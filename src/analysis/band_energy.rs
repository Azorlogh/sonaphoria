use iir_filters::{
	filter::{DirectForm2Transposed, Filter},
	filter_design::{butter, FilterType},
	sos::zpk2sos,
};

use crate::consts::SAMPLE_RATE;

use super::{Analyzer, BUFFER_SIZE};

pub struct BandEnergy {
	dft2: DirectForm2Transposed,
}

impl BandEnergy {
	pub fn new(low: f32, high: f32) -> Self {
		let zpk = butter(
			3,
			FilterType::BandPass(low as f64, high as f64),
			SAMPLE_RATE as f64,
		)
		.unwrap();
		let sos = zpk2sos(&zpk, None).unwrap();
		let dft2 = DirectForm2Transposed::new(&sos);
		Self { dft2 }
	}
}

impl Analyzer for BandEnergy {
	fn process(&mut self, buf: &[f32; BUFFER_SIZE]) -> f32 {
		let mut sum = 0.0;
		for low_mid_high in buf {
			let mid = self.dft2.filter(*low_mid_high as f64) as f32;
			sum += mid * mid;
		}
		let output = sum / buf.len() as f32 * 2.0;
		output
	}
}
