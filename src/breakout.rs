use glutin::{WindowedContext, PossiblyCurrent, ContextError};
use core::Framebuffer;

use std::collections::HashMap;
use glutin::event::{MouseButton, VirtualKeyCode, ModifiersState};

pub struct GlutinBreakout {
    pub context: WindowedContext<PossiblyCurrent>,
    pub fb: Framebuffer,
}

impl GlutinBreakout {
    /// Sets the current thread's OpenGL context to the one contained in this breakout.
    ///
    /// Historically, MGlFb did not support multiple windows. It owned its own event loop and you
    /// weren't allowed to use the library with your own. However, as of version 0.8, you are now
    /// expected to bring your own event loop to all functions that involve one. This means that
    /// multiple windows are very possible, and even supported, as long as you're willing to route
    /// events yourself... and manage all the OpenGL contexts.
    ///
    /// The problem with managing multiple OpenGL contexts from one thread is that the "current"
    /// context is set per-thread. That means you basically have to switch through them really
    /// quickly if you want to update multiple windows in "parallel". But how do you switch?
    ///
    /// Glutin has you partially covered on this one - it has
    /// [`make_current`][glutin::ContextWrapper<PossiblyCurrent, Window>::make_current]. However,
    /// that method takes `self` and emits a new `WindowedContext`, and you can't really move `self`
    /// into that function without unsafe code.
    ///
    /// Here is an unsafe function containing code that makes the context current, in-place. That
    /// way, you can switch contexts in one line of code, and focus on other stuff.
    ///
    /// # Usage
    ///
    /// ```
    /// # use mini_gl_fb::glutin::event_loop::{EventLoop, ControlFlow};
    /// # use mini_gl_fb::glutin::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
    /// # use mini_gl_fb::get_fancy;
    /// # use mini_gl_fb::Config;
    /// #
    /// # let mut event_loop = EventLoop::new();
    /// # let mut breakout = get_fancy(Config {
    /// #     window_title: "GlutinBreakout::make_current()",
    /// #     ..Config::<&str>::default()
    /// # }, &event_loop).glutin_breakout();
    /// #
    /// event_loop.run(move |event, _, flow| {
    /// #     *flow = ControlFlow::Wait;
    /// #
    ///     match event {
    ///         // ...
    /// #         Event::WindowEvent { event, .. } => {
    /// #             match event {
    /// #                 WindowEvent::CloseRequested |
    /// #                 WindowEvent::KeyboardInput {
    /// #                     input: KeyboardInput {
    /// #                         virtual_keycode: Some(VirtualKeyCode::Escape),
    /// #                         state: ElementState::Released,
    /// #                         ..
    /// #                     },
    /// #                     ..
    /// #                 } => *flow = ControlFlow::Exit,
    /// #                 _ => ()
    /// #             }
    /// #         },
    ///         Event::RedrawRequested(..) => {
    ///             unsafe { breakout.make_current().unwrap(); }
    ///             // ...
    /// #             let window = breakout.context.window();
    /// #             let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    /// #             let pixels = size.width.floor() as usize * size.height.floor() as usize;
    /// #             let your_buffer_here = vec![[0u8, 200, 240, 255]; pixels];
    ///             breakout.fb.update_buffer(&your_buffer_here);
    ///             breakout.context.swap_buffers();
    ///         }
    ///         // ...
    /// #         _ => {}
    ///     }
    /// })
    /// ```
    pub unsafe fn make_current(&mut self) -> Result<(), ContextError> {
        let context: WindowedContext<PossiblyCurrent> =
            std::ptr::read(&mut self.context as *mut _);
        let result = context.make_current();

        if let Err((context, err)) = result {
            std::ptr::write(&mut self.context as *mut _, context);
            Err(err)
        } else {
            std::ptr::write(&mut self.context as *mut _, result.unwrap());
            Ok(())
        }
    }
}

pub struct BasicInput {
    /// The mouse position in buffer coordinates.
    ///
    /// The bottom left of the window is (0, 0). Pixel centers are at multiples of (0.5, 0.5). If
    /// you want to use this to index into your buffer, in general the following is sufficient:
    ///
    /// - clamp each coordinate to the half-open range [0.0, buffer_size)
    /// - take the floor of each component
    /// - cast to usize and compute an index: `let index = y * WIDTH + x`
    pub mouse_pos: (f64, f64),
    /// Stores whether a mouse button was down and is down, in that order.
    ///
    /// If a button has not been pressed yet it will not be in the map.
    pub mouse: HashMap<MouseButton, (bool, bool)>,
    /// Stores the previous and current "key down" states, in that order.
    ///
    /// If a key has not been pressed yet it will not be in the map.
    pub keys: HashMap<VirtualKeyCode, (bool, bool)>,
    pub modifiers: ModifiersState,
    pub resized: bool,
}

impl BasicInput {
    // TODO: Do we want to add a `mouse_as_buffer_index` or method or something like that?

    /// If the mouse was pressed this last frame.
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        &(false, true) == self.mouse.get(&button).unwrap_or(&(false, false))
    }

    /// If the mouse is currently down.
    pub fn mouse_is_down(&self, button: MouseButton) -> bool {
        if let &(_, true) = self.mouse.get(&button).unwrap_or(&(false, false)) {
            true
        } else {
            false
        }
    }

    /// If the mouse was released this last frame.
    pub fn mouse_released(&self, button: MouseButton) -> bool {
        &(true, false) == self.mouse.get(&button).unwrap_or(&(false, false))
    }

    /// If the key was pressed this last frame.
    pub fn key_pressed(&self, button: VirtualKeyCode) -> bool {
        &(false, true) == self.keys.get(&button).unwrap_or(&(false, false))
    }

    /// If the key is currently down.
    pub fn key_is_down(&self, button: VirtualKeyCode) -> bool {
        if let &(_, true) = self.keys.get(&button).unwrap_or(&(false, false)) {
            true
        } else {
            false
        }
    }

    /// If the key was released this last frame.
    pub fn key_released(&self, button: VirtualKeyCode) -> bool {
        &(true, false) == self.keys.get(&button).unwrap_or(&(false, false))
    }
}
