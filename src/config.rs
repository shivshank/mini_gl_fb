/// Configuration for "advanced" use cases, when `gotta_go_fast` isn't doing what you need.
///
/// The following pattern is recommended when creating a config:
///
/// ```
/// use mini_gl_fb::Config;
///
/// let config: Config<&str> = Config {
///     /* specify whichever fields you need to set, for example: */
///     window_size: (100.0, 100.0),
///     resizable: false,
///     .. Default::default()
/// };
/// ```
///
/// If there's a config option you want to see or think is missing, please open an issue!
pub struct Config<S: ToString> {
    /// Sets the pixel dimensions of the buffer. The buffer will automatically scale to the size of
    /// the window. By default this will be the same as the window_size.
    pub buffer_size: (u32, u32),
    /// If this is true, the window created by mini_gl_fb will be set to resizable. This can be
    /// changed later.
    pub resizable: bool,
    pub window_title: S,
    pub window_size: (f64, f64),
    pub invert_y: bool
}

impl<S: ToString + Clone> Clone for Config<S> {
    fn clone(&self) -> Self {
        Self {
            buffer_size: self.buffer_size,
            resizable: self.resizable,
            window_title: self.window_title.clone(),
            window_size: self.window_size,
            invert_y: self.invert_y
        }
    }
}

impl<'a> Default for Config<&'a str> {
    fn default() -> Self {
        Config {
            buffer_size: (0, 0),
            resizable: false,
            // :^)
            window_title: "Super Mini GL Framebufferer 3!",
            window_size: (600.0, 480.0),
            invert_y: true
        }
    }
}

impl Default for Config<String> {
    fn default() -> Self {
        Config {
            buffer_size: (0, 0),
            resizable: false,
            // :^)
            window_title: "Super Mini GL Framebufferer 3!".to_string(),
            window_size: (600.0, 480.0),
            invert_y: true
        }
    }
}
