#![feature(let_chains, iter_array_chunks)]
use ::egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use remun::State;
use wgpu::util::DeviceExt;
use std::time::Instant;
use std::{env, error::Error, sync::Arc};
use visualizer::Visualizer;
use wgpu::{self, InstanceFlags};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use shared::Ines;

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
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

//const VERTICES: &[Vertex] = &[
//    // Changed
//    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
//    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
//    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
//    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
//    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
//];
//
//const INDICES: &[u16] = &[
//    0, 1, 4,
//    1, 2, 4,
//    2, 3, 4,
//];
const VERTICES: &[Vertex] = &[
    // top left
    Vertex { position: [-1., 1., 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [-1., -1., 0.0], tex_coords: [0.0, 1.0], },
    Vertex { position: [1., -1., 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [1., 1., 0.0], tex_coords: [1.0, 0.0], },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

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
        let render_state = pollster::block_on(RenderState::new(window_wrapper.clone(), &self.state.ines));

        // We use the egui_wgpu_backend crate as the render backend.
        let render_pass = RenderPass::new(&render_state.device, render_state.config.format, 1);

        // Display the demo application that ships with egui.
        //let mut demo_app = egui_demo_lib::DemoWindows::default();
        let visualizer = Visualizer::new(&platform.context(), &mut self.state);

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
                if let Some(text) = text
                    && text == "g"
                    && state == ElementState::Pressed
                {
                    self.overlay_hidden = !self.overlay_hidden;
                }
            }
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
        render_pass.set_pipeline(&render_state.render_pipeline);
        render_pass.set_bind_group(0, &render_state.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, render_state.vertex_buffer.slice(..));
        render_pass.set_index_buffer(render_state.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
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
    pub async fn new(window: Arc<Window>, ines: &Ines) -> Self {
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


        // Textures
        // TODO load texture from ines bytes, 1. start with bank 3
        let raw_texture = &ines.banks[shared::BANK_SIZE * 2..(shared::BANK_SIZE * 2 + shared::BANK_SIZE/2)];
        let color_lookup: [u32; 4] = [0x000000FF, 0xeb3000ff, 0x2ADD00FF, 0x46fff4ff];
        let mut color_buffer: Vec<u8> = vec![0 ; 16 * 16 * 8 * 8 * 4];
        raw_texture.iter().array_chunks::<16>().enumerate().for_each(|(tile_index, tile)| {
            for row in 0..8 {
                let mut b0 = *tile[row];
                let mut b1 = *tile[row + 8];
                // generate 8 colors
                for column in 0..8 {
                    let rgba = color_lookup[((b0 & 1) | ((b1 & 1) << 1)) as usize].to_be_bytes();
                    for (color_index, color) in rgba.iter().enumerate() {
                        let index = ((tile_index % 16) * 8 + 16*8*8*(tile_index/16) + (row*16*8) + column) * 4 + color_index;
                        color_buffer[index] = *color;
                    }
                    b0 >>= 1;
                    b1 >>= 1;
                }
            }
        });
        //let diffuse_bytes = &diffuse_bytes[..(diffuse_bytes.len()/2)];
        //let diffuse_bytes = include_bytes!("../logo.png");
        println!("db: {:?}", color_buffer.len());
        //println!("db2: {:?}", diffuse_bytes2.len());
        //let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        //let diffuse_rgba = diffuse_image.to_rgba8();
        
        //use image::GenericImageView;
        let dimensions = (16*8, 16*8);
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            }
        );

        // write texture to gpu
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &color_buffer,
            //&diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                // This should match the filterable field of the
                                // corresponding Texture entry above.
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );



        // Setup our pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ Vertex::desc() ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
        }
    }
}
