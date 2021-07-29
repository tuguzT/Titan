use std::ops::Deref;

use ash::version::DeviceV1_0;
use ash::vk;

pub use layout::PipelineLayout;
use proc_macro::SlotMappable;
pub use render_pass::RenderPass;

use crate::error::{Error, Result};

use super::{
    device::Device,
    ext::Swapchain,
    shader::{ShaderModule, FRAG_SHADER_CODE, VERT_SHADER_CODE},
    slotmap::{HasParent, SlotMappable},
    utils::{HasHandle, HasLoader},
};

pub mod layout;
pub mod render_pass;

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct GraphicsPipeline {
    key: Key,
    handle: vk::Pipeline,
    parent_render_pass: render_pass::Key,
    parent_pipeline_layout: layout::Key,
}

impl HasParent<RenderPass> for GraphicsPipeline {
    fn parent_key(&self) -> render_pass::Key {
        self.parent_render_pass
    }
}

impl HasParent<PipelineLayout> for GraphicsPipeline {
    fn parent_key(&self) -> layout::Key {
        self.parent_pipeline_layout
    }
}

impl HasHandle for GraphicsPipeline {
    type Handle = vk::Pipeline;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(&self.handle)
    }
}

impl GraphicsPipeline {
    pub fn new(render_pass_key: render_pass::Key, pipeline_layout_key: layout::Key) -> Result<Key> {
        let slotmap_pipeline_layout = SlotMappable::slotmap().read().unwrap();
        let pipeline_layout: &PipelineLayout = slotmap_pipeline_layout
            .get(pipeline_layout_key)
            .expect("pipeline layout not found");

        let slotmap_render_pass = SlotMappable::slotmap().read().unwrap();
        let render_pass: &RenderPass = slotmap_render_pass
            .get(render_pass_key)
            .expect("render pass not found");

        let swapchain_key = render_pass.parent_key();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let render_pass_swapchain: &Swapchain = slotmap_swapchain
            .get(swapchain_key)
            .expect("swapchain not found");

        let render_pass_device =
            <Swapchain as HasParent<Device>>::parent_key(render_pass_swapchain);
        let pipeline_layout_device = pipeline_layout.parent_key();
        if render_pass_device != pipeline_layout_device {
            return Err(Error::Other {
                message: String::from("pipeline layout and render pass must have the same parent"),
                source: None,
            });
        }
        let device_key = render_pass_device;
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");

        let vert_shader_module_key = ShaderModule::new(device_key, VERT_SHADER_CODE)?;
        let frag_shader_module_key = ShaderModule::new(device_key, FRAG_SHADER_CODE)?;
        let mut slotmap_shader = SlotMappable::slotmap().write().unwrap();
        let vert_shader_module: &ShaderModule = slotmap_shader
            .get(vert_shader_module_key)
            .expect("shader module not found");
        let frag_shader_module: &ShaderModule = slotmap_shader
            .get(frag_shader_module_key)
            .expect("shader module not found");

        let shader_stage_info_name = c_str_macro::c_str!("main");
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.handle())
            .name(shader_stage_info_name);
        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.handle())
            .name(shader_stage_info_name);
        let shader_stage_infos = [*vert_shader_stage_info, *frag_shader_stage_info];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: render_pass_swapchain.extent().width as f32,
            height: render_pass_swapchain.extent().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let viewports = [viewport];
        let scissor = vk::Rect2D::builder().extent(render_pass_swapchain.extent());
        let scissors = [*scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        let attachments = [*color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachments);

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_infos)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(**pipeline_layout.handle())
            .render_pass(**render_pass.handle())
            .subpass(0)
            .base_pipeline_index(-1);
        let create_infos = [*create_info];
        let handles = unsafe {
            device.loader().create_graphics_pipelines(
                vk::PipelineCache::default(),
                &create_infos,
                None,
            )
        };
        let handle = handles
            .map(|handles| {
                handles.into_iter().next().ok_or_else(|| Error::Other {
                    message: String::from("graphics pipeline was not created"),
                    source: None,
                })
            })
            .map_err(|error| Error::Graphics { result: error.1 })??;
        slotmap_shader.remove(frag_shader_module_key);
        slotmap_shader.remove(vert_shader_module_key);

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_render_pass: render_pass_key,
            parent_pipeline_layout: pipeline_layout_key,
        });
        Ok(key)
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        let slotmap_render_pass = SlotMappable::slotmap().read().unwrap();
        let render_pass_key = <Self as HasParent<RenderPass>>::parent_key(self);
        let render_pass: &RenderPass = slotmap_render_pass
            .get(render_pass_key)
            .expect("render pass not found");

        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(render_pass.parent_key())
            .expect("swapchain not found");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device_key = <Swapchain as HasParent<Device>>::parent_key(swapchain);
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let loader = device.loader();
        unsafe { loader.destroy_pipeline(self.handle, None) }
    }
}
