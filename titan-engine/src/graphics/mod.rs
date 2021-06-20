use ash::version::DeviceV1_0;
use ash::vk;
use winit::window::Window;

use command::{CommandBuffer, CommandPool};
use device::{Device, PhysicalDevice, Queue};
use ext::{debug_utils, swapchain, DebugUtils, Swapchain};
use framebuffer::Framebuffer;
use image::{Image, ImageView};
use instance::Instance;
use pipeline::{GraphicsPipeline, PipelineLayout, RenderPass};
use surface::Surface;
use sync::{fence, semaphore, Fence, Semaphore};

use crate::config::Config;
use crate::error::{Error, Result};

use self::slotmap::SlotMappable;

mod command;
mod device;
mod ext;
mod framebuffer;
mod image;
mod instance;
mod pipeline;
mod shader;
pub(crate) mod slotmap;
mod surface;
mod sync;
mod utils;

const MAX_FRAMES_IN_FLIGHT: usize = 10;

pub struct Renderer {
    frame_index: usize,
    images_in_flight: Vec<vk::Fence>,
    in_flight_fences: Vec<fence::Key>,
    render_finished_semaphores: Vec<semaphore::Key>,
    image_available_semaphores: Vec<semaphore::Key>,
    command_buffers: Vec<command::buffer::Key>,
    command_pool: command::pool::Key,
    framebuffers: Vec<framebuffer::Key>,
    graphics_pipeline: pipeline::Key,
    pipeline_layout: pipeline::layout::Key,
    render_pass: pipeline::render_pass::Key,
    swapchain_image_views: Vec<image::view::Key>,
    swapchain_images: Vec<image::Key>,
    swapchain: swapchain::Key,
    device_queues: Vec<device::queue::Key>,
    device: device::Key,
    physical_device: device::physical::Key,
    surface: surface::Key,
    debug_utils: Option<debug_utils::Key>,
    instance: instance::Key,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self> {
        let instance = Instance::new(config, window)?;

        let debug_utils = if instance::ENABLE_VALIDATION {
            let key = DebugUtils::new(instance)?;
            log::info!("debug_utils extension was attached to the instance");
            Some(key)
        } else {
            None
        };
        let surface = Surface::new(instance, window)?;

        let physical_device = {
            let physical_devices: Vec<_> = {
                let slotmap = SlotMappable::slotmap().read().unwrap();
                let surface: &Surface = slotmap.get(surface).expect("surface not found");
                let slotmap = SlotMappable::slotmap().read().unwrap();
                let instance: &Instance = slotmap.get(instance).expect("instance not found");

                let physical_devices = instance.enumerate_physical_devices()?;
                let mut slotmap = SlotMappable::slotmap().write().unwrap();
                let retain = physical_devices
                    .iter()
                    .filter(|&key| {
                        let physical_device: &PhysicalDevice =
                            slotmap.get(*key).expect("physical device not found");
                        let iter = surface
                            .physical_device_queue_family_properties_support(physical_device);
                        physical_device.is_suitable()
                            && surface.is_suitable(physical_device).unwrap_or(false)
                            && iter.peekable().peek().is_some()
                    })
                    .map(|key| *key)
                    .collect::<Vec<_>>();
                for key in physical_devices.iter() {
                    if !retain.contains(&key) {
                        slotmap.remove(*key);
                    }
                }
                retain
            };
            log::info!(
                "enumerated {} suitable physical devices",
                physical_devices.len(),
            );
            let mut slotmap = PhysicalDevice::slotmap().write().unwrap();
            let best_physical_device = *physical_devices
                .iter()
                .max_by_key(|&&key| slotmap.get(key))
                .ok_or_else(|| Error::Other {
                    message: String::from("no suitable physical devices were found"),
                    source: None,
                })?;
            for &physical_device in physical_devices.iter() {
                if physical_device != best_physical_device {
                    slotmap.remove(physical_device);
                }
            }
            best_physical_device
        };

        let device = Device::new(surface, physical_device)?;
        let device_queues = {
            let slotmap = SlotMappable::slotmap().read().unwrap();
            let device: &Device = slotmap.get(device).expect("device not found");
            device.enumerate_queues()?
        };

        let swapchain = Swapchain::new(window, device, surface)?;
        let swapchain_images = {
            let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
            let swapchain: &Swapchain = slotmap_swapchain
                .get(swapchain)
                .expect("swapchain not found");
            swapchain.enumerate_images()?
        };
        let swapchain_image_views = {
            let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
            let swapchain: &Swapchain = slotmap_swapchain
                .get(swapchain)
                .expect("swapchain not found");
            let slotmap_image = SlotMappable::slotmap().read().unwrap();
            swapchain_images
                .iter()
                .map(|&image_key| unsafe {
                    let image: &Image = slotmap_image.get(image_key).expect("image not found");
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
                    ImageView::new(image_key, &create_info)
                })
                .collect::<Result<Vec<_>>>()?
        };

        let render_pass = RenderPass::new(swapchain)?;
        let pipeline_layout = PipelineLayout::new(device)?;
        let graphics_pipeline = GraphicsPipeline::new(render_pass, pipeline_layout)?;

        let framebuffers = {
            let slotmap = SlotMappable::slotmap().read().unwrap();
            let render_pass: &RenderPass = slotmap.get(render_pass).expect("render pass not found");
            let slotmap = SlotMappable::slotmap().read().unwrap();
            let swapchain: &Swapchain = slotmap.get(swapchain).expect("swapchain not found");
            let slotmap = SlotMappable::slotmap().read().unwrap();
            swapchain_image_views
                .iter()
                .map(|&image_view| unsafe {
                    let image_view: &ImageView =
                        slotmap.get(image_view).expect("image view not found");
                    let attachments = [image_view.handle()];
                    let create_info = vk::FramebufferCreateInfo::builder()
                        .attachments(&attachments)
                        .render_pass(render_pass.handle())
                        .width(swapchain.extent().width)
                        .height(swapchain.extent().height)
                        .layers(1);
                    Framebuffer::new(device, &create_info)
                })
                .collect::<Result<Vec<_>>>()?
        };

        let command_pool = unsafe {
            let graphics_queue_family_index = {
                let slotmap = SlotMappable::slotmap().read().unwrap();
                let physical_device: &PhysicalDevice = slotmap
                    .get(physical_device)
                    .expect("physical device not found");
                physical_device.graphics_family_index().unwrap()
            };
            let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(graphics_queue_family_index);
            CommandPool::new(device, &command_pool_create_info)?
        };
        let command_buffers = {
            let slotmap_command_pool = SlotMappable::slotmap().read().unwrap();
            let command_pool: &CommandPool = slotmap_command_pool
                .get(command_pool)
                .expect("command pool not found");
            command_pool.enumerate_command_buffers(swapchain_image_views.len() as u32)?
        };

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let slotmap_command_buffer = SlotMappable::slotmap().read().unwrap();
        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let slotmap_render_pass = SlotMappable::slotmap().read().unwrap();
        let slotmap_framebuffer = SlotMappable::slotmap().read().unwrap();
        let slotmap_graphics_pipeline = SlotMappable::slotmap().read().unwrap();
        let render_pass_ref: &RenderPass = slotmap_render_pass
            .get(render_pass)
            .expect("render pass not found");
        let swapchain_ref: &Swapchain = slotmap_swapchain
            .get(swapchain)
            .expect("swapchain not found");
        let graphics_pipeline_ref: &GraphicsPipeline = slotmap_graphics_pipeline
            .get(graphics_pipeline)
            .expect("graphics pipeline not found");
        let device_ref: &Device = slotmap_device.get(device).expect("device not found");
        for (index, command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer: &CommandBuffer = slotmap_command_buffer
                .get(*command_buffer)
                .expect("command buffer not found");
            let framebuffer: &Framebuffer = slotmap_framebuffer
                .get(framebuffers[index])
                .expect("framebuffer not found");
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
                    .render_pass(render_pass_ref.handle())
                    .framebuffer(framebuffer.handle())
                    .render_area(vk::Rect2D {
                        offset: Default::default(),
                        extent: swapchain_ref.extent(),
                    })
                    .clear_values(&clear_values);
                render_pass_ref.begin(command_buffer, &begin_info, vk::SubpassContents::INLINE)?;

                device_ref.loader().cmd_bind_pipeline(
                    command_buffer.handle(),
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline_ref.handle(),
                );
                device_ref
                    .loader()
                    .cmd_draw(command_buffer.handle(), 3, 1, 0, 0);

                render_pass_ref.end(command_buffer)?;
                command_buffer.end()?;
            }
        }

        let create_semaphores = || {
            (0..MAX_FRAMES_IN_FLIGHT)
                .into_iter()
                .map(|_| Semaphore::new(device))
                .collect::<Result<Vec<_>>>()
        };
        let image_available_semaphores = create_semaphores()?;
        let render_finished_semaphores = create_semaphores()?;
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight_fences = (0..MAX_FRAMES_IN_FLIGHT)
            .into_iter()
            .map(|_| Fence::new(device, &fence_create_info))
            .collect::<Result<Vec<_>>>()?;

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
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight: vec![vk::Fence::null(); MAX_FRAMES_IN_FLIGHT],
            frame_index: 0,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        let slotmap_fence = SlotMappable::slotmap().read().unwrap();
        let in_flight_fence: &Fence = slotmap_fence
            .get(self.in_flight_fences[self.frame_index])
            .expect("fence not found");

        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(self.device).expect("device not found");

        let slotmap_swapchain = SlotMappable::slotmap().read().unwrap();
        let swapchain: &Swapchain = slotmap_swapchain
            .get(self.swapchain)
            .expect("swapchain not found");

        let slotmap_semaphore = SlotMappable::slotmap().read().unwrap();
        let image_available_semaphore: &Semaphore = slotmap_semaphore
            .get(self.image_available_semaphores[self.frame_index])
            .expect("semaphore not found");
        let render_finished_semaphore: &Semaphore = slotmap_semaphore
            .get(self.render_finished_semaphores[self.frame_index])
            .expect("semaphore not found");

        let slotmap_queue = SlotMappable::slotmap().read().unwrap();
        let queue: &Queue = slotmap_queue
            .get(self.device_queues[0])
            .expect("queue not found");

        unsafe {
            let fences = [in_flight_fence.handle()];
            device.loader().wait_for_fences(&fences, true, u64::MAX)?;
        }

        let image_index = unsafe {
            swapchain
                .loader()
                .acquire_next_image(
                    swapchain.handle(),
                    u64::MAX,
                    image_available_semaphore.handle(),
                    vk::Fence::null(),
                )?
                .0 as usize
        };
        let slotmap_command_buffer = SlotMappable::slotmap().read().unwrap();
        let command_buffer: &CommandBuffer = slotmap_command_buffer
            .get(self.command_buffers[image_index])
            .expect("command buffer not found");

        if self.images_in_flight[image_index] != vk::Fence::null() {
            let fences = [self.images_in_flight[image_index]];
            unsafe {
                device.loader().wait_for_fences(&fences, true, u64::MAX)?;
            }
        }
        self.images_in_flight[image_index] = in_flight_fence.handle();

        let wait_semaphores = [image_available_semaphore.handle()];
        let signal_semaphores = [render_finished_semaphore.handle()];
        let command_buffers = [command_buffer.handle()];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        let submits = [*submit_info];
        unsafe {
            let fences = [in_flight_fence.handle()];
            device.loader().reset_fences(&fences)?;
            device
                .loader()
                .queue_submit(queue.handle(), &submits, in_flight_fence.handle())?;
        }
        let swapchains = [swapchain.handle()];
        let image_indices = [image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        unsafe {
            swapchain
                .loader()
                .queue_present(queue.handle(), &present_info)?;
        }
        self.frame_index = (self.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub fn wait(&mut self) -> Result<()> {
        unsafe {
            let slotmap = SlotMappable::slotmap().read().unwrap();
            let device: &Device = slotmap.get(self.device).expect("device not found");
            device.loader().device_wait_idle()?;
        }
        Ok(())
    }
}
