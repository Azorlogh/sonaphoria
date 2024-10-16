use std::{
	sync::mpsc::{channel, Receiver, Sender},
	time::Duration,
};

mod band_energy;
mod beat_detector;
mod integrated;
mod smooth;
pub use smooth::Smoothing;

use anyhow::Result;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{HeapConsumer, HeapProducer};

use crate::{consts::BUFFER_SIZE, wallpaper::Signal};

use self::{
	band_energy::BandEnergy, beat_detector::BeatDetector, integrated::Integrated, smooth::Smooth,
};

pub struct AnalyzerController {
	_stream: Stream,
	pub signal_consumer: HeapConsumer<Vec<f32>>,
	event_sender: Sender<SetSignals>,
}

impl AnalyzerController {
	pub fn start(signals: &[Signal]) -> Result<Self> {
		let ring = ringbuf::HeapRb::new(BUFFER_SIZE * 64);

		let (prod, cons) = ring.split();

		let stream = audio_source(prod);

		let (signal_producer, signal_consumer) = ringbuf::HeapRb::new(256).split();

		let (event_sender, event_receiver) = channel();

		let signals = signals.to_owned();
		std::thread::spawn(move || {
			analyzer(cons, &signals, signal_producer, event_receiver);
		});

		Ok(Self {
			_stream: stream,
			signal_consumer,
			event_sender,
		})
	}

	pub fn set_signals(&self, new_signals: Vec<Signal>) {
		self.event_sender.send(SetSignals(new_signals)).unwrap();
	}
}

fn audio_source(mut prod: HeapProducer<f32>) -> Stream {
	let host = cpal::default_host();

	let input_device = host.default_input_device().unwrap();

	let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

	let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
		prod.push_iter(&mut data.iter().step_by(2).cloned());
	};

	let input_stream = input_device
		.build_input_stream(
			&config,
			input_data_fn,
			|e| eprintln!("error in input stream: {e}"),
			None,
		)
		.unwrap();

	input_stream.play().unwrap();
	input_stream
}

pub trait Analyzer {
	fn process(&mut self, buf: &[f32; BUFFER_SIZE]) -> f32;
}

fn create_analyzer(signal: Signal) -> Box<dyn Analyzer> {
	match signal {
		Signal::Beat => Box::new(BeatDetector::new()),
		Signal::BandEnergy { low, high } => Box::new(BandEnergy::new(low, high)),
		Signal::Integrated(inner) => Box::new(Integrated::new(create_analyzer(*inner))),
		Signal::Smooth {
			attack,
			release,
			inner,
		} => Box::new(Smooth {
			attack,
			release,
			value: 0.0,
			inner: create_analyzer(*inner),
		}),
	}
}

pub struct SetSignals(Vec<Signal>);

fn analyzer(
	mut cons: HeapConsumer<f32>,
	signals: &[Signal],
	mut signal_prod: HeapProducer<Vec<f32>>,
	event_recv: Receiver<SetSignals>,
) {
	let mut analyzers: Vec<Box<dyn Analyzer>> = vec![];
	for signal in signals {
		analyzers.push(create_analyzer(signal.clone()));
	}

	loop {
		let mut buffer = [0.0; BUFFER_SIZE];
		let mut len = 0;
		if let Some(new_signals) = event_recv.try_iter().last() {
			analyzers = vec![];
			for signal in new_signals.0 {
				analyzers.push(create_analyzer(signal.clone()));
			}
		}
		while len < BUFFER_SIZE {
			if let Some(smpl) = cons.pop() {
				buffer[len] = smpl;
				len += 1;
			} else {
				std::thread::sleep(Duration::from_millis(1));
			}
		}
		let mut output = vec![];
		for analyzer in &mut analyzers {
			output.push(analyzer.process(&buffer));
		}
		signal_prod.push(output).ok();
	}
}
