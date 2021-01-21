use breakout::{GlutinBreakout, BasicInput};

use rustic_gl;

use glutin::{ContextBuilder, WindowedContext, PossiblyCurrent};
use glutin::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};

use gl;
use gl::types::*;

use std::mem::size_of_val;
use glutin::window::WindowBuilder;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::platform::run_return::EventLoopExtRunReturn;
use glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, KeyboardInput};

/// Create a context using glutin given a configuration.
pub fn init_glutin_context<S: ToString, ET: 'static>(
    window_title: S,
    window_width: f64,
    window_height: f64,
    resizable: bool,
    event_loop: &EventLoop<ET>
) -> WindowedContext<PossiblyCurrent> {
    let window_size = LogicalSize::new(window_width, window_height);

    let window = WindowBuilder::new()
        .with_title(window_title.to_string())
        .with_inner_size(window_size)
        .with_resizable(resizable);

    let context: WindowedContext<PossiblyCurrent> = unsafe {
        ContextBuilder::new()
            .build_windowed(window, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    context
}

type VertexFormat = buffer_layout!([f32; 2], [f32; 2]);

/// Create the OpenGL resources needed for drawing to a buffer.
pub fn init_framebuffer(
    buffer_width: u32,
    buffer_height: u32,
    viewport_width: u32,
    viewport_height: u32,
    invert_y: bool
) -> Framebuffer {
    // The config takes the size in u32 because that's all that actually makes sense but since
    // OpenGL is from the Land of C where a Working Type System doesn't exist, we work with i32s
    let buffer_width = buffer_width as i32;
    let buffer_height = buffer_height as i32;
    let vp_width = viewport_width as i32;
    let vp_height = viewport_height as i32;

    let vertex_shader = rustic_gl::raw::create_shader(
        gl::VERTEX_SHADER,
        include_str!("./default_vertex_shader.glsl"),
    ).unwrap();
    let fragment_shader = rustic_gl::raw::create_shader(
        gl::FRAGMENT_SHADER,
        include_str!("./default_fragment_shader.glsl"),
    ).unwrap();

    let program = unsafe {
        build_program(&[
            Some(vertex_shader),
            Some(fragment_shader),
        ])
    };

    let sampler_location = unsafe {
        let location = gl::GetUniformLocation(program, b"u_buffer\0".as_ptr() as *const _);
        gl::UseProgram(program);
        gl::Uniform1i(location, 0);
        gl::UseProgram(0);
        location
    };

    let texture_format = (BufferFormat::RGBA, gl::UNSIGNED_BYTE);
    let texture = create_texture();

    let vao = rustic_gl::raw::create_vao().unwrap();
    let vbo = rustic_gl::raw::create_buffer().unwrap();

    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        VertexFormat::declare(0);

        let verts: [[f32; 2]; 12] = if invert_y {
            [
                [-1., 1.], [0., 1.], // top left
                [-1., -1.], [0., 0.], // bottom left
                [1., -1.], [1., 0.], // bottom right
                [1., -1.], [1., 0.], // bottom right
                [1., 1.], [1., 1.], // top right
                [-1., 1.], [0., 1.], // top left
            ]
        } else {
            [
                [-1., -1.], [0., 1.], // bottom left
                [1., 1.], [1., 0.], // top right
                [-1., 1.], [0., 0.], // top left
                [1., 1.], [1., 0.], // top right
                [-1., -1.], [0., 1.], // bottom left
                [1., -1.], [1., 1.], // bottom right
            ]
        };
        gl::BufferData(gl::ARRAY_BUFFER,
            size_of_val(&verts) as _,
            verts.as_ptr() as *const _,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // So the user doesn't have to consider alignment in their buffer
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    }

    Framebuffer {
        buffer_size: LogicalSize::new(buffer_width, buffer_height),
        vp_size: PhysicalSize::new(vp_width, vp_height),
        did_draw: false,
        inverted_y: invert_y,
        internal: FramebufferInternal {
            program,
            sampler_location,
            vertex_shader: Some(vertex_shader),
            geometry_shader: None,
            fragment_shader: Some(fragment_shader),
            texture,
            vao,
            vbo,
            texture_format,
        }
    }
}

/// Hides away the guts of the library.
///
/// Public methods are considered stable. Provides more advanced methods that may be difficult
/// or more complicated to use, but may be applicable to some use cases.
///
/// When `MiniGlFb` wraps a method from `Internal`, the documentation is provided there. If there
/// is no documentation and you find the method is non-trivial, it's a bug! Feel free to submit an
/// issue!
pub struct Internal {
    pub context: WindowedContext<PossiblyCurrent>,
    pub fb: Framebuffer,
}

impl Internal {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        self.fb.update_buffer(image_data);
        self.context.swap_buffers().unwrap();
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.context.window().set_resizable(resizable);
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.context.resize((width, height).into());
        self.fb.resize_viewport(width, height);
    }

    pub fn redraw(&mut self) {
        self.fb.redraw();
        self.context.swap_buffers().unwrap();
    }

    pub fn persist<ET: 'static>(&mut self, event_loop: &mut EventLoop<ET>) {
        self.persist_and_redraw(event_loop, false);
    }

    pub fn persist_and_redraw<ET: 'static>(&mut self, event_loop: &mut EventLoop<ET>, redraw: bool) {
        event_loop.run_return(|event, _, flow| {
            *flow = ControlFlow::Wait;

            let mut new_size = None;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(k) = input.virtual_keycode {
                            if k == VirtualKeyCode::Escape
                                    && input.state == ElementState::Pressed {
                                *flow = ControlFlow::Exit;
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        new_size = Some(physical_size);
                    }
                    _ => {},
                },
                _ => {},
            }

            if let Some(size) = new_size {
                self.resize_viewport(size.width, size.height);
                self.redraw();
            } else if redraw {
                self.fb.redraw();
                self.context.swap_buffers().unwrap();
            }
        });
    }

    pub fn glutin_handle_basic_input<ET: 'static, F: FnMut(&mut Framebuffer, &BasicInput) -> bool>(
        &mut self, event_loop: &mut EventLoop<ET>, mut handler: F
    ) {
        let mut previous_input: Option<BasicInput> = None;
        let mut input = BasicInput::default();

        event_loop.run_return(|event, _, flow| {
            let mut new_size = None;
            let mut new_mouse_pos: Option<PhysicalPosition<f64>> = None;

            // Copy the current states into the previous state for input
            for (_, val) in &mut input.keys {
                val.0 = val.1;
            }

            for (_, val) in &mut input.mouse {
                val.0 = val.1;
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(vk),
                            state,
                            ..
                        },
                        ..
                    } => {
                        let key = input.keys.entry(vk)
                            .or_insert((false, false));
                        key.1 = state == ElementState::Pressed;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        new_mouse_pos = Some(position);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let button = input.mouse.entry(button)
                            .or_insert((false, false));
                        button.1 = state == ElementState::Pressed;
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        input.modifiers = modifiers;
                    }
                    WindowEvent::Resized(logical_size) => {
                        new_size = Some(logical_size);
                    }
                    _ => {}
                },
                _ => {}
            }

            if let Some(size) = new_size {
                self.resize_viewport(size.width, size.height);
                input.resized = false;
            }

            if let Some(pos) = new_mouse_pos {
                let (x, y): (f64, f64) = pos.into();
                let x_scale = self.fb.buffer_size.width as f64 / (self.fb.vp_size.width as f64);
                let y_scale = self.fb.buffer_size.height as f64 / (self.fb.vp_size.height as f64);
                let mouse_pos = (
                    x * x_scale,
                    // use the OpenGL texture coordinate system instead of window coordinates
                    if self.fb.inverted_y {
                        self.fb.buffer_size.height as f64 - y * y_scale
                    } else {
                        y * y_scale
                    }
                );
                input.mouse_pos = mouse_pos;
            }

            if input.wait {
                *flow = ControlFlow::Wait;

                // handler only wants to be notified when the input changes
                if previous_input.as_ref().map_or(true, |p| *p != input) {
                    if !handler(&mut self.fb, &input) {
                        *flow = ControlFlow::Exit;
                    }
                }
            } else {
                // handler wants to be notified regardless
                if !handler(&mut self.fb, &input) {
                    *flow = ControlFlow::Exit;
                } else {
                    *flow = ControlFlow::Poll;
                }
            }

            previous_input = Some(input.clone());

            if self.fb.did_draw {
                self.context.swap_buffers().unwrap();
                self.fb.did_draw = false;
            }
        });
    }

    pub fn glutin_breakout(self) -> GlutinBreakout {
        GlutinBreakout {
            context: self.context,
            fb: self.fb,
        }
    }
}

/// Contains internal OpenGL things.
#[non_exhaustive]
#[derive(Debug)]
pub struct FramebufferInternal {
    pub program: GLuint,
    pub sampler_location: GLint,
    pub vertex_shader: Option<GLuint>,
    pub geometry_shader: Option<GLuint>,
    pub fragment_shader: Option<GLuint>,
    pub texture: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
    pub texture_format: (BufferFormat, GLenum),
}

/// The Framebuffer struct manages the framebuffer of a MGlFb window. Through this struct, you can
/// update the size and content of the buffer. Framebuffers are usually obtained through
/// [`MiniGlFb::glutin_breakout`][crate::MiniGlFb::glutin_breakout], but they're also returned by
/// [`init_framebuffer`].
///
/// # Basic usage
/// Firstly, one of the most important things to do when managing a Framebuffer manually is to make
/// sure that whenever the window is resized, the Framebuffer is the first to know. Usually, this is
/// handled for you by [`MiniGlFb`][crate::MiniGlFb], but that isn't the case when using the
/// [`GlutinBreakout`].
///
/// Whenever you receive a resize event for your window, make sure to call
/// [`Framebuffer::resize_viewport`] with the new physical dimensions of your window. You can also
/// figure out some logical dimensions and call [`Framebuffer::resize_buffer`] too.
///
/// Additionally, when managing multiple framebuffers at once, you should make sure to call
/// [`GlutinBreakout::make_current`] when appropriate, before calling any `Framebuffer` methods.
/// Forgetting to call `make_current` can cause OpenGL to get confused and draw to the wrong window,
/// which is probably not what you want.
#[derive(Debug)]
pub struct Framebuffer {
    /// The logical size of the buffer. When you update the buffer via
    /// [`update_buffer`][Framebuffer::update_buffer], it is expected to contain
    /// `buffer_size.width * buffer_size.height` pixels.
    pub buffer_size: LogicalSize<i32>,

    /// The physical size of the viewport. This should always be kept up to date with the size of
    /// the window, and there is no reason to set it otherwise unless you're drawing multiple
    /// buffers to one window or something funky like that.
    pub vp_size: PhysicalSize<i32>,

    /// This is set to `true` every time [`draw`][Framebuffer::draw] is called. (or, by extension,
    /// [`update_buffer`][Framebuffer::update_buffer])
    ///
    /// It's safe to set this to `false` afterwards, it's just a flag to let you know if code you're
    /// calling into has updated the buffer or not.
    pub did_draw: bool,

    /// True if the origin should be the bottom left of the screen instead of the top left. For
    /// historical reasons, this is the default. This should only be configured by changing the
    /// [`Config`][crate::Config] passed to [`get_fancy`][crate::get_fancy].
    pub inverted_y: bool,

    /// Contains internal OpenGL things.
    ///
    /// Accessing fields directly is not the intended usage. If a feature is missing please open an
    /// issue. The fields are public, however, so that while you are waiting for a feature to be
    /// exposed, if you need something in a pinch you can dig in easily and make it happen.
    ///
    /// The internal fields may change.
    pub internal: FramebufferInternal
}

impl Framebuffer {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        // Check the length of the passed slice so this is actually a safe method.
        let (format, kind) = self.internal.texture_format;
        let expected_size_in_bytes = size_of_gl_type_enum(kind)
            * format.components()
            * self.buffer_size.width as usize
            * self.buffer_size.height as usize;
        let actual_size_in_bytes = size_of_val(image_data);
        if actual_size_in_bytes != expected_size_in_bytes {
            panic!(
                "Expected a buffer of {} bytes, instead recieved one of {} bytes",
                expected_size_in_bytes,
                actual_size_in_bytes
            );
        }
        self.draw(|fb| {
            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    fb.buffer_size.width,
                    fb.buffer_size.height,
                    0,
                    format as GLenum,
                    kind,
                    image_data.as_ptr() as *const _,
                );
            }
        })
    }

    pub fn use_vertex_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.internal.vertex_shader, gl::VERTEX_SHADER, source);
        self.relink_program();
    }

    pub fn use_fragment_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.internal.fragment_shader, gl::FRAGMENT_SHADER, source);
        self.relink_program();
    }

    pub fn use_post_process_shader(&mut self, source: &str) {
        let source = make_post_process_shader(source);
        self.use_fragment_shader(&source);
    }

    pub fn use_geometry_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.internal.geometry_shader, gl::GEOMETRY_SHADER, source);
        self.relink_program();
    }

    pub fn use_grayscale_shader(&mut self) {
        self.use_fragment_shader(include_str!("./grayscale_fragment_shader.glsl"));
    }

    pub fn change_buffer_format<T: ToGlType>(
        &mut self,
        format: BufferFormat,
    ) {
        self.internal.texture_format = (format, T::to_gl_enum());
    }

    pub fn resize_buffer(&mut self, buffer_width: u32, buffer_height: u32) {
        self.buffer_size = LogicalSize::new(buffer_width, buffer_height).cast();
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.vp_size = PhysicalSize::new(width, height).cast();
    }

    pub fn redraw(&mut self) {
        self.draw(|_| {})
    }

    /// Draw the quad to the active context. Optionally issue other commands after binding
    /// everything but before drawing it.
    ///
    /// You probably want [`redraw`][Framebuffer::redraw] (equivalent to `.draw(|_| {})`).
    pub fn draw<F: FnOnce(&Framebuffer)>(&mut self, f: F) {
        unsafe {
            gl::Viewport(0, 0, self.vp_size.width, self.vp_size.height);
            gl::UseProgram(self.internal.program);
            gl::BindVertexArray(self.internal.vao);
            gl::ActiveTexture(0);
            gl::BindTexture(gl::TEXTURE_2D, self.internal.texture);
            f(self);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
        self.did_draw = true;
    }

    pub fn relink_program(&mut self) {
        unsafe {
            gl::DeleteProgram(self.internal.program);
            self.internal.program = build_program(&[
                self.internal.vertex_shader.clone(),
                self.internal.fragment_shader.clone(),
                self.internal.geometry_shader.clone(),
            ]);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum BufferFormat {
    R = gl::RED,
    RG = gl::RG,
    RGB = gl::RGB,
    BGR = gl::BGR,
    RGBA = gl::RGBA,
    BGRA = gl::BGRA,
}

impl BufferFormat {
    fn components(&self) -> usize {
        use self::BufferFormat::*;
        match self {
            R => 1,
            RG => 2,
            RGB | BGR => 3,
            RGBA | BGRA => 4,
        }
    }
}

pub trait ToGlType {
    fn to_gl_enum() -> GLenum;
}

macro_rules! impl_ToGlType {
    (
        $(
            $t:ty, $gl_type:expr
        ),+,
    ) => {
        $(
            impl ToGlType for $t {
                fn to_gl_enum() -> GLenum {
                    $gl_type
                }
            }
        )+
    }
}

impl_ToGlType!(
    u8, gl::UNSIGNED_BYTE,
    i8, gl::BYTE,
);

fn size_of_gl_type_enum(gl_enum: GLenum) -> usize {
    match gl_enum {
        gl::UNSIGNED_BYTE | gl::BYTE => 1,
        _ => panic!("Must pass a GL enum representing a type"),
    }
}

fn create_texture() -> GLuint {
    unsafe {
        let mut tex = 0;
        gl::GenTextures(1, &mut tex);
        if tex == 0 {
            // TODO
            panic!();
        }
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::BindTexture(gl::TEXTURE_2D, 0);
        tex
    }
}

fn make_post_process_shader(source: &str) -> String {
    format!(
        "
            #version 330 core

            in vec2 v_uv;

            out vec4 r_frag_color;

            uniform sampler2D u_buffer;

            {}

            void main() {{
                main_image(r_frag_color, v_uv);
            }}
        ",
        source,
    )
}

fn rebuild_shader(shader: &mut Option<GLuint>, kind: GLenum, source: &str) {
    if let Some(shader) = *shader {
        unsafe {
            gl::DeleteShader(shader);
        }
    }
    let compilation_result = rustic_gl::raw::create_shader(kind, source);
    match compilation_result {
        Ok(gl_id) => {
            *shader = Some(gl_id);
        },
        Err(rustic_gl::error::GlError::ShaderCompilation(info)) => {
            if let Some(log) = info {
                panic!("Shader compilation failed with the following information: {}", log);
            } else {
                panic!("Shader compilation failed without any information.")
            }
        },
        Err(err) => {
            panic!("An error occured while compiling shader: {}", err);
        }
    }
}

unsafe fn build_program(shaders: &[Option<GLuint>]) -> GLuint {
    let program = rustic_gl::raw::create_program()
        .unwrap();
    for shader in shaders.iter() {
        if let &Some(shader) = shader {
            gl::AttachShader(program, shader);
        }
    }
    gl::LinkProgram(program);
    rustic_gl::raw::get_link_status(program)
        .unwrap();
    for shader in shaders {
        if let &Some(shader) = shader {
            gl::DetachShader(program, shader);
        }
    }
    program
}
