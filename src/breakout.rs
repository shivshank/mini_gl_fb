use glutin::{WindowedContext, PossiblyCurrent};
use core::Framebuffer;

use std::collections::HashMap;
use glutin::event::{MouseButton, VirtualKeyCode, ModifiersState};
use glutin::event_loop::EventLoop;

pub struct GlutinBreakout<ET: 'static> {
    pub events_loop: EventLoop<ET>,
    pub context: WindowedContext<PossiblyCurrent>,
    pub fb: Framebuffer,
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
