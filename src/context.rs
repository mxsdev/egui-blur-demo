use winit::dpi::PhysicalSize;

pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub shader_copy_texture: wgpu::ShaderModule,
    pub shader_blur_rect: wgpu::ShaderModule,
}

impl RenderContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let shader_copy_texture =
            device.create_shader_module(wgpu::include_wgsl!("copy_texture.wgsl"));

        let shader_blur_rect = device.create_shader_module(wgpu::include_wgsl!("blur_rect.wgsl"));

        Self {
            instance,
            adapter,
            device,
            queue,

            shader_copy_texture,
            shader_blur_rect,
        }
    }

    pub(super) fn create_window_surface<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        &self,
        window: &W,
        size: PhysicalSize<u32>,
        adapter: &wgpu::Adapter,
    ) -> (wgpu::Surface, wgpu::SurfaceConfiguration) {
        let surface = unsafe { self.instance.create_surface(window) }.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let texture_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        (surface, config)
    }

    pub(super) fn configure_surface(
        &self,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration,
    ) {
        surface.configure(&self.device, config)
    }

    pub fn get_device(&self) -> (&wgpu::Device, &wgpu::Queue) {
        (&self.device, &self.queue)
    }
}
