use anyhow::{anyhow, Context, Result};
use wgpu::BufferUsages;

const BYTES_PER_ROW: u32 = (super::MAP_WIDTH * super::BYTES_PER_PIXEL) as u32;

pub struct GraphicsStuff {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    input_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
}

impl GraphicsStuff {
    pub async fn init() -> Result<GraphicsStuff> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow!("couldn't get adapter from wgpu"))?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::TEXTURE_FORMAT_16BIT_NORM,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .with_context(|| "getting device from wgpu adapter")?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
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

        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // input/output buffer have to be separate for performance reasons (?)
        let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: super::STATE_BYTES as u64,
            usage: BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: super::STATE_BYTES as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(GraphicsStuff {
            instance,
            device,
            queue,
            pipeline,
            texture,
            texture_view,
            input_buffer,
            output_buffer,
        })
    }

    pub async fn set_texture_contents(&self, data: &[u8]) -> Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();

        // map buffer as writeable
        let buffer_slice = self.input_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Write, move |r| {
            ready_sender
                .send(r)
                .expect("couldn't send to ready_sender in set_texture_contents")
        });
        self.device.poll(wgpu::Maintain::wait());
        ready_receiver.await??;

        {
            let mut buffer_view = buffer_slice.get_mapped_range_mut();
            buffer_view.copy_from_slice(data);
        }

        // buffers have to be unmapped before being used by the GPU
        self.input_buffer.unmap();

        // actually do copy with command encoder
        command_encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &self.input_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(BYTES_PER_ROW),
                    rows_per_image: Some(super::MAP_HEIGHT.try_into()?),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: super::MAP_WIDTH.try_into()?,
                height: super::MAP_HEIGHT.try_into()?,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(command_encoder.finish()));

        Ok(())
    }

    pub async fn get_texture_contents(&self, output: &mut [u8]) -> Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        command_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
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

        // buffer is now ready; copy data out of it
        {
            let buffer_view = buffer_slice.get_mapped_range();
            output.copy_from_slice(&buffer_view);
        }

        // buffers have to be unmapped before they can be used by the GPU
        self.output_buffer.unmap();

        Ok(())
    }
}
