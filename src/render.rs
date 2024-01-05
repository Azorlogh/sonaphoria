use std::{borrow::Cow, iter::once};

use encase::ShaderType;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{engine::Globals, wallpaper::Wallpaper};

pub struct Renderer {
	pub twin_buffers: Vec<[wgpu::TextureView; 2]>,
	pub twin_buffers_bind_groups: [wgpu::BindGroup; 2],
	pub twin_buffers_pipelines: Vec<wgpu::RenderPipeline>,
	pub signals_buf: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub render_pipeline: wgpu::RenderPipeline,
	pub wallpaper: Wallpaper,
}

fn make_twin_buffers(
	device: &wgpu::Device,
	size: PhysicalSize<u32>,
	wallpaper: &Wallpaper,
) -> (
	wgpu::BindGroupLayout,
	Vec<[wgpu::TextureView; 2]>,
	[wgpu::BindGroup; 2],
) {
	// BIND GROUP LAYOUT
	let twin_buffers_bind_group_layout =
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("main"),
			entries: &once(wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			})
			.chain(
				(0..wallpaper.buffers.len()).map(|i| wgpu::BindGroupLayoutEntry {
					binding: i as u32 + 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				}),
			)
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
				// format: wgpu::TextureFormat::Rgba8Unorm,
				usage: wgpu::TextureUsages::TEXTURE_BINDING
					| wgpu::TextureUsages::RENDER_ATTACHMENT,
				view_formats: &[],
			});
			let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
			twins.push((texture, texture_view));
		}
		twin_buffers.push(twins);
	}

	// SAMPLER
	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		label: Some("mip"),
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});

	// BIND GROUP
	let twin_buffers_bind_groups: [wgpu::BindGroup; 2] = (0..2)
		.map(|i| {
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout: &twin_buffers_bind_group_layout,
				entries: &once(wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Sampler(&sampler),
				})
				.chain(
					twin_buffers
						.iter()
						.enumerate()
						.map(|(j, twins)| wgpu::BindGroupEntry {
							binding: j as u32 + 1,
							resource: wgpu::BindingResource::TextureView(&twins[i].1),
						}),
				)
				.collect::<Vec<_>>(),
			})
		})
		.collect::<Vec<_>>()
		.try_into()
		.unwrap();

	(
		twin_buffers_bind_group_layout,
		twin_buffers
			.into_iter()
			.map(|mut twins| [twins.remove(0).1, twins.remove(0).1])
			.collect(),
		twin_buffers_bind_groups,
	)
}

fn make_render_pipeline(
	device: &wgpu::Device,
	pipeline_layout: &wgpu::PipelineLayout,
	swapchain_format: &wgpu::TextureFormat,
	wallpaper: &Wallpaper,
) -> wgpu::RenderPipeline {
	let fullscreen_vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("fullscreen_vs"),
		source: wgpu::ShaderSource::Wgsl(include_str!("fullscreen_vertex.wgsl").into()),
	});

	let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("main shader"),
		source: wgpu::ShaderSource::Naga(Cow::Owned(wallpaper.main.clone())),
	});

	device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &fullscreen_vertex_shader,
			entry_point: "vs_main",
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState {
			module: &module,
			entry_point: "main",
			targets: &[Some((*swapchain_format).into())],
		}),
		primitive: wgpu::PrimitiveState::default(),
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	})
}

impl Renderer {
	pub fn new(
		window: &Window,
		device: &wgpu::Device,
		globals_buf: &wgpu::Buffer,
		swapchain_format: &wgpu::TextureFormat,
		wallpaper: Wallpaper,
	) -> Self {
		let size = window.inner_size();
		let fullscreen_vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("fullscreen_vs"),
			source: wgpu::ShaderSource::Wgsl(include_str!("fullscreen_vertex.wgsl").into()),
		});

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("mip"),
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			..Default::default()
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
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		});

		let signals_buf = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Signals buf"),
			size: 4 * wallpaper.config.signals.len() as wgpu::BufferAddress,
			mapped_at_creation: false,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: globals_buf.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: signals_buf.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
			],
		});

		// TWIN BUFFERS BIND GROUP
		let (twin_buffers_bind_group_layout, twin_buffers, twin_buffers_bind_groups) =
			make_twin_buffers(&device, size, &wallpaper);

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout, &twin_buffers_bind_group_layout],
			push_constant_ranges: &[],
		});

		let render_pipeline =
			make_render_pipeline(&device, &pipeline_layout, &swapchain_format, &wallpaper);

		let twin_buffers_pipelines = wallpaper
			.buffers
			.iter()
			.map(|source| {
				let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: None,
					source: wgpu::ShaderSource::Naga(Cow::Owned(source.clone())),
				});
				device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: None,
					layout: Some(&pipeline_layout),
					vertex: wgpu::VertexState {
						module: &fullscreen_vertex_shader,
						entry_point: "vs_main",
						buffers: &[],
					},
					fragment: Some(wgpu::FragmentState {
						module: &shader,
						entry_point: "main",
						targets: &[Some(wgpu::ColorTargetState {
							format: wgpu::TextureFormat::Rgba32Float,
							// format: wgpu::TextureFormat::Rgba8Unorm,
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

		Self {
			signals_buf,
			twin_buffers,
			twin_buffers_bind_groups,
			bind_group,
			render_pipeline,
			twin_buffers_pipelines,
			wallpaper,
		}
	}

	pub fn resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>) {
		let (_, twin_buffers, twin_buffers_bind_groups) =
			make_twin_buffers(&device, size, &self.wallpaper);
		self.twin_buffers = twin_buffers;
		self.twin_buffers_bind_groups = twin_buffers_bind_groups;
	}

	pub fn render(
		&mut self,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
		frame_number: u32,
	) {
		// run twin buffers
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
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
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
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_bind_group(0, &self.bind_group, &[]);
			rpass.set_bind_group(1, &prev_twin_buffer_bind_group, &[]);
			rpass.set_pipeline(&self.render_pipeline);
			rpass.draw(0..3, 0..1);
		}
	}
}
