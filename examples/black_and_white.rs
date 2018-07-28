extern crate mini_gl_fb;

use mini_gl_fb::{Config, BufferFormat};

fn main() {
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "Hello world!",
        window_size: (800.0, 600.0),
        buffer_size: (2, 2),
        .. Default::default()
    });

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_grayscale_shader();

    let buffer = [128u8, 255, 50, 25];
    fb.update_buffer(&buffer);

    fb.persist();
}
