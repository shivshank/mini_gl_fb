extern crate mini_gl_fb;

use mini_gl_fb::{glutin, GlutinBreakout};

use glutin::GlContext;

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
    let buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
    fb.update_buffer(&buffer);

    let GlutinBreakout {
        mut events_loop,
        gl_window,
        mut fb,
    } = fb.glutin_breakout();

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            use glutin::{Event, ElementState, VirtualKeyCode};
            use glutin::WindowEvent::*;

            match event {
                Event::WindowEvent { event: CloseRequested, .. } => {
                    running = false;
                }
                Event::WindowEvent { event: KeyboardInput { input, .. }, .. } => {
                    if let Some(k) = input.virtual_keycode {
                        if k == VirtualKeyCode::Escape && input.state == ElementState::Released {
                            running = false;
                        }
                    }
                }
                Event::WindowEvent { event: Resized(logical_size), .. } => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                }
                _ => {}
            }
        });

        fb.redraw();
        gl_window.swap_buffers().unwrap();
    }
}
