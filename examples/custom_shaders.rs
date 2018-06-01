extern crate mini_gl_fb;

/// Geometry shaders allow you to procedurally generate new geometry from the vertex data.
///
/// This shader takes the two triangles submitted by mini_gl_fb and turns them into a circle!
const geometry_source: &str = r"
    #version 330 core

    layout (triangles) in;
    layout (triangle_strip, max_vertices = 8) out;

    in vec2 v_uv[];

    out vec2 g_uv;

    vec2 midpoint(vec2 a, vec2 b) {
        return (a + b) / 2.0;
    }

    void main() {
        // n.b., the way we do this relies almost entirely on what we know about the internals of
        // mini_gl_fb, so you will need to refer to the source if you want to play with this stuff.

        vec4 center = vec4(0.0, 0.0, 0.0, 1.0);
        // for the first triangle, the second (index 1) vertex is the top left
        // for the second triangle, it is the bottom right. we will treat that like a direction!
        vec2 dir = gl_in[1].gl_Position.xy;

        // we are turning each triangle into 4 triangles, which we output in triangle strips
        // (remember this shader will get run twice, once for each input triangle)

        vec4 top_left = vec4(normalize(vec2(dir.x, -dir.y)), 0.0, 1.0);
        vec4 left = vec4(sign(dir.x), 0.0, 0.0, 1.0);
        vec4 bottom_left = vec4(normalize(dir), 0.0, 1.0);
        vec4 bottom = vec4(0.0, sign(dir.y), 0.0, 1.0);
        vec4 bottom_right = vec4(normalize(vec2(-dir.x, dir.y)), 0.0, 1.0);

        gl_Position = top_left;
        g_uv = v_uv[0];
        EmitVertex();

        gl_Position = left;
        g_uv = midpoint(v_uv[0], v_uv[1]);
        EmitVertex();

        gl_Position = center;
        g_uv = midpoint(v_uv[0], v_uv[2]);
        EmitVertex();

        gl_Position = bottom_left;
        g_uv = v_uv[1];
        EmitVertex();

        EndPrimitive();

        gl_Position = bottom_left;
        g_uv = v_uv[1];
        EmitVertex();

        gl_Position = bottom;
        g_uv = midpoint(v_uv[1], v_uv[2]);
        EmitVertex();

        gl_Position = center;
        g_uv = midpoint(v_uv[0], v_uv[2]);
        EmitVertex();

        gl_Position = bottom_right;
        g_uv = v_uv[2];
        EmitVertex();

        EndPrimitive();
    }
";

const fragment_source: &str = r"
    #version 330 core

    in vec2 g_uv;

    out vec4 frag_color;

    // this is the texture uploaded by calls to `update_buffer`
    uniform sampler2D u_tex0;

    void main() {
        vec4 sample = texture(u_tex0, g_uv);
        vec4 color;
        if (sample.r == 1.0) {
            color = sample;
        } else {
            // render the uv coords as color otherwise
            color = vec4(g_uv, 0.0, 1.0);
        }
        frag_color = color;
    }
";

extern crate gl;

fn main() {
    let width = 800;
    let height = 600;

    let mut fb = mini_gl_fb::gotta_go_fast("Hello shaders!", width, height);

    let mut buffer = vec![[128u8, 0, 0, 255]; (width * height) as usize];
    // let's write a red line into the buffer roughly along the diagonal (misses many pixels)
    for i in 0..100 {
        let j = i as f32 / 100.0;
        let index = ((width as f32) * j * ((height as f32) + 1.0)).floor() as usize;
        buffer[index] = [255, 0, 0, 255];
    }

    // Let's keep using the default vertex shader
    // fb.use_vertex_shader(...);
    fb.use_geometry_shader(geometry_source);
    fb.use_fragment_shader(fragment_source);

    fb.update_buffer(&buffer);

    fb.persist_and_redraw(true);
}
