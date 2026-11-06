//! Opt-in wrappers for libvips 8.19, initially validated with 8.19.0-rc1.
use super::*;

impl<'a> VipsImage<'a> {
    /// Load a Quite OK Image (QOI) file. Requires native QOI support.
    pub fn qoiload_file(path: impl AsRef<Path>) -> Result<VipsImage<'static>> {
        let path = path_to_cstring(path.as_ref())?;
        let mut out = null_mut();
        // SAFETY: valid path and output pointer; varargs end in NULL.
        let rc =
            unsafe { vips_sys::vips_qoiload(path.as_ptr(), &mut out, null() as *const c_char) };
        result_with_ret(out, rc)
    }

    /// Decode QOI bytes, retaining the input lifetime throughout lazy evaluation.
    pub fn qoiload_buffer(buf: &'a [u8]) -> Result<VipsImage<'a>> {
        let mut out = null_mut();
        // SAFETY: libvips reads the buffer; the returned image retains its lifetime.
        let rc = unsafe {
            vips_sys::vips_qoiload_buffer(
                buf.as_ptr().cast_mut().cast(),
                buf.len(),
                &mut out,
                null() as *const c_char,
            )
        };
        result_with_ret(out, rc)
    }

    /// Save a QOI file using the native encoder.
    pub fn write_qoi(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        // SAFETY: live image, valid path and NULL-terminated varargs.
        ffi::ret_to_result(unsafe {
            vips_sys::vips_qoisave(self.as_ptr(), path.as_ptr(), null() as *const c_char)
        })
    }

    /// Encode QOI into native-owned storage, avoiding an additional Rust copy.
    pub fn write_qoi_buffer(&self) -> Result<crate::VipsOwnedBuffer> {
        let mut ptr = null_mut();
        let mut len = 0;
        // SAFETY: valid image and outputs; returned storage is released by g_free.
        let rc = unsafe {
            vips_sys::vips_qoisave_buffer(
                self.as_ptr(),
                &mut ptr,
                &mut len,
                null() as *const c_char,
            )
        };
        // SAFETY: adopt the output before propagating a native error.
        let buffer = unsafe { crate::VipsOwnedBuffer::from_raw(ptr, len) }?;
        ffi::ret_to_result(rc)?;
        Ok(buffer)
    }

    /// Convert CICP-tagged pixels to linear scRGB using their colour metadata.
    pub fn cicp2scrgb(&self) -> Result<VipsImage<'a>> {
        let mut out = null_mut();
        // SAFETY: live input, valid output, terminated varargs.
        let rc =
            unsafe { vips_sys::vips_CICP2scRGB(self.as_ptr(), &mut out, null() as *const c_char) };
        result_with_ret(out, rc)
    }

    /// Encode scRGB as full-range RGB CICP with the requested primaries and transfer.
    /// `depth` must be 8 or 16. Conversion may clip out-of-gamut values.
    pub fn scrgb2cicp(
        &self,
        primaries: vips_sys::VipsCICPColourPrimaries,
        transfer: vips_sys::VipsCICPTransferCharacteristics,
        depth: i32,
    ) -> Result<VipsImage<'a>> {
        if !matches!(depth, 8 | 16) {
            return Err(Error::Other("CICP depth must be 8 or 16".into()));
        }
        let mut out = null_mut();
        // SAFETY: enum arguments use C int promotion; depth has been validated.
        let rc = unsafe {
            vips_sys::vips_scRGB2CICP(
                self.as_ptr(),
                &mut out,
                c"colour_primaries".as_ptr(),
                primaries as c_int,
                c"transfer_characteristics".as_ptr(),
                transfer as c_int,
                c"depth".as_ptr(),
                depth,
                null() as *const c_char,
            )
        };
        result_with_ret(out, rc)
    }

    /// BT.709 luminance percentile for a linear RGB float image (at least 3 bands).
    /// The histogram spans `0..=max`, in input units. scRGB uses 1.0 = 80 nits.
    pub fn percent_lum(&self, percent: f64, max: f64) -> Result<f64> {
        if !(0.0..=100.0).contains(&percent) || !(1e-9..=1e9).contains(&max) {
            return Err(Error::Other(
                "invalid luminance percentile or histogram maximum".into(),
            ));
        }
        let mut threshold = 0.0;
        // SAFETY: live image and double output, finite validated scalar arguments.
        let rc = unsafe {
            vips_sys::vips_percent_lum(
                self.as_ptr(),
                percent,
                &mut threshold,
                c"max".as_ptr(),
                max,
                null() as *const c_char,
            )
        };
        ffi::ret_to_result(rc)?;
        Ok(threshold)
    }

    /// Animation loop count, with libvips' legacy GIF handling and fallback to zero.
    pub fn loop_count(&self) -> i32 {
        // SAFETY: accessor borrows a live image.
        unsafe { vips_sys::vips_image_get_loop(self.as_ptr()) }
    }

    /// Borrow blob metadata without copying it. The bytes remain tied to this image.
    /// Missing fields and fields with a different metadata type return an error.
    pub fn blob_value(&self, name: &str) -> Result<&[u8]> {
        let name =
            CString::new(name).map_err(|_| Error::Other("metadata name contains NUL".into()))?;
        let mut blob = null_mut();
        // SAFETY: getter returns a borrowed blob owned by the image.
        ffi::ret_to_result(unsafe {
            vips_sys::vips_image_get_blob_value(self.as_ptr(), name.as_ptr(), &mut blob)
        })?;
        if blob.is_null() {
            return Err(Error::Other("libvips returned a null metadata blob".into()));
        }
        let mut len = 0;
        // SAFETY: live borrowed blob and valid length output.
        let data = unsafe { vips_sys::vips_blob_get(blob, &mut len) }.cast::<u8>();
        if len > isize::MAX as usize || (len > 0 && data.is_null()) {
            return Err(Error::Other(
                "invalid metadata blob length or pointer".into(),
            ));
        }
        if len == 0 {
            return Ok(&[]);
        }
        // SAFETY: immutable blob storage remains alive for the borrow of self.
        Ok(unsafe { std::slice::from_raw_parts(data, len) })
    }

    /// Crop around a specified input-pixel point of interest.
    pub fn smartcrop_at(&self, width: i32, height: i32, x: i32, y: i32) -> Result<VipsImage<'a>> {
        let mut out = null_mut();
        // SAFETY: native code validates dimensions and coordinates; varargs match C types.
        let rc = unsafe {
            vips_sys::vips_smartcrop(
                self.as_ptr(),
                &mut out,
                width,
                height,
                c"interesting".as_ptr(),
                VipsInteresting::VIPS_INTERESTING_SPECIFIC as c_int,
                c"interesting_x".as_ptr(),
                x,
                c"interesting_y".as_ptr(),
                y,
                null() as *const c_char,
            )
        };
        result_with_ret(out, rc)
    }

    /// Load and crop a thumbnail around an input-pixel point, using shrink-on-load.
    pub fn thumbnail_file_at(
        path: impl AsRef<Path>,
        width: i32,
        height: i32,
        x: i32,
        y: i32,
    ) -> Result<VipsImage<'static>> {
        let path = path_to_cstring(path.as_ref())?;
        let mut out = null_mut();
        // SAFETY: valid path/output; native loader validates dimensions and coordinates.
        let rc = unsafe {
            vips_sys::vips_thumbnail(
                path.as_ptr(),
                &mut out,
                width,
                c"height".as_ptr(),
                height,
                c"crop".as_ptr(),
                VipsInteresting::VIPS_INTERESTING_SPECIFIC as c_int,
                c"interesting_x".as_ptr(),
                x,
                c"interesting_y".as_ptr(),
                y,
                null() as *const c_char,
            )
        };
        result_with_ret(out, rc)
    }

    /// Decode and crop a thumbnail around an input-pixel point, using shrink-on-load.
    pub fn thumbnail_buffer_at(
        buf: &'a [u8],
        width: i32,
        height: i32,
        x: i32,
        y: i32,
    ) -> Result<VipsImage<'a>> {
        let mut out = null_mut();
        // SAFETY: the result retains buf's lifetime; varargs match C types.
        let rc = unsafe {
            vips_sys::vips_thumbnail_buffer(
                buf.as_ptr().cast_mut().cast(),
                buf.len(),
                &mut out,
                width,
                c"height".as_ptr(),
                height,
                c"crop".as_ptr(),
                VipsInteresting::VIPS_INTERESTING_SPECIFIC as c_int,
                c"interesting_x".as_ptr(),
                x,
                c"interesting_y".as_ptr(),
                y,
                null() as *const c_char,
            )
        };
        result_with_ret(out, rc)
    }
}
