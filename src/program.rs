use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    context::RenderContext, pipeline::WindowPipelineRegistry, surface::SurfaceRenderer,
    window_texture::WindowTexture,
};

pub struct Program {
    window: winit::window::Window,
    event_loop: Option<EventLoop<()>>,

    render_ctx: RenderContext,
    surface: SurfaceRenderer,

    egui_ctx: egui::Context,
    egui_winit_bridge: egui_winit::State,
    egui_wgpu_renderer: egui_wgpu::Renderer,

    ferris_img: egui::TextureHandle,
}

impl Program {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(455., 330.))
            .with_title("Blur Rect Demo")
            .build(&event_loop)
            .unwrap();

        let render_ctx = RenderContext::new().await;
        let surface = SurfaceRenderer::from_window(&window, &render_ctx);

        let egui_ctx = egui::Context::default();

        let egui_wgpu_renderer =
            egui_wgpu::Renderer::new(render_ctx.get_device().0, surface.format(), None, 1);

        let mut egui_winit_bridge = egui_winit::State::new(&window);
        egui_winit_bridge.set_pixels_per_point(window.scale_factor() as f32);

        let image = image::load_from_memory(include_bytes!("cuddlyferris.png").as_slice()).unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        let ferris_img = egui_ctx.load_texture(
            "cuddlyferris",
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
            egui::TextureOptions::LINEAR,
        );

        let mut res = Self {
            window,
            event_loop: Some(event_loop),

            render_ctx,
            surface,

            egui_ctx,
            egui_wgpu_renderer,
            egui_winit_bridge,

            ferris_img,
        };

        res.generate_window_texture();

        res
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => {
                self.handle_window_event(&event, control_flow);
            }

            Event::RedrawRequested(..) => {
                self.handle_redraw_request(control_flow);
            }

            Event::MainEventsCleared => {
                self.window.request_redraw();
            }

            _ => (),
        })
    }

    fn resize(&mut self, new_inner_size: PhysicalSize<u32>, new_scale_factor: Option<f64>) {
        if let Some(new_scale_factor) = new_scale_factor {
            self.surface.set_scale_factor(new_scale_factor)
        }

        self.surface.resize(new_inner_size, &self.render_ctx);
        self.generate_window_texture();
    }

    fn generate_window_texture(&mut self) {
        let window_texture = WindowTexture::from_surface(&self.surface, &self.render_ctx);

        self.egui_wgpu_renderer
            .paint_callback_resources
            .insert(window_texture);
    }

    pub fn handle_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            winit::event::WindowEvent::Resized(new_size) => {
                self.resize(*new_size, None);
            }

            winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                self.egui_winit_bridge
                    .set_pixels_per_point(*scale_factor as f32);
                self.resize(**new_inner_size, Some(*scale_factor));
            }

            _ => (),
        }

        let response = self.egui_winit_bridge.on_event(&self.egui_ctx, event);

        if !response.consumed {
            match event {
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                            ..
                        },
                    ..
                }
                | winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                _ => (),
            }
        }
    }

    pub fn handle_redraw_request(&mut self, control_flow: &mut ControlFlow) {
        match self.draw() {
            Ok(_) => {}
            Err(SurfaceError::Lost) => self.surface.reconfigure(&self.render_ctx),
            Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::ExitWithCode(1),
            Err(e) => eprintln!("{:?}", e),
        }
    }

    pub fn draw(&mut self) -> Result<(), SurfaceError> {
        let raw_input = self.egui_winit_bridge.take_egui_input(&self.window);

        let full_output = self
            .egui_ctx
            .run(raw_input, |ctx| crate::ui::ui_main(ctx, &self.ferris_img));

        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes);
        let screen_descriptor = self.surface.screen_descriptor();

        let (device, queue) = self.render_ctx.get_device();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUI Render Encoder"),
        });

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_wgpu_renderer
                .update_texture(device, queue, *id, image_delta)
        }

        self.egui_wgpu_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let wt = self
                .egui_wgpu_renderer
                .paint_callback_resources
                .get::<WindowTexture>()
                .unwrap();

            let view = wt.view();

            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("EGUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };

            let mut render_pass = encoder.begin_render_pass(&descriptor);
            self.egui_wgpu_renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        let output = self.surface.get_current_texture(&self.render_ctx)?;

        {
            let surface_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut copy_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("EGUI copy render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let resources = &mut self.egui_wgpu_renderer.paint_callback_resources;

            let WindowPipelineRegistry {
                copy_pipeline,
                copy_bind_group,
                ..
            } = &resources
                .get::<WindowTexture>()
                .unwrap()
                .pipeline_registry();

            copy_render_pass.set_pipeline(copy_pipeline);
            copy_render_pass.set_bind_group(0, copy_bind_group, &[]);
            copy_render_pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for id in &full_output.textures_delta.free {
            self.egui_wgpu_renderer.free_texture(id);
        }

        self.egui_winit_bridge.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            full_output.platform_output,
        );

        Ok(())
    }
}
