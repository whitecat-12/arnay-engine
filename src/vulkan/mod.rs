// src/vulkan/mod.rs
mod device;
mod swapchain;
mod pipeline;
mod buffer;
mod texture;

use ash::{vk, Entry, Instance};
use ash_window::enumerate_required_extensions;
use winit::window::Window;
use std::sync::Arc;
use tracing::{info, error};

pub struct VulkanContext {
    entry: Entry,
    instance: Instance,
    device: device::Device,
    swapchain: swapchain::Swapchain,
    pipeline: pipeline::Pipeline,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    surface: vk::SurfaceKHR,
    sync_objects: SyncObjects,
}

struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing Vulkan...");
        
        let entry = Entry::linked()?;
        let instance = Self::create_instance(&entry, window)?;
        let surface = Self::create_surface(&entry, &instance, window)?;
        
        let device = device::Device::new(&instance, &surface)?;
        let swapchain = swapchain::Swapchain::new(&instance, &device, &surface, window)?;
        let render_pass = Self::create_render_pass(&instance, &device)?;
        let pipeline = pipeline::Pipeline::new(&instance, &device, render_pass)?;
        
        let (command_pool, command_buffers) = Self::create_command_buffers(&device, &swapchain)?;
        let framebuffers = Self::create_framebuffers(&instance, &device, &swapchain, render_pass)?;
        let sync_objects = Self::create_sync_objects(&device)?;

        Ok(VulkanContext {
            entry,
            instance,
            device,
            swapchain,
            pipeline,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,
            surface,
            sync_objects,
        })
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, Box<dyn std::error::Error>> {
        let app_name = std::ffi::CString::new("Godot-RS Editor")?;
        let engine_name = std::ffi::CString::new("Vulkan")?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let mut extensions = enumerate_required_extensions(window)?
            .iter()
            .map(|ext| *ext)
            .collect::<Vec<_>>();
        extensions.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr());

        let mut layers = vec![];
        #[cfg(debug_assertions)]
        {
            layers.push(b"VK_LAYER_KHRONOS_validation\0".as_ptr());
        }

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        unsafe {
            Ok(entry.create_instance(&instance_create_info, None)?)
        }
    }

    fn create_surface(entry: &Entry, instance: &Instance, window: &Window) -> Result<vk::SurfaceKHR, Box<dyn std::error::Error>> {
        unsafe {
            Ok(ash_window::create_surface(entry, instance, window, None)?)
        }
    }

    fn create_render_pass(instance: &Instance, device: &device::Device) -> Result<vk::RenderPass, Box<dyn std::error::Error>> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(device.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&[color_attachment_ref])
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
            .subpasses(&[subpass])
            .dependencies(&[subpass_dependency]);

        unsafe {
            Ok(device.device.create_render_pass(&render_pass_info, None)?)
        }
    }

    fn create_framebuffers(
        instance: &Instance,
        device: &device::Device,
        swapchain: &swapchain::Swapchain,
        render_pass: vk::RenderPass,
    ) -> Result<Vec<vk::Framebuffer>, Box<dyn std::error::Error>> {
        let mut framebuffers = Vec::new();

        for image_view in &swapchain.image_views {
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&[*image_view])
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);

            unsafe {
                framebuffers.push(device.device.create_framebuffer(&framebuffer_info, None)?);
            }
        }

        Ok(framebuffers)
    }

    fn create_command_buffers(
        device: &device::Device,
        swapchain: &swapchain::Swapchain,
    ) -> Result<(vk::CommandPool, Vec<vk::CommandBuffer>), Box<dyn std::error::Error>> {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue_family_index);

        let command_pool = unsafe {
            device.device.create_command_pool(&pool_info, None)?
        };

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain.images.len() as u32);

        let command_buffers = unsafe {
            device.device.allocate_command_buffers(&allocate_info)?
        };

        Ok((command_pool, command_buffers))
    }

    fn create_sync_objects(device: &device::Device) -> Result<SyncObjects, Box<dyn std::error::Error>> {
        let max_frames = 2;
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();

        for _ in 0..max_frames {
            unsafe {
                image_available_semaphores.push(device.device.create_semaphore(&semaphore_info, None)?);
                render_finished_semaphores.push(device.device.create_semaphore(&semaphore_info, None)?);
                in_flight_fences.push(device.device.create_fence(&fence_info, None)?);
            }
        }

        Ok(SyncObjects {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        })
    }

    pub fn render(&mut self, camera: &scene::Camera, objects: &[scene::RenderObject]) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let device = &self.device.device;
            let current_frame = self.sync_objects.current_frame;

            device.wait_for_fence(
                self.sync_objects.in_flight_fences[current_frame],
                true,
                u64::MAX,
            )?;

            let (image_index, _) = self.swapchain.acquire_next_image(
                device,
                self.sync_objects.image_available_semaphores[current_frame],
            )?;

            device.reset_fence(self.sync_objects.in_flight_fences[current_frame])?;

            let command_buffer = self.command_buffers[image_index as usize];

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device.begin_command_buffer(command_buffer, &begin_info)?;

            // Render pass
            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.07, 0.07, 0.12, 1.0],
                },
            };

            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .clear_values(&[clear_color]);

            device.cmd_begin_render_pass(command_buffer, &render_pass_info, vk::SubpassContents::INLINE);
            
            // Bind pipeline
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);

            // Set viewport and scissor
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            // Push constants for camera
            let camera_data = self.build_camera_data(camera);
            device.cmd_push_constants(
                command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(&camera_data),
            );

            // Draw objects
            for obj in objects {
                let model_matrix = self.build_model_matrix(obj);
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    size_of::<CameraData>() as u32,
                    bytemuck::cast_slice(&model_matrix),
                );

                // Draw primitive based on type
                self.pipeline.draw_primitive(command_buffer, obj.object_type);
            }

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)?;

            // Submit command buffer
            let wait_semaphores = [self.sync_objects.image_available_semaphores[current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [self.sync_objects.render_finished_semaphores[current_frame]];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&[command_buffer])
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(
                self.device.graphics_queue,
                &[submit_info],
                self.sync_objects.in_flight_fences[current_frame],
            )?;

            // Present
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&[self.swapchain.swapchain])
                .image_indices(&[image_index]);

            self.swapchain.present(device, &present_info)?;

            self.sync_objects.current_frame = (current_frame + 1) % 2;
        }

        Ok(())
    }

    fn build_camera_data(&self, camera: &scene::Camera) -> CameraData {
        CameraData {
            view: camera.view_matrix().as_slice().try_into().unwrap(),
            projection: camera.projection_matrix().as_slice().try_into().unwrap(),
        }
    }

    fn build_model_matrix(&self, obj: &scene::RenderObject) -> ModelData {
        let translation = nalgebra::Matrix4::new_translation(&obj.position);
        let rotation = nalgebra::Matrix4::new_nonuniform_scaling(&obj.scale);
        // Simplified: just use position and scale for now
        let model = translation * rotation;
        ModelData {
            model: model.as_slice().try_into().unwrap(),
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device.device;
            
            for fence in &self.sync_objects.in_flight_fences {
                device.destroy_fence(*fence, None);
            }
            for semaphore in &self.sync_objects.image_available_semaphores {
                device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.sync_objects.render_finished_semaphores {
                device.destroy_semaphore(*semaphore, None);
            }
            
            for framebuffer in &self.framebuffers {
                device.destroy_framebuffer(*framebuffer, None);
            }
            
            self.pipeline.destroy(device);
            device.destroy_render_pass(self.render_pass, None);
            
            self.swapchain.destroy(device);
            
            device.destroy_command_pool(self.command_pool, None);
            
            self.device.destroy();
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelData {
    model: [[f32; 4]; 4],
}
