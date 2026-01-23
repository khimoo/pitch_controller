{
  description = "Pitch Controller - MIDI controller application with game controller";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      {
        packages.pitch_controller = pkgs.rustPlatform.buildRustPackage {
          pname = "pitch_controller";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = with pkgs; [
            portmidi
            SDL2
            alsa-lib
            fontconfig
            ipafont
            pkg-config
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];

          postFixup = ''
            wrapProgram $out/bin/pitch_controller \
              --set FONTCONFIG_FILE ${pkgs.fontconfig.out}/etc/fonts/fonts.conf \
              --set FONTCONFIG_PATH ${pkgs.fontconfig.out}/etc/fonts \
              --set PITCH_CONTROLLER_FONT ${pkgs.ipafont}/share/fonts/opentype/ipag.ttf \
              --prefix XDG_DATA_DIRS :${pkgs.ipafont}/share
          '';
        };

        packages.default = self.packages.${system}.pitch_controller;

        apps.pitch_controller = {
          type = "app";
          program = "${self.packages.${system}.pitch_controller}/bin/pitch_controller";
        };

        apps.default = self.apps.${system}.pitch_controller;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            portmidi
            SDL2
            alsa-lib
            fontconfig
            ipafont
            pkg-config
          ];

          shellHook = ''
            export PITCH_CONTROLLER_FONT=${pkgs.ipafont}/share/fonts/opentype/ipag.ttf
          '';
        };
      }
    );
}
