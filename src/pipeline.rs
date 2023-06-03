use std::num::NonZeroU64;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferBindingType, BufferDescriptor,
};
use winit::dpi::LogicalSize;

use crate::{context::RenderContext, surface::SurfaceRenderer};

const VS_MAIN: &str = "vs_main";
const FS_MAIN: &str = "fs_main";

pub struct WindowPipelineRegistry {
    pub copy_pipeline: wgpu::RenderPipeline,
    pub copy_bind_group: wgpu::BindGroup,

    pub copy_back_pipeline: wgpu::RenderPipeline,
    pub copy_back_bind_group: wgpu::BindGroup,

    pub blur_rect_pipeline: wgpu::RenderPipeline,
    pub blur_rect_bind_group: wgpu::BindGroup,

    rect_uniform: wgpu::Buffer,
}

impl WindowPipelineRegistry {
    pub fn set_rect(&self, rect: egui::epaint::Rect, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.rect_uniform,
            0,
            bytemuck::cast_slice(&[rect.left(), rect.top(), rect.right(), rect.bottom()]),
        );
    }
}

pub struct PipelineRegistry {}

impl PipelineRegistry {
    pub fn create_window_size_buffer(
        context: &RenderContext,
        size: LogicalSize<f32>,
    ) -> wgpu::Buffer {
        let (device, ..) = context.get_device();

        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Window Size Buffer"),
            contents: bytemuck::cast_slice(&[size.width, size.height]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        })
    }

    fn create_copy_pipeline(
        context: &RenderContext,
        surface: &SurfaceRenderer,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
        let (device, _queue) = context.get_device();
        let shader = &context.shader_copy_texture;

        let copy_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                // label: Some("window_texture_bind_group_layout"),
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let copy_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &copy_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            // label: Some("diffuse_bind_group"),
            label: None,
        });

        let copy_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                // label: Some("Render Pipeline Layout"),
                label: None,
                bind_group_layouts: &[&copy_bind_group_layout], // NEW!
                push_constant_ranges: &[],
            });

        let copy_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            // label: Some("Render Pipeline"),
            label: None,
            layout: Some(&copy_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: VS_MAIN,
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: FS_MAIN,
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface.format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        return (copy_pipeline, copy_bind_group);
    }

    pub fn generate_for_window(
        context: &RenderContext,
        surface: &SurfaceRenderer,
        view: &wgpu::TextureView,
        back_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        back_sampler: &wgpu::Sampler,
        window_size_uniform: &wgpu::Buffer,
    ) -> WindowPipelineRegistry {
        let (device, queue) = context.get_device();

        let (copy_pipeline, copy_bind_group) =
            Self::create_copy_pipeline(context, surface, view, sampler);

        let (copy_back_pipeline, copy_back_bind_group) =
            Self::create_copy_pipeline(context, surface, back_view, back_sampler);

        let size: [f32; 2] = surface.logical_size().cast::<f32>().into();
        queue.write_buffer(window_size_uniform, 0, bytemuck::cast_slice(&size));

        let rect_uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let blur_rect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blur_rect"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(8),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let blur_rect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &blur_rect_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: window_size_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rect_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let blur_rect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blur_rect"),
                bind_group_layouts: &[&blur_rect_bind_group_layout],
                push_constant_ranges: &[],
            });

        let blur_rect_shader = &context.shader_blur_rect;

        let blur_rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blur_rect"),
            layout: Some(&blur_rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: blur_rect_shader,
                entry_point: VS_MAIN,
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: blur_rect_shader,
                entry_point: FS_MAIN,
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface.format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        WindowPipelineRegistry {
            copy_pipeline,
            copy_bind_group,
            copy_back_pipeline,
            copy_back_bind_group,
            rect_uniform,
            blur_rect_bind_group,
            blur_rect_pipeline,
        }
    }
}
