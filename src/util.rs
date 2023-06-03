use std::{any::Any, num::NonZeroU64};

pub struct ObjectId {
    id: Option<NonZeroU64>,
}

struct RenderPassExposed<'a> {
    id: ObjectId,
    data: Box<dyn Any + Send + Sync>,
    pub parent: &'a mut wgpu::CommandEncoder,
}

pub trait NewRenderPass<'a> {
    fn encoder(&mut self) -> &'a mut wgpu::CommandEncoder;
    fn begin_new_render_pass(&mut self, descriptor: &wgpu::RenderPassDescriptor<'a, '_>);
}

impl<'a> NewRenderPass<'a> for wgpu::RenderPass<'a> {
    fn begin_new_render_pass(&mut self, descriptor: &wgpu::RenderPassDescriptor<'a, '_>) {
        *self = self.encoder().begin_render_pass(descriptor);
    }

    fn encoder(&mut self) -> &'a mut wgpu::CommandEncoder {
        unsafe { std::mem::transmute::<&mut wgpu::RenderPass, &mut RenderPassExposed>(self).parent }
    }
}
