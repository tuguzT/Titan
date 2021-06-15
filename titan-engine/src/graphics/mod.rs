use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use self::slotmap::*;
use commands::{CommandBuffer, CommandPool};
use device::{Device, PhysicalDevice, Queue};
use ext::{DebugUtils, Swapchain};
use framebuffer::Framebuffer;
use image::{Image, ImageView};
use instance::Instance;
use pipeline::{GraphicsPipeline, PipelineLayout, RenderPass};
use surface::Surface;
use sync::{Fence, Semaphore};

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
pub(crate) mod slotmap;
mod surface;
mod sync;
mod utils;

const MAX_FRAMES_IN_FLIGHT: usize = 10;

pub struct Renderer {
    frame_index: usize,
    images_in_flight: Vec<vk::Fence>,
    in_flight_fences: Vec<FenceKey>,
    render_finished_semaphores: Vec<SemaphoreKey>,
    image_available_semaphores: Vec<SemaphoreKey>,
    command_buffers: Vec<CommandBufferKey>,
    command_pool: CommandPoolKey,
    framebuffers: Vec<FramebufferKey>,
    graphics_pipeline: GraphicsPipelineKey,
    pipeline_layout: PipelineLayoutKey,
    render_pass: RenderPassKey,
    swapchain_image_views: Vec<ImageViewKey>,
    swapchain_images: Vec<ImageKey>,
    swapchain: SwapchainKey,
    device_queues: Vec<QueueKey>,
    device: DeviceKey,
    physical_device: PhysicalDeviceKey,
    surface: SurfaceKey,
    debug_utils: Option<DebugUtilsKey>,
    instance: InstanceKey,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self, Box<dyn Error>> {
        let instance = {
            let mut slotmap = SLOTMAP_INSTANCE.write()?;
            let key =
                slotmap.insert_with_key(|key| Instance::new(key, config, window.window()).unwrap());
            let instance = slotmap.get_mut(key).unwrap();
            log::info!(
                "instance was created! Vulkan API version is {}",
                instance.version(),
            );
            key
        };

        let debug_utils = if instance::ENABLE_VALIDATION {
            let mut slotmap = SLOTMAP_DEBUG_UTILS.write()?;
            let key = slotmap.insert(DebugUtils::new(instance)?);
            log::info!("debug_utils extension was attached to the instance");
            Some(key)
        } else {
            None
        };
        let surface = {
            let mut slotmap = SLOTMAP_SURFACE.write()?;
            slotmap.insert(Surface::new(instance, window.window())?)
        };

        let physical_device = {
            let mut physical_devices: Vec<_> = {
                let slotmap_surface = SLOTMAP_SURFACE.read()?;
                let surface = slotmap_surface.get(surface).unwrap();
                let slotmap_instance = SLOTMAP_INSTANCE.read()?;
                let instance = slotmap_instance.get(instance).unwrap();
                instance
                    .enumerate_physical_devices()?
                    .into_iter()
                    .filter(|item| {
                        let iter = surface.physical_device_queue_family_properties_support(item);
                        item.is_suitable()
                            && surface.is_suitable(item).unwrap_or(false)
                            && iter.peekable().peek().is_some()
                    })
                    .collect()
            };
            log::info!(
                "enumerated {} suitable physical devices",
                physical_devices.len(),
            );
            physical_devices.sort_unstable();
            physical_devices.reverse();
            let physical_device = physical_devices
                .into_iter()
                .next()
                .ok_or_else(|| utils::make_error("no suitable physical devices were found"))?;
            let mut slotmap = SLOTMAP_PHYSICAL_DEVICE.write()?;
            slotmap.insert(physical_device)
        };

        let device = {
            let mut slotmap = SLOTMAP_DEVICE.write()?;
            slotmap.insert_with_key(|key| Device::new(key, surface, physical_device).unwrap())
        };
        let device_queues = {
            let slotmap_device = SLOTMAP_DEVICE.read()?;
            let device = slotmap_device.get(device).unwrap();
            let mut slotmap_queue = SLOTMAP_QUEUE.write()?;
            device
                .enumerate_queues()?
                .into_iter()
                .map(|queue| slotmap_queue.insert(queue))
                .collect()
        };

        let swapchain = {
            let mut slotmap = SLOTMAP_SWAPCHAIN.write()?;
            slotmap.insert(Swapchain::new(window, device, surface)?)
        };
        let swapchain_images: Vec<_> = {
            let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
            let swapchain = slotmap_swapchain.get(swapchain).unwrap();
            let mut slotmap_image = SLOTMAP_IMAGE.write()?;
            swapchain
                .enumerate_images()?
                .into_iter()
                .map(|image| slotmap_image.insert(image))
                .collect()
        };
        let swapchain_image_views = {
            let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
            let swapchain = slotmap_swapchain.get(swapchain).unwrap();
            let slotmap_image = SLOTMAP_IMAGE.read()?;
            let mut slotmap_image_view = SLOTMAP_IMAGE_VIEW.write()?;
            swapchain_images
                .iter()
                .map(|&image_key| unsafe {
                    let image = slotmap_image.get(image_key).unwrap();
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
                        .map(|image_view| slotmap_image_view.insert(image_view))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let render_pass = {
            let mut slotmap = SLOTMAP_RENDER_PASS.write()?;
            slotmap.insert(RenderPass::new(swapchain)?)
        };
        let pipeline_layout = {
            let mut slotmap = SLOTMAP_PIPELINE_LAYOUT.write()?;
            slotmap.insert(PipelineLayout::new(device)?)
        };
        let graphics_pipeline = {
            let mut slotmap = SLOTMAP_GRAPHICS_PIPELINE.write()?;
            slotmap.insert(GraphicsPipeline::new(render_pass, pipeline_layout)?)
        };

        let framebuffers = {
            let mut slotmap_framebuffer = SLOTMAP_FRAMEBUFFER.write()?;
            let slotmap_image_view = SLOTMAP_IMAGE_VIEW.read()?;
            let slotmap_render_pass = SLOTMAP_RENDER_PASS.read()?;
            let render_pass = slotmap_render_pass.get(render_pass).unwrap();
            let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
            let swapchain = slotmap_swapchain.get(swapchain).unwrap();
            swapchain_image_views
                .iter()
                .map(|&image_view| unsafe {
                    let image_view = slotmap_image_view.get(image_view).unwrap();
                    let attachments = [image_view.handle()];
                    let create_info = vk::FramebufferCreateInfo::builder()
                        .attachments(&attachments)
                        .render_pass(render_pass.handle())
                        .width(swapchain.extent().width)
                        .height(swapchain.extent().height)
                        .layers(1);
                    Framebuffer::new(device, &create_info)
                        .map(|framebuffer| slotmap_framebuffer.insert(framebuffer))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let command_pool = unsafe {
            let graphics_queue_family_index = {
                let slotmap = SLOTMAP_PHYSICAL_DEVICE.read()?;
                let physical_device = slotmap.get(physical_device).unwrap();
                physical_device.graphics_family_index().unwrap()
            };
            let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(graphics_queue_family_index);
            let mut slotmap = SLOTMAP_COMMAND_POOL.write()?;
            slotmap.insert_with_key(|key| {
                CommandPool::new(key, device, &command_pool_create_info).unwrap()
            })
        };
        let command_buffers: Vec<_> = {
            let slotmap_command_pool = SLOTMAP_COMMAND_POOL.read()?;
            let command_pool = slotmap_command_pool.get(command_pool).unwrap();
            let mut slotmap_command_buffer = SLOTMAP_COMMAND_BUFFER.write()?;
            command_pool
                .enumerate_command_buffers(swapchain_image_views.len() as u32)?
                .into_iter()
                .map(|command_buffer| slotmap_command_buffer.insert(command_buffer))
                .collect()
        };

        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let slotmap_command_buffer = SLOTMAP_COMMAND_BUFFER.read()?;
        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let slotmap_render_pass = SLOTMAP_RENDER_PASS.read()?;
        let slotmap_framebuffer = SLOTMAP_FRAMEBUFFER.read()?;
        let slotmap_graphics_pipeline = SLOTMAP_GRAPHICS_PIPELINE.read()?;
        let render_pass_ref = slotmap_render_pass.get(render_pass).unwrap();
        let swapchain_ref = slotmap_swapchain.get(swapchain).unwrap();
        let graphics_pipeline_ref = slotmap_graphics_pipeline.get(graphics_pipeline).unwrap();
        let device_ref = slotmap_device.get(device).unwrap();
        for (index, command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer = slotmap_command_buffer.get(*command_buffer).unwrap();
            let framebuffer = slotmap_framebuffer.get(framebuffers[index]).unwrap();
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
            let slotmap = SLOTMAP_SEMAPHORE.write();
            if let Ok(mut slotmap) = slotmap {
                (0..MAX_FRAMES_IN_FLIGHT)
                    .into_iter()
                    .map(|_| Semaphore::new(device).map(|semaphore| slotmap.insert(semaphore)))
                    .collect::<Result<Vec<_>, _>>()
            } else {
                Err(utils::make_error("error").into())
            }
        };
        let image_available_semaphores = create_semaphores()?;
        let render_finished_semaphores = create_semaphores()?;
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight_fences = {
            let mut slotmap = SLOTMAP_FENCE.write()?;
            (0..MAX_FRAMES_IN_FLIGHT)
                .into_iter()
                .map(|_| Fence::new(device, &fence_create_info).map(|fence| slotmap.insert(fence)))
                .collect::<Result<Vec<_>, _>>()?
        };

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

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let slotmap_fence = SLOTMAP_FENCE.read()?;
        let in_flight_fence = slotmap_fence
            .get(self.in_flight_fences[self.frame_index])
            .unwrap();

        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device.get(self.device).unwrap();

        let slotmap_swapchain = SLOTMAP_SWAPCHAIN.read()?;
        let swapchain = slotmap_swapchain.get(self.swapchain).unwrap();

        let slotmap_semaphore = SLOTMAP_SEMAPHORE.read()?;
        let image_available_semaphore = slotmap_semaphore
            .get(self.image_available_semaphores[self.frame_index])
            .unwrap();
        let render_finished_semaphore = slotmap_semaphore
            .get(self.render_finished_semaphores[self.frame_index])
            .unwrap();

        let slotmap_queue = SLOTMAP_QUEUE.read()?;
        let queue = slotmap_queue
            .get(self.device_queues[0])
            .unwrap();

        unsafe {
            let fences = [in_flight_fence.handle()];
            device
                .loader()
                .wait_for_fences(&fences, true, u64::MAX)?;
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
        let slotmap_command_buffer = SLOTMAP_COMMAND_BUFFER.read()?;
        let command_buffer = slotmap_command_buffer
            .get(self.command_buffers[image_index])
            .unwrap();

        if self.images_in_flight[image_index] != vk::Fence::null() {
            let fences = [self.images_in_flight[image_index]];
            unsafe {
                device
                    .loader()
                    .wait_for_fences(&fences, true, u64::MAX)?;
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
            device.loader().queue_submit(
                queue.handle(),
                &submits,
                in_flight_fence.handle(),
            )?;
        }
        let swapchains = [swapchain.handle()];
        let image_indices = [image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        unsafe {
            let queue_handle = queue.handle();
            swapchain
                .loader()
                .queue_present(queue_handle, &present_info)?;
        }
        self.frame_index = (self.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub fn wait(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            let slotmap = SLOTMAP_DEVICE.read()?;
            let device = slotmap
                .get(self.device)
                .ok_or_else(|| utils::make_error("Wait failure: device not found"))?;
            device.loader().device_wait_idle()?;
        }
        Ok(())
    }
}
