//! Contains the [`GlutinBreakout`] struct, which is a way to "break out" the Glutin context and
//! [`Framebuffer`] object and manipulate them directly.

use glutin::{WindowedContext, PossiblyCurrent, ContextError};
use core::Framebuffer;

use std::collections::HashMap;
use glutin::event::{MouseButton, VirtualKeyCode, ModifiersState};

/// `GlutinBreakout` is useful when you are growing out of the basic input methods and synchronous
/// nature of [`MiniGlFb`][crate::MiniGlFb], since it's more powerful than the the higher-level
/// abstrations. You can obtain it by calling
/// [`MiniGlFb::glutin_breakout()`][crate::MiniGlFb::glutin_breakout].
///
/// # Usage for multiple windows
/// The basic idea for managing multiple windows is to check each incoming event to determine which
/// window it's for. In order to draw to multiple windows individually, you have to switch the
/// context using [`make_current`][GlutinBreakout::make_current] before updating the window.
///
/// Here's a basic implementation (there's a lot of boilerplate because we're not using the
/// [`MiniGlFb`][crate::MiniGlFb] API - it's closer to using
/// [`winit`](https://docs.rs/winit/0.24.0/winit/index.html) directly):
///
/// ```
/// use mini_gl_fb::GlutinBreakout;
/// use mini_gl_fb::glutin::window::{Window, WindowId};
/// use mini_gl_fb::glutin::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
/// use mini_gl_fb::glutin::event_loop::{EventLoop, ControlFlow};
/// use mini_gl_fb::config;
///
/// struct TrackedWindow {
///     pub breakout: GlutinBreakout,
///     pub background: [u8; 4]
/// }
///
/// impl TrackedWindow {
///     fn window(&self) -> &Window { self.breakout.context.window() }
///     fn matches_id(&self, id: WindowId) -> bool { id == self.breakout.context.window().id() }
///
///     pub fn handle_event(&mut self, event: &Event<()>) -> bool {
///         match event {
///             Event::WindowEvent { window_id: id, event, .. } if self.matches_id(*id) => {
///                 match event {
///                     WindowEvent::CloseRequested |
///                     WindowEvent::KeyboardInput {
///                         input: KeyboardInput {
///                             virtual_keycode: Some(VirtualKeyCode::Escape),
///                             state: ElementState::Released,
///                             ..
///                         },
///                         ..
///                     } => return false,
///                     WindowEvent::Resized(size) => {
///                         self.breakout.fb.resize_viewport(size.width, size.height);
///                         let size = size.to_logical(self.window().scale_factor());
///                         self.breakout.fb.resize_buffer(size.width, size.height);
///                     }
///                     _ => {
///                         // do other stuff?
///                     }
///                 }
///             }
///             Event::RedrawRequested(id) if self.matches_id(*id) => {
///                 // If you don't do this, OpenGL will get confused and only draw to one window.
///                 unsafe { self.breakout.make_current().unwrap(); }
///
///                 let size = self.window().inner_size().to_logical::<f64>(self.window().scale_factor());
///
///                 // Unfortunately the performance of this is abysmal. Usually you should cache
///                 // your buffer and only update it when needed or when the window is resized.
///                 let pixels = size.width.floor() as usize * size.height.floor() as usize;
///                 self.breakout.fb.update_buffer(&vec![self.background; pixels]);
///                 self.breakout.context.swap_buffers();
///             }
///             _ => {}
///         }
///
///         true
///     }
/// }
///
/// fn main() {
///     let event_loop = EventLoop::new();
///     let mut windows: Vec<Option<TrackedWindow>> = vec![];
///
///     let config = config! {
///         resizable: true
///     };
///
///     windows.push(Some(TrackedWindow {
///         breakout: mini_gl_fb::get_fancy(config.clone(), &event_loop).glutin_breakout(),
///         background: [224u8, 66, 26, 255]
///     }));
///
///     windows.push(Some(TrackedWindow {
///         breakout: mini_gl_fb::get_fancy(config.clone(), &event_loop).glutin_breakout(),
///         background: [26u8, 155, 224, 255]
///     }));
///
///     // run event loop
///     event_loop.run(move |event, _, flow| {
///         *flow = ControlFlow::Wait;
///
///         for option in &mut windows {
///             if let Some(window) = option {
///                 if !window.handle_event(&event) {
///                     option.take();
///                 }
///             }
///         }
///
///         windows.retain(Option::is_some);
///
///         if windows.is_empty() {
///             *flow = ControlFlow::Exit;
///         }
///     })
/// }
/// ```
///
/// It's hard to come up with a generalized, flexible implementation of this, especially if you need
/// to open more windows based on user input, or run tasks in other threads, etc. Basically, it's
/// open for you to play with, but it's not functionality that MGlFb wants to include first-class
/// just yet.
#[derive(Debug)]
pub struct GlutinBreakout {
    /// Contains the OpenGL context and its associated window. This is a
    /// [`glutin`](https://docs.rs/glutin/0.26.0/glutin/) struct; go see their documentation on
    /// [`WindowedContext`] for more information.
    pub context: WindowedContext<PossiblyCurrent>,
    /// Contains the [`Framebuffer`] for that context. Consult its documentation for information on
    /// how to use it.
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
    /// # use mini_gl_fb::{config, get_fancy};
    /// #
    /// # let mut event_loop = EventLoop::new();
    /// # let mut breakout = get_fancy(config! {
    /// #     window_title: String::from("GlutinBreakout::make_current()")
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
        let context_ptr: *mut _ = &mut self.context;
        let context = std::ptr::read(context_ptr);
        let result = context.make_current();

        if let Err((context, err)) = result {
            std::ptr::write(context_ptr, context);
            Err(err)
        } else {
            std::ptr::write(context_ptr, result.unwrap());
            Ok(())
        }
    }
}

/// Used for [`MiniGlFb::glutin_handle_basic_input`][crate::MiniGlFb::glutin_handle_basic_input].
/// Contains the current state of the window in a polling-like fashion.
#[non_exhaustive]
#[derive(Default, Clone, PartialEq, Debug)]
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
    /// The current modifier keys that are being pressed.
    pub modifiers: ModifiersState,
    /// This is set to `true` when the window is resized outside of your callback. If you do not
    /// update the buffer in your callback, you should still draw it if this is `true`.
    pub resized: bool,
    /// If this is set to `true` by your callback, it will not be called as fast as possible, but
    /// rather only when the input changes.
    pub wait: bool,
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
