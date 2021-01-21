#[macro_use]
extern crate mini_gl_fb;

use mini_gl_fb::BufferFormat;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::dpi::LogicalSize;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut fb = mini_gl_fb::get_fancy(config! {
        window_title: String::from("Hello world!"),
        window_size: LogicalSize::new(800.0, 600.0),
        buffer_size: Some(LogicalSize::new(2, 2))
    }, &event_loop);

    fb.change_buffer_format::<u8>(BufferFormat::RG);

    // This should panic! We should only be providing two components but we provide 4!
    let buffer = vec![[0u8, 50, 128, 255]; 4];
    fb.update_buffer(&buffer);

    fb.persist(&mut event_loop);
}
