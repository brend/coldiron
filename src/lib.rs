use std::io::{self, BufRead, BufReader, Read, Write};

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

impl Format {
    pub fn from_magic_number(magic_number: &str) -> Option<Self> {
        match magic_number {
            "P1" | "P4" => Some(Format::Bitmap),
            "P2" | "P5" => Some(Format::Graymap),
            "P3" | "P6" => Some(Format::Pixmap),
            _ => None,
        }
    }
}

/// Encoding for writing a Netpbm image
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Encoding {
    /// Color values stored as plain text
    Ascii,
    /// Color values stored as binary values
    Binary,
}

impl Encoding {
    pub fn from_magic_number(magic_number: &str) -> Option<Encoding> {
        match magic_number {
            "P1" | "P3" | "P5" => Some(Encoding::Ascii),
            "P2" | "P4" | "P6" => Some(Encoding::Binary),
            _ => None,
        }
    }
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

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Set the pixel at the given coordinates to a new color
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        match &mut self.data {
            ImageData::Bitmap(data) => data[y * self.width + x] = value,
            ImageData::Graymap8(data) => data[y * self.width + x] = value,
            ImageData::Pixmap(data) => data[y * self.width + x] = Color8::new(value, value, value),
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        match &self.data {
            ImageData::Bitmap(data) => data[y * self.width + x],
            ImageData::Graymap8(data) => data[y * self.width + x],
            ImageData::Pixmap(_) => unimplemented!(),
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

    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut reader = BufReader::new(reader);
        let (magic_number, width, height, max_value) = read_header(&mut reader)?;
        let format = Format::from_magic_number(&magic_number).unwrap();
        let encoding = Encoding::from_magic_number(&magic_number).unwrap();
        let data = match (format, encoding) {
            (Format::Bitmap, Encoding::Ascii) => read_pbm_ascii(&mut reader, width, height),
            (Format::Bitmap, Encoding::Binary) => read_pbm_binary(&mut reader, width, height),
            (Format::Graymap, Encoding::Ascii) => {
                read_pgm_ascii(&mut reader, width, height, max_value)
            }
            (Format::Graymap, Encoding::Binary) => {
                read_pgm_binary(&mut reader, width, height, max_value)
            }
            (Format::Pixmap, Encoding::Ascii) => {
                read_ppm_ascii(&mut reader, width, height, max_value)
            }
            (Format::Pixmap, Encoding::Binary) => {
                read_ppm_binary(&mut reader, width, height, max_value)
            }
        }?;

        Ok(Image {
            format,
            width,
            height,
            data,
        })
    }
}

fn read_pbm_ascii<R: Read>(
    reader: &mut BufReader<R>,
    width: usize,
    height: usize,
) -> io::Result<ImageData> {
    let byte_count = height * width;
    let mut bytes = Vec::with_capacity(byte_count);
    let mut line = String::new();

    let mut next_line = || -> io::Result<String> {
        line.clear();
        loop {
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Ok(trimmed.to_string());
            }
            line.clear();
        }
    };

    while bytes.len() < byte_count {
        let line = next_line()?;
        for token in line.chars() {
            if token.is_whitespace() {
                continue;
            }
            let b = if token == '0' { 0 } else { 1 };
            bytes.push(b);
        }
    }

    Ok(ImageData::Bitmap(bytes))
}

fn read_pbm_binary<R: Read>(
    reader: &mut BufReader<R>,
    width: usize,
    height: usize,
) -> io::Result<ImageData> {
    let byte_count = height * width.div_ceil(8);
    let mut buf = vec![0u8; byte_count];
    reader.read_exact(&mut buf)?;
    // the bitmap is packed into bytes we need to unpack
    let mut bytes = vec![0u8; height * width];
    for b in buf {
        for i in 0..8 {
            let bit = if b & (1 << i) == 0 { 0 } else { 1 };
            bytes.push(bit)
        }
    }
    Ok(ImageData::Bitmap(bytes))
}

fn read_pgm_ascii<R: Read>(
    reader: &mut BufReader<R>,
    width: usize,
    height: usize,
    max_value: Option<u16>,
) -> io::Result<ImageData> {
    let byte_count = height * width;
    let mut bytes = Vec::with_capacity(byte_count);
    let mut line = String::new();

    let mut next_line = || -> io::Result<String> {
        line.clear();
        loop {
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Ok(trimmed.to_string());
            }
            line.clear();
        }
    };

    while bytes.len() < byte_count {
        let line = next_line()?;
        for token in line.split_whitespace() {
            let value: u16 = token.parse().expect("Invalid color value");
            let b = match max_value {
                Some(max_value) => (value as f32 / max_value as f32) as u8,
                None => value as u8,
            };
            bytes.push(b);
        }
    }

    Ok(ImageData::Graymap8(bytes))
}

fn read_pgm_binary<R: Read>(
    reader: &mut R,
    width: usize,
    height: usize,
    max_value: Option<u16>,
) -> io::Result<ImageData> {
    let size = match max_value {
        Some(max_value) if max_value >= 256 => 2,
        _ => 1,
    };
    if size != 1 {
        unimplemented!();
    }
    let byte_count = height * width * size;
    let mut buf = vec![0u8; byte_count];
    reader.read_exact(&mut buf)?;
    Ok(ImageData::Bitmap(buf))
}

fn read_ppm_ascii<R: Read>(
    reader: &mut BufReader<R>,
    width: usize,
    height: usize,
    max_value: Option<u16>,
) -> io::Result<ImageData> {
    let pixel_count = height * width;
    let mut pixels = Vec::with_capacity(pixel_count);
    let mut line = String::new();

    let mut next_line = || -> io::Result<String> {
        line.clear();
        loop {
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Ok(trimmed.to_string());
            }
            line.clear();
        }
    };

    let mut rgb_index = 0;
    let mut rgb = [0, 0, 0];
    while pixels.len() < pixel_count {
        let line = next_line()?;
        for token in line.split_whitespace() {
            let value: u16 = token.parse().expect("Invalid color value");
            let b = match max_value {
                Some(max_value) => (value as f32 / max_value as f32) as u8,
                None => value as u8,
            };
            rgb[rgb_index] = b;
            rgb_index += 1;
            if rgb_index == 3 {
                pixels.push(Color8::new(rgb[0], rgb[1], rgb[2]));
            }
        }
    }

    Ok(ImageData::Pixmap(pixels))
}

fn read_ppm_binary<R: Read>(
    reader: &mut R,
    width: usize,
    height: usize,
    max_value: Option<u16>,
) -> io::Result<ImageData> {
    let size = match max_value {
        Some(max_value) if max_value >= 256 => 2,
        _ => 1,
    };
    if size != 1 {
        unimplemented!();
    }
    let byte_count = height * width * size;
    let mut buf = vec![0u8; byte_count];
    reader.read_exact(&mut buf)?;
    Ok(ImageData::Bitmap(buf))
}

fn read_header<R: Read>(
    reader: &mut BufReader<R>,
) -> io::Result<(String, usize, usize, Option<u16>)> {
    let mut line = String::new();

    // Helper to read the next non-comment, non-empty line
    let mut next_line = || -> io::Result<String> {
        line.clear();
        loop {
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Ok(trimmed.to_string());
            }
            line.clear();
        }
    };

    // Read magic number
    let magic = next_line()?;

    // Read width and height (might be on same line or separate lines)
    let mut dimensions = Vec::new();
    while dimensions.len() < 2 {
        let text = next_line()?;
        let tokens: Vec<_> = text.split_whitespace().collect();
        for tok in tokens {
            if let Ok(n) = tok.parse::<usize>() {
                dimensions.push(n);
                if dimensions.len() == 2 {
                    break;
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid dimension value",
                ));
            }
        }
    }

    let width = dimensions[0];
    let height = dimensions[1];

    // For binary formats like P5 or P6, there is a maxval line before the pixel data
    let maxval = match magic.as_str() {
        "P2" | "P3" | "P5" | "P6" => {
            let val = next_line()?
                .parse::<u16>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid maxval"))?;
            Some(val)
        }
        _ => None, // e.g., PBM formats like P1, P4 don't use a maxval
    };

    Ok((magic, width, height, maxval))
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

pub struct Kernel {
    size: usize,
    weights: Vec<f32>,
}

impl Kernel {
    pub fn new(size: usize, weights: Vec<f32>) -> Kernel {
        assert!(weights.len() == size * size);
        Kernel { size, weights }
    }

    pub fn apply(&self, src: &Image, dst: &mut Image) {
        assert!(src.width() == dst.width());
        assert!(src.height() == dst.height());
        let k = self.size;
        // for y in 0..src.height() {
        //     for x in 0..src.width() {
        for y in 1..src.height() - 1 {
            for x in 1..src.width() - 1 {
                let mut value = 0.0;
                for j in 0..k {
                    for i in 0..k {
                        value += src.get_pixel(x + i - k / 2, y + j - k / 2) as f32
                            * self.weights[j * k + i];
                    }
                }
                dst.set_pixel(x, y, value as u8);
            }
        }
    }
}
