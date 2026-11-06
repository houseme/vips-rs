use vips::VipsImage;

fn main() {
    let image;
    {
        let bytes = vec![0; 32];
        image = VipsImage::qoiload_buffer(&bytes).unwrap();
    }
    let _ = image.write_to_memory();
}
