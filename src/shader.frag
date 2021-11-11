#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform texture2D WebMaterial_color;
layout(set = 0, binding = 2) uniform sampler WebMaterial_color_sampler;

void main() {
    vec4 color = texture(sampler2D(WebMaterial_color, WebMaterial_color_sampler), v_Uv);
    o_Target = color;
}