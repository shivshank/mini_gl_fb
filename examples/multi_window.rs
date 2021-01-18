extern crate mini_gl_fb;
extern crate glutin;

use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::event::{Event, WindowEvent, MouseButton};
use mini_gl_fb::{get_fancy, GlutinBreakout, Config};
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::window::Window;
use mini_gl_fb::glutin::window::WindowId;
use mini_gl_fb::glutin::event_loop::ControlFlow;
use std::cell::Cell;
use mini_gl_fb::glutin::platform::run_return::EventLoopExtRunReturn;
use mini_gl_fb::glutin::{WindowedContext, PossiblyCurrent};
use mini_gl_fb::glutin::event::ElementState;

struct TrackedWindowImpl {
    pub breakout: GlutinBreakout,
    pub buffer: Vec<u8>,
    pub buffer_size: LogicalSize<u32>,
    pub bg: [u8; 4],
    pub fg: [u8; 4],
    mouse_state: ElementState,
}

impl TrackedWindowImpl {
    fn window(&self) -> &Window {
        self.breakout.context.window()
    }

    pub fn matches_id(&self, id: WindowId) -> bool {
        id == self.window().id()
    }

    unsafe fn make_current(&mut self) {
        let mut context: WindowedContext<PossiblyCurrent> = std::ptr::read(&mut self.breakout.context as *mut _);
        context = context.make_current().unwrap();
        std::ptr::write(&mut self.breakout.context as *mut _, context);
    }

    fn redraw(&mut self) {
        self.breakout.fb.update_buffer(&self.buffer);
        self.breakout.context.swap_buffers().unwrap();
    }

    fn resize(&mut self, new_size: LogicalSize<u32>) {
        let mut new_buffer = vec![0u8; new_size.width as usize * new_size.height as usize * 4];
        new_buffer.chunks_exact_mut(4).for_each(|c| c.copy_from_slice(&self.bg));

        if self.buffer_size.width > 0 {
            for (old_line, new_line) in self.buffer.rchunks_exact(self.buffer_size.width as usize * 4)
                .zip(new_buffer.rchunks_exact_mut(new_size.width as usize * 4)) {
                if old_line.len() <= new_line.len() {
                    new_line[0..old_line.len()].copy_from_slice(old_line)
                } else {
                    new_line.copy_from_slice(&old_line[0..new_line.len()])
                }
            }
        }

        self.buffer = new_buffer;
        self.buffer_size = new_size;
        self.breakout.fb.resize_buffer(new_size.width, new_size.height);
    }

    fn request_redraw(&self) {
        self.window().request_redraw();
    }

    pub fn new(event_loop: &EventLoop<()>, bg: [u8; 4], fg: [u8; 4]) -> Self {
        let mut new = Self {
            breakout: get_fancy::<&str, ()>(Config {
                resizable: true,
                ..Default::default()
            }, &event_loop).glutin_breakout(),
            buffer: vec![],
            buffer_size: LogicalSize::new(0, 0),
            bg,
            fg,
            mouse_state: ElementState::Released,
        };
        new.resize(new.window().inner_size().to_logical(new.window().scale_factor()));
        new
    }
}

trait TrackedWindow {
    /// Handles one event from the event loop. Returns true if the window needs to be kept alive,
    /// otherwise it will be closed. Should manually check IDs coming in to make sure they are
    /// relevant.
    fn handle_event(&mut self, event: &Event<()>) -> bool;
}

impl TrackedWindow for TrackedWindowImpl {
    fn handle_event(&mut self, event: &Event<()>) -> bool {
        match *event {
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if self.matches_id(id) {
                    return false;
                }
            }
            Event::RedrawRequested(id) => {
                if self.matches_id(id) {
                    unsafe { self.make_current(); }
                    self.redraw();
                }
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::Resized(size),
                ..
            } => {
                if self.matches_id(id) {
                    unsafe { self.make_current(); }
                    self.breakout.fb.resize_viewport(size.width, size.height);
                    self.resize(size.to_logical(self.window().scale_factor()));
                    self.request_redraw();
                }
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                },
                ..
            } => {
                if self.matches_id(id) {
                    self.mouse_state = state;
                }
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::CursorMoved {
                    position,
                    ..
                },
                ..
            } => {
                if self.mouse_state == ElementState::Pressed && self.matches_id(id) {
                    let mut position = position.to_logical::<u32>(self.window().scale_factor());
                    position.x = std::cmp::min(position.x, self.buffer_size.width - 1);
                    position.y = (self.buffer_size.height - 1) - std::cmp::min(position.y, self.buffer_size.height - 1);
                    let index = (position.x + position.y * self.buffer_size.width) as usize * 4;
                    self.buffer[index..index + 4].copy_from_slice(&self.fg);
                    self.request_redraw();
                }
            }
            _ => {}
        }

        true
    }
}

struct MultiWindow {
    windows: Vec<Cell<Box<dyn TrackedWindow>>>,
}

impl MultiWindow {
    pub fn new() -> Self {
        MultiWindow {
            windows: vec![],
        }
    }

    pub fn add(&mut self, window: Box<dyn TrackedWindow>) {
        self.windows.push(Cell::new(window))
    }

    pub fn run(&mut self, event_loop: &mut EventLoop<()>) {
        event_loop.run_return(|event, _, flow| {
            *flow = ControlFlow::Wait;

            self.windows.retain(|window|
                unsafe { &mut *window.as_ptr() }.handle_event(&event)
            );

            if self.windows.is_empty() {
                *flow = ControlFlow::Exit;
            }
        })
    }
}

fn main() {
    let mut event_loop = EventLoop::new();
    let mut multi_window = MultiWindow::new();
    multi_window.add(Box::new(TrackedWindowImpl::new(&event_loop, [25u8, 33, 40, 255], [54u8, 165, 209, 255])));
    multi_window.add(Box::new(TrackedWindowImpl::new(&event_loop, [25u8, 40, 33, 255], [54u8, 209, 82, 255])));
    multi_window.add(Box::new(TrackedWindowImpl::new(&event_loop, [40u8, 33, 25, 255], [209u8, 82, 54, 255])));
    multi_window.run(&mut event_loop);
}
