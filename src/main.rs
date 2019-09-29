use gfx_backend_vulkan::Instance;
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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        _ => *control_flow = ControlFlow::Wait,
    });
}

pub struct Application {}

impl Application {
    fn init() -> Self {
        let instance = Instance::create("vulkan_tutorial_but_its_gfx_hal", 0);

        Self {}
    }
}
