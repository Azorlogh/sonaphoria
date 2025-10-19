{
	description = "Audio-reactive shader engine";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
	};

	outputs = { self, nixpkgs }: {
		packages.x86_64-linux.sonaphoria =
			let
				pkgs = import nixpkgs { system = "x86_64-linux"; };
			in
				pkgs.rustPlatform.buildRustPackage rec {
					pname = "sonaphoria";
					version = "0.1.0";
					src = ./.;
					cargoLock.lockFile = ./Cargo.lock;

					nativeBuildInputs = with pkgs; [
						pkg-config
					];

					buildInputs = with pkgs; [
						alsa-lib
						jack2
						xorg.libX11
						xorg.libXcursor
						xorg.libxcb
						xorg.libXi

						aubio
						libsndfile
						libsamplerate
						fftw

						libxkbcommon
						libGL
						vulkan-loader
						makeWrapper
					];

					postFixup = ''
						patchelf $out/bin/sonaphoria \
							--add-rpath ${pkgs.lib.makeLibraryPath buildInputs}
					'';
				};

		defaultPackage.x86_64-linux = self.packages.x86_64-linux.sonaphoria;
	};
}
