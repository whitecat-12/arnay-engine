// src/vulkan/device.rs
use ash::{vk, Instance, Device as AshDevice};
use std::sync::Arc;
use tracing::{info, warn};

pub struct Device {
    pub device: AshDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_family_index: u32,
    pub present_queue_family_index: u32,
    pub surface_format: vk::SurfaceFormatKHR,
}

impl Device {
    pub fn new(instance: &Instance, surface: &vk::SurfaceKHR) -> Result<Self, Box<dyn std::error::Error>> {
        let (physical_device, queue_family_index, present_queue_family_index) = 
            Self::pick_physical_device(instance, surface)?;

        let surface_format = Self::choose_surface_format(instance, physical_device, surface)?;

        let device = Self::create_logical_device(instance, physical_device, queue_family_index, present_queue_family_index)?;

        let graphics_queue = unsafe {
            device.get_device_queue(queue_family_index, 0)
        };
        let present_queue = unsafe {
            device.get_device_queue(present_queue_family_index, 0)
        };

        Ok(Device {
            device,
            graphics_queue,
            present_queue,
            queue_family_index,
            present_queue_family_index,
            surface_format,
        })
    }

    fn pick_physical_device(
        instance: &Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<(vk::PhysicalDevice, u32, u32), Box<dyn std::error::Error>> {
        let physical_devices = unsafe {
            instance.enumerate_physical_devices()?
        };

        for device in physical_devices {
            let properties = unsafe {
                instance.get_physical_device_properties(device)
            };

            let queue_family_properties = unsafe {
                instance.get_physical_device_queue_family_properties(device)
            };

            let mut graphics_queue_family = None;
            let mut present_queue_family = None;

            for (index, queue_family) in queue_family_properties.iter().enumerate() {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_queue_family = Some(index as u32);
                }

                let present_support = unsafe {
                    instance.get_physical_device_surface_support_khr(device, index as u32, *surface)?
                };
                if present_support {
                    present_queue_family = Some(index as u32);
                }

                if graphics_queue_family.is_some() && present_queue_family.is_some() {
                    break;
                }
            }

            if let (Some(graphics), Some(present)) = (graphics_queue_family, present_queue_family) {
                info!("Selected physical device: {:?}", properties.device_name);
                return Ok((device, graphics, present));
            }
        }

        Err("No suitable physical device found".into())
    }

    fn choose_surface_format(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> Result<vk::SurfaceFormatKHR, Box<dyn std::error::Error>> {
        let formats = unsafe {
            instance.get_physical_device_surface_formats_khr(physical_device, *surface)?
        };

        for format in formats {
            if format.format == vk::Format::B8G8R8A8_SRGB ||
               format.format == vk::Format::R8G8B8A8_SRGB {
                return Ok(format);
            }
        }

        Ok(formats[0])
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
        present_queue_family: u32,
    ) -> Result<AshDevice, Box<dyn std::error::Error>> {
        let queue_priorities = [1.0];
        let queue_create_infos = [
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_queue_family)
                .queue_priorities(&queue_priorities)
                .build(),
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(present_queue_family)
                .queue_priorities(&queue_priorities)
                .build(),
        ];

        let extensions = [
            vk::KHR_SWAPCHAIN_NAME.as_ptr(),
        ];

        let features = vk::PhysicalDeviceFeatures::builder()
            .fill_mode_non_solid(true);

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        unsafe {
            Ok(instance.create_device(physical_device, &device_create_info, None)?)
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
