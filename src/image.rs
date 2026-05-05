use crate::BasicPdfWriter;
use crate::page::Page;
use format_bytes::write_bytes as wb;

/// PDF image - byte data and attributes that describe how image is encoded.
pub struct Image<'a> {
    /// obj id
    pub obj: usize,
    /// Image data - size is width * height * (bits_per_component/8) * 3 (for RGB).
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

impl<'a> Image<'a>
{
    /// Set the obj number, write the image attributes and data to the PDF.
    pub fn init(&mut self, w: &mut BasicPdfWriter) {
        self.obj = w.begin();
        let _ = wb!(
            &mut w.b,
            b"<</Type/XObject/Subtype/Image/Width {}/Height {}/ColorSpace{}/BitsPerComponent {}/Length {}{}>>stream\n",
            self.width, self.height, self.color_space, self.bits_per_component, self.data.len(), self.other
        );
        w.b.extend_from_slice(&self.data);
        w.b.extend_from_slice(b"\nendstream");
        w.end();
    }

    /// Draw image on page.
    pub fn draw(&self, page: &mut Page, x: f32, y: f32, scale:f32)
    {
       let obj = self.obj;
       let w = (self.width as f32) * scale;
       let h = (self.height as f32) * scale;
       page.xobjs.insert(obj);
       let _ = wb!(&mut page.os, b"\nq {} 0 0 {} {} {} cm /X{} Do Q", w,h,x,y,obj);
    }
}
