use std::io::BufWriter;

use coldiron::{Encoding, Format, Image, Kernel};

fn main() {
    let mut reader = std::fs::File::open("images/lightning.pgm").unwrap();
    let src = Image::read_from(&mut reader).unwrap();

    let file = std::fs::File::create("src.pgm").expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    src.write_to(&mut writer, Encoding::Ascii)
        .expect("Failed to write image to file");

    let kernel = Kernel::new(3, vec![0.0, 0.125, 0.0, 0.125, 0.5, 0.125, 0.0, 0.125, 0.0]);
    let mut dst = Image::new(Format::Graymap, 128, 128);

    kernel.apply(&src, &mut dst);

    let file = std::fs::File::create("dst.pgm").expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    dst.write_to(&mut writer, Encoding::Binary)
        .expect("Failed to write image to file");
}

// fn create_test_image(format: Format) -> Image {
//     let mut image = Image::new(format, 300, 200);
//     for i in 0..image.height {
//         for j in 0..image.width {
//             image.set_pixel(j, i, ((j as f32 / 10.0).sin() * 128.0) as u8);
//         }
//     }
//     image
// }
