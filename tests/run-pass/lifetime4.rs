use vips::VipsBandFormat;
use vips::VipsImage;
use vips::VipsInstance;
use vips::VipsSize;

fn main() {
    let _instance = VipsInstance::new("lifetime_test", true).unwrap();
    let pixels = vec![0; 256 * 256 * 3];

    let img: VipsImage =
        VipsImage::from_memory_reference(&pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)
            .unwrap();
    let _thumbnail: VipsImage = img.thumbnail(234, 123, VipsSize::VIPS_SIZE_FORCE).unwrap();
}
