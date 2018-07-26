//! Hardware accelerated library inspired by minifb and friends.
//!
//! Powered by OpenGL. Default context is provided by glutin, but this may be optional in the
//! future so that you can create your own context any way you like.
//!
//! # Note on possible context creation failure:
//!
//! Currently uses the `gl` crate for OpenGL loading. OpenGL context creation may fail if your
//! setup does not support the newest OpenGL. This bug needs to be verified and is be fixable.
//! OpenGL ~3 is currently required, but OpenGL 2.1 support should be feasible if requested.

#[macro_use]
pub extern crate rustic_gl;

pub extern crate glutin;
pub extern crate gl;

mod config;
mod core;
// mod breakout;

pub use config::Config;
pub use core::{Internal, BufferFormat};

/*
// TODO: Support mixed { prop, prop: value, .. } for creating configs through the macro

#[macro_export]
macro_rules! get_fancy {
    (
        $($setting:ident: $setting_value:expr),*
    ) => {
        // Support both no trailing comma and trailing comma
        // (The core macro impl assumes trailing comma)
        get_fancy!($($setting: $setting_value),*,)
    };

    (
        $($setting:ident),*
    ) => {
        // Support both no trailing comma and trailing comma
        // (The core macro impl assumes trailing comma)
        get_fancy!($($setting),*,)
    };

    (
        $($setting:ident),*,
    ) => {
        get_fancy!($($setting: $setting),*,)
    };

    (
        $($setting:ident: $setting_value:expr),*,
    ) => {{
        let config = $crate::Config {
            $(
                $setting: $setting_value
            ),*,
            .. Default::default()
        };
        $crate::get_fancy(config)
    }};
}*/

pub fn gotta_go_fast<S: ToString>(
    window_title: S,
    window_width: f64,
    window_height: f64
) -> MiniGlFb {
    let config = Config {
        window_title: window_title.to_string(),
        window_size: (window_width, window_height),
        .. Default::default()
    };
    get_fancy(config)
}

pub fn get_fancy<S: ToString>(config: Config<S>) -> MiniGlFb {
    let (events_loop, gl_window) = core::init_glutin_context(&config);
    let fb = core::init_framebuffer(&config);

    MiniGlFb {
        internal: Internal {
            events_loop,
            gl_window,
            fb,
        }
    }
}

pub struct MiniGlFb {
    /// All fields are exposed for your convienience, but use at your own risk.
    ///
    /// Anything accessed through `internal` is not considered a public API and may be subject to
    /// breaking API changes. Only access this field as a last resort if the MiniGlFb API fails
    /// to fit your exact use case.
    pub internal: Internal,
}

impl MiniGlFb {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        self.internal.update_buffer(image_data);
    }

    /// Keeps the window open until the user closes it.
    pub fn persist(&mut self) {
        self.internal.persist();
    }

    /// `persist` implementation.
    ///
    /// When redraw is true, redraws as fast as possible. This function is primarily for debugging.
    pub fn persist_and_redraw(&mut self, redraw: bool) {
        self.internal.persist_and_redraw(redraw);
    }

    // TODO: resize_buffer
    // TODO: set_resizable
    // TODO: change_buffer_format
}
