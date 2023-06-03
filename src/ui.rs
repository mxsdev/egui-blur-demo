use std::sync::Arc;

use egui::{epaint::Shadow, *};

pub use util::*;
use wgpu::RenderPassDescriptor;

use crate::{pipeline::WindowPipelineRegistry, util::NewRenderPass, window_texture::WindowTexture};

pub fn ui_main<'a>(ctx: &egui::Context, image: &egui::TextureHandle) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.image(image, image.size_vec2());
    });

    let layer = LayerId::new(Order::Middle, Id::from("test_window_bg"));
    let painter = ctx.layer_painter(layer);
    let shape_idx = painter.add(Shape::Noop);

    let blur_window = egui::Window::new("Test")
        .id(layer.id)
        .frame(
            Frame::window(&ctx.style())
                .fill(Color32::TRANSPARENT)
                .shadow(Shadow::NONE),
        )
        .resizable(true)
        .default_size(vec2(200., 260.))
        .show(ctx, |ui| {
            ui.allocate_space(ui.available_size());
        })
        .unwrap()
        .response;

    {
        let rect = blur_window.rect;

        if rect.size().length() > 0.0 {
            painter.set(
                shape_idx,
                Shape::Callback(PaintCallback {
                    rect,
                    callback: Arc::new(
                        egui_wgpu::CallbackFn::new()
                            .prepare(move |_device, queue, _encoder, resources| {
                                let wt = resources.get::<WindowTexture>().unwrap();
                                wt.pipeline_registry().set_rect(blur_window.rect, queue);

                                vec![]
                            })
                            .paint(move |_info, render_pass, resources| {
                                let wt = resources.get::<WindowTexture>().unwrap();

                                let size = wt.physical_size();

                                let WindowPipelineRegistry {
                                    copy_back_bind_group,
                                    copy_back_pipeline,
                                    blur_rect_bind_group,
                                    blur_rect_pipeline,
                                    ..
                                } = wt.pipeline_registry();

                                // first pass
                                *render_pass = render_pass.encoder().begin_render_pass(
                                    &RenderPassDescriptor {
                                        label: None,
                                        color_attachments: &[Some(
                                            wgpu::RenderPassColorAttachment {
                                                view: wt.back_view(),
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                                        r: 0.0,
                                                        g: 0.0,
                                                        b: 0.0,
                                                        a: 0.0,
                                                    }),
                                                    store: true,
                                                },
                                            },
                                        )],
                                        depth_stencil_attachment: None,
                                    },
                                );

                                let min =
                                    (rect.min.to_vec2() * wt.pixels_per_point() as f32).round();
                                let max =
                                    (rect.max.to_vec2() * wt.pixels_per_point() as f32).round();

                                render_pass.set_viewport(
                                    min.x,
                                    min.y,
                                    max.x - min.x,
                                    max.y - min.y,
                                    0.0,
                                    1.0,
                                );

                                render_pass.set_pipeline(blur_rect_pipeline);
                                render_pass.set_bind_group(0, blur_rect_bind_group, &[]);
                                render_pass.draw(0..4, 0..1);

                                // second pass
                                render_pass.begin_new_render_pass(&RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: wt.view(),
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: true,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                });

                                render_pass.set_viewport(
                                    0.0,
                                    0.0,
                                    size.width as f32,
                                    size.height as f32,
                                    0.0,
                                    1.0,
                                );

                                render_pass.set_pipeline(copy_back_pipeline);
                                render_pass.set_bind_group(0, copy_back_bind_group, &[]);
                                render_pass.draw(0..4, 0..1);
                            }),
                    ),
                }),
            );
        }
    }
}
