/// Coordinate axis shader.

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var position = vec4(model.position, 0.0) - vec4(camera.view_pos.xyz, 0.0);
    position = camera.view_proj * vec4<f32>(position.xyz, 1.0);

    var out: VertexOutput;
    out.clip_position = position;
    out.color = model.color;

    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let result = in.color.rgb;

    return vec4<f32>(result, 1.0);
}
