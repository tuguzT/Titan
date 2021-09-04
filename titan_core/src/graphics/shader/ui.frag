#version 450

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 outColor;

layout(binding = 0, set = 0) uniform sampler2D fontTexture;

void main() {
    outColor = color * texture(fontTexture, uv);
}
