//! This crate implements minimal conversion from HTML to PDF.
//!
//! ToDo:
//! Proper parsing of tag attibutes.
//! Font sizing, html tables.
//! A whole lot more.

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
//!<p>Hello <b>something</p>
//!<p>Hi <i>italic test</i>
//!<p>Hi <i><b>bold italic test</b></i>
//!<p>Test <sup>sup</sup>
//!<p>Test <sub>sub</sub>
//!<p>{}
//!</body>
//!</html>
//!","Some words to cause Line and Page wrapping ".repeat(200));
//!    let mut w = Writer::default();
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

use basic::*;
use font::*;
use page::*;
pub use writer::{html, Writer};
