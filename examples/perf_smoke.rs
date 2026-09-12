use std::time::Instant;
use vips::*;

fn main() -> Result<()> {
    let _instance = VipsInstance::new("perf_smoke", false)?;

    let img = VipsImage::from_file("./examples/images/kodim01.png")?;
    let (w, h) = img.size();
    assert!(w > 0 && h > 0 && img.bands() > 0);

    // Path API + large downscale via pre-shrink
    let t0 = Instant::now();
    let thumb = img.resize_reduce(0.05, None, None)?;
    let d_reduce = t0.elapsed();
    let (tw, th) = thumb.size();
    assert!(tw > 0 && th > 0 && tw < w && th < h);

    let t1 = Instant::now();
    let _plain = img.resize(0.05, None, None)?;
    let d_plain = t1.elapsed();

    // Owned memory round-trip
    let pixels = vec![128u8; 64 * 64 * 3];
    let mem = VipsImage::from_memory(pixels, 64, 64, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
    let mem_thumb = mem.thumbnail(32, 32, VipsSize::VIPS_SIZE_FORCE)?;
    let bytes = mem_thumb.write_to_memory()?;
    assert!(!bytes.is_empty());

    mem_thumb.write_to_file("/tmp/perf_smoke.png")?;
    mem_thumb.write_jpeg("/tmp/perf_smoke.jpg", Some(80))?;

    println!(
        "size={w}x{h} thumb={tw}x{th} reduce={d_reduce:?} plain={d_plain:?} mem_bytes={}",
        bytes.len()
    );
    println!("concurrency={} version={}", concurrency(), version_string());
    Ok(())
}
