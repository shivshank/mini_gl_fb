use glutin::dpi::LogicalSize;

/// Configuration for "advanced" use cases, when `gotta_go_fast` isn't doing what you need.
///
/// The following pattern is recommended when creating a config:
///
/// ```
/// use mini_gl_fb::Config;
/// use mini_gl_fb::glutin::dpi::LogicalSize;
///
/// let config = Config {
///     /* specify whichever fields you need to set, for example: */
///     window_size: LogicalSize::new(100.0, 100.0),
///     resizable: true,
///     .. Default::default()
/// };
/// ```
///
/// If there's a config option you want to see or think is missing, please open an issue!
pub struct Config {
    /// Sets the pixel dimensions of the buffer. The buffer will automatically stretch to fill the
    /// whole window. By default this will be the same as the window_size.
    pub buffer_size: Option<LogicalSize<u32>>,
    /// If this is true, the window created by mini_gl_fb will be set to resizable. This can be
    /// changed later. Please note that the buffer itself will not be automatically resized, only
    /// the viewport.
    pub resizable: bool,
    /// The title of the window that will be created.
    pub window_title: String,
    /// The logical size of the window that gets created. On HiDPI screens the actual size may be
    /// larger than this
    pub window_size: LogicalSize<f64>,
    /// By default, the origin of the buffer is the bottom-left. This is known as "inverted Y", as
    /// most screen-space coordinate systems begin from the top-left. By explicitly setting this
    /// option to `false`, you can switch to screen-space coordinates rather than OpenGL
    /// coordinates. Otherwise, you will have to invert all mouse events received from winit/glutin.
    pub invert_y: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            buffer_size: None,
            resizable: false,
            // :^)
            window_title: String::from("Super Mini GL Framebufferer 3!"),
            window_size: LogicalSize::new(600.0, 480.0),
            invert_y: true
        }
    }
}
