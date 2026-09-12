//! Smoke-check APIs aligned with common php-vips methods.
use vips::*;

fn main() -> Result<()> {
    let _instance = VipsInstance::new("phpvips_align", false)?;

    // Build a 64x64 RGB image.
    let pixels = vec![64u8; 64 * 64 * 3];
    let img = VipsImage::from_memory(pixels, 64, 64, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;

    let cropped = img.crop(8, 8, 32, 32)?;
    assert_eq!(cropped.size(), (32, 32));

    let flipped = cropped.fliphor()?.flipver()?;
    assert_eq!(flipped.size(), (32, 32));

    let rotated = flipped.rot90()?;
    assert_eq!(rotated.size(), (32, 32));

    let zoomed = cropped.zoom(2, 2)?;
    assert_eq!(zoomed.size(), (64, 64));

    let band = cropped.extract_band(0)?;
    assert_eq!(band.bands(), 1);

    let joined = band.bandjoin2(&band)?;
    assert_eq!(joined.bands(), 2);

    let with_alpha = cropped.bandjoin_const(&[255.0])?;
    assert_eq!(with_alpha.bands(), 4);

    let bright = cropped.linear1(1.0, 10.0)?;
    let inverted = bright.invert()?;
    let avg = inverted.avg()?;
    assert!(avg.is_finite());

    let blurred = cropped.gaussblur(1.5)?;
    assert_eq!(blurred.size(), (32, 32));

    let cast = cropped.cast(VipsBandFormat::VIPS_FORMAT_USHORT)?;
    assert_eq!(cast.size(), (32, 32));

    let embedded = cropped.embed(4, 4, 40, 40, Some(VipsExtend::VIPS_EXTEND_WHITE))?;
    assert_eq!(embedded.size(), (40, 40));

    let point = cropped.getpoint(0, 0)?;
    assert_eq!(point.len(), 3);

    let jpg = cropped.write_to_buffer(".jpg")?;
    assert!(!jpg.is_empty());

    let png = cropped.write_to_buffer(".png")?;
    assert!(!png.is_empty());

    let loader = find_load("./examples/images/kodim01.png");
    assert!(loader.is_some());

    println!(
        "aligned ops ok: crop={:?} avg={avg:.3} loader={loader:?} jpg={} png={}",
        cropped.size(),
        jpg.len(),
        png.len()
    );
    Ok(())
}
