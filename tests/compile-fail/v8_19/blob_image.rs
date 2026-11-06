use vips::black;

fn main() {
    let bytes;
    {
        let image = black(1, 1).unwrap();
        bytes = image.blob_value("blob").unwrap();
    }
    println!("{bytes:?}");
}
