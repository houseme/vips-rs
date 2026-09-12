//! Smoke-check the high-level `vips` operation wrappers.
use vips::*;

fn main() -> Result<()> {
    let _instance = VipsInstance::new("ops_smoke", false)?;

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

    // Convenience helpers
    let mask = cropped.less(200.0)?;
    assert_eq!(mask.size(), (32, 32));
    let selected = mask
        .bandand()?
        .ifthenelse(&cropped, &cropped.linear1(0.5, 0.0)?)?;
    assert_eq!(selected.size(), (32, 32));

    let floored = cropped.floor()?.ceil()?.rint()?;
    assert_eq!(floored.size(), (32, 32));

    let med = cropped.median(3)?;
    assert_eq!(med.size(), (32, 32));

    let edges = cropped.sobel()?;
    assert_eq!(edges.size(), (32, 32));

    let black_img = black(8, 8)?;
    assert_eq!(black_img.size(), (8, 8));

    let coord = xyz(4, 4)?;
    assert_eq!(coord.bands(), 2);

    let split = cropped.bandsplit()?;
    assert_eq!(split.len(), 3);

    let d = cropped.deviate()?;
    assert!(d.is_finite());

    // Additional geometry / hist / create
    let tiled = cropped.replicate(2, 2)?;
    assert_eq!(tiled.size(), (64, 64));
    let wrapped = tiled.wrap()?;
    assert_eq!(wrapped.size(), (64, 64));
    let gamma = cropped.gamma(None)?;
    assert_eq!(gamma.size(), (32, 32));
    let hist = band.hist_find()?;
    assert!(hist.bands() >= 1);
    let grey_img = grey(8, 8)?;
    assert_eq!(grey_img.size(), (8, 8));

    println!(
        "ops ok: crop={:?} avg={avg:.3} deviate={d:.3} loader={loader:?} jpg={} png={}",
        cropped.size(),
        jpg.len(),
        png.len()
    );
    Ok(())
}
