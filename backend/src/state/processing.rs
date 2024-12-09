use std::ops::Neg;

use anyhow::{anyhow, Context, Result};
use wgpu::BufferUsages;

use super::{MAP_HEIGHT, MAP_WIDTH};

const BYTES_PER_ROW: u32 = (super::MAP_WIDTH * super::BYTES_PER_PIXEL) as u32;

pub struct GraphicsStuff {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    texture1: wgpu::Texture,
    texture2: wgpu::Texture,
    render_to_texture2: bool,
    output_buffer: wgpu::Buffer,
}

impl GraphicsStuff {
    /// Initializes all the `wgpu` backend shenanigans necessary to render textures & stuff.
    pub async fn init() -> Result<GraphicsStuff> {
        // NOTE: we don't need to keep the instance around according to wgpu docs; everything else we kinda need though
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow!("couldn't get adapter from wgpu"))?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .with_context(|| "getting device from wgpu adapter")?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let fragment_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&fragment_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 2 textures to allow for alternating which one gets rendered to
        let texture1 = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: super::MAP_WIDTH.try_into()?,
                height: super::MAP_HEIGHT.try_into()?,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let texture2 = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: super::MAP_WIDTH.try_into()?,
                height: super::MAP_HEIGHT.try_into()?,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: super::STATE_BYTES as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(GraphicsStuff {
            device,
            queue,
            pipeline,
            texture1,
            texture2,
            render_to_texture2: true,
            output_buffer,
        })
    }

    /// Sets the contents of the texture that will next be used as a render source.
    pub async fn set_source_texture_contents(&self, data: &[u8]) -> Result<()> {
        let texture = if self.render_to_texture2 {
            &self.texture1
        } else {
            &self.texture2
        };

        // write data to texture via the queue.
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_ROW),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: MAP_WIDTH.try_into()?,
                height: MAP_HEIGHT.try_into()?,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([]);

        Ok(())
    }

    /// Applies the vertex & fragment shaders that advance the state.
    pub fn apply_shader(&mut self) -> Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // choose source/render textures based on current render target
        let (render_texture, source_texture) = if self.render_to_texture2 {
            (&self.texture2, &self.texture1)
        } else {
            (&self.texture1, &self.texture2)
        };

        let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.pipeline);

            // add source texture to bind group, which was configured as part of the layout in init()
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                }],
            });
            render_pass.set_bind_group(0, &bind_group, &[]);

            // just draw a triangle (lol) - covers the entire viewport thing
            render_pass.draw(0..3, 0..1);
        }

        // submit render pass to GPU queue
        self.queue.submit(Some(command_encoder.finish()));

        // alternate which texture gets rendered to
        self.render_to_texture2 = !self.render_to_texture2;

        Ok(())
    }

    /// Copies the last rendered texture's contents into the provided buffer.
    ///
    /// Panics if the provided buffer doesn't match the size of the texture's raw RGBA data.
    pub async fn get_texture_contents(&self, output: &mut [u8]) -> Result<()> {
        // fetch contents of texture that was last rendered to (i.e., not the next render target)
        let source_texture = if self.render_to_texture2 {
            &self.texture1
        } else {
            &self.texture2
        };

        // step 1: copy texture contents to intermediate buffer
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        command_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &source_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(BYTES_PER_ROW),
                    rows_per_image: Some(super::MAP_HEIGHT.try_into()?),
                },
            },
            wgpu::Extent3d {
                width: super::MAP_WIDTH.try_into()?,
                height: super::MAP_HEIGHT.try_into()?,
                depth_or_array_layers: 1,
            },
        );

        // execute copy from texture to buffer
        self.queue.submit(Some(command_encoder.finish()));

        // step 2: map buffer as readable asynchronously (but not async)
        let buffer_slice = self.output_buffer.slice(..);

        // map buffer as readable
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
            ready_sender
                .send(r)
                .expect("couldn't send to ready_sender in get_texture_contents")
        });
        self.device.poll(wgpu::Maintain::wait());
        ready_receiver.await??;

        // buffer is now mapped; copy data out of it
        {
            let buffer_view = buffer_slice.get_mapped_range();
            output.copy_from_slice(&buffer_view);
        }

        // buffers have to be unmapped before they can be used by the GPU
        self.output_buffer.unmap();

        Ok(())
    }
}

#[allow(unused)]
pub fn precompute_gaussian(width: usize, scale: i8) -> Vec<i8> {
    let mut kernel = Vec::with_capacity(width.pow(2));

    let n_plus_1_over_2 = (width as f64 + 1.) / 2.;
    let mut sum = 0.;

    // one indexing :skull:
    for i in 1..=width {
        for j in 1..=width {
            let squares =
                (i as f64 - n_plus_1_over_2).powi(2) + (j as f64 - n_plus_1_over_2).powi(2);
            let entry = squares.neg().exp();
            kernel.push(entry);
            sum += entry;
        }
    }

    kernel
        .iter()
        .map(|n| (n / sum * scale as f64) as i8)
        .collect()
}
