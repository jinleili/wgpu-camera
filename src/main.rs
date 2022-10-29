#[cfg(any(target_os = "macos", target_os = "window", target_os = "linux"))]
fn main() {
    use app_surface::AppSurface;
    use gpu_image4::WgpuCanvas;
    use std::time::{Duration, Instant};
    use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};

    let events_loop = EventLoop::new();
    let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize {
        width: 512.0,
        height: 512.0,
    });
    let builder = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_max_inner_size(size)
        .with_transparent(true)
        .with_title("GPUImage4 on Desktop");
    let window = builder.build(&events_loop).unwrap();
    let app_surface = pollster::block_on(AppSurface::new(window));
    let mut canvas = WgpuCanvas::new(app_surface);

    let (texture, size) = gpu_image4::ffi::get_a_texture(&canvas.app_surface);
    canvas.set_external_texture(texture, (size.width as f32, size.height as f32));

    let mut last_update_inst = Instant::now();
    let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
    let spawner = Spawner::new();

    events_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            Event::RedrawEventsCleared => {
                let time_since_last_frame = last_update_inst.elapsed();
                if time_since_last_frame >= target_frametime {
                    canvas.app_surface.view.request_redraw();
                    last_update_inst = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_frametime - time_since_last_frame,
                    );
                }

                spawner.run_until_stalled();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                canvas.resize();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                canvas.enter_frame();
            }
            _ => (),
        }
    });
}

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl std::future::Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}
