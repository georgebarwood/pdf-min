use crate::font::Font;
use crate::*;
use format_bytes::write_bytes as wb;
use std::collections::BTreeSet;

/// PDF Page, has buffered text stream, text position, font.
#[derive(Default)]
pub struct Page {
    /// Page width.
    pub width: Px,

    /// Page height.
    pub height: Px,

    /// Output buffer.
    pub os: Vec<u8>,

    /// Text stream.
    pub ts: Vec<u8>,

    /// Current text buffer.
    pub text: Vec<u8>,

    /// Current line position ( from left of page ).
    pub x: Px,

    /// Current line position ( from bottom of page ).
    pub y: Px,

    /// Current font.
    font_obj: usize,

    /// Current font size.
    font_size: Px,

    /// Current super
    pub sup: Px,

    /// For checking whether font has changed.
    last_font_obj: usize,

    /// For checking whether font size has changed.
    last_font_size: Px,

    /// Set of font obj numbers used by page.
    pub fonts: BTreeSet<usize>,

    /// Set of other obj numbers used by page.
    pub xobjs: BTreeSet<usize>,
}

impl Page {
    /// Start a new line ( absolute position ).
    pub fn goto(&mut self, x: Px, y: Px) {
        self.td(x - self.x, y - self.y);
    }

    /// Start a new line ( relative to previous line ).
    pub fn td(&mut self, x: Px, y: Px) {
        self.flush_text();
        let _ = wb!(&mut self.ts, b"\n{} {} Td ", x, y);
        self.x += x;
        self.y += y;
    }

    /// Append text ( encoded with font ).
    pub fn text(&mut self, font: &dyn Font, size: Px, s: &str) {
        if size != self.font_size || font.obj() != self.font_obj {
            self.flush_text();
            self.font_obj = font.obj();
            self.font_size = size;
        }
        font.encode(s, &mut self.text);
    }

    /// Leave some space.
    pub fn space(&mut self, amount: MPx) {
        let amount = amount / (self.font_size as MPx);
        let _ = wb!(&mut self.ts, b"[{}] TJ ", -amount);
    }

    /// Flush text using tj.
    pub fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }
        if self.font_obj != self.last_font_obj || self.font_size != self.last_font_size {
            self.fonts.insert(self.font_obj);
            let obj = self.font_obj;
            let size = self.font_size;
            let _ = wb!(&mut self.ts, b"/F{} {} Tf", obj, size);
            self.last_font_obj = obj;
            self.last_font_size = size;
        }
        let mut hex = false;
        for b in &self.text {
            if *b < 32 || *b >= 128 {
                hex = true;
                break;
            }
        }
        if hex {
            self.ts.push(b'<');
            for b in &self.text {
                let x = *b >> 4;
                self.ts.push(x + if x < 10 { 48 } else { 55 });
                let x = *b & 15;
                self.ts.push(x + if x < 10 { 48 } else { 55 });
            }
            self.ts.extend_from_slice(b"> Tj");
        } else {
            self.ts.push(b'(');
            for b in &self.text {
                let b = *b;
                if b == b'(' || b == b')' || b == b'\\' {
                    self.ts.push(b'\\');
                }
                self.ts.push(b);
            }
            self.ts.extend_from_slice(b") Tj");
        }
        self.text.clear();
    }

    /// Finish page by appending self.ts to self.os enclosed by "BT" and "ET".
    pub fn finish(&mut self) {
        self.flush_text();
        self.os.extend_from_slice(b"\nBT");
        self.os.extend_from_slice(&self.ts);
        self.ts.clear();
        self.os.extend_from_slice(b"\nET");
    }

    // Graphics operations

    /// Draw a line from (x0,y0) to (x1,y1)
    pub fn line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        let _ = wb!(&mut self.os, b"\n{} {} m {} {} l S", x0, y0, x1, y1);
    }

    /// Draw a rectangle with corners (x0,y0) to (x1,y1)
    pub fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        let _ = wb!(&mut self.os, b"\n{} {} {} {} re S", x0, y0, x1, y1);
    }

    /// Set level of text on line.
    pub fn set_sup(&mut self, sup: Px) {
        if self.sup != sup {
            self.flush_text();
            self.sup = sup;
            let _ = wb!(&mut self.ts, b" {} Ts", sup);
        }
    }
}
