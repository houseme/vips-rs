use vips::VipsBandFormat;
use vips::VipsImage;
use vips::VipsInstance;
use vips::VipsSize;

fn main() {
    let _instance = VipsInstance::new("lifetime_test", true).unwrap();

    // Assert only borrow lifecycle errors
    let _thumbnail: VipsImage = {
        let pixels = vec![0; 256 * 256 * 3];
        let img = VipsImage::from_memory_reference(
            &pixels, //~ ERROR E0597
            256,
            256,
            3,
            VipsBandFormat::VIPS_FORMAT_UCHAR,
        )
        .unwrap();
        img.thumbnail(234, 123, VipsSize::VIPS_SIZE_FORCE).unwrap() //~ ERROR E0597
    };
}
