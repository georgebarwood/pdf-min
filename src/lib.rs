//! This crate implements minimal conversion from HTML to PDF.
//!
//! ToDo:
//! Proper parsing of tag attibutes.
//! Font sizing, html tables.
//! Img tag in html, with support for jpegs ( using new image module ).

//!# Test example
//!
//! ```
//!    use pdf_min::*;
//!    let source = format!("
//!<html>
//!<head>
//!   <title>Rust is Great</title>
//!</head>
//!<body>
//!<h1>Important Notice&excl;</h1>
//!<p>Hello <b>something bold</b> ok</p>
//!<p>Hi <i>italic test</i>
//!<p>Hi <i><b>bold italic test</b> ok</i>
//!<p>Hi <sup>sup test</sup> ok
//!<p>Hi <sub>sub text</sub> ok
//!<p>{}
//!</body>
//!</html>
//!","Some words to cause Line and Page wrapping ".repeat(200));
//!    let mut w = Writer::default();
//!    w.b.nocomp = true;
//!    w.line_pad = 8; // Other Writer default values could be adjusted here.
//!    html(&mut w, source.as_bytes());
//!    w.finish();
//!
//!    use std::fs::File;
//!    use std::io::prelude::*;
//!
//!    let mut file = File::create("test.pdf").unwrap();
//!    file.write_all(&w.b.b).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Low level PDF writer.
pub mod basic;
/// PDF fonts.
pub mod font;
/// PDF page.
pub mod page;
/// High level PDF writer.
pub mod writer;
/// PDF images.
pub mod image;

use basic::*;
use font::*;
use page::*;
pub use writer::{html, Writer};

#[test]
fn image_test()
{
   use crate::image::Image;
   
   let mut w = Writer::default();
   w.b.nocomp = true;
   
   let mut data = Vec::new();
   for i in 0..3 * 16 * 16 { 
      data.push( i as u8 );  // Red
      data.push( ( i + 85 ) as u8 ); // Green
      data.push( ( i + 85 + 85 ) as u8 ); // Blue
   }
   
   let mut im = Image{ obj:0, data: &data, width:16, height:16, bits_per_component:8, color_space: b"/DeviceRGB", other: b"" };

   // Write the image to the PDF.
   im.init(&mut w.b);

   // Draw some text on the current page.
   html( &mut w, b"<p>Hello <b>there</b><p>Hello <i>again</i>" );

   // Draw a rectangle on the current page.
   w.p.rect( 100.0, 200.0, 160.0, 100.0 );

   // Draw the image on the current page.
   im.draw( &mut w.p, 100.0, 300.0, 10.0 );

   // Flush text
   w.output_line();

   // Finish the page
   w.save_page();

   // Draw some more text on the next page.
   html( &mut w, b"<p>Some <b>more</b> text" );

   w.finish();

   use std::fs::File;
   use std::io::prelude::*;

   let mut file = File::create("image_test.pdf").unwrap();
   file.write_all(&w.b.b).unwrap();
}

#[test]
fn image_jpg_test()
{
  // ToDo....
}