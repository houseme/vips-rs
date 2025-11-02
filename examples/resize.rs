use vips::*;

fn resize_file() {
    let img: VipsImage = VipsImage::from_file("./examples/images/kodim01.png").unwrap();
    let thumbnail = img.thumbnail(123, 123, VipsSize::VIPS_SIZE_FORCE).unwrap();
    thumbnail.write_to_file("kodim01_123x234.png").unwrap();
}

fn resize_mem() {
    let pixels = vec![0; 256 * 256 * 3];
    let img: VipsImage =
        VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR).unwrap();
    let thumbnail = img.thumbnail(234, 123, VipsSize::VIPS_SIZE_FORCE).unwrap();
    thumbnail.write_to_file("black_mem_234_123.png").unwrap();
}

fn resize_mem_ref() {
    let pixels = vec![0; 256 * 256 * 3];
    let img: VipsImage =
        VipsImage::from_memory_reference(&pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)
            .unwrap();
    let thumbnail = img.thumbnail(234, 123, VipsSize::VIPS_SIZE_FORCE).unwrap();
    thumbnail.write_to_file("black_ref_234x123.png").unwrap();
}

fn main() {
    let _instance = VipsInstance::new("app_test", true).unwrap();
    resize_file();
    resize_mem();
    resize_mem_ref();
}
