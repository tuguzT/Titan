use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::shaders::{ShaderModule, FRAG_SHADER_CODE, VERT_SHADER_CODE};
use super::slotmap::{
    DeviceKey, PipelineLayoutKey, RenderPassKey, SwapchainKey, SLOTMAP_DEVICE,
    SLOTMAP_PIPELINE_LAYOUT, SLOTMAP_RENDER_PASS, SLOTMAP_SWAPCHAIN,
};
use super::utils;
use super::CommandBuffer;

pub struct GraphicsPipeline {
    handle: vk::Pipeline,
    parent_render_pass: RenderPassKey,
    parent_pipeline_layout: PipelineLayoutKey,
}

impl GraphicsPipeline {
    pub fn new(
        render_pass_key: RenderPassKey,
        pipeline_layout_key: PipelineLayoutKey,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_pipeline_layout = SLOTMAP_PIPELINE_LAYOUT.read()?;
        let pipeline_layout = slotmap_pipeline_layout
            .get(pipeline_layout_key)
            .ok_or_else(|| utils::make_error("pipeline layout not found"))?;

        let slotmap_render_pass = SLOTMAP_RENDER_PASS.read()?;
        let render_pass = slotmap_render_pass
            .get(render_pass_key)
            .ok_or_else(|| utils::make_error("render pass not found"))?;

        let swapchain_key = render_pass.parent_swapchain();
        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let render_pass_swapchain = slotmap_swapchain
            .get(swapchain_key)
            .ok_or_else(|| utils::make_error("swapchain not found"))?;

        let render_pass_device = render_pass_swapchain.parent_device();
        let pipeline_layout_device = pipeline_layout.parent_device();
        if render_pass_device != pipeline_layout_device {
            return Err(utils::make_error(
                "pipeline layout and render pass must have the same parent",
            )
            .into());
        }
        let device_key = render_pass_device;
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let vert_shader_module = ShaderModule::new(device_key, VERT_SHADER_CODE)?;
        let frag_shader_module = ShaderModule::new(device_key, FRAG_SHADER_CODE)?;

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
            parent_render_pass: render_pass_key,
            parent_pipeline_layout: pipeline_layout_key,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn parent_render_pass(&self) -> RenderPassKey {
        self.parent_render_pass
    }

    pub fn parent_pipeline_layout(&self) -> PipelineLayoutKey {
        self.parent_pipeline_layout
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        let slotmap_render_pass = match SLOTMAP_RENDER_PASS.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let render_pass = match slotmap_render_pass.get(self.parent_render_pass()) {
            None => return,
            Some(value) => value,
        };

        let slotmap_swapchain = match SLOTMAP_SWAPCHAIN.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let swapchain = match slotmap_swapchain.get(render_pass.parent_swapchain()) {
            None => return,
            Some(value) => value,
        };

        let slotmap_device = match SLOTMAP_DEVICE.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(swapchain.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline(self.handle, None) }
    }
}

pub struct PipelineLayout {
    handle: vk::PipelineLayout,
    parent_device: DeviceKey,
}

impl PipelineLayout {
    pub unsafe fn with(
        device_key: DeviceKey,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_pipeline_layout(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: device_key,
        })
    }

    pub fn new(device_key: DeviceKey) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::PipelineLayoutCreateInfo::default();
        unsafe { Self::with(device_key, &create_info) }
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn parent_device(&self) -> DeviceKey {
        self.parent_device
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let slotmap_device = match SLOTMAP_DEVICE.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline_layout(self.handle, None) }
    }
}

pub struct RenderPass {
    handle: vk::RenderPass,
    parent_swapchain: SwapchainKey,
}

impl RenderPass {
    pub fn new(swapchain_key: SwapchainKey) -> Result<Self, Box<dyn Error>> {
        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let swapchain = slotmap_swapchain
            .get(swapchain_key)
            .ok_or_else(|| utils::make_error("swapchain not found"))?;

        let device_key = swapchain.parent_device();
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

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

        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::default())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        let dependencies = [*subpass_dependency];

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        let handle = unsafe { device.loader().create_render_pass(&create_info, None)? };
        Ok(Self {
            handle,
            parent_swapchain: swapchain_key,
        })
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.handle
    }

    pub fn parent_swapchain(&self) -> SwapchainKey {
        self.parent_swapchain
    }

    pub unsafe fn begin(
        &self,
        command_buffer: &CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) -> Result<(), Box<dyn Error>> {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let swapchain = slotmap_swapchain
            .get(swapchain_key)
            .ok_or_else(|| utils::make_error("swapchain not found"))?;

        let device_key = swapchain.parent_device();
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        Ok(device
            .loader()
            .cmd_begin_render_pass(command_buffer.handle(), &begin_info, contents))
    }

    pub unsafe fn end(&self, command_buffer: &CommandBuffer) -> Result<(), Box<dyn Error>> {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let swapchain = slotmap_swapchain
            .get(swapchain_key)
            .ok_or_else(|| utils::make_error("swapchain not found"))?;

        let device_key = swapchain.parent_device();
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        Ok(device.loader().cmd_end_render_pass(command_buffer.handle()))
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        let swapchain_key = self.parent_swapchain();
        let slotmap_swapchain = match SLOTMAP_SWAPCHAIN.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let swapchain = match slotmap_swapchain.get(swapchain_key) {
            None => return,
            Some(value) => value,
        };

        let device_key = swapchain.parent_device();
        let slotmap_device = match SLOTMAP_DEVICE.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(device_key) {
            None => return,
            Some(value) => value,
        };

        unsafe { device.loader().destroy_render_pass(self.handle, None) }
    }
}
