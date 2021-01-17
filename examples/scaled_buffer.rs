extern crate mini_gl_fb;

use mini_gl_fb::Config;
use mini_gl_fb::glutin::event_loop::EventLoop;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut fb = mini_gl_fb::get_fancy(Config {
        window_title: "Hello world!",
        window_size: (800.0, 600.0),
        buffer_size: (2, 2),
        .. Default::default()
    }, &event_loop);

    let mut buffer = vec![[128u8, 0, 0, 255]; 4];
    buffer[3] = [255, 255, 255, 255];

    fb.update_buffer(&buffer);

    fb.persist(&mut event_loop);
}
