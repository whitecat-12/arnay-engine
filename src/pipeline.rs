// src/vulkan/pipeline.rs
use ash::{vk, Device, Instance};
use std::fs;
use tracing::{info, error};

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl Pipeline {
    pub fn new(
        instance: &Instance,
        device: &Device,
        render_pass: vk::RenderPass,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let vert_shader = Self::create_shader_module(device, include_bytes!("../shaders/vert.spv"))?;
        let frag_shader = Self::create_shader_module(device, include_bytes!("../shaders/frag.spv"))?;

        let vert_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader)
            .name(std::ffi::CString::new("main")?.as_c_str());

        let frag_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader)
            .name(std::ffi::CString::new("main")?.as_c_str());

        let shader_stages = [vert_shader_stage.build(), frag_shader_stage.build()];

        let binding_description = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(size_of::<[f32; 3]>() as u32),
            vk::VertexInputAttributeDescription::builder()
                .location(2)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(size_of::<[f32; 6]>() as u32),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[binding_description])
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: 800, height: 600 },
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[viewport])
            .scissors(&[scissor]);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_attachment]);

        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size((size_of::<CameraData>() + size_of::<ModelData>()) as u32);

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[push_constant_range]);

        let pipeline_layout = unsafe {
            device.device.create_pipeline_layout(&layout_info, None)?
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)?
        }[0];

        unsafe {
            device.device.destroy_shader_module(vert_shader, None);
            device.device.destroy_shader_module(frag_shader, None);
        }

        Ok(Pipeline {
            pipeline,
            pipeline_layout,
        })
    }

    fn create_shader_module(
        device: &Device,
        code: &[u8],
    ) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
        let code_u32 = code.chunks_exact(4)
            .map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
            .collect::<Vec<_>>();

        let shader_module_info = vk::ShaderModuleCreateInfo::builder()
            .code(&code_u32);

        unsafe {
            Ok(device.device.create_shader_module(&shader_module_info, None)?)
        }
    }

    pub fn draw_primitive(&self, command_buffer: vk::CommandBuffer, object_type: scene::ObjectType) {
        unsafe {
            let vertex_count = match object_type {
                scene::ObjectType::Cube => 36,
                scene::ObjectType::Sphere => 288,
                scene::ObjectType::Cylinder => 72,
                scene::ObjectType::Torus => 96,
                scene::ObjectType::Wall => 36,
                scene::ObjectType::Floor => 36,
                scene::ObjectType::Beam => 36,
                scene::ObjectType::StairStep => 36,
                _ => 36,
            };
            device.device.cmd_draw(command_buffer, vertex_count, 1, 0, 0);
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.device.destroy_pipeline(self.pipeline, None);
            device.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelData {
    pub model: [[f32; 4]; 4],
}
