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
    var position = vec4<f32>(model.position, 1.0);

    let rotation_matrix = mat4x4<f32>(
        vec4(camera.view_proj[0].xyz, 0.0),
        vec4(camera.view_proj[1].xyz, 0.0),
        vec4(camera.view_proj[2].xyz, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );

    let transposed_view_proj: mat4x4<f32> = transpose(camera.view_proj);
    let cam_forward = normalize(vec4<f32>(transposed_view_proj[2].xyz, 0.0));
    
    position += cam_forward;

    let scale: f32 = 0.1;
    let scale_matrix = mat4x4<f32>(
        vec4<f32>(scale, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );    
    
    position = rotation_matrix * position;
    
    let translation = vec4<f32>(-0.9, -0.9, scale, 0.0) / scale;
    position += translation;
    
    position = scale_matrix * position;
    
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
