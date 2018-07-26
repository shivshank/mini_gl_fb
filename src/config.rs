/// Configuration for "advanced" use cases, when `gotta_go_fast` isn't doing what you need.
///
/// The following pattern is reccomended when creating a config:
///
/// ```rust
/// let config = Config {
///     /* specify whichever fields you need to set, for example: */
///     window_size: (100.0, 100.0),
///     resizable: false,
///     .. Default::default()
/// }
/// ```
///
/// To streamline this pattern and save you imports, see the `get_fancy!` macro.
///
/// If there's a config option you want to see or think is missing, please open an issue!
pub struct Config<S: ToString> {
    /// Sets the scale of the buffer. The buffer will automatically scale to the size of the
    /// window. By default this will be the same size as the window_size.
    pub buffer_size: (u32, u32),
    pub resizable: bool,
    pub window_title: S,
    pub window_size: (f64, f64),
}

impl Default for Config<String> {
    fn default() -> Self {
        Config {
            buffer_size: (0, 0),
            resizable: false,
            // :^)
            window_title: "Super Mini GL Framebufferer 3!".to_string(),
            window_size: (600.0, 480.0),
        }
    }
}
