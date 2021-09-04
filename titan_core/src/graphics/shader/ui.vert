#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec2 outUV;

layout(push_constant) uniform PushConstants {
    vec2 screen_size;
} pushConstants;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = vec4(2.0 * position / pushConstants.screen_size - 1.0, 0.0, 1.0);
    outColor = color;
    outUV = uv;
}
