extern crate mini_gl_fb;

use mini_gl_fb::{glutin, GlutinBreakout};

use glutin::GlContext;

use std::cmp::{min, max};

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
    let mut buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
    fb.update_buffer(&buffer);

    let GlutinBreakout {
        mut events_loop,
        gl_window,
        mut fb,
    } = fb.glutin_breakout();

    let mut running = true;
    let mut mouse_x = 0;
    let mut mouse_y = 0;
    let mut mouse_down = false;
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
                Event::WindowEvent { event: CursorMoved { position, .. }, .. } => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    let (x, y) = position.to_physical(dpi_factor).into();
                    mouse_x = min(max(x, 0), 800 - 1);
                    mouse_y = min(max(y, 0), 600 - 1);
                    if mouse_down {
                        buffer[(mouse_x + mouse_y * 800) as usize] = [64, 128, 255, 255];
                        fb.update_buffer(&buffer);
                    }
                }
                Event::WindowEvent { event: MouseInput { state, .. }, .. } => {
                    if state == ElementState::Pressed {
                        mouse_down = true;
                    } else {
                        mouse_down = false;
                    }
                }
                _ => {}
            }
        });

        fb.redraw();
        gl_window.swap_buffers().unwrap();
    }
}
