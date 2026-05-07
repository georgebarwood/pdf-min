//! This crate implements minimal conversion from HTML to PDF.
//!
//! ToDo:
//! Maybe html tables.
//! Proper calculation of char display widths ( for text wrapping, image positioning )

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
//!    let bytes = w.finish();
//!
//!    use std::fs::File;
//!    use std::io::prelude::*;
//!
//!    let mut file = File::create("test.pdf").unwrap();
//!    file.write_all(bytes).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Low level PDF writer.
pub mod basic;
/// PDF fonts.
pub mod font;
/// PDF images.
pub mod image;
/// PDF page.
pub mod page;
/// High level PDF writer.
pub mod writer;
/// Conversion from HTML to PDF.
pub mod html;
/// Character sizes for standard fonts.
pub mod metric;

pub use writer::Writer;
pub use html::html;

use basic::*;
use font::*;
use image::*;
use page::*;
use writer::*;

/// Page unit
pub type Px = i32;
/// 1/1000 of a page unit (for text width calculations)
pub type MPx = i64; 
