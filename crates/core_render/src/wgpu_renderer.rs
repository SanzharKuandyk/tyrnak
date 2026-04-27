//! wgpu-based renderer — real GPU rendering implementation.
//!
//! Handles device initialization, surface management, render pipelines,
//! and frame presentation.

use crate::backend::{RenderBackend, RenderError};
use core_proto::RenderSnapshot;
use std::sync::Arc;

/// WGSL shader source for the basic triangle/quad pipeline.
const SHADER_SRC: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

/// Vertex data: position (x, y) + color (r, g, b).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x3,
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// A colored triangle — used for the initial proof-of-concept pipeline.
const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
];

/// GPU renderer using wgpu.
pub struct WgpuRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
    current_texture: Option<wgpu::SurfaceTexture>,
}

impl WgpuRenderer {
    /// Create a new wgpu renderer from an existing window.
    ///
    /// This is async because wgpu adapter/device creation is async.
    pub async fn new(
        window: Arc<dyn winit::window::Window>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| RenderError::Other(format!("Failed to create surface: {e}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RenderError::Other(format!("No suitable GPU adapter found: {e}")))?;

        tracing::info!("GPU adapter: {:?}", adapter.get_info());

        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("tyrnak_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::Other(format!("Failed to create device: {e}")))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        // Uniform buffer (4x4 matrix = 64 bytes)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("basic_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("basic_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Vertex buffer
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            size: (std::mem::size_of::<Vertex>() * TRIANGLE_VERTICES.len()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(TRIANGLE_VERTICES));

        // Write identity matrix as initial uniform
        let identity = glam::Mat4::IDENTITY;
        let cols = identity.to_cols_array();
        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&cols));

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            width,
            height,
            current_texture: None,
        })
    }

    /// Get a reference to the wgpu device.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get a reference to the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get the surface format.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    /// Update the view-projection matrix uniform.
    pub fn set_view_projection(&self, vp: &glam::Mat4) {
        let cols = vp.to_cols_array();
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&cols));
    }
}

impl RenderBackend for WgpuRenderer {
    fn begin_frame(&mut self) -> Result<(), RenderError> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                tracing::warn!("Surface texture is suboptimal, consider reconfiguring");
                texture
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(RenderError::SurfaceLost);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(RenderError::Other(
                    "Surface not available (timeout/occluded)".into(),
                ));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::Other("Surface validation error".into()));
            }
        };
        self.current_texture = Some(output);
        Ok(())
    }

    fn render_snapshot(&mut self, _snapshot: &RenderSnapshot) -> Result<(), RenderError> {
        let output = self
            .current_texture
            .as_ref()
            .ok_or_else(|| RenderError::Other("begin_frame not called".into()))?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..TRIANGLE_VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        if let Some(texture) = self.current_texture.take() {
            texture.present();
        }
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            tracing::debug!("Renderer resized to {width}x{height}");
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
