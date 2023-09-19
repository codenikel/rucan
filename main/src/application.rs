use radiant_core::{RadiantMessage, RadiantResponse, RadiantScene, ScreenDescriptor};
use winit::window::Window;
use winit::{event::*, event_loop::ControlFlow};

pub struct RadiantApp {
    pub window: Window,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub scene: RadiantScene,

    mouse_position: [f32; 2],
    mouse_dragging: bool,
}

impl RadiantApp {
    pub async fn new(window: Window, handler: Box<dyn Fn(RadiantResponse)>) -> Self {
        let size = window.inner_size();

        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let scene = RadiantScene::new(config, surface, device, queue, screen_descriptor, handler);

        Self {
            window,
            size,
            scene,
            mouse_position: [0.0, 0.0],
            mouse_dragging: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.scene.resize([new_size.width, new_size.height]);
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn handle_event(
        &mut self,
        event: &Event<()>,
        control_flow: &mut ControlFlow,
    ) -> Option<RadiantResponse> {
        log::debug!("Event: {:?}", event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &self.window.id() => {
                if !self.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.resize(**new_inner_size);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if button == &MouseButton::Left {
                                let is_pressed = *state == ElementState::Pressed;
                                if is_pressed {
                                    if self.scene.on_mouse_down(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                } else {
                                    if self.scene.on_mouse_up(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                }
                                self.mouse_dragging = is_pressed;
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let current_position = [position.x as f32, position.y as f32];
                            // let transform = [
                            //     current_position[0] - self.mouse_position[0],
                            //     current_position[1] - self.mouse_position[1],
                            // ];
                            self.mouse_position = current_position;
                            if self.scene.on_mouse_move(self.mouse_position) {
                                self.window.request_redraw();
                            }
                        }
                        WindowEvent::Touch(Touch {
                            location, phase, ..
                        }) => {
                            let current_position = [location.x as f32, location.y as f32];
                            // let transform = [
                            //     current_position[0] - self.mouse_position[0],
                            //     current_position[1] - self.mouse_position[1],
                            // ];
                            self.mouse_position = current_position;
                            match phase {
                                TouchPhase::Started => {
                                    if self.scene.on_mouse_down(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                }
                                TouchPhase::Moved => {
                                    if self.scene.on_mouse_move(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                }
                                TouchPhase::Ended => {
                                    if self.scene.on_mouse_up(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                }
                                TouchPhase::Cancelled => {
                                    if self.scene.on_mouse_up(self.mouse_position) {
                                        self.window.request_redraw();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == &self.window.id() => {
                match self.scene.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => {}
        }
        None
    }
}

impl RadiantApp {
    pub fn handle_message(&mut self, message: RadiantMessage) -> Option<RadiantResponse> {
        self.scene.handle_message(message)
    }
}
