use std::io::BufWriter;

use coldiron::{Encoding, Format, Image};

fn main() {
    for format in &[Format::Bitmap, Format::Graymap, Format::Pixmap] {
        for encoding in &[Encoding::Ascii, Encoding::Binary] {
            let image = create_test_image(*format);
            let extension = match format {
                Format::Bitmap => "pbm",
                Format::Graymap => "pgm",
                Format::Pixmap => "ppm",
            };
            let file =
                std::fs::File::create(format!("output_{:?}_{:?}.{}", format, encoding, extension))
                    .expect("Failed to create file");
            let mut writer = BufWriter::new(file);
            image
                .write_to(&mut writer, *encoding)
                .expect("Failed to write image to file");
        }
    }
}

fn create_test_image(format: Format) -> Image {
    let mut image = Image::new(format, 300, 200);
    for i in 0..image.height {
        for j in 0..image.width {
            image.set_pixel(j, i, ((j as f32 / 10.0).sin() * 128.0) as u8);
        }
    }
    image
}
