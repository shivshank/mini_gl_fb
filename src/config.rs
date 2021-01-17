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
    /// If this is true, the buffer will automatically resize if the event loop's events are being
    /// managed by mini_gl_fb.
    pub resizable: bool,
    pub window_title: S,
    pub window_size: (f64, f64)
}

impl<S: ToString + Clone> Clone for Config<S> {
    fn clone(&self) -> Self {
        Self {
            buffer_size: self.buffer_size,
            resizable: self.resizable,
            window_title: self.window_title.clone(),
            window_size: self.window_size
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
            window_size: (600.0, 480.0)
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
            window_size: (600.0, 480.0)
        }
    }
}
