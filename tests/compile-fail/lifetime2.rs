use vips::VipsBandFormat;
use vips::VipsImage;
use vips::VipsInstance;
use vips::VipsSize;

fn main() {
    let _instance = VipsInstance::new("lifetime_test", true).unwrap();

    // Only assert the borrow life cycle error triggered by the escape of the thumbnail return value to avoid being "short-circuited" by upstream errors.
    let _thumbnail: VipsImage = {
        let pixels = vec![0; 256 * 256 * 3];
        // Do not explicitly mark the type, avoid triggering E0597 here first, and ensure that the error occurs at the return expression
        let img = VipsImage::from_memory_reference(
            &pixels,
            256,
            256,
            3,
            VipsBandFormat::VIPS_FORMAT_UCHAR,
        )
        .unwrap();
        img.thumbnail(234, 123, VipsSize::VIPS_SIZE_FORCE).unwrap() //~ ERROR E0597
    };
}
