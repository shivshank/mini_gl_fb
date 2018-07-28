//! Hardware accelerated library inspired by minifb and friends.
//!
//! # Basic Usage
//!
//! Start with the function `gotta_go_fast`. This will create a basic window and give you a buffer
//! that you can draw to in one line.
//!
//! ```rust
//! extern crate mini_gl_fb;
//!
//! fn main() {
//!     let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
//!     let buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
//!     fb.update_buffer(&buffer);
//!     fb.persist();
//! }
//! ```
//!
//! The default buffer format is 32bit RGBA, so every pixel is four bytes. Buffer[0] is the top
//! left pixel. The buffer should be tightly packed with no padding after each row.
//!
//! # Interlude: Library philosophy
//!
//! All of the internals of this library are exposed. Any fields behind `mini_gl_fb.internal`
//! are not considered a part of the public API but are exposed in case the library is missing a
//! feature that you need "right now." This library is not here to box you in.
//!
//! Likewise, by exposing as much as possible it allows you to grow what may have started as a
//! simple project without hassle. This allows you to slowly move away from `mini_gl_fb` if
//! necessary, without requiring you to completely drop the library the second you need to do
//! something "advanced."
//!
//! This also means there's a number of ways to do the same thing, but this seems like a fair
//! compromise.
//!
//! # More advanced configuration
//!
//! Use the `get_fancy` function for more settings. See `Config` for what's available. This allows
//! you to, for instance, create a window with a buffer of a different size than the window.
//!
//! ```rust
//! let config = Config {
//!    window_title: window_title.to_string(),
//!    window_size: (window_width, window_height),
//!    .. Default::default()
//! };
//! let fb = get_fancy(config);
//! ```
//!
//! If you think something else should be exposed as an option, open an issue!
//!
//! # Bring your own context (and event handling)!
//!
//! Default context is provided by glutin. If that's not good enough for you [grr! ;^)], there's
//! the function `core::init_framebuffer`. Create your own OpenGL context, load the OpenGL
//! functions, and then call `core::init_framebuffer` to get a framebuffer with a texture already
//! set up.
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

pub mod config;
pub mod core;
pub mod breakout;

pub use breakout::GlutinBreakout;
pub use config::Config;
pub use core::{Internal, BufferFormat};

use core::ToGlType;

/// Creates a non resizable window and framebuffer with a given size in pixels.
///
/// Please note that the window size is in logical device pixels, so on a high DPI monitor the
/// physical window size may be larger. In this case, the rendered buffer will be scaled it
/// automatically by OpenGL.
pub fn gotta_go_fast<S: ToString>(
    window_title: S,
    window_width: f64,
    window_height: f64
) -> MiniGlFb {
    let config = Config {
        window_title: window_title.to_string(),
        window_size: (window_width, window_height),
        resizable: false,
        .. Default::default()
    };
    get_fancy(config)
}

/// Create a window with a custom configuration.
///
/// If this configuration is not sufficient for you, check out the source for this function.
/// Creating the MiniGlFb instance is just a call to two functions!
///
/// Many window settings can be changed after creation, so you most likely don't ever need to call
/// `get_fancy` with a custom config. However, if there is a bug in the OS/windowing system or
/// glutin or in this library, this function exists as a possible work around (or in case for some
/// reason everything must be absolutely correct at window creation)
pub fn get_fancy<S: ToString>(config: Config<S>) -> MiniGlFb {
    let buffer_width = if config.buffer_size.0 == 0 { config.window_size.0.round() as _ }
        else { config.buffer_size.0 };
    let buffer_height = if config.buffer_size.1 == 0 { config.window_size.1.round() as _ }
        else { config.buffer_size.1 };

    let (events_loop, gl_window) = core::init_glutin_context(
        config.window_title,
        config.window_size.0,
        config.window_size.1,
        config.resizable,
    );

    let dpi_factor = gl_window.get_hidpi_factor();
    let (vp_width, vp_height) = gl_window.get_inner_size()
        .unwrap()
        .to_physical(dpi_factor)
        .into();

    let fb = core::init_framebuffer(
        buffer_width,
        buffer_height,
        vp_width,
        vp_height,
    );

    MiniGlFb {
        internal: Internal {
            events_loop,
            gl_window,
            fb,
        }
    }
}

/// Main wrapper type.
///
/// **Any fields accessed through `internal` are not considered a public API and may be subject to
/// breaking API changes.** Only access this field as a last resort if the MiniGlFb API fails
/// to fit your exact use case.
///
/// Public methods of `Internal` are considered stable, but may be more complicated to use.
///
/// # Basic Usage
///
/// See the `update_buffer` and `persist` methods.
pub struct MiniGlFb {
    pub internal: Internal,
}

impl MiniGlFb {
    /// Updates the backing buffer and draws immediately (swaps buffers).
    ///
    /// The main drawing function.
    ///
    /// # Panics
    ///
    /// Panics if the size of the buffer does not exactly match the correct size of the texture
    /// data required based on the buffers format.
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        self.internal.update_buffer(image_data);
    }

    /// Changes the format of the image buffer.
    ///
    /// OpenGL will interpret any missing components as 0, except the alpha which it will assume is
    /// 255. For instance, if you set the format to BufferFormat::RG, OpenGL will render every
    /// pixel reading the two values you passed for the first two components, and then assume 0
    /// for the blue component, and 255 for the alpha.
    ///
    /// If you want to render in grayscale by providing a single component for each pixel, set
    /// the buffer format to BufferFormat::R, and call `use_grayscale_shader` (which will replace
    /// the fragment shader with one that sets all components equal to the red component).
    ///
    /// The type `T` does not affect how the texture is sampled, only how the buffer you pass is
    /// interpreted. Since there is no way exposed to change the internal format of the texture,
    /// (for instance if you wanted to make it an HDR image with floating point components) only
    /// the types `u8` and `i8` are supported. Open an issue if you have a use case for other
    /// types.
    ///
    /// # Example
    ///
    /// ```rust
    /// fb.change_buffer_format::<u8>(BufferFormat::R);
    /// fb.use_grayscale_shader();
    /// ```
    pub fn change_buffer_format<T: ToGlType>(&mut self, format: BufferFormat) {
        self.internal.fb.change_buffer_format::<T>(format);
    }

    /// Resizes the buffer.
    ///
    /// This does not affect the size of the window. The texture will be scaled to fit.
    pub fn resize_buffer(&mut self, buffer_width: u32, buffer_height: u32) {
        self.internal.fb.resize_buffer(buffer_width, buffer_height);
    }

    /// Switch to a shader that only uses the first component from your buffer.
    ///
    /// This **does not** switch to a shader which converts RGB(A) images to grayscale, for
    /// instance, by preserving percieved luminance.
    pub fn use_grayscale_shader(&mut self) {
        self.internal.fb.use_grayscale_shader();
    }

    /// Set the size of the OpenGL viewport (does not trigger a redraw).
    ///
    /// This does not resize the window or image buffer, only the area to which OpenGL draws. You
    /// only need to call this function when you are handling events manually and have a resizable
    /// window.
    ///
    /// You will know if you need to call this function, as in that case only part of the window
    /// will be getting drawn, typically after an update.
    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.internal.fb.resize_viewport(width, height);
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

    /// Need full access to Glutin's event handling? No problem!
    ///
    /// Hands you the event loop and the window we created, so you can handle events however you
    /// want, and the Framebuffer, so you can still draw easily!
    ///
    /// **IMPORTANT:** You should make sure to render something before swapping buffers or **the
    /// window may flash violently**. You can call `fb.redraw()` directly before if you are unsure
    /// that an OpenGL draw call was issued. `fb.update_buffer` will typically issue a draw call.
    pub fn glutin_breakout(self) -> GlutinBreakout {
        self.internal.glutin_breakout()
    }

    // TODO: set_resizable
}
