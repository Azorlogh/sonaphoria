# Sonaphoria

This is an animated wallpaper program that can display custom shaders and react to sound.
It is written in Rust and uses WGPU in order to remain cross-platform and driver-agnostic.

# Usage

You can try out the built-in wallpapers by running the following command:

`sonaphoria --built-in kanagawa`

# Creating custom wallpapers

Wallpaper are defined using a [.ron file](https://github.com/ron-rs/ron) with the following syntax:
```rust
Config (
	signals: [
		BandEnergy (
			low: 0.1,
			high: 150.0,
		),
		Integrate (
			Smooth (
				attack: Exp(100.0),
				release: Exp(10.0),
				inner: BandEnergy (
					low: 1000.0,
					high: 20000.0,
				),
			),
		),
	],
	main: "example.wgsl",
)
```

Signals are inputs for your shaders that are derived from your PC's sound output.
You can use them to create wallpapers that react to music.

Shaders are written in WGSL, the shading language for WebGPU.

You can define additional buffers by adding a buffers field your config. It has to contain an array of paths (strings). You can use those to persist data throughout multiple frames to create various effects.


Currently, the following signals are available:

```
// Computes the energy in a certain frequency band
BandEnergy (
	low: <frequency in hertz>,
	high: <frequency in hertz>,
)

// Outputs the time since the last detected beat
Beat

// Smoothes a signal in time
Smooth (
	attack: <None | Linear(<slope>) | Exp(<factor>)>,
	release: <None | Linear(<slope>) | Exp(<factor>)>,
	inner: <signal>
)

// Integrates a signal in time (useful for accelerating time when the bass hits)
Integrate(<signal>)
```

