extern crate mini_gl_fb;

use mini_gl_fb::{Config, BufferFormat};
use mini_gl_fb::glutin::event_loop::EventLoop;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "Hello world!",
        window_size: (800.0, 600.0),
        buffer_size: (2, 2),
        .. Default::default()
    }, &event_loop);

    fb.change_buffer_format::<u8>(BufferFormat::R);
    fb.use_grayscale_shader();

    let buffer = [128u8, 255, 50, 25];
    fb.update_buffer(&buffer);

    fb.persist(&mut event_loop);
}
