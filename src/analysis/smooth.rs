use serde::{Deserialize, Serialize};

use super::{Analyzer, BUFFER_SIZE, DT};

#[derive(Clone, Serialize, Deserialize)]
pub enum Smoothing {
	None,
	Linear(f32),
	Exp(f32),
}

pub struct Smooth {
	pub attack: Smoothing,
	pub release: Smoothing,
	pub value: f32,
	pub inner: Box<dyn Analyzer>,
}

impl Analyzer for Smooth {
	fn process(&mut self, buf: &[f32; BUFFER_SIZE]) -> f32 {
		let target = self.inner.process(buf);
		let smoothing = match target > self.value {
			true => &self.attack,
			false => &self.release,
		};
		match smoothing {
			Smoothing::None => self.value = target,
			Smoothing::Linear(speed) => {
				let dir = (target - self.value).signum();
				self.value += dir * speed * DT;
				if dir != (target - self.value).signum() {
					// went too far
					self.value = target;
				}
			}
			Smoothing::Exp(speed) => {
				let interp_t = 1.0 - (-speed * DT).exp();
				self.value = self.value * (1.0 - interp_t) + target * interp_t;
			}
		}
		self.value
	}
}
