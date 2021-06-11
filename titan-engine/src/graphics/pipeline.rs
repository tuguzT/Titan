use std::error::Error;
use std::sync::{Arc, Weak};

use ash::version::DeviceV1_0;
use ash::vk;

use super::ext::Swapchain;
use super::shaders::{ShaderModule, FRAG_SHADER_CODE, VERT_SHADER_CODE};
use super::utils;
use super::CommandBuffer;
use super::Device;

pub struct GraphicsPipeline {
    handle: vk::Pipeline,
    parent_render_pass: Weak<RenderPass>,
    parent_pipeline_layout: Weak<PipelineLayout>,
}

impl GraphicsPipeline {
    pub fn new(
        render_pass: &Arc<RenderPass>,
        pipeline_layout: &Arc<PipelineLayout>,
    ) -> Result<Self, Box<dyn Error>> {
        let pipeline_layout_device = pipeline_layout
            .parent_device()
            .ok_or_else(|| utils::make_error("pipeline layout parent was lost"))?;
        let swapchain = render_pass
            .parent_swapchain()
            .ok_or_else(|| utils::make_error("render pass parent was lost"))?;
        let device = swapchain
            .parent_device()
            .ok_or_else(|| utils::make_error("swapchain parent was lost"))?;
        if device.handle() != pipeline_layout_device.handle() {
            return Err(utils::make_error(
                "pipeline layout and render pass must have the same parent",
            )
            .into());
        }

        let vert_shader_module = ShaderModule::new(&device, VERT_SHADER_CODE)?;
        let frag_shader_module = ShaderModule::new(&device, FRAG_SHADER_CODE)?;

        let shader_stage_info_name = crate::c_str!("main");
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
            width: swapchain.extent().width as f32,
            height: swapchain.extent().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let viewports = [viewport];
        let scissor = vk::Rect2D::builder().extent(swapchain.extent());
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
            .layout(pipeline_layout.handle())
            .render_pass(render_pass.handle())
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
                handles
                    .into_iter()
                    .next()
                    .ok_or_else(|| utils::make_error("graphics pipeline was not created"))
            })
            .map_err(|_| utils::make_error("graphics pipeline was not created"))??;
        Ok(Self {
            handle,
            parent_render_pass: Arc::downgrade(render_pass),
            parent_pipeline_layout: Arc::downgrade(pipeline_layout),
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn parent_render_pass(&self) -> Option<Arc<RenderPass>> {
        self.parent_render_pass.upgrade()
    }

    pub fn parent_pipeline_layout(&self) -> Option<Arc<PipelineLayout>> {
        self.parent_pipeline_layout.upgrade()
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        let render_pass = match self.parent_render_pass() {
            None => return,
            Some(value) => value,
        };
        let swapchain = match render_pass.parent_swapchain() {
            None => return,
            Some(value) => value,
        };
        let device = match swapchain.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline(self.handle, None) }
    }
}

pub struct PipelineLayout {
    handle: vk::PipelineLayout,
    parent_device: Weak<Device>,
}

impl PipelineLayout {
    pub unsafe fn with(
        device: &Arc<Device>,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let handle = device.loader().create_pipeline_layout(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: Arc::downgrade(device),
        })
    }

    pub fn new(device: &Arc<Device>) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::PipelineLayoutCreateInfo::default();
        unsafe { Self::with(device, &create_info) }
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let device = match self.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline_layout(self.handle, None) }
    }
}

pub struct RenderPass {
    handle: vk::RenderPass,
    parent_swapchain: Weak<Swapchain>,
}

impl RenderPass {
    pub fn new(swapchain: &Arc<Swapchain>) -> Result<Self, Box<dyn Error>> {
        let device = swapchain
            .parent_device()
            .ok_or_else(|| utils::make_error("device parent was lost"))?;

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.format().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        let color_attachments = [*color_attachment];

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachment_refs = [*color_attachment_ref];

        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);
        let subpasses = [*subpass_description];

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses);
        let handle = unsafe { device.loader().create_render_pass(&create_info, None)? };
        Ok(Self {
            handle,
            parent_swapchain: Arc::downgrade(swapchain),
        })
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.handle
    }

    pub fn parent_swapchain(&self) -> Option<Arc<Swapchain>> {
        self.parent_swapchain.upgrade()
    }

    pub unsafe fn begin(
        &self,
        command_buffer: &CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) -> Result<(), Box<dyn Error>> {
        let swapchain = self
            .parent_swapchain()
            .ok_or_else(|| utils::make_error("parent was lost"))?;
        let device = swapchain
            .parent_device()
            .ok_or_else(|| utils::make_error("swapchain parent was lost"))?;
        Ok(device
            .loader()
            .cmd_begin_render_pass(command_buffer.handle(), &begin_info, contents))
    }

    pub unsafe fn end(&self, command_buffer: &CommandBuffer) -> Result<(), Box<dyn Error>> {
        let swapchain = self
            .parent_swapchain()
            .ok_or_else(|| utils::make_error("parent was lost"))?;
        let device = swapchain
            .parent_device()
            .ok_or_else(|| utils::make_error("swapchain parent was lost"))?;
        Ok(device.loader().cmd_end_render_pass(command_buffer.handle()))
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        let swapchain = match self.parent_swapchain() {
            None => return,
            Some(value) => value,
        };
        let device = match swapchain.parent_device() {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_render_pass(self.handle, None) }
    }
}
