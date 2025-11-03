// assumptions:
// * same bind group
// * 1 draw call
// everything is stored in a consecutive buffer

use shared::Ines;
use wgpu::{BufferDescriptor, Queue, RenderPass, util::DeviceExt};

// f32 * vertex * quad * (nr_sprites + background + foreground)
const VERTEX_BUFFER_SIZE: usize = 4 * 4 * 4 * (64 + 31 * 29);

// index buffer always the same
const SQUARE_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

const WHOLE_SCREEN_VERTICES: &[Vertex] = &[
    // top left
    Vertex {
        position: [-1., 1.],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-1., -1.],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1., -1.],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1., 1.],
        tex_coords: [1.0, 0.0],
    },
];

const TEST_2_VERT: &[Vertex] = &[
    // top left
    Vertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5],
        tex_coords: [1.0, 0.0],
    },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct NesGraphics {
    // background, sprites, foreground
    // everything drawn in order
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
}
impl NesGraphics {
    pub fn draw(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(6 * (64 + 2)), 0, 0..1);
    }

    pub fn update_buffers(&self, queue: &Queue) {
        // clear buffer (unnecessary)
        queue.write_buffer(&self.vertex_buffer, 0, &[0; (4 * 4) * 4 * (64 + 2)]);
        // testing quads
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(WHOLE_SCREEN_VERTICES),
        );
        queue.write_buffer(
            &self.vertex_buffer,
            4 * 4 * 6,
            bytemuck::cast_slice(TEST_2_VERT),
        );
    }

    pub fn new(
        device: &wgpu::Device,
        queue: &Queue,
        ines: &Ines,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // vertex buffer
        // 4 * 4 bytes per vertex, 4 vertices per quad, 64 + 2 quads
        let size = (4 * 4) * 4 * (64 + 2);
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gleeby goo"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // index buffer
        let mut index_buffer_cpu: [u16; 6 * (64 + 2)] = [0; 6 * (64 + 2)];
        [SQUARE_INDICES; 64 + 2]
            .iter()
            .enumerate()
            .for_each(|(square_index, square_indices)| {
                square_indices
                    .iter()
                    .enumerate()
                    .for_each(|(index_index, vertex_index)| {
                        index_buffer_cpu[square_index * 6 + index_index] =
                            square_index as u16 * 3 + *vertex_index;
                    });
            });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_buffer_cpu),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Textures
        // the memory layout of the pattern tables: https://www.nesdev.org/wiki/PPU_pattern_tables
        let chr_bank_start = shared::BANK_SIZE * (ines.inesprg as usize * 2);
        let raw_texture = &ines.banks[chr_bank_start..chr_bank_start + shared::BANK_SIZE / 2];
        let color_lookup: [u32; 4] = [0x000000FF, 0xeb3000ff, 0x2ADD00FF, 0x46fff4ff];
        let mut diffuse_rgba: Vec<u8> = vec![0; 16 * 16 * 8 * 8 * 4];
        raw_texture
            .iter()
            .array_chunks::<16>()
            .enumerate()
            .for_each(|(tile_index, tile)| {
                // NOTE changes the texture so it conforms with the standard way of displaying pattern
                // tables.
                let tile_index =
                    tile_index / 2 + (16 * (tile_index % 2)) + ((tile_index / 32) * 16);
                for row in 0..8 {
                    let mut b0 = *tile[row];
                    let mut b1 = *tile[row + 8];
                    // generate 8 colors
                    for column in 0..8 {
                        let rgba =
                            color_lookup[((b0 & 1) | ((b1 & 1) << 1)) as usize].to_be_bytes();
                        for (color_index, color) in rgba.iter().enumerate() {
                            // the following offsets are in pixels
                            let tile_x_offset = (tile_index % 16) * 8;
                            let tile_y_offset = 16 * 8 * 8 * (tile_index / 16);
                            let row_offset = row * 16 * 8 + 7;
                            let index = (tile_x_offset + tile_y_offset + row_offset - column) * 4
                                + color_index;
                            diffuse_rgba[index] = *color;
                        }
                        b0 >>= 1;
                        b1 >>= 1;
                    }
                }
            });

        let dimensions = (16 * 8, 16 * 8);
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        // write texture to gpu
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            //&diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
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

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

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
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
            render_pipeline,
        }
    }
}
