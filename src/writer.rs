use crate::*;

///
pub struct Writer {
    ///
    pub b: BasicPdfWriter,
    ///
    pub p: Page,
    ///
    pub fonts: Vec<Box<dyn Font>>,
    ///
    pub cur_font: usize, // index into fonts
    ///
    pub font_size: i16,
    ///
    pub sup: i16,
    ///
    pub mode: Mode,
    ///
    pub title: String,
    ///
    pub pages: Vec<Page>,
    ///
    pub new_page: bool,

    ///
    pub line_pad: i16,
    ///
    pub margin_left: i16,
    ///
    pub margin_right: i16,
    ///
    pub margin_top: i16,
    ///
    pub margin_bottom: i16,
    ///
    pub page_width: i16,
    ///
    pub page_height: i16,
    ///
    pub skip_space: bool, // White space is to be ignored.
    ///
    pub line_used: i16,
    ///
    pub line: Vec<Item>,
    ///
    pub max_font_size: i16,
    ///
    pub center: i16,
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
            sup: 0,
            p: Page::default(),
            pages: Vec::new(),
            new_page: true,

            page_width: 400,
            page_height: 600,
            line_pad: 4,
            margin_left: 20,
            margin_right: 20,
            margin_top: 20,
            margin_bottom: 20,
            skip_space: true,
            line_used: 0,
            line: Vec::new(),
            max_font_size: 0,
            center: 0,
        };
        for _ in 0..4 {
            x.fonts.push(Box::<StandardFont>::default());
        }
        x
    }
}

impl Writer {
    ///
    fn init_page(&mut self) {
        self.p.width = self.page_width;
        self.p.height = self.page_height;
        self.p.goto(
            self.margin_left,
            self.p.height - self.font_size - self.margin_top,
        );
        if self.sup != 0 {
            self.p.set_sup(self.sup);
        }
        self.new_page = false;
    }

    ///
    fn save_page(&mut self) {
        let p = std::mem::take(&mut self.p);
        self.pages.push(p);
        self.new_page = true;
    }

    ///
    fn init_font(&mut self, x: usize) {
        let f = &mut self.fonts[x];
        f.init(&mut self.b, HELVETICA[x]);
    }

    ///
    fn width(&self, _c: char) -> u64 {
        // Ought to take some account of upper/lower case.
        // This is rather preliminary.
        if (self.cur_font & 1) == 1 {
            550
        } else {
            500
        }
    }

    ///
    fn line_len(&self) -> i16 {
        self.page_width - self.margin_left - self.margin_right
    }

    ///
    fn wrap_text(&mut self, s: &str) {
        if self.new_page {
            self.init_page();
        }

        let mut w = 0;
        for c in s.chars() {
            w += self.width(c);
        }
        let w = (w * self.font_size as u64 / 1000) as i16;

        if self.line_used + w > self.line_len() {
            self.output_line();
            if s == " " {
                return;
            }
        }
        self.line_used += w;
        self.init_font(self.cur_font);
        if self.font_size > self.max_font_size {
            self.max_font_size = self.font_size;
        }
        self.line
            .push(Item::Text(s.to_string(), self.cur_font, self.font_size));
    }

    ///
    pub fn output_line(&mut self) {
        if self.new_page {
            self.init_page();
        } else {
            let cx = if self.center == 1 {
                (self.line_len() - self.line_used) / 2
            } else {
                0
            };
            let h = self.max_font_size + self.line_pad;
            if self.p.y >= h + self.margin_bottom {
                self.p.td(self.margin_left + cx - self.p.x, -h);
            } else {
                self.save_page();
                self.init_page();
            }
        }
        for item in &self.line {
            match item {
                Item::Text(s, f, x) => {
                    let fp = &*self.fonts[*f];
                    self.p.text(fp, *x, s);
                }
                Item::Sup(x) => {
                    self.p.set_sup(*x);
                }
            }
        }
        self.line.clear();
        self.line_used = 0;
        self.max_font_size = 0;
    }

    ///
    pub fn text(&mut self, s: &str) {
        match self.mode {
            Mode::Normal => {
                self.wrap_text(s);
            }
            Mode::Title => {
                self.title += s;
            }
            Mode::Head => {}
        }
    }

    ///
    pub fn space(&mut self) {
        self.text(" ");
    }

    ///
    pub fn set_sup(&mut self, sup: i16) {
        self.line.push(Item::Sup(sup));
    }

    ///
    pub fn finish(&mut self) {
        self.output_line();
        self.init_font(0);
        self.save_page();
        let n = self.pages.len();
        let mut pnum = 1;
        let font_size = 8;
        for p in &mut self.pages {
            p.goto(self.margin_left, self.line_pad);
            p.text(
                &*self.fonts[0],
                font_size,
                &format!("Page {} of {}", pnum, n),
            );
            p.finish();
            pnum += 1;
        }
        self.b.finish(&self.pages, self.title.as_bytes());
    }
}

///
#[derive(Clone, Copy)]
pub enum Mode {
    ///
    Normal,
    ///
    Head,
    ///
    Title,
}

/// Items that define a line of text.
pub enum Item {
    ///
    Text(String, usize, i16),
    ///
    Sup(i16),
}

/// Convert byte slice into string.
fn _tos(s: &[u8]) -> String {
    std::str::from_utf8(s).unwrap().to_string()
}

/// Convert byte slice into string.
fn tosl(s: &[u8]) -> &str {
    std::str::from_utf8(s).unwrap()
}

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
                let s = tosl(p.tvalue());
                let s = &html_escape::decode_html_entities(s);
                w.text(s);
                w.skip_space = false;
                p.read_token();
            }
            Token::Tag => {
                w.skip_space = true;
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
                } else {
                    let save_mode = w.mode;
                    let save_font = w.cur_font;
                    let save_font_size = w.font_size;
                    let mut save: i16 = 0;
                    match tag {
                        b"p" => w.output_line(),
                        b"h1" => {
                            w.font_size = 14;
                            w.output_line();
                            save = w.center;
                            w.center = 1;
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
                            w.center = save;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
