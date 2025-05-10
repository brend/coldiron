use coldiron::Image;
use macroquad::prelude::*;

#[macroquad::main("Coldiron")]
async fn main() {
    let mut reader = std::fs::File::open("images/feep.pgm").unwrap();
    let src = Image::read_from(&mut reader).unwrap();

    loop {
        let w = screen_width();
        let h = screen_height();
        let sx = 40.0;
        let ox = (w - sx * (src.width()+1) as f32) / 2.0;
        let oy = (h - sx * (src.height()+1) as f32) / 2.0;

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
