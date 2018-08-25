use breakout::{GlutinBreakout, BasicInput};

use rustic_gl;

use glutin::{
    EventsLoop,
    WindowBuilder,
    ContextBuilder,
    GlWindow,
    GlContext,
    Event,
    WindowEvent,
    VirtualKeyCode,
    ElementState,
};
use glutin::dpi::{LogicalSize, LogicalPosition};

use gl;
use gl::types::*;

use std::mem::size_of_val;
use std::collections::HashMap;

/// Create a context using glutin given a configuration.
pub fn init_glutin_context<S: ToString>(
    window_title: S,
    window_width: f64,
    window_height: f64,
    resizable: bool,
) -> (EventsLoop, GlWindow) {
    let window_size = LogicalSize::new(window_width, window_height);

    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title(window_title.to_string())
        .with_dimensions(window_size)
        .with_resizable(resizable);

    let context = ContextBuilder::new();
    let gl_window = GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    (events_loop, gl_window)
}

type VertexFormat = buffer_layout!([f32; 2], [f32; 2]);

/// Create the OpenGL resources needed for drawing to a buffer.
pub fn init_framebuffer(
    buffer_width: u32,
    buffer_height: u32,
    viewport_width: u32,
    viewport_height: u32
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

        let verts: [[f32; 2]; 12] = [
            [-1., 1.], [0., 1.], // top left
            [-1., -1.], [0., 0.], // bottom left
            [1., -1.], [1., 0.], // bottom right
            [1., -1.], [1., 0.], // bottom right
            [1., 1.], [1., 1.], // top right
            [-1., 1.], [0., 1.], // top left
        ];
        use std::mem::size_of_val;
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
        buffer_width,
        buffer_height,
        vp_width,
        vp_height,
        program,
        sampler_location,
        vertex_shader: Some(vertex_shader),
        geometry_shader: None,
        fragment_shader: Some(fragment_shader),
        texture,
        vao,
        vbo,
        texture_format,
        did_draw: false,
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
    pub events_loop: EventsLoop,
    pub gl_window: GlWindow,
    pub fb: Framebuffer,
}

impl Internal {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        self.fb.update_buffer(image_data);
        self.gl_window.swap_buffers().unwrap();
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.gl_window.set_resizable(resizable);
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.gl_window.resize((width, height).into());
        self.fb.resize_viewport(width, height);
    }

    pub fn is_running(&mut self) -> bool {
        let mut running = true;
        let mut resized = None;
        self.events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(k) = input.virtual_keycode {
                            if k == VirtualKeyCode::Escape
                                    && input.state == ElementState::Released {
                                running = false;
                            }
                        }
                    }
                    WindowEvent::Resized(logical_size) => {
                        resized = Some(logical_size);
                    }
                    _ => {},
                },
                _ => {},
            }
        });
        if let Some(size) = resized {
            let dpi_factor = self.gl_window.get_hidpi_factor();
            let (x, y) = size.to_physical(dpi_factor).into();
            self.resize_viewport(x, y);
        }
        running
    }

    pub fn redraw(&mut self) {
        self.fb.redraw();
        self.gl_window.swap_buffers().unwrap();
    }

    pub fn persist(&mut self) {
        self.persist_and_redraw(false);
    }

    pub fn persist_and_redraw(&mut self, redraw: bool) {
        let mut running = true;
        while running {
            let mut new_size = None;
            self.events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(k) = input.virtual_keycode {
                                if k == VirtualKeyCode::Escape
                                        && input.state == ElementState::Released {
                                    running = false;
                                }
                            }
                        }
                        WindowEvent::Resized(logical_size) => {
                            new_size = Some(logical_size);
                        }
                        _ => {},
                    },
                    _ => {},
                }
            });
            if let Some(size) = new_size {
                let dpi_factor = self.gl_window.get_hidpi_factor();
                let (x, y) = size.to_physical(dpi_factor).into();
                self.resize_viewport(x, y);
                // TODO: We should store window size in BasicInput, but as what type? OpenGL wants
                // integer window sizes but we also have to deal with physical vs. logical size
                self.redraw();
            } else {
                if redraw {
                    self.fb.redraw();
                    self.gl_window.swap_buffers().unwrap();
                }
            }
        }
    }

    pub fn glutin_handle_basic_input<F: FnMut(&mut Framebuffer, &BasicInput) -> bool>(
        &mut self, mut handler: F
    ) {
        let mut running = true;
        let mut input = BasicInput {
            // Not sure how to set mouse pos at start
            mouse_pos: (0.0, 0.0),
            mouse: HashMap::new(),
            keys: HashMap::new(),
            modifiers: Default::default(),
            resized: false,
        };
        while running {
            let mut new_size = None;
            let mut new_mouse_pos: Option<LogicalPosition> = None;
            self.events_loop.poll_events(|event| {
                // Copy the current states into the previous state for input
                for (_, val) in &mut input.keys {
                    val.0 = val.1;
                }
                for (_, val) in &mut input.mouse {
                    val.0 = val.1;
                }
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::KeyboardInput { input: event_input, .. } => {
                            if let Some(vk) = event_input.virtual_keycode {
                                let key = input.keys.entry(vk)
                                    .or_insert((false, false));
                                key.1 = event_input.state == ElementState::Pressed;
                            }
                            input.modifiers = event_input.modifiers;
                        }
                        WindowEvent::CursorMoved { position, modifiers, ..} => {
                            new_mouse_pos = Some(position);
                            input.modifiers = modifiers;
                        }
                        WindowEvent::MouseInput { state, button, modifiers, .. } => {
                            let button = input.mouse.entry(button)
                                .or_insert((false, false));
                            button.1 = state == ElementState::Pressed;
                            input.modifiers = modifiers;
                        }
                        WindowEvent::Resized(logical_size) => {
                            new_size = Some(logical_size);
                        }
                        _ => {},
                    },
                    _ => {},
                }
            });
            if let Some(size) = new_size {
                let dpi_factor = self.gl_window.get_hidpi_factor();
                let (x, y) = size.to_physical(dpi_factor).into();
                self.resize_viewport(x, y);
                input.resized = false;
            }
            if let Some(pos) = new_mouse_pos {
                let dpi_factor = self.gl_window.get_hidpi_factor();
                let (x, y): (f64, f64) = pos.to_physical(dpi_factor).into();
                let x_scale = self.fb.buffer_width as f64 / (self.fb.vp_width as f64);
                let y_scale = self.fb.buffer_height as f64 / (self.fb.vp_height as f64);
                let mouse_pos = (
                    x * x_scale,
                    // use the OpenGL texture coordinate system instead of window coordinates
                    self.fb.buffer_height as f64 - y * y_scale
                );
                input.mouse_pos = mouse_pos;
            }

            if running {
                running = handler(&mut self.fb, &input);
                if self.fb.did_draw {
                    self.gl_window.swap_buffers().unwrap();
                    self.fb.did_draw = false;
                }
            }
        }
    }

    pub fn glutin_breakout(self) -> GlutinBreakout {
        GlutinBreakout {
            events_loop: self.events_loop,
            gl_window: self.gl_window,
            fb: self.fb,
        }
    }
}

/// Provides the drawing functionality.
///
/// You can get direct access by using a breakout function, such as breakout_glutin.
///
/// # Disclaimer:
///
/// Accessing fields directly is not the intended usage. If a feature is missing please open an
/// issue. The fields are public, however, so that while you are waiting for a feature to be
/// exposed, if you need something in a pinch you can dig in easily and make it happen.
///
/// The internal fields may change.
///
/// TODO: Possibly create a FramebufferInternal struct?
pub struct Framebuffer {
    pub buffer_width: i32,
    pub buffer_height: i32,
    pub vp_width: i32,
    pub vp_height: i32,
    pub program: GLuint,
    pub sampler_location: GLint,
    pub vertex_shader: Option<GLuint>,
    pub geometry_shader: Option<GLuint>,
    pub fragment_shader: Option<GLuint>,
    pub texture: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
    pub texture_format: (BufferFormat, GLenum),
    pub did_draw: bool,
}

impl Framebuffer {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        // Check the length of the passed slice so this is actually a safe method.
        let (format, kind) = self.texture_format;
        let expected_size_in_bytes = size_of_gl_type_enum(kind)
            * format.components()
            * self.buffer_width as usize
            * self.buffer_height as usize;
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
                    fb.buffer_width,
                    fb.buffer_height,
                    0,
                    format as GLenum,
                    kind,
                    image_data.as_ptr() as *const _,
                );
            }
        })
    }

    pub fn use_vertex_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.vertex_shader, gl::VERTEX_SHADER, source);
        self.relink_program();
    }

    pub fn use_fragment_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.fragment_shader, gl::FRAGMENT_SHADER, source);
        self.relink_program();
    }

    pub fn use_post_process_shader(&mut self, source: &str) {
        let source = make_post_process_shader(source);
        self.use_fragment_shader(&source);
    }

    pub fn use_geometry_shader(&mut self, source: &str) {
        rebuild_shader(&mut self.geometry_shader, gl::GEOMETRY_SHADER, source);
        self.relink_program();
    }

    pub fn use_grayscale_shader(&mut self) {
        self.use_fragment_shader(include_str!("./grayscale_fragment_shader.glsl"));
    }

    pub fn change_buffer_format<T: ToGlType>(
        &mut self,
        format: BufferFormat,
    ) {
        self.texture_format = (format, T::to_gl_enum());
    }

    pub fn resize_buffer(&mut self, buffer_width: u32, buffer_height: u32) {
        self.buffer_width = buffer_width as _;
        self.buffer_height = buffer_height as _;
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.vp_width = width as _;
        self.vp_height = height as _;
    }

    pub fn redraw(&mut self) {
        self.draw(|_| {})
    }

    /// Draw the quad to the active context. Optionally issue other commands after binding
    /// everything but before
    ///
    /// You probably want `redraw` (equivalent to `.draw(|_| {})`).
    pub fn draw<F: FnOnce(&Framebuffer)>(&mut self, f: F) {
        unsafe {
            gl::Viewport(0, 0, self.vp_width, self.vp_height);
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
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
            gl::DeleteProgram(self.program);
            self.program = build_program(&[
                self.vertex_shader,
                self.fragment_shader,
                self.geometry_shader,
            ]);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
