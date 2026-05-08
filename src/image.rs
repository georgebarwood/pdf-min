//!# Test example
//!
//! ```
//!      use pdf_min::{Writer, html, writer::Fetcher, image::{ImageSpec,Image}};
//!      struct MyFetcher;
//!      impl Fetcher for MyFetcher {
//!         fn image(&mut self, w: &mut Writer, _name: &str) -> Image {         
//!             let mut data = Vec::new();
//!             for i in 0..3 * 16 * 16 {
//!                 data.push( i as u8 );  // Red
//!                 data.push( (i + 85 ) as u8 ); // Green
//!                 data.push( (i + 170 ) as u8 ); // Blue
//!             }
//!             let ims = ImageSpec{ data: &data, width:16, height:16,
//!                 bits_per_component:8, color_space: b"/DeviceRGB", other: b"" };
//!             Image::new( &ims, &mut w.b )
//!         }
//!      }
//!      let mut w = Writer::default();
//!      w.b.nocomp = true;
//!      w.font_size = 20;
//!      w.fetcher = Some(Box::new(MyFetcher));
//!   
//!      // Draw text with image
//!      html( &mut w, b"<p><b>Bold Text Before Image</b> <img width=32 src=myimg> Text after image" );
//!      let bytes = w.finish();
//!   
//!      use std::fs::File;
//!      use std::io::prelude::*;
//!   
//!      let mut file = File::create("image_test.pdf").unwrap();
//!      file.write_all(bytes).unwrap();
//! ```
//!
//!# JPG Test example
//! ```
//!    // Setup the PDF Writer
//!    let mut doc = pdf_min::Writer::default();
//!    doc.b.nocomp = true;
//!
//!    // Read jpg from file
//!    let file_bytes = std::fs::read("one.jpg").unwrap();
//!
//!    // Use jpeg_decoder::Decoder to get jpg info ( color space, bits_per_component, width, height ).
//!    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(&file_bytes));
//!    decoder.read_info().unwrap();
//!    let info = decoder.info().unwrap();
//!
//!    use jpeg_decoder::{PixelFormat};
//!    
//!    let color_space: &[u8] = match info.pixel_format {
//!        PixelFormat::RGB24 => b"/DeviceRGB",
//!        PixelFormat::CMYK32 => b"/DeviceCMYK",
//!        PixelFormat::L8 | PixelFormat::L16 => b"/DeviceGray",
//!    };
//!
//!    let bits_per_component = match info.pixel_format {
//!        PixelFormat::L16 => 16,
//!        _ => 8
//!    };
//!
//!    // Use img_parts::jpeg::Jpeg to make DCT (Discrete Cosine Transform) compressed data.
//!    let cdata =
//!    {
//!        let mut cdata = Vec::new();
//!        let jpeg = img_parts::jpeg::Jpeg::from_bytes(file_bytes.into()).unwrap();
//!        jpeg.encoder().write_to(&mut cdata).unwrap();
//!        cdata
//!    };
//!
//!    // Make the ImageSpec.
//!    use pdf_min::{Px, image::{ImageSpec, Image}};
//!    let ims = ImageSpec {
//!        data: &cdata,
//!        width: info.width as Px,
//!        height: info.height as Px,
//!        color_space,
//!        bits_per_component,
//!        other: b"/Filter/DCT",
//!    };
//!    
//!    // Make the Image from the ImageSpec.
//!    let im = Image::new(&ims, &mut doc.b);
//!
//!    // Draw the image on the current page.
//!    im.draw(&mut doc.p, 20.0, 40.0, 0.20);
//!
//!    // Save the pdf as a file.
//!    let bytes = doc.finish();
//!    let mut file = std::fs::File::create("jpg_image_test.pdf").unwrap();
//!    use std::io::Write;
//!    file.write_all(bytes).unwrap();
//! ```

use crate::BasicPdfWriter;
use crate::page::Page;
use crate::*;
use format_bytes::write_bytes as wb;

/// PDF image specification - byte data and attributes that describe how image is encoded.
pub struct ImageSpec<'a> {
    /// Image data - length is width * height * (bits_per_component/8) * 3 (for RGB).
    pub data: &'a [u8],
    /// Width
    pub width: Px,
    /// Height
    pub height: Px,
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
    pub width: Px,
    /// Height
    pub height: Px,
}

impl Image {
    /// Writes the specified image attributes and data to the PDF, returns Image with obj id, width and height.
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
            width: s.width,
            height: s.height,
        }
    }

    /// Draw image on page.
    pub fn draw(&self, page: &mut Page, x: f32, y: f32, scale: f32) {
        let w = (self.width as f32) * scale;
        let h = (self.height as f32) * scale;
        page.xobjs.insert(self.obj);
        let _ = wb!(
            &mut page.os,
            b"\nq {} 0 0 {} {} {} cm /X{} Do Q",
            w,
            h,
            x,
            y,
            self.obj
        );
    }
}
