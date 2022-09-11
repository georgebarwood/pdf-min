use crate::basic::BasicPdfWriter;
use format_bytes::write_bytes as wb;

/// Names of standard Times font.
pub static TIMES: [&str; 4] = [
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
];

/// Names of standard Helvetica font.
pub static HELVETICA: [&str; 4] = [
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
];

/// Names of standard Courier font.
pub static COURIER: [&str; 4] = [
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
];

pub trait Font {
    fn obj(&self) -> usize;
    fn encode(&self, s: &str, to: &mut Vec<u8>);
    fn init(&mut self, w: &mut BasicPdfWriter, name: &str);
}

#[derive(Default)]
pub struct StandardFont {
    /// PDF object ID
    pub obj: usize,
}

impl Font for StandardFont {
    fn obj(&self) -> usize {
        self.obj
    }

    fn init(&mut self, w: &mut BasicPdfWriter, name: &str) {
        if self.obj == 0 {
            self.obj = w.obj();
            w.start(self.obj);
            let _ = wb!(
                &mut w.b,
                b"<</Type/Font/Subtype/Type1/Name/F{}/BaseFont/{}/Encoding/WinAnsiEncoding>>",
                self.obj,
                name.as_bytes()
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

impl StandardFont {
    pub fn new() -> Self {
        Self { obj: 0 }
    }
}
