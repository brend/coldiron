use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Bitmap,
    Graymap,
    Pixmap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Encoding {
    Ascii,
    Binary,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Color8 {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color8 {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color8 { red, green, blue }
    }
}

enum ImageData {
    Bitmap(Vec<u8>),
    Graymap8(Vec<u8>),
    Pixmap(Vec<Color8>),
}

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub format: Format,
    data: ImageData,
}

impl Image {
    pub fn new(image_type: Format, width: usize, height: usize) -> Image {
        let count = width * height;
        let data = match image_type {
            Format::Bitmap => ImageData::Bitmap(vec![0; count]),
            Format::Graymap => ImageData::Graymap8(vec![0; count]),
            Format::Pixmap => ImageData::Pixmap(vec![Color8::default(); count]),
        };

        Image {
            format: image_type,
            width,
            height,
            data,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        match &mut self.data {
            ImageData::Bitmap(data) => data[y * self.width + x] = value,
            ImageData::Graymap8(data) => data[y * self.width + x] = value,
            ImageData::Pixmap(data) => data[y * self.width + x] = Color8::new(value, value, value),
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W, encoding: Encoding) -> std::io::Result<()> {
        match (&self.data, encoding) {
            (ImageData::Pixmap(data), Encoding::Binary) => {
                write_ppm_binary(self.width, self.height, data, writer)
            }
            (ImageData::Pixmap(data), Encoding::Ascii) => {
                write_ppm_ascii(self.width, self.height, data, writer)
            }
            (ImageData::Graymap8(data), Encoding::Binary) => {
                write_pgm_binary(self.width, self.height, data, writer)
            }
            (ImageData::Graymap8(data), Encoding::Ascii) => {
                write_pgm_ascii(self.width, self.height, data, writer)
            }
            (ImageData::Bitmap(data), Encoding::Binary) => {
                write_pbm_binary(self.width, self.height, data, writer)
            }
            (ImageData::Bitmap(data), Encoding::Ascii) => {
                write_pbm_ascii(self.width, self.height, data, writer)
            }
        }
    }
}

fn write_ppm_binary<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<Color8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P6", width, height, writer, Some(255))?;
    for pixel in data {
        writer.write_all(&[pixel.red, pixel.green, pixel.blue])?;
    }
    Ok(())
}

fn write_ppm_ascii<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<Color8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P3", width, height, writer, Some(255))?;
    for pixel in data {
        write!(writer, "{} {} {} ", pixel.red, pixel.green, pixel.blue)?;
    }
    Ok(())
}

fn write_pgm_binary<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<u8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P5", width, height, writer, Some(255))?;
    writer.write(data)?;
    Ok(())
}

fn write_pgm_ascii<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<u8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P2", width, height, writer, Some(255))?;
    for &value in data {
        write!(writer, "{} ", value)?;
    }
    Ok(())
}

fn write_pbm_binary<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<u8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P4", width, height, writer, None)?;
    writer.write(data)?;
    Ok(())
}

fn write_pbm_ascii<W: Write>(
    width: usize,
    height: usize,
    data: &Vec<u8>,
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P1", width, height, writer, None)?;
    for &value in data {
        write!(writer, "{} ", value)?;
    }
    Ok(())
}

fn write_header<W: Write>(
    magic_number: &str,
    width: usize,
    height: usize,
    writer: &mut W,
    max_value: Option<u16>,
) -> std::io::Result<()> {
    writeln!(writer, "{}", magic_number)?;
    writeln!(writer, "{} {}", width, height)?;
    if let Some(max_value) = max_value {
        writeln!(writer, "{}", max_value)?;
    }
    Ok(())
}
