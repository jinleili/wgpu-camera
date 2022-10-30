use app_surface::AppSurface;
use gpu_image4::WgpuCanvas;
use std::future::Future;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    env_logger::init();

    let (event_loop, instance) = pollster::block_on(create_instance());
    start_event_loop(event_loop, instance);
}

#[cfg(target_arch = "wasm32")]
pub fn run() {
    use wasm_bindgen::{prelude::*, JsCast};

    console_log::init_with_level(log::Level::Warn).expect("无法初始化日志库");

    wasm_bindgen_futures::spawn_local(async move {
        let (event_loop, instance) = create_instance().await;
        let run_closure = Closure::once_into_js(move || start_event_loop(event_loop, instance));

        // 处理运行过程中抛出的 JS 异常。
        // 否则 wasm_bindgen_futures 队列将中断，且不再处理任何任务。
        if let Err(error) = call_catch(&run_closure) {
            let is_control_flow_exception = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                e.message().includes("Using exceptions for control flow", 0)
            });

            if !is_control_flow_exception {
                web_sys::console::error_1(&error);
            }
        }

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
            fn call_catch(this: &JsValue) -> Result<(), JsValue>;
        }
    });
}

async fn create_instance() -> (EventLoop<()>, WgpuCanvas) {
    let event_loop = EventLoop::new();
    let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize {
        width: 256.0,
        height: 256.0,
    });
    let builder = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_max_inner_size(size)
        .with_transparent(true)
        .with_title("GPUImage4 on Desktop");
    let window = builder.build(&event_loop).unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                match doc.get_element_by_id("gpu-image4-container") {
                    Some(dst) => {
                        window.set_inner_size(PhysicalSize::new(768, 500));
                        let _ = dst.append_child(&web_sys::Element::from(window.canvas()));
                    }
                    None => {
                        window.set_inner_size(PhysicalSize::new(800, 800));
                        let canvas = window.canvas();
                        canvas.style().set_css_text(
                            "background-color: black; display: block; margin: 20px auto;",
                        );
                        doc.body().and_then(|body| {
                            Some(body.append_child(&web_sys::Element::from(canvas)))
                        });
                    }
                };
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    };

    let app_surface = AppSurface::new(window).await;
    let mut canvas = WgpuCanvas::new(app_surface);

    let (texture, size) = gpu_image4::ffi::get_a_texture(&canvas.app_surface);
    canvas.set_external_texture(texture, (size.width as f32, size.height as f32));

    (event_loop, canvas)
}

fn start_event_loop(event_loop: EventLoop<()>, canvas: WgpuCanvas) {
    let spawner = Spawner::new();
    let mut canvas = canvas;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            Event::RedrawEventsCleared => {
                #[cfg(not(target_arch = "wasm32"))]
                spawner.run_until_stalled();

                canvas.app_surface.view.request_redraw();
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

fn main() {
    run();
}

#[cfg(not(target_arch = "wasm32"))]
struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
struct Spawner {}

#[cfg(target_arch = "wasm32")]
impl Spawner {
    fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
