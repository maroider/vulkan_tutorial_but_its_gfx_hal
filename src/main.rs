use gfx_backend_vulkan as backend;
use gfx_hal::{
    adapter::DeviceType,
    queue::{QueueFamilyId, QueueType},
    Adapter, Features, Instance, PhysicalDevice, QueueFamily,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::from((800, 600)))
        .with_title("Vulkan but it's gfx-hal")
        .build(&event_loop)
        .unwrap();

    let app = Application::init();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        _ => *control_flow = ControlFlow::Wait,
    });
}

pub struct Application {
    instance: backend::Instance,
    adapter: Adapter<backend::Backend>,
    graphics_family_id: QueueFamilyId,
}

impl Application {
    fn init() -> Self {
        let instance = backend::Instance::create("vulkan_tutorial_but_its_gfx_hal", 0);

        let adapters = instance.enumerate_adapters();
        let adapter = adapters
            .into_iter()
            .find(|adapter| {
                adapter.info.device_type == DeviceType::DiscreteGpu
                    && adapter
                        .physical_device
                        .features()
                        .contains(Features::GEOMETRY_SHADER)
            })
            .expect("Could not find suitable graphics adapter");

        let graphics_family = adapter
            .queue_families
            .iter()
            .find(|family| family.queue_type() == QueueType::Graphics)
            .expect("Could not find a graphics queue family");
        let graphics_family_id = graphics_family.id();

        Self {
            instance,
            adapter,
            graphics_family_id,
        }
    }
}
