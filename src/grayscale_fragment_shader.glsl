#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform sampler2D u_buffer;

void main() {
    frag_color = texture(u_buffer, v_uv).rrra;
}
