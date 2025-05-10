use coldiron::{Format, Image, Kernel};
use macroquad::prelude::*;

#[macroquad::main("Coldiron Netpbm Viewer")]
async fn main() {
    let mut reader = std::fs::File::open("images/lightning.pgm").unwrap();
    let mut src = Image::read_from(&mut reader).unwrap();

    let mut dst = Image::new(Format::Graymap, src.width(), src.height());
    let kernel = Kernel::new(
        3,
        vec![
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
            1.0 / 9.0,
        ],
    );
    kernel.apply(&src, &mut dst);
    kernel.apply(&dst, &mut src);

    loop {
        let w = screen_width();
        let h = screen_height();
        let border = 0.0;
        let sx = w / src.width() as f32;
        let sy = h / src.height() as f32;
        let sx = if sx > sy { sy } else { sx };
        let ox = (w - border - sx * (src.width() + 1) as f32) / 2.0;
        let oy = (h - border - sx * (src.height() + 1) as f32) / 2.0;

        for y in 0..src.height() {
            for x in 0..src.width() {
                let p = src.get_pixel(x, y);
                let color = Color::from_rgba(p, p, p, 255);
                draw_rectangle(ox + x as f32 * sx, oy + y as f32 * sx, sx, sx, color);
            }
        }

        next_frame().await;
    }
}
