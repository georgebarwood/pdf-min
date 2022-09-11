use crate::*;
use format_bytes::write_bytes as wb;
use std::collections::BTreeSet;

/// Low level PDF writing.
pub struct BasicPdfWriter {
    /// Output buffer.
    pub b: Vec<u8>,
    /// Offset in file of each object.
    pub xref: Vec<usize>,
    /// For compressing streams.
    pub comp: flate3::Compressor,
    /// Suppresses compression.
    pub nocomp: bool,
}

impl Default for BasicPdfWriter {
    fn default() -> Self {
        let mut b = Vec::new();
        b.extend_from_slice(b"%PDF-1.4\n");
        Self {
            b,
            xref: Vec::new(),
            comp: flate3::Compressor::default(),
            nocomp: true,
        }
    }
}

impl BasicPdfWriter {
    pub fn standard_font(&mut self, name: &str) -> Box<dyn Font> {
        let mut f = StandardFont::new();
        f.init(self, name);
        Box::new(f)
    }

    /// Allocate PDF object number.
    pub fn obj(&mut self) -> usize {
        self.xref.push(0);
        self.xref.len()
    }

    /// Start definition of PDF object.
    pub fn start(&mut self, obj_num: usize) {
        self.xref[obj_num - 1] = self.b.len();
        let _ = wb!(&mut self.b, b"{} 0 obj\n", obj_num);
    }

    /// End definition of PDF object.
    pub fn end(&mut self) {
        self.b.extend_from_slice(b"\nendobj\n");
    }

    /// Finish the PDF by writing a list of pages.
    pub fn finish(&mut self, pages: &[Page], title: &[u8]) {
        let mut kids = Vec::new();
        let pagesobj = self.obj();
        for p in pages {
            let contentobj = self.put_stream(&p.os);
            let pageobj = self.obj();
            self.start(pageobj);
            let _ = wb!(&mut kids, b"{} 0 R ", pageobj);
            let _ = wb!(
                &mut self.b,
                b"<</Type/Page/Parent {} 0 R/MediaBox[0 0 {} {}]/Contents {} 0 R/Resources <<",
                pagesobj,
                p.width,
                p.height,
                contentobj
            );

            self.put_resource_set(&p.fonts, b"/Font", b"/F");
            self.put_resource_set(&p.xobjs, b"/XObject", b"/X");
            self.b.extend_from_slice(b" >> >>");
            self.end();
        }
        self.start(pagesobj);
        let _ = wb!(
            &mut self.b,
            b"<</Type/Pages/Count {}/Kids[{}]>>",
            pages.len(),
            kids
        );
        self.end();

        let cat = self.obj();
        self.start(cat);
        let _ = wb!(&mut self.b, b"<</Type/Catalog/Pages {} 0 R>>", pagesobj);
        self.end();

        let info = self.obj();
        self.start(info);
        let _ = wb!(&mut self.b, b"<</Title({})>>", title);
        self.end();

        let startxref = self.b.len();
        let xc = self.xref.len() + 1;
        let _ = wb!(&mut self.b, b"xref\n0 {}\n0000000000 65535 f\n", xc);

        for i in 0..self.xref.len() {
            let x = decimal(self.xref[i], 10);
            let _ = wb!(&mut self.b, b"{} 00000 n\n", x);
        }

        let _ = wb!(
            &mut self.b,
            b"trailer\n<</Size {}/Root {} 0 R/Info {} 0 R>>",
            xc,
            cat,
            info
        );
        let _ = wb!(&mut self.b, b"\nstartxref\n{}\n%%EOF\n", startxref);
    }

    fn put_resource_set(&mut self, s: &BTreeSet<usize>, n1: &[u8], n2: &[u8]) {
        if !s.is_empty() {
            let _ = wb!(&mut self.b, b"{}<<", n1);
            for i in s {
                let _ = wb!(&mut self.b, b"{}{} {} 0 R", n2, i, i);
            }
            self.b.extend_from_slice(b">>");
        }
    }

    fn put_stream(&mut self, data: &[u8]) -> usize {
        let obj = self.obj();
        self.start(obj);
        if self.nocomp {
            let _ = wb!(&mut self.b, b"<</Length {}>>stream\n", data.len());
            self.b.extend_from_slice(data);
        } else {
            let cb: Vec<u8> = self.comp.deflate(data);
            let _ = wb!(
                &mut self.b,
                b"<</Filter/FlateDecode/Length {}>>stream\n",
                cb.len()
            );
            self.b.extend_from_slice(&cb);
        }

        self.b.extend_from_slice(b"\nendstream");
        self.end();
        obj
    }
}

/// Format x as decimal padded to length n with zeros.
fn decimal(mut x: usize, mut n: usize) -> Vec<u8> {
    let mut result = vec![b'0'; n];
    while x != 0 {
        n -= 1;
        result[n] = b'0' + (x % 10) as u8;
        x /= 10;
    }
    result
}
