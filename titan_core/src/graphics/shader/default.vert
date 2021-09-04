#version 450

layout(binding = 0) uniform CameraUBO {
    mat4 projection;
    mat4 model;
    mat4 view;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 outColor;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = ubo.projection * ubo.view * ubo.model * vec4(position, 1.0);
    outColor = color;
}
