#![cfg(feature = "v8_19")]

use vips::*;

fn rgb() -> Result<VipsImage<'static>> {
    VipsImage::from_memory(
        vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255],
        2,
        2,
        3,
        VipsBandFormat::VIPS_FORMAT_UCHAR,
    )
}

#[test]
fn qoi_buffer_and_file_round_trip() -> Result<()> {
    init(None)?;
    let source = rgb()?;
    let encoded = source.write_qoi_buffer()?;
    assert_eq!(&encoded[..4], b"qoif");
    assert_eq!(
        VipsImage::qoiload_buffer(&encoded)?.write_to_memory()?,
        source.write_to_memory()?
    );
    assert!(VipsImage::qoiload_buffer(b"invalid qoi").is_err());
    let file = std::env::temp_dir().join(format!("vips-qoi-{}.qoi", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(file.clone());
    source.write_qoi(&file)?;
    assert_eq!(
        VipsImage::qoiload_file(&file)?.write_to_memory()?,
        source.write_to_memory()?
    );
    Ok(())
}

#[test]
fn cicp_colour_and_luminance() -> Result<()> {
    init(None)?;
    let source = rgb()?.colourspace(VipsInterpretation::VIPS_INTERPRETATION_scRGB)?;
    let primaries = VipsCICPColourPrimaries::VIPS_CICP_COLOUR_PRIMARIES_BT709;
    let transfer = VipsCICPTransferCharacteristics::VIPS_CICP_TRANSFER_SRGB;
    let encoded = source.scrgb2cicp(primaries, transfer, 16)?;
    let restored = encoded.cicp2scrgb()?;
    assert!(restored.subtract(&source)?.abs()?.max_value()? < 0.001);
    assert!(source.scrgb2cicp(primaries, transfer, 12).is_err());
    let threshold = source.percent_lum(100.0, 2.0)?;
    assert!((threshold - 1.0).abs() < 0.01);
    assert!(source.percent_lum(f64::NAN, 2.0).is_err());
    assert!(source.percent_lum(50.0, 0.0).is_err());
    assert!(source.percent_lum(101.0, 2.0).is_err());
    assert!(source.percent_lum(50.0, f64::INFINITY).is_err());
    Ok(())
}

#[test]
fn metadata_blob_is_borrowed_and_loop_is_normalized() -> Result<()> {
    init(None)?;
    let image = rgb()?;
    assert_eq!(image.loop_count(), 0);
    // SAFETY: metadata is changed on a private image before any borrow is made.
    unsafe {
        vips_sys::vips_image_set_int(image.as_ptr(), c"loop".as_ptr(), 3);
        vips_sys::vips_image_set_blob_copy(
            image.as_ptr(),
            c"test-blob".as_ptr(),
            b"hello".as_ptr().cast(),
            5,
        );
    }
    assert_eq!(image.loop_count(), 3);
    let blob = image.blob_value("test-blob")?;
    assert_eq!(blob, b"hello");
    assert_eq!(blob.as_ptr(), image.blob_value("test-blob")?.as_ptr());
    assert!(image.blob_value("missing-blob").is_err());
    assert!(image.blob_value("loop").is_err());
    assert!(image.blob_value("bad\0name").is_err());
    Ok(())
}

#[test]
fn specific_crop_and_thumbnail_evaluate_pixels() -> Result<()> {
    init(None)?;
    let source = rgb()?;
    assert_eq!(
        source.smartcrop_at(1, 1, 1, 1)?.write_to_memory()?,
        [255, 255, 255]
    );
    let encoded = source.write_to_buffer_owned(".png")?;
    let thumbnail = VipsImage::thumbnail_buffer_at(&encoded, 1, 2, 1, 1)?;
    assert_eq!(thumbnail.size(), (1, 2));
    assert_eq!(thumbnail.write_to_memory()?.len(), 6);
    let thumbnail = VipsImage::thumbnail_file_at("examples/images/kodim01.png", 32, 32, 100, 100)?;
    assert_eq!(thumbnail.size(), (32, 32));
    assert_eq!(thumbnail.write_to_memory()?.len(), 32 * 32 * 3);
    Ok(())
}
