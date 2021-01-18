extern crate mini_gl_fb;
extern crate glutin;

use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::event::{Event, WindowEvent, MouseButton, VirtualKeyCode, KeyboardInput, ElementState};
use mini_gl_fb::{get_fancy, GlutinBreakout, Config};
use mini_gl_fb::glutin::dpi::{LogicalSize, LogicalPosition};
use mini_gl_fb::glutin::window::{Window, WindowId, CursorIcon};
use mini_gl_fb::glutin::event_loop::ControlFlow;
use std::cell::Cell;
use mini_gl_fb::glutin::platform::run_return::EventLoopExtRunReturn;
use mini_gl_fb::glutin::{WindowedContext, PossiblyCurrent};

/// A window being tracked by a `MultiWindow`. All tracked windows will be forwarded all events
/// received on the `MultiWindow`'s event loop.
trait TrackedWindow {
    /// Handles one event from the event loop. Returns true if the window needs to be kept alive,
    /// otherwise it will be closed. Window events should be checked to ensure that their ID is one
    /// that the TrackedWindow is interested in.
    fn handle_event(&mut self, event: &Event<()>) -> bool;
}

/// Manages multiple `TrackedWindow`s by forwarding events to them.
struct MultiWindow {
    windows: Vec<Cell<Box<dyn TrackedWindow>>>,
}

impl MultiWindow {
    /// Creates a new `MultiWindow`.
    pub fn new() -> Self {
        MultiWindow {
            windows: vec![],
        }
    }

    /// Adds a new `TrackedWindow` to the `MultiWindow`.
    pub fn add(&mut self, window: Box<dyn TrackedWindow>) {
        self.windows.push(Cell::new(window))
    }

    /// Runs the event loop until all `TrackedWindow`s are closed.
    pub fn run(&mut self, event_loop: &mut EventLoop<()>) {
        if !self.windows.is_empty() {
            event_loop.run_return(|event, _, flow| {
                *flow = ControlFlow::Wait;

                self.windows.retain(|window|
                    unsafe { &mut *window.as_ptr() }.handle_event(&event)
                );

                if self.windows.is_empty() {
                    *flow = ControlFlow::Exit;
                }
            });
        }
    }
}

/// A basic window that allows you to draw in it. An example of how to implement a `TrackedWindow`.
struct DrawWindow {
    pub breakout: GlutinBreakout,
    pub buffer: Vec<u8>,
    pub buffer_size: LogicalSize<u32>,
    pub bg: [u8; 4],
    pub fg: [u8; 4],
    mouse_state: ElementState,
    line_start: Option<LogicalPosition<i32>>,
}

impl DrawWindow {
    fn window(&self) -> &Window {
        self.breakout.context.window()
    }

    pub fn matches_id(&self, id: WindowId) -> bool {
        id == self.window().id()
    }

    /// A call to this function is required before updating the window's buffer or doing any other
    /// OpenGL things, since all windows are run on the same thread.
    ///
    /// # Panics
    /// Panics if the OpenGL context cannot be made current.
    unsafe fn make_current(&mut self) {
        let mut context: WindowedContext<PossiblyCurrent> = std::ptr::read(&mut self.breakout.context as *mut _);
        context = context.make_current().unwrap();
        std::ptr::write(&mut self.breakout.context as *mut _, context);
    }

    /// Updates the window's buffer. Should only be done inside of RedrawRequested events; outside
    /// of them, use `request_redraw` instead.
    fn redraw(&mut self) {
        self.breakout.fb.update_buffer(&self.buffer);
        self.breakout.context.swap_buffers().unwrap();
    }

    /// Requests a redraw event for this window.
    fn request_redraw(&self) {
        self.window().request_redraw();
    }

    /// Resizes the window's buffer to a new size, attempting to preserve the current content as
    /// much as possible. Fills new space with the background color, and deletes overflowing space.
    fn resize(&mut self, new_size: LogicalSize<u32>) {
        let mut new_buffer = vec![0u8; new_size.width as usize * new_size.height as usize * 4];
        new_buffer.chunks_exact_mut(4).for_each(|c| c.copy_from_slice(&self.bg));

        if self.buffer_size.width > 0 {
            // use rchunks for inverted y
            for (old_line, new_line) in self.buffer.chunks_exact(self.buffer_size.width as usize * 4)
                .zip(new_buffer.chunks_exact_mut(new_size.width as usize * 4)) {
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

    fn plot(&mut self, position: LogicalPosition<i32>) {
        if position.x < 0 || position.x >= self.buffer_size.width as i32 ||
            position.y < 0 || position.y >= self.buffer_size.height as i32 {
            return
        }

        let position = position.cast::<u32>();
        let index = (position.x + position.y * self.buffer_size.width) as usize * 4;
        self.buffer[index..index + 4].copy_from_slice(&self.fg);
    }

    // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
    fn plot_line(&mut self, start: LogicalPosition<i32>, end: LogicalPosition<i32>) {
        let (mut x0, mut y0): (i32, i32) = start.into();
        let (x1, y1): (i32, i32) = end.into();
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        while x0 != x1 || y0 != y1 {
            self.plot(LogicalPosition::new(x0, y0));
            if x0 == x1 && y0 == y1 { break; }
            let e2 = err * 2;
            if e2 > dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }

        self.plot(end);
    }

    /// Creates a new `DrawWindow` for the specified event loop, using the specified background and
    /// foreground colors.
    pub fn new(event_loop: &EventLoop<()>, bg: [u8; 4], fg: [u8; 4]) -> Self {
        let mut new = Self {
            breakout: get_fancy::<&str, ()>(Config {
                resizable: true,
                invert_y: false,
                ..Default::default()
            }, &event_loop).glutin_breakout(),
            buffer: vec![],
            buffer_size: LogicalSize::new(0, 0),
            bg,
            fg,
            mouse_state: ElementState::Released,
            line_start: None,
        };
        new.resize(new.window().inner_size().to_logical(new.window().scale_factor()));
        new.window().set_cursor_icon(CursorIcon::Crosshair);
        new
    }
}

impl TrackedWindow for DrawWindow {
    fn handle_event(&mut self, event: &Event<()>) -> bool {
        match *event {
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::CloseRequested,
                ..
            } if self.matches_id(id) => {
                return false;
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state: ElementState::Released,
                        ..
                    },
                    ..
                },
                ..
            } if self.matches_id(id) => {
                return false;
            }
            Event::RedrawRequested(id) if self.matches_id(id) => {
                unsafe { self.make_current(); }
                self.redraw();
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::Resized(size),
                ..
            } if self.matches_id(id) => {
                unsafe { self.make_current(); }
                self.breakout.fb.resize_viewport(size.width, size.height);
                self.resize(size.to_logical(self.window().scale_factor()));
                self.request_redraw();
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                },
                ..
            } if self.matches_id(id) => {
                self.mouse_state = state;
                self.line_start = None;
            }
            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::CursorMoved {
                    position,
                    ..
                },
                ..
            } if self.matches_id(id) => {
                if self.mouse_state == ElementState::Pressed {
                    let mut position = position.to_logical::<i32>(self.window().scale_factor());

                    if let Some(line_start) = self.line_start {
                        self.plot_line(line_start, position);
                    } else {
                        self.plot(position);
                    }

                    self.line_start = Some(position);

                    self.request_redraw();
                }
            }
            _ => {}
        }

        true
    }
}

fn main() {
    let mut event_loop = EventLoop::new();
    let mut multi_window = MultiWindow::new();
    multi_window.add(Box::new(DrawWindow::new(&event_loop, [25u8, 33, 40, 255], [54u8, 165, 209, 255])));
    multi_window.add(Box::new(DrawWindow::new(&event_loop, [25u8, 40, 33, 255], [54u8, 209, 82, 255])));
    multi_window.add(Box::new(DrawWindow::new(&event_loop, [40u8, 33, 25, 255], [209u8, 82, 54, 255])));
    multi_window.run(&mut event_loop);
}
