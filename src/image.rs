//!# Test example
//!
//! ```
//!      use pdf_min::{Writer, html, image::{ImageSpec,Image}};
//!   
//!      let mut w = Writer::default();
//!      w.b.nocomp = true;
//!      
//!      let mut data = Vec::new();
//!      for i in 0..3 * 16 * 16 {
//!         data.push( i as u8 );  // Red
//!         data.push( ( i + 85 ) as u8 ); // Green
//!         data.push( ( i + 85 + 85 ) as u8 ); // Blue
//!      }
//!      
//!      let ims = ImageSpec{ data: &data, width:16, height:16, bits_per_component:8, color_space: b"/DeviceRGB", other: b"" };
//!      let im = Image::new( &ims, &mut w.b );
//!   
//!      // Draw some text on the current page.
//!      html( &mut w, b"<p>Hello <b>there</b><p>Hello <i>again</i>" );
//!   
//!      // Draw a rectangle on the current page.
//!      w.p.rect( 100.0, 200.0, 160.0, 100.0 );
//!   
//!      // Draw the image on the current page.
//!      im.draw( &mut w.p, 100.0, 300.0, 10.0 );
//!   
//!      // Flush text
//!      w.output_line();
//!   
//!      // Finish the page
//!      w.save_page();
//!   
//!      // Draw some more text on the next page.
//!      html( &mut w, b"<p>Some <b>more</b> text" );
//!   
//!      let bytes = w.finish();
//!   
//!      use std::fs::File;
//!      use std::io::prelude::*;
//!   
//!      let mut file = File::create("image_test.pdf").unwrap();
//!      file.write_all(bytes).unwrap();
//!
//! ```

use crate::BasicPdfWriter;
use crate::page::Page;
use format_bytes::write_bytes as wb;

/// PDF image specification - byte data and attributes that describe how image is encoded.
pub struct ImageSpec<'a> {
    /// Image data - length is width * height * (bits_per_component/8) * 3 (for RGB).
    pub data: &'a [u8],
    /// Width
    pub width: usize,
    /// Height
    pub height: usize,
    /// Bits per component, usually 8
    pub bits_per_component: u8,
    /// Color space, such as b"/DeviceGray", b"/DeviceRGB", b"/DeviceCMYK"
    pub color_space: &'a [u8],
    /// Any other attributes, e.g. b"/Filter/DCT" for a jpeg
    pub other: &'a [u8],
}

/// PDF image - obj id, width and height
pub struct Image {
    /// PDF obj id
    pub obj: usize,
    /// Width
    pub width: usize,
    /// Height
    pub height: usize,
}

impl Image {
    /// Writes the specificed image attributes and data to the PDF, returns Image with obj id, width and height.
    pub fn new(s: &ImageSpec, w: &mut BasicPdfWriter) -> Image {
        let obj = w.begin();
        let _ = wb!(
            &mut w.b,
            b"<</Type/XObject/Subtype/Image/Width {}/Height {}/ColorSpace{}/BitsPerComponent {}/Length {}{}>>stream\n",
            s.width, s.height, s.color_space, s.bits_per_component, s.data.len(), s.other
        );
        w.b.extend_from_slice(s.data);
        w.b.extend_from_slice(b"\nendstream");
        w.end();
        Image {
            obj,
            height: s.height,
            width: s.width,
        }
    }

    /// Draw image on page.
    pub fn draw(&self, page: &mut Page, x: f32, y: f32, scale: f32) {
        let obj = self.obj;
        let w = (self.width as f32) * scale;
        let h = (self.height as f32) * scale;
        page.xobjs.insert(obj);
        let _ = wb!(
            &mut page.os,
            b"\nq {} 0 0 {} {} {} cm /X{} Do Q",
            w,
            h,
            x,
            y,
            obj
        );
    }
}
