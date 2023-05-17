use crate::renderer::vertex::Vertex;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AxisVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex for AxisVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<AxisVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub const AXIS_VERTICES: [AxisVertex; 36] = [
    // X-Axis 1
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, -0.025, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.025, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, -0.05, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [1.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.05, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    // X-Axis 2
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.0, -0.025],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.0, 0.025],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.0, -0.05],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [1.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0], // Red
    },
    AxisVertex {
        position: [0.9, 0.0, 0.05],
        color: [1.0, 0.0, 0.0], // Red
    },
    // Y-Axis 1
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [-0.025, 0.9, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.025, 0.9, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [-0.05, 0.9, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.05, 0.9, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    // Y-Axis 2
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 0.9, -0.025],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 0.9, 0.025],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 0.9, -0.05],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0], // Green
    },
    AxisVertex {
        position: [0.0, 0.9, 0.05],
        color: [0.0, 1.0, 0.0], // Green
    },
    // Z-Axis 1
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, -0.025, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, 0.025, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, -0.05, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, 0.05, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    // Z-Axis 2
    AxisVertex {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [-0.025, 0.0, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.025, 0.0, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [-0.05, 0.0, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0], // Blue
    },
    AxisVertex {
        position: [0.05, 0.0, 0.9],
        color: [0.0, 0.0, 1.0], // Blue
    },
];
