# Mini GL "Framebuffer" (MGlFb)

[![Version](https://img.shields.io/crates/v/mini_gl_fb.svg)](https://crates.io/crates/mini_gl_fb)
[![Docs.rs](https://docs.rs/mini_gl_fb/badge.svg)](https://docs.rs/mini_gl_fb)

Mini GL Framebuffer provides an easy way to draw to a window from a pixel buffer. OpenGL
alternative to other easy framebuffer libraries.

It's designed to be dead simple and easy to remember when you just want to get something on the
screen ASAP. It's also built to be super flexible and easy to grow out of in case your project
gets serious (MGlFb exposes all of its internals so you can iteratively remove it as a
dependency over time!).

# Usage

```rust
extern crate mini_gl_fb;

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
    let buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
    fb.update_buffer(&buffer);
    fb.persist();
}
```

`fb.update_buffer` can be called as many times as you like and will redraw the screen each
time. You can bring your own timing mechanism, whether it's just `sleep(ms)` or something more
sophisticated.

# Support for quick and easy, simple input handling

Get access to mouse position and key inputs with no hassle. The following is extracted from the
Game of Life example:

```rust

let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);
let buffer = vec![[128u8, 0, 0, 255]; 800 * 600];

// ...

fb.glutin_handle_basic_input(|fb, input| {
    let elapsed = previous.elapsed().unwrap();
    let seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

    if input.key_is_down(VirtualKeyCode::Escape) {
        return false;
    }

    if input.mouse_is_down(MouseButton::Left) {
        // Mouse was pressed
        let (x, y) = input.mouse_pos;
        cells[y * WIDTH + x] = true;
        fb.update_buffer(&cells);
        // Give the user extra time to make something pretty each time they click
        previous = SystemTime::now();
        extra_delay = (extra_delay + 0.5).min(2.0);
    }

    // Each generation should stay on screen for half a second
    if seconds > 0.5 + extra_delay {
        previous = SystemTime::now();
        calculate_neighbors(&mut cells, &mut neighbors);
        make_some_babies(&mut cells, &mut neighbors);
        fb.update_buffer(&cells);
        extra_delay = 0.0;
    } else if input.resized {
        fb.redraw();
    }

    true
});
```

# Shader playground

Post process with GLSL shaders inspired by ShaderToy. This is a work in progress but there is
basic support for simple effects. The following is the default behavior but illustrates the
API:

```rust
fb.use_post_process_shader("
void main_image( out vec4 r_frag_color, in vec2 v_uv ) {
    r_frag_color = texture(u_buffer, v_uv);
}
");
```

# Get full access to glutin/winit for custom event handling

You can also "breakout" and get access to the underlying glutin window while still having easy
setup:

```rust
let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800.0, 600.0);

let GlutinBreakout {
    mut events_loop,
    gl_window,
    mut fb,
} = fb.glutin_breakout();

fb.update_buffer(/*...*/);
```

# Other features

 - Black and white rendering, specifying one byte per pixel
 - Hardware accelerated buffer scaling (window and buffer can have different sizes)
 - Exposes a function for creating a context with glutin in one line
 - Exposes a function for creating a VAO, VBO, quad, and blank texture in one line
 - If you don't want to use glutin you can **bring your own context** too!

See the [docs](https://docs.rs/mini_gl_fb/) for more info.

# Planned Features (depends on demand)

Feel free to open an issue if you have a suggestion or want to see one of these soon!

 - More kinds of simplified input handling methods

 - Enhanced and more thorough shader playground

 - Support for running ShaderToy examples directly (a conversion function)

 - Support for more textures, possibly actual OpenGL framebuffers for complex sequences of
    post processing

 - An HTML canvas-like API that allows drawing over your buffer???
