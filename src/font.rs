use crate::MPx;
use crate::basic::BasicPdfWriter;
use crate::metric::*;
use format_bytes::write_bytes as wb;

/// Font
pub trait Font {
    /// Get the PDF object number.
    fn obj(&self) -> usize;

    /// Encode string.
    fn encode(&self, s: &str, to: &mut Vec<u8>);

    /// Get char width
    fn width(&self, c: char) -> MPx;

    /// Initialise the font by writing defition to w.
    fn init(&mut self, w: &mut BasicPdfWriter);
}

/// Font family - normal, bold, italic, bold italic
pub type FontFamily = [Box<dyn Font>; 4];

/// Helvetica standard font family
pub fn helvetica() -> FontFamily {
    let a = StandardFont::make(&HELVETICA0[..], HELVETICA[0]);
    let b = StandardFont::make(&HELVETICA1[..], HELVETICA[1]);
    let c = StandardFont::make(&HELVETICA2[..], HELVETICA[2]);
    let d = StandardFont::make(&HELVETICA3[..], HELVETICA[3]);
    [a, b, c, d]
}

/// Times standard font family
pub fn times() -> FontFamily {
    let a = StandardFont::make(&TIMES0[..], TIMES[0]);
    let b = StandardFont::make(&TIMES1[..], TIMES[1]);
    let c = StandardFont::make(&TIMES2[..], TIMES[2]);
    let d = StandardFont::make(&TIMES3[..], TIMES[3]);
    [a, b, c, d]
}

/// Courier standard font family
pub fn courier() -> FontFamily {
    let a = StandardFont::make(&COURIER0[..], COURIER[0]);
    let b = StandardFont::make(&COURIER1[..], COURIER[1]);
    let c = StandardFont::make(&COURIER2[..], COURIER[2]);
    let d = StandardFont::make(&COURIER3[..], COURIER[3]);
    [a, b, c, d]
}

/// Standard Font
#[derive(Default)]
pub struct StandardFont {
    obj: usize,
    size_data: &'static [u16],
    name: &'static str,
}

impl StandardFont {
    fn make(size_data: &'static [u16], name: &'static str) -> Box<dyn Font> {
        Box::new(Self {
            obj: 0,
            size_data,
            name,
        })
    }
}

impl Font for StandardFont {
    fn obj(&self) -> usize {
        self.obj
    }

    fn width(&self, c: char) -> MPx {
        let mut c = c as usize;
        if c < 32 || c - 32 >= self.size_data.len() {
            c = 32;
        }
        self.size_data[c - 32] as MPx
    }

    fn init(&mut self, w: &mut BasicPdfWriter) {
        if self.obj == 0 {
            self.obj = w.begin();
            let _ = wb!(
                &mut w.b,
                b"<</Type/Font/Subtype/Type1/Name/F{}/BaseFont/{}/Encoding/WinAnsiEncoding>>",
                self.obj,
                self.name.as_bytes()
            );
            w.end();
        }
    }

    fn encode(&self, s: &str, to: &mut Vec<u8>) {
        let mut e = encoding_rs::WINDOWS_1252.new_encoder();
        let x = e
            .max_buffer_length_from_utf8_without_replacement(s.len())
            .unwrap();
        to.reserve(x); // Weird that this is necessary.
        let (r, _n) = e.encode_from_utf8_to_vec_without_replacement(s, to, false);
        assert!(r == encoding_rs::EncoderResult::InputEmpty);
    }
}

/// Names of standard Times font.
static TIMES: [&str; 4] = [
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
];

/// Names of standard Helvetica font.
static HELVETICA: [&str; 4] = [
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
];

/// Names of standard Courier font.
static COURIER: [&str; 4] = [
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
];
