use crate::*;

/// Writer - has support for wrapping text, page layout, fonts, etc.
pub struct Writer {
    /// Underlying Basic Writer
    pub b: BasicPdfWriter,
    /// Current Page
    pub p: Page,
    /// List of fonts
    pub fonts: FontFamily,
    /// Index into fonts
    pub cur_font: usize,
    /// Current font size, default is 10
    pub font_size: Px,
    /// Current sup ( raises text up off line ), use set_sup to adjust it
    pub sup: Px,
    /// Writing mode
    pub mode: Mode,
    /// PDF title
    pub title: String,
    /// List of Pages
    pub pages: Vec<Page>,
    /// Page is new ( not yet initialised )
    pub new_page: bool,
    /// Line padding ( space between lines ) default is 4
    pub line_pad: Px,
    /// Line margin ( left ), default is 20
    pub margin_left: Px,
    /// Line margin ( right ), default is 20
    pub margin_right: Px,
    /// Top margin, default is 20
    pub margin_top: Px,
    /// Bottom margin, default is 20
    pub margin_bottom: Px,
    /// Page width, default is 600
    pub page_width: Px,
    /// Page height, default is 800
    pub page_height: Px,
    /// Line used ( controls word-wrapping )
    pub line_used: MPx,
    /// Line items
    pub line: Vec<Item>,
    /// Largest font for current line
    pub max_font_size: Px,
    /// Default is zero, set to 1 to center output lines
    pub center: bool,
    /// For fetching fonts and images
    pub fetcher: Option<Box<dyn Fetcher>>,
    /// Cache of images
    pub image_cache: BTreeMap<String,Image>,
}

impl Default for Writer {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            title: String::new(),
            b: BasicPdfWriter::default(),
            fonts: helvetica(),
            cur_font: 0,
            font_size: 10,
            sup: 0,
            p: Page::default(),
            pages: Vec::new(),
            new_page: true,

            page_width: 600,
            page_height: 800,
            line_pad: 4,
            margin_left: 20,
            margin_right: 20,
            margin_top: 20,
            margin_bottom: 20,
            line_used: 0,
            line: Vec::new(),
            max_font_size: 0,
            center: false,
            fetcher: None,
            image_cache: BTreeMap::new(),
        }
    }
}

impl Writer {
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

    /// Completes current page.
    pub fn save_page(&mut self) {
        let p = std::mem::take(&mut self.p);
        self.pages.push(p);
        self.new_page = true;
    }

    fn init_font(&mut self, x: usize) {
        let f = &mut self.fonts[x];
        f.init(&mut self.b);
    }

    fn width(&self, c: char) -> MPx {
        let f = &self.fonts[self.cur_font];
        f.width(c) * self.font_size as MPx
    }

    fn line_len(&self) -> MPx {
        ((self.page_width - self.margin_left - self.margin_right) as MPx) * 1000
    }

    fn wrap_init(&mut self) {
        if self.new_page {
            self.init_page();
        }
    }

    fn wrap_text(&mut self, s: &str) {
        self.wrap_init();

        let mut width: MPx = 0;
        for c in s.chars() {
            width += self.width(c); // May depend on current font.
        }

        if self.line_used + width > self.line_len() {
            self.output_line();
            if s == " " {
                return;
            }
        }
        self.line_used += width;

        self.init_font(self.cur_font);
        if self.font_size > self.max_font_size {
            self.max_font_size = self.font_size;
        }

        self.line.push(Item::Text(
            s.to_string(),
            self.cur_font,
            self.font_size,
            width,
        ));
    }

    fn wrap_image(&mut self, im: Image, width: Px, scale: f32) {
        self.wrap_init();

        let width = (width as MPx) * 1000; // Convert width to MPx

        if self.line_used + width > self.line_len() {
            self.output_line();
        }

        self.line_used += width;
        self.line.push(Item::Img(im, width, scale));
    }

    /// Outputs current line ( consisting of items ).
    pub fn output_line(&mut self) {
        if self.new_page {
            self.init_page();
        } else {
            let cx = if self.center {
                ((self.line_len() - self.line_used) / 2000) as Px
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
        let mut cx: MPx = 0;
        for item in &self.line {
            match item {
                Item::Text(s, f, x, w) => {
                    let fp = &*self.fonts[*f];
                    self.p.text(fp, *x, s);
                    cx += w;
                }
                Item::Sup(x) => {
                    self.p.set_sup(*x);
                }
                Item::Img(im, width, scale) => {
                    self.p.flush_text();
                    let x: f32 = (self.p.x as f32) + (cx as f32 / 1000.0);
                    let y = self.p.y as f32;
                    im.draw(&mut self.p, x, y, *scale);
                    cx += width;
                    self.p.space(*width);
                }
            }
        }
        self.line.clear();
        self.line_used = 0;
        self.max_font_size = 0;
    }

    /// Writes word-wrapped text if mode is Normal, adds text to title if mode is Title.
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

    fn fetch_image(&mut self, src: &str) -> Option<Image>
    {
        let mut result = None;
        if let Some(im) = self.image_cache.get(src)
        {
           result = Some(im.clone());
        }
        else
        {
            let mut bf = std::mem::take(&mut self.fetcher);
            if let Some(f) = &mut bf {
                let im = f.image(self, src);
                self.image_cache.insert( src.to_owned(), im.clone() );
                result = Some(im);
            }
            self.fetcher = bf;
        }
        result
    }   

    /// Write image
    pub fn image(&mut self, src: &str, awidth: Option<Px>, aheight: Option<Px>) {
        if let Some(im) = self.fetch_image( src ) {
            let mut width: Px = im.width;
            let mut scale: f32 = 1.0;
            if let Some(awidth) = awidth {
                scale = awidth as f32 / width as f32;
                width = awidth;
            } else if let Some(aheight) = aheight {
                scale = aheight as f32 / im.height as f32;
                width = (width as f32 * scale) as Px;
            }
            self.wrap_image(im, width, scale);
        } else {
            self.text("error : no fetcher in pdf-min::Writer");
        }
    }

    /// Adds a space to text.
    pub fn space(&mut self) {
        self.text(" ");
    }

    /// Sets sup
    pub fn set_sup(&mut self, sup: Px) {
        self.line.push(Item::Sup(sup));
        self.sup = sup;
    }

    /// Flushes output line, writes page footers, saves pages, sets title, returns finished PDF as byte slice.
    pub fn finish(&mut self) -> &[u8] {
        self.output_line();
        self.init_font(0);
        self.save_page();
        let n = self.pages.len();
        let mut pnum = 1;
        let font_size = 8;
        #[allow(clippy::explicit_counter_loop)]
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
        &self.b.b
    }
}

/// Writing mode (for html)
#[derive(Clone, Copy)]
pub enum Mode {
    /// Normal
    Normal,
    /// Text output is suppressed
    Head,
    /// Text is appended to title
    Title,
}

/// Items that define a line of text.
pub enum Item {
    /// Text, font index, font size, width
    Text(String, usize, Px, MPx),
    /// Sup value ( raise text above base line )
    Sup(Px),
    /// Image, image, width, scale
    Img(Image, MPx, f32),
}

/// Instances can fetch an image or font
pub trait Fetcher {
    /// Fetch named image
    fn image(&mut self, _w: &mut Writer, _name: &str) -> Image {
        todo!()
    }
    /// Fetch specified font
    fn font(&mut self, _w: &mut Writer, _name: &str) -> Box<dyn Font> {
        todo!()
    }
}
