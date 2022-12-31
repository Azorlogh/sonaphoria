mod band_energy;
mod beat_detector;
mod integrated;
mod smooth;
pub use smooth::Smoothing;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapConsumer, HeapProducer};
use triple_buffer::triple_buffer;

use crate::wallpaper::Signal;

use self::{
	band_energy::BandEnergy, beat_detector::BeatDetector, integrated::Integrated, smooth::Smooth,
};

const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = 256;
const HOP_SIZE: usize = 256;
const DT: f32 = BUFFER_SIZE as f32 / SAMPLE_RATE as f32;

pub fn run(signals: &[Signal]) -> Result<triple_buffer::Output<Vec<f32>>> {
	let ring = ringbuf::HeapRb::new(BUFFER_SIZE * 4);

	let (prod, cons) = ring.split();

	std::thread::spawn(|| {
		audio_source(prod);
	});

	let (buf_input, buf_output) = triple_buffer(&vec![0.0; signals.len()]);

	let signals = signals.to_owned();
	std::thread::spawn(move || {
		analyzer(cons, &signals, buf_input);
	});

	Ok(buf_output)
}

fn audio_source(mut prod: HeapProducer<f32>) {
	let host = cpal::default_host();

	let input_device = host.default_input_device().unwrap();

	let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

	let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
		prod.push_slice(data);
	};

	let input_stream = input_device
		.build_input_stream(&config, input_data_fn, |e| {
			eprintln!("error in input stream: {e}")
		})
		.unwrap();

	input_stream.play().unwrap();

	loop {}
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

fn analyzer(
	mut cons: HeapConsumer<f32>,
	signals: &[Signal],
	mut input: triple_buffer::Input<Vec<f32>>,
) {
	let mut analyzers: Vec<Box<dyn Analyzer>> = vec![];
	for signal in signals {
		analyzers.push(create_analyzer(signal.clone()));
	}

	loop {
		let mut buffer = [0.0; BUFFER_SIZE];
		let mut len = 0;
		while len < BUFFER_SIZE {
			if let Some(smpl) = cons.pop() {
				buffer[len] = smpl;
				len += 1;
			}
		}
		let mut output = vec![];
		for analyzer in &mut analyzers {
			output.push(analyzer.process(&buffer));
		}
		input.write(output);
		// println!("{thing}");
	}
}
