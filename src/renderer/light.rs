#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32, // Due to uniforms requiring 16 byte (4 float) spacing.
    pub color: [f32; 3],
    pub _padding_2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
            _padding_2: 0,
        }
    }
}
