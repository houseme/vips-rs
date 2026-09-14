use crate::Result;
use crate::VipsImage;

/// Extension trait for thumbnailing encoded image bytes.
///
/// Prefer this over `from_buffer` + [`VipsImage::thumbnail`]: `vips_thumbnail_buffer`
/// combines decode + resize so libvips can shrink-on-load.
pub trait VipsBuffer {
    /// Decode `self` as an encoded image and produce a `width`×`height` thumbnail.
    ///
    /// The returned image may retain a lazy load pipeline that reads from
    /// `self`; keep the buffer alive until the thumbnail is fully consumed.
    ///
    /// # Errors
    /// Returns an error if the buffer is not a valid image or thumbnail fails.
    fn thumbnail(&self, width: u32, height: u32, size: vips_sys::VipsSize)
    -> Result<VipsImage<'_>>;
}

impl VipsBuffer for [u8] {
    fn thumbnail(
        &self,
        width: u32,
        height: u32,
        size: vips_sys::VipsSize,
    ) -> Result<VipsImage<'_>> {
        VipsImage::thumbnail_buffer(self, width, height, size)
    }
}
