use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result};
use encase::{ShaderType, UniformBuffer};
use notify::{RecursiveMode, Watcher};
use wgpu::util::DeviceExt;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ EventLoop, EventLoopBuilder, EventLoopProxy},
	window::{Window, WindowBuilder},
};

use crate::{analysis::AnalyzerController, consts::DT, render::Renderer, wallpaper::Wallpaper};

pub enum EngineEvent {
	SetWallpaper(Wallpaper),
}

#[derive(Debug, Default, ShaderType)]
pub struct Globals {
	resolution: glam::Vec2,
	time: f32,
	frame: u32,
}

pub struct Engine {
	pub event_loop: EventLoop<EngineEvent>,
	pub window: Arc<Window>,
	pub device: wgpu::Device,
	pub surface: wgpu::Surface<'static>,
	pub surface_cfg: wgpu::SurfaceConfiguration,
	pub queue: wgpu::Queue,
	pub globals: Globals,
	pub globals_buf: wgpu::Buffer,
}

impl Engine {
	pub async fn new() -> Result<Self> {
		let event_loop = EventLoopBuilder::with_user_event().build()?;
		let window = WindowBuilder::new().build(&event_loop)?;
		let size = window.inner_size();
		let instance = wgpu::Instance::default();
		let window = Arc::new(window);
		let surface = instance.create_surface(window.clone())?;
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Failed to find an appropriate adapter");

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					required_limits: wgpu::Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
					memory_hints: wgpu::MemoryHints::MemoryUsage,
				},
				None,
			)
			.await
			.expect("Failed to create device");

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		// let surface_cfg = wgpu::SurfaceConfiguration {
		// 	alpha_mode: wgpu::CompositeAlphaMode::Auto,
		// 	usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		// 	format: swapchain_format,
		// 	width: size.width,
		// 	height: size.height,
		// 	present_mode: wgpu::PresentMode::Fifo,
		// 	view_formats: vec![],
		// };

		let surface_cfg = surface.get_default_config(&adapter, size.width, size.height).context("failed to get surface configuration")?;

		surface.configure(&device, &surface_cfg);

		let globals = Globals::default();
		let mut buffer = UniformBuffer::new(Vec::new());
		buffer.write(&globals).unwrap();
		let globals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform buffer"),
			contents: &buffer.into_inner(),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		Ok(Self {
			event_loop,
			window,
			device,
			surface,
			surface_cfg,
			queue,
			globals,
			globals_buf,
		})
	}

	pub fn proxy(&self) -> EventLoopProxy<EngineEvent> {
		self.event_loop.create_proxy()
	}

	pub fn run(
		mut self,
		wallpaper: Wallpaper,
		mut analyzer: AnalyzerController,
		mut watcher: Option<impl Watcher + 'static>,
	) -> Result<()> {
		let mut watched_paths = vec![];
		if let Some(watcher) = &mut watcher {
			watched_paths = wallpaper.paths();
			for path in &watched_paths {
				watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
			}
		}

		let mut renderer = Renderer::new(
			&self.window,
			&self.device,
			&self.globals_buf,
			&self.surface_cfg.format,
			wallpaper,
		);
		let mut start = Instant::now();
		let mut start_analysis = Instant::now();
		let mut nb_read = 0;
		let mut underrun = true;
		let mut frame_number = 0;
		#[allow(unused)]
		let mut last_frame = Instant::now();
		let mut last_frame2 = Instant::now();

		self.event_loop.run(move |event, elwt| {
			match event {
				Event::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					self.surface_cfg.width = size.width;
					self.surface_cfg.height = size.height;
					self.surface.configure(&self.device, &self.surface_cfg);

					renderer.resize(&self.device, size);

					// Needed on macos
					self.window.request_redraw();
				}
				Event::AboutToWait => {
					while (Instant::now() - last_frame).as_secs_f64() < 1.0 / 144.0 {
						std::thread::sleep_ms(1);
					}
					last_frame = Instant::now();
					self.window.request_redraw();
				}
				Event::UserEvent(event) => match event {
					EngineEvent::SetWallpaper(wallpaper) => {
						if let Some(watcher) = &mut watcher {
							for path in watched_paths.drain(..) {
								watcher.unwatch(&path).unwrap();
							}
							watched_paths = wallpaper.paths();
							for path in watched_paths.iter() {
								watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
							}
						}

						analyzer.set_signals(wallpaper.config.signals.clone());
						renderer = Renderer::new(
							&self.window,
							&self.device,
							&self.globals_buf,
							&self.surface_cfg.format,
							wallpaper,
						);
						self.globals = Globals::default();
						start = Instant::now();
						start_analysis = Instant::now();
						nb_read = 0;
						underrun = true;
						frame_number = 0;
					}
				},
				Event::WindowEvent {
					event: WindowEvent::RedrawRequested,
					..
				} => {
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

					
					let now = Instant::now();
					// println!("{}", 1.0/(now- last_frame2).as_secs_f64());
					last_frame2 = now;

					// update globals
					{
						let now = Instant::now();
						let nb_total_must_be_read =
							((now - start_analysis).as_secs_f32() / DT * 1.1) as usize;
						// println!("must_be_read: {}", nb_total_must_be_read);
						let mut signals = None;
						for _ in 0..(nb_total_must_be_read.saturating_sub(nb_read)) {
							// attempt to read a signals buffer
							if let Some(new_signals) = analyzer.signal_consumer.pop() {
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

						// println!("{:?}", self.globals.time);
						self.globals.frame = frame_number;
						let mut buffer = UniformBuffer::new(Vec::new());
						buffer.write(&self.globals).unwrap();
						self.queue
							.write_buffer(&self.globals_buf, 0, &buffer.into_inner());

						if let Some(signals) = signals {
							self.queue.write_buffer(
								&renderer.signals_buf,
								0,
								bytemuck::cast_slice(&signals),
							);
						}
					}

					renderer.render(&mut encoder, &view, frame_number);

					self.queue.submit(Some(encoder.finish()));
					frame.present();

					frame_number += 1;
				}
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => elwt.exit(),
				_ => {}
			}
		})?;

		Ok(())
	}
}
