#![feature(let_chains)]
use ::egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use remun::State;
use std::time::Instant;
use std::{env, error::Error, path::Path, sync::Arc};
use visualizer::Visualizer;
use wgpu::{self, InstanceFlags};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

mod visualizer;

struct App<'window> {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState<'window>>,
    egui_overlay: Option<EguiOverlay>,
    overlay_hidden: bool,
    start_time: Instant,
    state: State,
}

struct EguiOverlay {
    platform: Platform,
    render_pass: RenderPass,
    visualizer: Visualizer,
}

struct RenderState<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl App<'_> {
    fn new(state: State) -> Self {
        App {
            window: None,
            render_state: None,
            egui_overlay: None,
            overlay_hidden: false,
            start_time: Instant::now(),
            state,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let ines = remun::load_from_file(env::args().nth(1).ok_or("please give file as argument")?)?;
    let state = remun::State::new(ines);
    let event_loop = EventLoop::new()?;
    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    //event_loop.run_app(App { window: None, render_state: None })?;
    event_loop.run_app(&mut App::new(state))?;
    Ok(())
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("resuming");
        let window_attributes = WindowAttributes::default();
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(err) => {
                eprintln!("error creating window: {err}");
                event_loop.exit();
                return;
            }
        };
        let window_size = window.inner_size();

        // We use the egui_winit_platform crate as the platform.
        let platform = Platform::new(PlatformDescriptor {
            physical_width: window_size.width,
            physical_height: window_size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let window_wrapper = Arc::new(window);
        let render_state = pollster::block_on(RenderState::new(window_wrapper.clone()));

        // We use the egui_wgpu_backend crate as the render backend.
        let render_pass = RenderPass::new(&render_state.device, render_state.config.format, 1);

        // Display the demo application that ships with egui.
        //let mut demo_app = egui_demo_lib::DemoWindows::default();
        let visualizer = Visualizer::new(&platform.context());

        let egui_overlay = EguiOverlay {
            platform,
            render_pass,
            visualizer,
        };

        self.window = Some(window_wrapper);
        self.render_state = Some(render_state);
        self.egui_overlay = Some(egui_overlay);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        log::trace!("{event:?}");
        self.egui_overlay
            .as_mut()
            .unwrap()
            .platform
            .handle_event(&event);
        match event {
            WindowEvent::CloseRequested => {
                println!("Close was requested; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                log::info!("resizing requested 📐");
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(new_size);
                }
                self.window
                    .as_ref()
                    .expect("resize event without a window")
                    .request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let window = self
                    .window
                    .as_ref()
                    .expect("redraw request without a window");

                // Notify that you're about to draw.
                window.pre_present_notify();

                // Draw.
                render(self).unwrap();

                // LOL do it again
                let window = self
                    .window
                    .as_ref()
                    .expect("redraw request without a window");

                // For contiguous redraw loop you can request a redraw from here.
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;
                let winit::event::KeyEvent { text, state, .. } = event;
                if let Some(text) = text && text == "g" && state == ElementState::Pressed {
                    self.overlay_hidden = !self.overlay_hidden;
                }
            },
            _ => (),
        }
    }
}

fn render(app: &mut App) -> Result<(), wgpu::SurfaceError> {
    let egui_overlay = app.egui_overlay.as_mut().unwrap();
    let start_time = &mut app.start_time;
    let render_state = app.render_state.as_mut().unwrap();
    let state = &mut app.state;
    let platform = &mut egui_overlay.platform;
    let window = app.window.as_mut().unwrap();
    let device = &mut render_state.device;
    let queue = &mut render_state.queue;
    let egui_rpass = &mut egui_overlay.render_pass;
    let dimensions = window.inner_size();
    let width = dimensions.width;
    let height = dimensions.height;

    // Update when
    platform.update_time(start_time.elapsed().as_secs_f64());

    let output_frame = render_state.surface.get_current_texture()?;
    let output_view = output_frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Begin to draw the UI frame.
    platform.begin_pass();

    // Draw the demo application.
    egui_overlay.visualizer.update(&platform.context(), state);

    // End the UI frame. We could now handle the output and draw the UI with the backend.
    let full_output = platform.end_pass(Some(window));
    let context = &platform.context();
    // NOTE don't know if the pixels_per_point is fine
    let paint_jobs = context.tessellate(full_output.shapes, context.pixels_per_point());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder"),
    });

    // render 🐻
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            occlusion_query_set: None,
            timestamp_writes: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.89,
                        g: 0.725,
                        b: 0.91,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
        });
        // TODO do rendering pass here for non-egui
    }
    // Egui render pass
    if !app.overlay_hidden {
        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        egui_rpass
            .add_textures(device, queue, &tdelta)
            .expect("add texture ok");
        egui_rpass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        egui_rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();

        // Clean up textures
        egui_rpass
            .remove_textures(tdelta)
            .expect("remove texture ok");
    }

    // Submit the commands.
    queue.submit(std::iter::once(encoder.finish()));
    // Redraw egui
    output_frame.present();

    Ok(())
}

impl RenderState<'_> {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        } else {
            log::error!("trying to resize to a size 0");
        }
    }
    pub async fn new(window: Arc<Window>) -> Self {
        let width = window.inner_size().width;
        let height = window.inner_size().height;
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: InstanceFlags::default(),
            backend_options: wgpu::BackendOptions {
                gl: wgpu::GlBackendOptions {
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                },
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::default(),
                },
            },
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::info!("GPU info: {:?}", adapter.get_info());

        for backend in instance.enumerate_adapters(wgpu::Backends::all()) {
            println!("Other backend: {:?}", backend.get_info());
        }

        // Use adapter to create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            desired_maximum_frame_latency: 2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            //present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }
}
