extern crate mini_gl_fb;
extern crate glutin;

use mini_gl_fb::{Config, BufferFormat};
use glutin::event_loop::EventLoop;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "Hello world!",
        window_size: (800.0, 600.0),
        buffer_size: (2, 2),
        .. Default::default()
    }, &event_loop);

    fb.change_buffer_format::<u8>(BufferFormat::RG);

    // This should panic! We should only be providing two components but we provide 4!
    let buffer = vec![[0u8, 50, 128, 255]; 4];
    fb.update_buffer(&buffer);

    fb.persist(&mut event_loop);
}
