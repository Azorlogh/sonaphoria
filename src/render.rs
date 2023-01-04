use std::{borrow::Cow, time::Instant};

use encase::{ShaderType, UniformBuffer};
use ringbuf::HeapConsumer;
use wgpu::util::DeviceExt;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Window, WindowBuilder},
};

use crate::{consts::DT, wallpaper::Wallpaper, Globals};

pub struct Renderer {
	event_loop: EventLoop<()>,
	window: Window,
	device: wgpu::Device,
	surface: wgpu::Surface,
	surface_cfg: wgpu::SurfaceConfiguration,
	queue: wgpu::Queue,
	globals: Globals,
	globals_buf: wgpu::Buffer,
	signals: HeapConsumer<Vec<f32>>,
	twin_buffers: Vec<[wgpu::TextureView; 2]>,
	twin_buffers_bind_groups: [wgpu::BindGroup; 2],
	twin_buffers_pipelines: Vec<wgpu::RenderPipeline>,
	signals_buf: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
	pub async fn new(wallpaper: &Wallpaper, mut signals: HeapConsumer<Vec<f32>>) -> Self {
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
			label: Some("main"),
			entries: &[
				// globals
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
				// signals
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

		while signals.is_empty() {}
		let signals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Signals buf"),
			contents: bytemuck::cast_slice(&signals.pop().unwrap()),
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

		// TWIN BUFFERS BIND GROUP

		// BIND GROUP LAYOUT
		let twin_buffers_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("main"),
				entries: &(0..wallpaper.buffers.len())
					.map(|i| wgpu::BindGroupLayoutEntry {
						binding: i as u32,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: false },
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
						count: None,
					})
					.collect::<Vec<_>>(),
			});

		// TEXTURES
		let mut twin_buffers = vec![];
		for _ in 0..wallpaper.buffers.len() {
			let mut twins = vec![];
			for _ in 0..2 {
				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: None,
					size: wgpu::Extent3d {
						width: size.width,
						height: size.height,
						depth_or_array_layers: 1,
					},
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba32Float,
					usage: wgpu::TextureUsages::TEXTURE_BINDING
						| wgpu::TextureUsages::RENDER_ATTACHMENT,
				});
				let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
				twins.push((texture, texture_view));
			}
			twin_buffers.push(twins);
		}

		// BIND GROUP

		let twin_buffers_bind_groups: [wgpu::BindGroup; 2] = (0..2)
			.map(|i| {
				device.create_bind_group(&wgpu::BindGroupDescriptor {
					label: None,
					layout: &twin_buffers_bind_group_layout,
					entries: &twin_buffers
						.iter()
						.enumerate()
						.map(|(j, twins)| wgpu::BindGroupEntry {
							binding: j as u32,
							resource: wgpu::BindingResource::TextureView(&twins[i].1),
						})
						.collect::<Vec<_>>(),
				})
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap();

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout, &twin_buffers_bind_group_layout],
			push_constant_ranges: &[],
		});

		let swapchain_format = surface.get_supported_formats(&adapter)[1];

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

		let twin_buffers_pipelines = wallpaper
			.buffers
			.iter()
			.map(|source| {
				let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: None,
					source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&source)),
				});
				device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
						targets: &[Some(wgpu::ColorTargetState {
							format: wgpu::TextureFormat::Rgba32Float,
							blend: None,
							write_mask: wgpu::ColorWrites::ALL,
						})],
					}),
					primitive: wgpu::PrimitiveState::default(),
					depth_stencil: None,
					multisample: wgpu::MultisampleState::default(),
					multiview: None,
				})
			})
			.collect::<Vec<_>>();

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
			twin_buffers: twin_buffers
				.into_iter()
				.map(|mut twins| [twins.remove(0).1, twins.remove(0).1])
				.collect(),
			twin_buffers_bind_groups,
			bind_group,
			render_pipeline,
			twin_buffers_pipelines,
		}
	}

	pub fn run(mut self) {
		let start = Instant::now();
		let mut start_analysis = Instant::now();
		let mut nb_read = 0;
		let mut underrun = true;
		let mut frame_number = 0;

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
						let nb_total_must_be_read =
							((now - start_analysis).as_secs_f32() / DT * 1.05) as usize;
						let mut signals = None;
						for _ in 0..(nb_total_must_be_read.saturating_sub(nb_read)) {
							// attempt to read a signals buffer
							if let Some(new_signals) = self.signals.pop() {
								// previous frame had an underrun, reset timing to avoid future underruns
								signals = Some(new_signals);
								if underrun {
									start_analysis = Instant::now();
									nb_read = 1;
									underrun = false;
								} else {
									nb_read += 1
								}
							} else {
								underrun = true;
							}
						}
						self.globals.resolution = glam::Vec2::new(
							self.surface_cfg.width as f32,
							self.surface_cfg.height as f32,
						);
						self.globals.time = (now - start).as_secs_f32();
						self.globals.frame = frame_number;
						let mut buffer = UniformBuffer::new(Vec::new());
						buffer.write(&self.globals).unwrap();
						self.queue
							.write_buffer(&self.globals_buf, 0, &buffer.into_inner());

						if let Some(signals) = signals {
							self.queue.write_buffer(
								&self.signals_buf,
								0,
								bytemuck::cast_slice(&signals),
							);
						}
					}

					let prev_twin_buffer_bind_group = match frame_number % 2 == 0 {
						false => &self.twin_buffers_bind_groups[0],
						true => &self.twin_buffers_bind_groups[1],
					};
					for i in 0..self.twin_buffers.len() {
						let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: None,
							color_attachments: &[Some(wgpu::RenderPassColorAttachment {
								view: &self.twin_buffers[i][((frame_number + 0) % 2) as usize],
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
									store: true,
								},
							})],
							depth_stencil_attachment: None,
						});
						rpass.set_bind_group(0, &self.bind_group, &[]);
						rpass.set_bind_group(1, &prev_twin_buffer_bind_group, &[]);
						rpass.set_pipeline(&self.twin_buffers_pipelines[i]);
						rpass.draw(0..3, 0..1);
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
						rpass.set_bind_group(1, &prev_twin_buffer_bind_group, &[]);
						rpass.set_pipeline(&self.render_pipeline);
						rpass.draw(0..3, 0..1);
					}

					self.queue.submit(Some(encoder.finish()));
					frame.present();

					frame_number += 1;
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
