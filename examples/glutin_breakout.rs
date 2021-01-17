extern crate mini_gl_fb;
extern crate glutin;

use std::cmp::{min, max};
use mini_gl_fb::GlutinBreakout;
use mini_gl_fb::glutin::event::WindowEvent::KeyboardInput;
use mini_gl_fb::glutin::event::{Event, VirtualKeyCode, ElementState, WindowEvent};
use mini_gl_fb::glutin::event_loop::ControlFlow;

fn main() {
    let (event_loop, mut fb) = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
    let mut buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
    fb.update_buffer(&buffer);

    let GlutinBreakout {
        context,
        mut fb,
    } = fb.glutin_breakout();

    let mut mouse_down = false;

    event_loop.run(move |event, _, flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: KeyboardInput { input, .. }, .. } => {
                if let Some(k) = input.virtual_keycode {
                    if k == VirtualKeyCode::Escape && input.state == ElementState::Released {
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                context.resize(size);
                context.window().request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                let (x, y) = position.to_logical::<f64>(context.window().scale_factor()).into();
                println!("({}, {})", x, y);
                let mouse_x = min(max(x, 0), 800 - 1);
                let mouse_y = min(max(fb.buffer_height - y, 0), 600 - 1);
                if mouse_down {
                    buffer[(mouse_x + mouse_y * 800) as usize] = [64, 128, 255, 255];
                    fb.update_buffer(&buffer);
                    context.window().request_redraw();
                }
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state, .. }, .. } => {
                if state == ElementState::Pressed {
                    mouse_down = true;
                } else {
                    mouse_down = false;
                }
            }
            Event::RedrawRequested(_) => {
                fb.redraw();
                context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}
