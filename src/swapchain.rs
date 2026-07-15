// src/vulkan/swapchain.rs
use ash::{vk, Device, Instance};
use winit::window::Window;
use tracing::{info, error};

pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        device: &Device,
        surface: &vk::SurfaceKHR,
        window: &Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let capabilities = unsafe {
            instance.get_physical_device_surface_capabilities_khr(
                device.physical_device,
                *surface,
            )?
        };

        let extent = Self::choose_extent(window, capabilities);
        let image_count = Self::choose_image_count(capabilities);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(device.surface_format.format)
            .image_color_space(device.surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe {
            device.device.create_swapchain_khr(&swapchain_create_info, None)?
        };

        let images = unsafe {
            device.device.get_swapchain_images_khr(swapchain)?
        };

        let image_views = Self::create_image_views(device, &images, device.surface_format.format)?;

        Ok(Swapchain {
            swapchain,
            images,
            image_views,
            extent,
        })
    }

    fn choose_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn choose_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32 {
        let mut count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && count > capabilities.max_image_count {
            count = capabilities.max_image_count;
        }
        count
    }

    fn create_image_views(
        device: &Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, Box<dyn std::error::Error>> {
        let mut image_views = Vec::new();

        for &image in images {
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = unsafe {
                device.device.create_image_view(&view_info, None)?
            };
            image_views.push(image_view);
        }

        Ok(image_views)
    }

    pub fn acquire_next_image(
        &self,
        device: &Device,
        semaphore: vk::Semaphore,
    ) -> Result<(u32, bool), Box<dyn std::error::Error>> {
        let result = unsafe {
            device.device.acquire_next_image_khr(
                self.swapchain,
                u64::MAX,
                semaphore,
                vk::Fence::null(),
            )?
        };
        Ok(result)
    }

    pub fn present(
        &self,
        device: &Device,
        present_info: &vk::PresentInfoKHR,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            device.device.queue_present_khr(device.present_queue, present_info)?;
        }
        Ok(())
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            for &image_view in &self.image_views {
                device.device.destroy_image_view(image_view, None);
            }
            device.device.destroy_swapchain_khr(self.swapchain, None);
        }
    }
}
