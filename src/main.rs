use gfx_backend_vulkan as backend;
use gfx_hal::{
    pool::CommandPoolCreateFlags,
    pso::{self, ShaderStageFlags},
    Adapter, DescriptorPool, Device, Instance, PhysicalDevice, QueueGroup, Surface,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::mem::ManuallyDrop;

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { pos: [ -0.5, 0.33 ], uv: [0.0, 1.0] },
    Vertex { pos: [  0.5, 0.33 ], uv: [1.0, 1.0] },
    Vertex { pos: [  0.5,-0.33 ], uv: [1.0, 0.0] },

    Vertex { pos: [ -0.5, 0.33 ], uv: [0.0, 1.0] },
    Vertex { pos: [  0.5,-0.33 ], uv: [1.0, 0.0] },
    Vertex { pos: [ -0.5,-0.33 ], uv: [0.0, 0.0] },
];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::from((800, 600)))
        .with_title("Vulkan but it's gfx-hal")
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let instance = backend::Instance::create("vulkan_tutorial_but_its_gfx_hal", 0);
    let adapters = instance.enumerate_adapters();
    let surface = instance
        .create_surface_from_raw(&window)
        .expect("Could not create surface");

    let app = Application::init(&window, adapters, surface);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        _ => *control_flow = ControlFlow::Wait,
    });
}

pub struct Application<B: gfx_hal::Backend> {
    device: B::Device,
    queue_group: QueueGroup<B, gfx_hal::Graphics>,
    desc_pool: ManuallyDrop<B::DescriptorPool>,
    surface: B::Surface,
    adapter: gfx_hal::adapter::Adapter<B>,
    format: gfx_hal::format::Format,
    dimensions: gfx_hal::window::Extent2D,
    viewport: gfx_hal::pso::Viewport,
    render_pass: ManuallyDrop<B::RenderPass>,
    pipeline: ManuallyDrop<B::GraphicsPipeline>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    desc_set: B::DescriptorSet,
    set_layout: ManuallyDrop<B::DescriptorSetLayout>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    vertex_buffer: ManuallyDrop<B::Buffer>,
    image_upload_buffer: ManuallyDrop<B::Buffer>,
    image_logo: ManuallyDrop<B::Image>,
    image_srv: ManuallyDrop<B::ImageView>,
    buffer_memory: ManuallyDrop<B::Memory>,
    image_memory: ManuallyDrop<B::Memory>,
    image_upload_memory: ManuallyDrop<B::Memory>,
    sampler: ManuallyDrop<B::Sampler>,
    frames_in_flight: usize,
    frame: u64,
}

impl<B: gfx_hal::Backend> Application<B> {
    fn init(window: &Window, adapters: Vec<Adapter<B>>, surface: B::Surface) -> Self {
        let mut adapters = adapters;
        let mut surface = surface;

        let adapter = adapters.remove(0);
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();

        let (device, mut queue_group) = adapter
            .open_with::<_, gfx_hal::Graphics>(1, |family| surface.supports_queue_family(family))
            .unwrap();

        let mut command_pool = unsafe {
            device.create_command_pool_typed(&queue_group, CommandPoolCreateFlags::empty())
        }
        .expect("Can't create command pool");

        let set_layout = unsafe {
            device.create_descriptor_set_layout(
                &[
                    pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: pso::DescriptorType::SampledImage,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
                &[],
            )
        }
        .expect("Can't create descriptor set layout");

        let mut desc_pool = unsafe {
            device.create_descriptor_pool(
                1,
                &[
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::SampledImage,
                        count: 1,
                    },
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::Sampler,
                        count: 1,
                    },
                ],
                pso::DescriptorPoolCreateFlags::empty(),
            )
        }
        .expect("Can't create descriptor pool");
        let desc_set = unsafe { desc_pool.allocate_set(&set_layout) };

        let buffer_stride = std::mem::size_of::<Vertex>() as u64;
        let buffer_len = VERTICES.len() as u64 * buffer_stride;

        assert_ne!(buffer_len, 0);
        let mut vertex_buffer =
            unsafe { device.create_buffer(buffer_len, gfx_hal::buffer::Usage::VERTEX) }.unwrap();

        let buffer_req = unsafe { device.get_buffer_requirements(&vertex_buffer) };

        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                // type_mask is a bit field where each bit represents a memory type. If the bit is set
                // to 1 it means we can use that type for our buffer. So this code finds the first
                // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                buffer_req.type_mask & (1 << id) != 0
                    && mem_type
                        .properties
                        .contains(gfx_hal::memory::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        let buffer_memory =
            unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();

        unsafe { device.bind_buffer_memory(&buffer_memory, 0, &mut vertex_buffer) }.unwrap();

        // TODO: Check if the todo that is here in the example still exists on latest master
        unsafe {
            let mut vertices = device
                .acquire_mapping_writer::<Vertex>(&buffer_memory, 0..buffer_req.size)
                .unwrap();
            vertices[0..VERTICES.len()].copy_from_slice(VERTICES);
            device.release_mapping_writer(vertices).unwrap();
        }

        // TODO: Image

        // Self {  }
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}
