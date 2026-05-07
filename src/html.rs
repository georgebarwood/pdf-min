use crate::*;
use std::collections::BTreeMap;

/// Convert source html to PDF using Writer w.
pub fn html(w: &mut Writer, source: &[u8]) {
    let mut p = Parser::new(source);
    p.read_token();
    html_inner(w, &mut p, b"");
}

#[derive(Debug)]
enum Token {
    Text,
    Tag,
    WhiteSpace,
    Eof,
}

struct Parser<'a> {
    source: &'a [u8],
    position: usize,
    token_start: usize,
    token_end: usize,
    end_tag: bool,
    token: Token,
    attr: BTreeMap< &'a [u8], &'a [u8] >,
}

impl<'a> Parser<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
            token_start: 0,
            token_end: 0,
            end_tag: false,
            token: Token::Eof,
            attr: BTreeMap::new(),
        }
    }

    fn tvalue(&self) -> &'a [u8] {
        &self.source[self.token_start..self.token_end]
    }

    /// Get attribute value,
    fn avalue(&self, name: &'a[u8] ) -> Option<&&'a[u8]> {
        self.attr.get(name)
    }

    /// Get integer attribute, return None on not present or error.
    fn aint(&self, name: &'a[u8] ) -> Option<Px> {
        if let Some(s) = self.avalue(name) && let Ok(x) = tos(s).parse::<Px>() {
            return Some(x) 
        }
        None
    }

    fn next(&mut self) -> u8 {
        if self.position == self.source.len() {
            0
        } else {
            let c = self.source[self.position];
            self.position += 1;
            c
        }
    }

    fn next_non_space(&mut self) -> u8
    {
        loop
        {
           let c = self.next();
           if c != b' ' { return c; }
        }
    }   

    fn read_tag_attributes(&mut self)
    {
       // Example: width = 15 alt = "something" src = "something" >
       loop
       {
           let mut c = self.next_non_space();
           let attr_name_start = self.position-1;
           while c != b'=' && c != b' ' && c != b'>' && c != 0 {
              c = self.next();
           }
           if c == b'>' { return; }
           let attr_name = &self.source[attr_name_start..self.position-1];
           if c == b' ' { c = self.next_non_space(); }
           if c != b'=' { return; }
           c = self.next_non_space();
           let start = self.position - 1;
           let attr = if c == b'"' { // Read quoted attribute
               c = self.next();
               while c != b'"' && c != 0 { c = self.next(); }
               if c != b'"' { return; } 
               &self.source[start+1..self.position-1]
           } else { // Read unquoted attribute
               while c != b' ' && c != b'>' && c != 0 { c = self.next(); }
               &self.source[start..self.position-1] 
           };
           self.attr.insert(attr_name, attr);
           if c == b'>' { return; }
       }
    }

    fn read_token(&mut self) {
        let c = self.next();
        if c == 0 {
            self.token = Token::Eof;
        } else if c == b' ' || c == b'\n' {
            self.token = Token::WhiteSpace;
            loop {
                let c = self.next();
                if c != b' ' || c != b'\n' {
                    if c != 0 {
                        self.position -= 1;
                    }
                    break;
                }
            }
        } else if c == b'<' {
            // e.g. <h1 name=x> or </h1>
            self.token = Token::Tag;
            self.token_start = self.position;
            self.end_tag = false;
            let mut c = self.next();
            if c == b'/' {
                self.end_tag = true;
                self.token_start = self.position;
                c = self.next();
            }
            loop { // To find end of tag name
                if c == b' ' {
                    self.token_end = self.position - 1;
                    self.read_tag_attributes();
                    break;
                } else if c == b'>' {
                    self.token_end = self.position - 1;
                    break; // No attributes to parse
                } else if c == 0 {
                    self.token =  Token::Eof; // Error
                    return;
                } else {
                    c = self.next();
                }
            }
        } else {
            self.token = Token::Text;
            self.token_start = self.position - 1;
            let mut c = self.next();
            loop {
                if c == b'<' || c == b' ' || c == b'\n' {
                    self.position -= 1;
                    self.token_end = self.position;
                    break;
                } else if c == 0 {
                    self.token_end = self.position;
                    break;
                }
                c = self.next();
            }
        }
    }
}

fn html_inner(w: &mut Writer, p: &mut Parser, endtag: &[u8]) {
    loop {
        match p.token {
            Token::Eof => {
                return;
            }
            Token::WhiteSpace => {
                w.space();
                p.read_token();
            }
            Token::Text => {
                let s = tos(p.tvalue());
                let s = &html_escape::decode_html_entities(s);
                w.text(s);
                p.read_token();
            }
            Token::Tag => {
                let tag = p.tvalue();
                if p.end_tag {
                    if tag == endtag {
                        p.read_token();
                    }
                    return;
                } else if tag == b"p" && tag == endtag {
                    return;
                }
                p.read_token();
                if tag == b"br" || tag == b"br/" {
                    w.output_line();
                } else if tag == b"img" {
                    if let Some(src) = p.avalue( b"src" )
                    {
                        let width = p.aint( b"width" );
                        let height = p.aint( b"height" );
                        w.image( tos(src), width, height );
                    }
                } else {
                    let save_mode = w.mode;
                    let save_font = w.cur_font;
                    let save_font_size = w.font_size;
                    let mut save: Px = 0;
                    match tag {
                        b"p" => w.output_line(),
                        b"h1" => {
                            w.font_size = 14;
                            w.output_line();
                            save = if w.center { 1 } else { 0 };
                            w.center = true;
                        }
                        b"b" => w.cur_font |= 1,
                        b"i" => w.cur_font |= 2,
                        b"title" => w.mode = Mode::Title,
                        b"html" | b"head" => w.mode = Mode::Head,
                        b"body" => w.mode = Mode::Normal,
                        b"sup" => {
                            save = w.sup;
                            w.set_sup(w.font_size / 2);
                        }
                        b"sub" => {
                            save = w.sup;
                            w.set_sup(-w.font_size / 2);
                        }
                        _ => {}
                    }
                    html_inner(w, p, tag);
                    w.mode = save_mode;
                    w.font_size = save_font_size;
                    w.cur_font = save_font;
                    match tag {
                        b"sup" | b"sub" => w.set_sup(save),
                        b"h1" => {
                            w.output_line();
                            w.center = save == 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Convert byte slice into string.
fn tos(s: &[u8]) -> &str {
    std::str::from_utf8(s).unwrap()
}
