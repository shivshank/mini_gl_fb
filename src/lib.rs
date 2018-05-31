//! Hardware accelerated library inspired by minifb and friends.
//!
//! Powered by OpenGL. Default context is provided by glutin, but

extern crate glutin;
#[macro_use]
extern crate rustic_gl;
extern crate gl;

use glutin::GlContext;

use gl::types::*;

use std::ptr::null;

type VertexFormat = buffer_layout!([i8; 2], [u8; 2]);

pub fn gotta_go_fast<S: ToString>(window_title: S, window_width: i32, window_height: i32) -> Framebuffer {
    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title(window_title.to_string())
        .with_dimensions(window_width as _, window_height as _);
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

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
        let location = gl::GetUniformLocation(program, b"u_tex0\0".as_ptr() as *const _);
        gl::UseProgram(program);
        gl::Uniform1i(location, 0);
        gl::UseProgram(0);
        location
    };

    let texture_format = (BufferFormat::RGBA, gl::UNSIGNED_BYTE);
    let texture = create_texture(window_width, window_height, texture_format.0, texture_format.1);

    let vao = create_vao().unwrap();
    let vbo = create_gl_buffer().unwrap();

    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        VertexFormat::declare(0);
        let verts = [
            [-1i8, 1], [0, 0],
            [-1, -1], [0, 1],
            [1, -1], [1, 1],
            [1, -1], [1, 1],
            [1, 1], [1, 0],
            [-1, 1], [0, 0],
        ];
        gl::BufferData(gl::ARRAY_BUFFER, (verts.len() * 2) as _, verts.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    Framebuffer {
        width: window_width,
        height: window_height,
        events_loop,
        gl_window,
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

pub struct Framebuffer {
    width: i32,
    height: i32,
    events_loop: glutin::EventsLoop,
    gl_window: glutin::GlWindow,
    program: GLuint,
    sampler_location: GLint,
    vertex_shader: Option<GLuint>,
    geometry_shader: Option<GLuint>,
    fragment_shader: Option<GLuint>,
    texture: GLuint,
    vao: GLuint,
    vbo: GLuint,
    texture_format: (BufferFormat, GLenum),
}

impl Framebuffer {
    pub fn update_buffer<T>(&mut self, image_data: &[T]) {
        // TODO: Safety check on the length of the passed slice so this is actually a safe method
        self.draw(|fb| {
            unsafe {
                let (format, kind) = fb.texture_format;
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    fb.width,
                    fb.height,
                    0,
                    format as GLenum,
                    kind,
                    image_data.as_ptr() as *const _,
                );
            }
        })
    }

    pub fn change_buffer_format<T: ToGlType>(&mut self, format: BufferFormat) {
        self.texture_format = (format, T::to_gl_enum());
    }

    /// Keeps the window open until the user closes it.
    pub fn persist(&mut self) {
        self.persist_and_redraw(false);
    }

    /// Persist implementation.
    ///
    /// When redraw is true, redraws as fast as possible. This function is primarily for debugging.
    pub fn persist_and_redraw(&mut self, redraw: bool) {
        let mut running = true;
        while running {
            self.events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent{ event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        _ => {},
                    },
                    _ => {},
                }
            });
            if redraw {
                self.draw(|_| {});
            }
        }
    }

    fn draw<F: FnOnce(&Framebuffer)>(&mut self, f: F) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            f(self);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);

            self.gl_window.swap_buffers().unwrap();
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

fn create_texture(width: i32, height: i32, format: BufferFormat, buffer_kind: GLenum) -> GLuint {
    unsafe {
        let mut tex = 0;
        gl::GenTextures(1, &mut tex);
        if tex == 0 {
            // TODO
            panic!();
        }
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _, width, height, 0, format as GLenum, buffer_kind, null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::BindTexture(gl::TEXTURE_2D, 0);
        tex
    }
}

fn create_vao() -> Option<GLuint> {
    unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        if vao == 0 {
            return None;
        }
        Some(vao)
    }
}

fn create_gl_buffer() -> Option<GLuint> {
    unsafe {
        let mut b = 0;
        gl::GenBuffers(1, &mut b);
        if b == 0 {
            return None;
        }
        Some(b)
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
