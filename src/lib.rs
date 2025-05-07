use std::io::Write;

/// Format of a Netpbm image
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    /// Portable BitMap format (.pbm, black and white)
    Bitmap,
    /// Portable GrayMap format (.pgm, grayscale)
    Graymap,
    /// Portable PixMap format (.ppm, RGB color)
    Pixmap,
}

/// Encoding for writing a Netpbm image
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Encoding {
    /// Color values stored as plain text
    Ascii,
    /// Color values stored as binary values
    Binary,
}

/// RGB color structure with 8 bits per color channel,
/// i.e. 24 bits per pixel
#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Color8 {
    /// Red color component
    red: u8,
    /// Green color component
    green: u8,
    /// Blue color component
    blue: u8,
}

impl Color8 {
    /// Create a new color from red, green, and blue components
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color8 { red, green, blue }
    }
}

/// A data structure to hold the data of Netpbm images
/// in various formats.
enum ImageData {
    /// BitMap data; each bit is stored in one byte
    Bitmap(Vec<u8>),
    /// 8-bit GrayMap data
    Graymap8(Vec<u8>),
    /// 24-bit-per-pixel PixMap data
    Pixmap(Vec<Color8>),
}

/// A Netpbm image
pub struct Image {
    /// Width of the image in pixels
    pub width: usize,
    /// Height of the image in pixels
    pub height: usize,
    /// Format of the image (pbm, pgm or ppm)
    pub format: Format,
    /// Pixel data of the image
    data: ImageData,
}

impl Image {
    /// Create a new image of the given format and dimensions
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

    /// Set the pixel at the given coordinates to a new color
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        match &mut self.data {
            ImageData::Bitmap(data) => data[y * self.width + x] = value,
            ImageData::Graymap8(data) => data[y * self.width + x] = value,
            ImageData::Pixmap(data) => data[y * self.width + x] = Color8::new(value, value, value),
        }
    }

    /// Write the image to a writer
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

/// Write BitMap image data to a writer using binary encoding
fn write_pbm_binary<W: Write>(
    width: usize,
    height: usize,
    data: &[u8],
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P4", width, height, writer, None)?;

    for y in 0..height {
        let mut byte = 0u8;
        let mut bit_count = 0;

        for x in 0..width {
            let index = y * width + x;
            let bit = if data[index] != 0 { 0 } else { 1 }; // 1 is black, 0 is white in PBM
            byte = (byte << 1) | bit;
            bit_count += 1;

            if bit_count == 8 {
                writer.write_all(&[byte])?;
                byte = 0;
                bit_count = 0;
            }
        }

        // If the row's width isn't divisible by 8, pad the last byte with zeros
        if bit_count > 0 {
            byte <<= 8 - bit_count;
            writer.write_all(&[byte])?;
        }
    }

    Ok(())
}

/// Write a Netpbm header to a writer
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

/// Write BitMap image data to a writer using plain text encoding
fn write_pbm_ascii<W: Write>(
    width: usize,
    height: usize,
    data: &[u8],
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P1", width, height, writer, None)?;
    for &value in data {
        let bit = if value == 0 { 1 } else { 0 };
        write!(writer, "{} ", bit)?;
    }
    Ok(())
}

/// Write GrayMap image data to a writer using binary encoding
fn write_pgm_binary<W: Write>(
    width: usize,
    height: usize,
    data: &[u8],
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P5", width, height, writer, Some(255))?;
    _ = writer.write(data)?;
    Ok(())
}

/// Write GrayMap image data to a writer using plain text encoding
fn write_pgm_ascii<W: Write>(
    width: usize,
    height: usize,
    data: &[u8],
    writer: &mut W,
) -> std::io::Result<()> {
    write_header("P2", width, height, writer, Some(255))?;
    for &value in data {
        write!(writer, "{} ", value)?;
    }
    Ok(())
}

/// Write PixMap image data to a writer using binary encoding
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

/// Write PixMap image data to a writer using plain text encoding
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
