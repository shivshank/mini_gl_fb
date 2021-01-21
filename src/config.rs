use glutin::dpi::LogicalSize;

/// Configuration for "advanced" use cases, when [`gotta_go_fast`][crate::gotta_go_fast] isn't doing
/// what you need.
///
/// The following pattern is recommended when creating a config:
///
/// ```
/// use mini_gl_fb::config;
/// use mini_gl_fb::glutin::dpi::LogicalSize;
///
/// let config = config! {
///     /* specify whichever fields you need to set, for example: */
///     window_size: LogicalSize::new(100.0, 100.0),
///     resizable: true,
/// };
/// ```
///
/// Since `Config` is `#[non_exhaustive]`, you cannot construct it directly, and can only obtain one
/// from a trait like [`Default`]. The [`config!`][config] macro makes it much less tedious to
/// construct custom configs. See its documentation for more information.
///
/// If there's a config option you want to see or think is missing, please open an issue!
#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
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


/// The `config!` macro is intended to make it easy for us to add new fields in the future while
/// staying backwards-compatible. This is done by making [`Config`] `#[non_exhaustive]` but still
/// providing [`Default`], so that users can obtain the defaults and modify it to their liking. The
/// `config!` macro automates this, and makes custom configs just as easy as constructing `Config`
/// directly would be.
///
/// You can use the macro like this:
///
/// ```
/// # use mini_gl_fb::config;
/// #
/// let config = config! {
///     resizable: true,
///     invert_y: false
/// };
/// ```
///
/// As you can see, it's identical to a struct construction, you just use this macro in place of
/// `Config`. As such, it has a minimal impact on user code. That invocation roughly expands to:
///
/// ```
/// # use mini_gl_fb::Config;
/// #
/// let config = {
///     let mut config = Config::default();
///     config.resizable = true;
///     config.invert_y = false;
///     config
/// };
/// ```
///
/// This way, adding new fields will not affect existing code.
#[macro_export]
macro_rules! config {
    {$($k:ident: $v:expr),+$(,)?} => {{
        let mut config: ::mini_gl_fb::Config = ::std::default::Default::default();
        $(config.$k = $v;
        )*config
    }};
    {} => { <::mini_gl_fb::Config as ::std::default::Default>::default() }
}
