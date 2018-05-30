# Mini GL "Framebuffer"

Provides an easy way to draw a window from a pixel buffer. OpenGL alternative to other
easy "framebuffer" libraries.

Designed to be dead simple and easy to remember when you just want to get something on the
screen ASAP!

```rust
extern crate mini_gl_fb;

fn main() {
    let mut fb = mini_gl_fb::gotta_go_fast("Hello world!", 800, 600);
    let buffer = vec![[128u8, 0, 0, 255]; 800 * 600];
    fb.update_buffer(&buffer);
    fb.persist();
}
```

`fb.update_buffer` can be called as many times as you like and will redraw the screen each
time. You can bring your own timing mechanism, whether it's just `sleep(ms)` or something more
sophisticated.

# Planned Features

Listed in rough order of importance and ease (which are surprisingly correlated here!).

 - Bounds check on `update_buffer` which will currently segfault if you pass the wrong size.

 - Provide a way to break out of the `fb` object into the raw backing glutin window and event
    loop so that you can easily provide interactivity.

 - Shader playground. Add a method for using shadertoy-like fragment shaders to be applied to
    your submitted pixel data.

 - Some built in managed ways of getting interactivity, possibly such as a functional reactive
    style draw function that renders based on a provided Store struct. Other simpler
    alternatives include automatically drawing after running some event handlers and some
    methods for drawing at intervals (fixed/dynamic delta time).

 - Provide a way to bring your own context/window.

 - Fully replace and customize the vertex, geometry, and fragment shader, including adding your
    own uniforms (but probably not adding any vertex attributes).

 - Support for more textures, possibly actual OpenGL framebuffers for complex sequences of
    post processing. I am undecided on whether this is appropriate for this library.
