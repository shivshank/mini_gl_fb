use glutin::{
    GlWindow,
    EventsLoop,
    VirtualKeyCode,
    MouseButton,
    ModifiersState,
};
use core::Framebuffer;

use std::collections::HashMap;

pub struct GlutinBreakout {
    pub events_loop: EventsLoop,
    pub gl_window: GlWindow,
    pub fb: Framebuffer,
}

pub struct BasicInput {
    /// The mouse position in buffer coordinates.
    ///
    /// The bottom left of the window is (0, 0).
    pub mouse_pos: (usize, usize),
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
