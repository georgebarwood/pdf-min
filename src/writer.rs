use crate::*;

#[derive(Clone, Copy)]
pub enum Mode {
    Normal,
    Head,
    Title,
}

pub struct Writer {
    pub b: BasicPdfWriter,
    pub p: Page,
    pub fonts: Vec<Box<dyn Font>>,
    pub cur_font: usize, // index into fonts
    pub font_size: i16,
    pub mode: Mode,
    pub title: String,
    pub pages: Vec<Page>,
    pub new_page: bool,

    pub line_pad: i16,
    pub margin_left: i16,
    pub margin_bottom: i16,
    pub page_width: i16,
    pub page_height: i16,

    pub skip_space: bool, // White space is to be ignored.
}

impl Default for Writer {
    fn default() -> Self {
        let mut x = Self {
            mode: Mode::Normal,
            title: String::new(),
            b: BasicPdfWriter::default(),
            fonts: Vec::new(),
            cur_font: 0,
            font_size: 10,
            p: Page::default(),
            pages: Vec::new(),
            new_page: true,

            page_width: 400,
            page_height: 600,
            line_pad: 4,
            margin_left: 10,
            margin_bottom: 10,
            skip_space: true,
        };
        for _ in 0..4 {
            x.fonts.push(Box::new(StandardFont::new()));
        }
        x
    }
}

impl Writer {
    pub fn init_page(&mut self) {
        self.p.width = self.page_width;
        self.p.height = self.page_height;
        self.p.goto(
            self.margin_left,
            self.p.height - self.font_size - self.line_pad,
        );
    }

    pub fn new_page(&mut self) {
        let p = std::mem::replace(&mut self.p, Page::default());
        self.pages.push(p);
        self.new_page = true;
    }

    fn init_font(&mut self, x: usize) {
        let f = &mut self.fonts[x];
        f.init(&mut self.b, HELVETICA[x]);
    }

    pub fn text(&mut self, s: &[u8]) {
        match self.mode {
            Mode::Normal => {
                self.init_font(self.cur_font);
                if self.new_page {
                    self.init_page();
                    self.new_page = false;
                }
                let f = &mut self.fonts[self.cur_font];
                self.p.text(f, self.font_size, tosl(s));
            }
            Mode::Title => {
                self.title += tosl(s);
            }
            Mode::Head => {}
        }
    }

    pub fn space(&mut self) {
        self.text(b" ");
    }

    pub fn new_line(&mut self) {
        if self.new_page {
            self.init_page();
            self.new_page = false;
        } else {
            let h = self.font_size + self.line_pad;
            if self.p.y >= h + self.margin_bottom {
                self.p.td(0, -h);
            } else {
                self.new_page();
                self.init_page();
                self.new_page = false;
            }
        }
    }

    pub fn finish(&mut self) {
        self.init_font(0);
        self.new_page();
        let n = self.pages.len();
        let mut pnum = 1;
        let font_size = self.font_size;
        for p in &mut self.pages {
            p.goto(self.margin_left, self.line_pad);
            p.text(
                &self.fonts[0],
                font_size,
                &format!("Page {} of {}", pnum, n),
            );
            p.finish();
            pnum += 1;
        }
        self.b.finish(&self.pages, &self.title.as_bytes());
    }
}

/// Convert byte slice into string.
fn _tos(s: &[u8]) -> String {
    std::str::from_utf8(s).unwrap().to_string()
}

/// Convert byte slice into string.
fn tosl(s: &[u8]) -> &str {
    std::str::from_utf8(s).unwrap()
}

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
        }
    }

    fn tvalue(&self) -> &'a [u8] {
        &self.source[self.token_start..self.token_end]
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
            // e.g. <h1 name=x> or </h1>s
            // Find tag name, then read to end of tag.

            self.token = Token::Tag;
            self.token_start = self.position;
            self.end_tag = false;
            let mut c = self.next();
            if c == b'/' {
                self.end_tag = true;
                self.token_start = self.position;
                c = self.next();
            }
            let mut got_end = false;
            loop {
                if c == b' ' {
                    if !got_end {
                        self.token_end = self.position;
                        got_end = true;
                    }
                } else if c == b'>' {
                    if !got_end {
                        self.token_end = self.position - 1;
                        break;
                    }
                } else if c == 0 {
                    self.token = Token::Eof;
                    break;
                }
                c = self.next();
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
        // println!("token {:?} tvalue={} end_tag={}", p.token, tosl(p.tvalue()), p.end_tag);
        match p.token {
            Token::Eof => {
                return;
            }
            Token::WhiteSpace => {
                if !w.skip_space {
                    w.space();
                }
                p.read_token();
            }
            Token::Text => {
                w.text(p.tvalue());
                w.skip_space = false;
                p.read_token();
            }
            Token::Tag => {
                w.skip_space = true;
                let tag = p.tvalue();

                if tag == b"p" && tag == endtag {
                    return;
                } else if p.end_tag {
                    if tag == endtag {
                        p.read_token();
                    }
                    return;
                }
                p.read_token();
                if tag == b"br" || tag == b"br/" {
                    w.new_line();
                } else {
                    let save_mode = w.mode;
                    let save_font = w.cur_font;
                    let save_font_size = w.font_size;
                    let mut save: i16 = 0;
                    match tag {
                        b"p" => w.new_line(),
                        b"h1" => {
                            w.font_size = 14;
                            w.new_line();
                        }
                        b"b" => w.cur_font |= 1,
                        b"i" => w.cur_font |= 2,
                        b"title" => w.mode = Mode::Title,
                        b"html" | b"head" => w.mode = Mode::Head,
                        b"body" => w.mode = Mode::Normal,
                        b"sup" => {
                            save = w.p.sup;
                            w.p.set_sup(w.font_size as i16 / 2);
                        }
                        b"sub" => {
                            save = w.p.sup;
                            w.p.set_sup(-(w.font_size as i16) / 2);
                        }
                        _ => {}
                    }
                    html_inner(w, p, tag);
                    w.mode = save_mode;
                    w.font_size = save_font_size;
                    w.cur_font = save_font;
                    match tag {
                        b"sup" | b"sub" => w.p.set_sup(save),
                        b"h1" => w.new_line(),
                        _ => {}
                    }
                }
            }
        }
    }
}