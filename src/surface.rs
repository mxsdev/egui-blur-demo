use crate::context::RenderContext;

pub(crate) type SurfaceError = wgpu::SurfaceError;

pub struct SurfaceRenderer {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,

    size: winit::dpi::PhysicalSize<u32>,
    scale_fac: f64,
}

impl SurfaceRenderer {
    pub fn from_window(window: &winit::window::Window, ctx: &RenderContext) -> Self {
        let size = window.inner_size();
        let scale_fac = window.scale_factor();

        let (surface, config) = ctx.create_window_surface(window, size, &ctx.adapter);

        let res = Self {
            surface,
            size,
            scale_fac,
            config,
        };

        res.configure(&ctx);

        res
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn configure(&self, ctx: &RenderContext) {
        ctx.configure_surface(&self.surface, &self.config)
    }

    pub fn set_width(&mut self, new_width: u32) {
        self.config.width = new_width
    }

    pub fn set_height(&mut self, new_height: u32) {
        self.config.height = new_height
    }

    pub fn get_current_texture(
        &mut self,
        context: &RenderContext,
    ) -> Result<wgpu::SurfaceTexture, SurfaceError> {
        match self.surface.get_current_texture() {
            Err(wgpu::SurfaceError::Lost) => {
                self.resize(self.size(), context);
                Err(wgpu::SurfaceError::Lost)
            }

            x => x,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, context: &RenderContext) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.set_width(new_size.width);
            self.set_height(new_size.height);
            self.configure(&context);
        }
    }

    pub fn reconfigure(&mut self, context: &RenderContext) {
        self.resize(self.size, context)
    }

    pub fn set_scale_factor(&mut self, new_scale_fac: f64) {
        self.scale_fac = new_scale_fac
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn screen_descriptor(&self) -> egui_wgpu::renderer::ScreenDescriptor {
        egui_wgpu::renderer::ScreenDescriptor {
            pixels_per_point: self.scale_fac as f32,
            size_in_pixels: self.size.into(),
        }
    }

    pub fn logical_size(&self) -> winit::dpi::LogicalSize<f64> {
        return self.size().to_logical(self.scale_fac);
    }

    pub fn get_scale_fac(&self) -> f64 {
        self.scale_fac
    }
}
