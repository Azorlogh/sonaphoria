use std::{borrow::Cow, time::Instant};

use encase::{ShaderType, UniformBuffer};
use wgpu::util::DeviceExt;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Window, WindowBuilder},
};

use crate::{wallpaper::Wallpaper, Globals};

pub struct Renderer {
	event_loop: EventLoop<()>,
	window: Window,
	device: wgpu::Device,
	surface: wgpu::Surface,
	surface_cfg: wgpu::SurfaceConfiguration,
	queue: wgpu::Queue,
	globals: Globals,
	globals_buf: wgpu::Buffer,
	signals: triple_buffer::Output<Vec<f32>>,
	signals_buf: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
	pub async fn new(wallpaper: &Wallpaper, mut signals: triple_buffer::Output<Vec<f32>>) -> Self {
		let event_loop = EventLoop::new();
		let window = WindowBuilder::new().build(&event_loop).unwrap();
		let size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(&window) };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Failed to find an appropriate adapter");

		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					features: wgpu::Features::empty(),
					// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
					limits: wgpu::Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
				},
				None,
			)
			.await
			.expect("Failed to create device");

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&wallpaper.main)),
		});

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: Some(Globals::min_size()),
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: Some(
							wgpu::BufferSize::new(4 * wallpaper.config.signals.len() as u64)
								.unwrap(),
						),
					},
					count: None,
				},
			],
		});

		let globals = Globals::default();
		let mut buffer = UniformBuffer::new(Vec::new());
		buffer.write(&globals).unwrap();
		let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform buffer"),
			contents: &buffer.into_inner(),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let signals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Signals buf"),
			contents: bytemuck::cast_slice(signals.read()),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: uniform_buf.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: signals_buf.as_entire_binding(),
				},
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let swapchain_format = surface.get_supported_formats(&adapter)[1];
		// println!("{:?}", surface.get_supported_formats(&adapter));

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(swapchain_format.into())],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});

		let surface_cfg = wgpu::SurfaceConfiguration {
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: swapchain_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};

		surface.configure(&device, &surface_cfg);

		Self {
			event_loop,
			window,
			device,
			surface,
			surface_cfg,
			queue,
			globals_buf: uniform_buf,
			globals,
			signals,
			signals_buf,
			bind_group,
			render_pipeline,
		}
	}

	pub fn run(mut self) {
		let start = Instant::now();

		self.event_loop.run(move |event, _, control_flow| {
			// Have the closure take ownership of the resources.
			// `event_loop.run` never returns, therefore we must do this to ensure
			// the resources are properly cleaned up.
			// let _ = (&instance, &adapter, &shader, &pipeline_layout);

			*control_flow = ControlFlow::Poll;
			match event {
				Event::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					// Reconfigure the surface with the new size
					self.surface_cfg.width = size.width;
					self.surface_cfg.height = size.height;
					self.surface.configure(&self.device, &self.surface_cfg);
					// On macos the window needs to be redrawn manually after resizing
					self.window.request_redraw();
				}
				Event::MainEventsCleared => self.window.request_redraw(),
				Event::RedrawRequested(_) => {
					let frame = self
						.surface
						.get_current_texture()
						.expect("Failed to acquire next swap chain texture");
					let view = frame
						.texture
						.create_view(&wgpu::TextureViewDescriptor::default());
					let mut encoder = self
						.device
						.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

					// update globals
					{
						let now = Instant::now();
						let signals = self.signals.read();
						self.globals.time = (now - start).as_secs_f32();
						self.globals.resolution = glam::Vec2::new(
							self.surface_cfg.width as f32,
							self.surface_cfg.height as f32,
						);
						let mut buffer = UniformBuffer::new(Vec::new());
						buffer.write(&self.globals).unwrap();
						self.queue
							.write_buffer(&self.globals_buf, 0, &buffer.into_inner());

						println!("signals: {signals:.4?}");
						self.queue.write_buffer(
							&self.signals_buf,
							0,
							bytemuck::cast_slice(signals),
						);
					}

					{
						let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: None,
							color_attachments: &[Some(wgpu::RenderPassColorAttachment {
								view: &view,
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
									store: true,
								},
							})],
							depth_stencil_attachment: None,
						});
						rpass.set_bind_group(0, &self.bind_group, &[]);
						rpass.set_pipeline(&self.render_pipeline);
						rpass.draw(0..3, 0..1);
					}

					self.queue.submit(Some(encoder.finish()));
					frame.present();
				}
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => *control_flow = ControlFlow::Exit,
				_ => {}
			}
		});
	}
}
