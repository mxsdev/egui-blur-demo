use winit::dpi::PhysicalSize;

use crate::{
    context::RenderContext,
    pipeline::{PipelineRegistry, WindowPipelineRegistry},
    surface::SurfaceRenderer,
};
pub struct WindowTexture {
    texture: wgpu::Texture,
    back_texture: wgpu::Texture,

    texture_view: wgpu::TextureView,
    back_texture_view: wgpu::TextureView,

    sampler: wgpu::Sampler,

    pixels_per_point: f64,
    size: PhysicalSize<u32>,

    pipeline_registry: WindowPipelineRegistry,
    window_size_uniform: wgpu::Buffer,
}

impl WindowTexture {
    fn create_texture(renderer: &SurfaceRenderer, device: &wgpu::Device) -> wgpu::Texture {
        let format = renderer.format();
        let size = renderer.size();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        texture
    }

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    pub(super) fn from_surface(renderer: &SurfaceRenderer, render_ctx: &RenderContext) -> Self {
        let (device, ..) = render_ctx.get_device();

        let texture = Self::create_texture(renderer, device);
        let texture_view = Self::texture_view(&texture);
        let sampler = Self::create_sampler(device);

        let back_texture = Self::create_texture(renderer, device);
        let back_texture_view = Self::texture_view(&back_texture);
        let back_sampler = Self::create_sampler(device);

        let window_size_uniform =
            PipelineRegistry::create_window_size_buffer(render_ctx, renderer.logical_size().cast());

        let pipeline_registry = PipelineRegistry::generate_for_window(
            render_ctx,
            renderer,
            &texture_view,
            &back_texture_view,
            &sampler,
            &back_sampler,
            &window_size_uniform,
        )
        .into();

        let pixels_per_point = renderer.get_scale_fac();
        let size = renderer.size();

        Self {
            sampler,
            texture,
            pipeline_registry,
            window_size_uniform,
            back_texture,
            texture_view,
            back_texture_view,
            pixels_per_point,
            size,
        }
    }

    pub fn pipeline_registry(&self) -> &WindowPipelineRegistry {
        &self.pipeline_registry
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn back_view(&self) -> &wgpu::TextureView {
        &self.back_texture_view
    }

    fn texture_view(texture: &wgpu::Texture) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn pixels_per_point(&self) -> f64 {
        self.pixels_per_point
    }

    pub fn physical_size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
