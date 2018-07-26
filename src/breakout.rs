use glutin::{GlWindow, EventsLoop};
use core::Framebuffer;

pub struct GlutinBreakout {
    pub events_loop: EventsLoop,
    pub gl_window: GlWindow,
    pub fb: Framebuffer,
}
