# Mini GL "Framebuffer"

Provides an easy way to draw a window from a pixel buffer. OpenGL alternative to other
easy framebuffer libraries.

Designed to be dead simple and easy to remember when you just want to get something on the
screen ASAP!

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

# Get full access to glutin for custom event handling

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

 - Hardware accelerated buffer scaling (window and buffer can have different sizes)
 - Exposes a function for creating a context with glutin in one line
 - Exposes a function for creating a VAO, VBO, quad, and blank texture in one line
 - If you don't want to use glutin you can **bring your own context** too!

See the docs for more info.

# Planned Features

Listed in rough order of importance and ease (which are surprisingly correlated here!).

 - Shader playground. Add a method for using shadertoy-like fragment shaders to be applied to
    your submitted pixel data.

 - Some built in managed ways of getting interactivity, possibly such as a functional reactive
    style draw function that renders based on a provided Store struct. Other simpler
    alternatives include automatically drawing after running some event handlers and some
    methods for drawing at intervals (fixed/dynamic delta time).

 - Fully replace and customize the vertex, geometry, and fragment shader, including adding your
    own uniforms (but probably not adding any vertex attributes).

 - Support for more textures, possibly actual OpenGL framebuffers for complex sequences of
    post processing. I am undecided on whether this is appropriate for this library.
