use std::error::Error;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::vk;

use commands::{CommandBuffer, CommandPool};
use device::{Device, PhysicalDevice, Queue};
use ext::{DebugUtils, Swapchain};
use framebuffer::Framebuffer;
use image::{Image, ImageView};
use instance::Instance;
use pipeline::{GraphicsPipeline, PipelineLayout, RenderPass};
use surface::Surface;

use super::config::Config;
use super::impl_window::Window;

mod commands;
mod device;
mod ext;
mod framebuffer;
mod image;
mod instance;
mod pipeline;
mod shaders;
mod surface;
mod utils;

pub struct Renderer {
    command_buffers: Vec<Arc<CommandBuffer>>,
    command_pool: Arc<CommandPool>,
    framebuffers: Vec<Arc<Framebuffer>>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    pipeline_layout: Arc<PipelineLayout>,
    render_pass: Arc<RenderPass>,
    swapchain_image_views: Vec<Arc<ImageView>>,
    swapchain_images: Vec<Arc<Image>>,
    swapchain: Arc<Swapchain>,
    device_queues: Vec<Arc<Queue>>,
    device: Arc<Device>,
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
    debug_utils: Option<Arc<DebugUtils>>,
    instance: Arc<Instance>,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self, Box<dyn Error>> {
        let instance = Arc::new(Instance::new(config, window.window())?);
        log::info!(
            "instance was created! Vulkan API version is {}",
            instance.version(),
        );
        let debug_utils = if instance::ENABLE_VALIDATION {
            log::info!("debug_utils was attached to instance");
            Some(Arc::new(DebugUtils::new(&instance)?))
        } else {
            None
        };
        let surface = Arc::new(Surface::new(&instance, window.window())?);

        let mut physical_devices: Vec<PhysicalDevice> =
            Instance::enumerate_physical_devices(&instance)?
                .into_iter()
                .filter(|item| {
                    let iter = surface.physical_device_queue_family_properties_support(item);
                    item.is_suitable()
                        && surface.is_suitable(item).unwrap_or(false)
                        && iter.peekable().peek().is_some()
                })
                .collect();
        log::info!(
            "enumerated {} suitable physical devices",
            physical_devices.len(),
        );
        physical_devices.sort_unstable();
        physical_devices.reverse();
        let physical_device = Arc::new(
            physical_devices
                .into_iter()
                .next()
                .ok_or_else(|| utils::make_error("no suitable physical devices were found"))?,
        );
        let device = Arc::new(Device::new(&surface, &physical_device)?);
        let device_queues = Device::enumerate_queues(&device)
            .into_iter()
            .map(|queue| Arc::new(queue))
            .collect();

        let swapchain = Arc::new(Swapchain::new(window, &device, &surface)?);
        let swapchain_images = Swapchain::enumerate_images(&swapchain)?
            .into_iter()
            .map(|image| Arc::new(image))
            .collect::<Vec<_>>();
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| unsafe {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image.handle())
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain.format().format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                ImageView::new(image, &create_info).map(|image_view| Arc::new(image_view))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let render_pass = Arc::new(RenderPass::new(&swapchain)?);
        let pipeline_layout = Arc::new(PipelineLayout::new(&device)?);
        let graphics_pipeline = Arc::new(GraphicsPipeline::new(&render_pass, &pipeline_layout)?);

        let framebuffers = swapchain_image_views
            .iter()
            .map(|image_view| unsafe {
                let attachments = [image_view.handle()];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .attachments(&attachments)
                    .render_pass(render_pass.handle())
                    .width(swapchain.extent().width)
                    .height(swapchain.extent().height)
                    .layers(1);
                Framebuffer::new(&device, &create_info).map(|framebuffer| Arc::new(framebuffer))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let graphics_queue_family_index = physical_device.graphics_family_index().unwrap();
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(graphics_queue_family_index);
        let command_pool =
            unsafe { Arc::new(CommandPool::new(&device, &command_pool_create_info)?) };
        let command_buffers: Vec<_> = CommandPool::enumerate_command_buffers(
            &command_pool,
            swapchain_image_views.len() as u32,
        )?
        .into_iter()
        .map(|command_buffer| Arc::new(command_buffer))
        .collect();

        for (index, command_buffer) in command_buffers.iter().enumerate() {
            let begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                command_buffer.begin(&begin_info)?;
                let clear_color = vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                };
                let clear_values = [clear_color];
                let begin_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(render_pass.handle())
                    .framebuffer(framebuffers[index].handle())
                    .render_area(vk::Rect2D {
                        offset: Default::default(),
                        extent: swapchain.extent(),
                    })
                    .clear_values(&clear_values);
                render_pass.begin(command_buffer, &begin_info, vk::SubpassContents::INLINE)?;

                device.loader().cmd_bind_pipeline(
                    command_buffer.handle(),
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.handle(),
                );
                device
                    .loader()
                    .cmd_draw(command_buffer.handle(), 3, 1, 0, 0);

                render_pass.end(command_buffer)?;
                command_buffer.end()?;
            }
        }

        Ok(Self {
            instance,
            debug_utils,
            surface,
            physical_device,
            device,
            device_queues,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            framebuffers,
            command_pool,
            command_buffers,
        })
    }

    pub fn render(&self) {
        log::trace!("rendering a frame!");
    }
}
