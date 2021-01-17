//! Hardware accelerated library inspired by minifb and friends.
//!
//! # Basic Usage
//!
//! Start with the function `gotta_go_fast`. This will create a basic window and give you a buffer
//! that you can draw to in one line. The main public API is available through the `MiniGlFb` type.
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
//! The default buffer format is 32bit RGBA, so every pixel is four bytes. Buffer[0] is the bottom
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
//! use mini_gl_fb::{get_fancy, Config};
//! # let window_title = "foo";
//! # let window_width = 800.0;
//! # let window_height = 600.0;
//!
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

pub use breakout::{GlutinBreakout, BasicInput};
pub use config::Config;
pub use core::{Internal, BufferFormat, Framebuffer};

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
) -> MiniGlFb<()> {
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
pub fn get_fancy<S: ToString, ET: 'static>(config: Config<S, ET>) -> MiniGlFb<ET> {
    let buffer_width = if config.buffer_size.0 == 0 { config.window_size.0.round() as _ }
        else { config.buffer_size.0 };
    let buffer_height = if config.buffer_size.1 == 0 { config.window_size.1.round() as _ }
        else { config.buffer_size.1 };

    let (events_loop, context) = core::init_glutin_context(
        config.window_title,
        config.window_size.0,
        config.window_size.1,
        config.resizable,
        config.event_loop
    );

    let (vp_width, vp_height) = context.window().inner_size().into();

    let fb = core::init_framebuffer(
        buffer_width,
        buffer_height,
        vp_width,
        vp_height,
    );

    MiniGlFb {
        internal: Internal {
            event_loop: events_loop,
            context,
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
pub struct MiniGlFb<ET: 'static> {
    pub internal: Internal<ET>,
}

impl<ET: 'static> MiniGlFb<ET> {
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

    /// Checks if escape has been pressed or the window has been asked to close.
    ///
    /// This function is a good choice for a while loop condition when you are making a simulation
    /// that needs to progress over time but does not need to handle user input.
    ///
    /// Calling this function clears the event queue and also handles resizes for you (if your
    /// window is resizable). This does not resize the image buffer; the rendered buffer will
    /// instead scale to fit the window.
    ///
    /// Please note that if your window does change size, for buffer to appear scaled it must
    /// be redrawn, typically either by calling `redraw` or `update_buffer`.
    pub fn is_running(&mut self) -> bool {
        self.internal.is_running()
    }

    pub fn redraw(&mut self) {
        self.internal.redraw();
    }

    /// Use a custom post process shader written in GLSL (version 330 core).
    ///
    /// The interface is unapologetically similar to ShaderToy's. It works by inserting your code
    /// (it is implemented as literal substitution) into a supplied fragment shader and calls
    /// a function `main_image` that it assumes you define.
    ///
    /// # Example usage
    ///
    /// The behavior of the default fragment shader can be emulated by the following:
    ///
    /// ```rust
    /// # use mini_gl_fb::get_fancy;
    /// # let mut fb = get_fancy::<&str>(Default::default());
    /// fb.use_post_process_shader("
    ///     void main_image( out vec4 r_frag_color, in vec2 v_uv ) {
    ///         r_frag_color = texture(u_buffer, v_uv);
    ///     }
    /// ");
    /// ```
    ///
    /// Regardless of the format of your buffer, the internal texture is always stored as RGBA,
    /// so sampling u_buffer will yield a vec4 representing an RGBA color. The built in grayscale
    /// shader, for instance, only stores Red components, and then uses the red component for the
    /// green and blue components to create gray.
    ///
    /// The output color is determined by the value of the first output parameter, `r_frag_color`.
    /// Your buffer is accessible as a 2D sampler uniform named `u_buffer`. The first input
    /// parameter `v_uv` is a vec2 UV coordinate. UV (0, 0) represents the bottom left of the
    /// screen and (1, 1) represents the top right.
    ///
    /// An API for exposing more built in and custom uniforms is planned, along with support for
    /// an arbitrary number of render targets and possibly more user supplied textures.
    pub fn use_post_process_shader(&mut self, source: &str) {
        self.internal.fb.use_post_process_shader(source);
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
    /// use mini_gl_fb::BufferFormat;
    /// # use mini_gl_fb::get_fancy;
    /// # let mut fb = get_fancy::<&str>(Default::default());
    ///
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
    /// For high DPI screens this is the physical size of the viewport.
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

    /// Set whether or not the window is resizable.
    ///
    /// Please note that if you are handling events yourself that you need to call
    /// `resize_viewport` when the window is resized, otherwise the buffer will only be drawn to
    /// a small portion of the window.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.internal.set_resizable(resizable);
    }

    /// Keeps the window open until the user closes it.
    ///
    /// Supports pressing escape to quit. Automatically scales the rendered buffer to the size of
    /// the window if the window is resiable (but this does not resize the buffer).
    pub fn persist(&mut self) {
        self.internal.persist();
    }

    /// `persist` implementation.
    ///
    /// When redraw is true, redraws as fast as possible. This function is primarily for debugging.
    ///
    /// See `persist` method documentation for more info.
    pub fn persist_and_redraw(&mut self, redraw: bool) {
        self.internal.persist_and_redraw(redraw);
    }

    /// Provides an easy interface for rudimentary input handling.
    ///
    /// Automatically handles close events and partially handles resizes (the caller chooses if
    /// a redraw is necessary; and the window will only actually physically change size if it is
    /// a resizable window).
    ///
    /// Polls for window events and summarizes the input events for you each frame. See
    /// `BasicInput` for the information that is provided to you. You will need to use some
    /// glutin types (which just wraps the crate winit's input types), so glutin is re-expoted
    /// by this library. You can access it via `use mini_gl_fb::glutin`.
    ///
    /// You can cause the handler to exit by returning false from it. This does not kill the
    /// window, so as long as you still have it in scope, you can actually keep using it and,
    /// for example, resume handling input but with a different handler callback.
    pub fn glutin_handle_basic_input<F: FnMut(&mut Framebuffer, &BasicInput) -> bool>(
        &mut self, handler: F
    ) {
        self.internal.glutin_handle_basic_input(handler);
    }

    /// Need full access to Glutin's event handling? No problem!
    ///
    /// Hands you the event loop and the window we created, so you can handle events however you
    /// want, and the Framebuffer, so you can still draw easily!
    ///
    /// **IMPORTANT:** You should make sure to render something before swapping buffers or **the
    /// window may flash violently**. You can call `fb.redraw()` directly before if you are unsure
    /// that an OpenGL draw call was issued. `fb.update_buffer` will typically issue a draw call.
    pub fn glutin_breakout(self) -> GlutinBreakout<ET> {
        self.internal.glutin_breakout()
    }
}
