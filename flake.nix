{
  nixConfig = {
    extra-substituters = [
      "https://cache.nixos.org"
      "https://nix-community.cachix.org"
      "https://cache.garnix.io"
      "https://numtide.cachix.org"
      "https://devenv.cachix.org"
    ];
    extra-trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
      "numtide.cachix.org-1:2ps1kLBUWjxIneOy1Ik6cQjb41X0iXVXeHigGmycPPE="
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, fenix, utils, crane, nixpkgs }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        toolchain = with fenix.packages.${system}; fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: toolchain);
        src = craneLib.cleanCargoSource ./.;

      in {
        defaultPackage = with pkgs; craneLib.buildPackage {
          src = ./.;
          nativeBuildInputs = [
            # sdl3
            sdl3

            # shaderc
            cmake libcxx git python3
          ];
          preBuild = ''
            export LD_LIBRARY_PATH=${lib.makeLibraryPath [ sdl3 stdenv.cc.cc ]}:$LD_LIBRARY_PATH
          '';
        };

        devShell = with pkgs; mkShell {
          buildInputs = [
            # rust
            toolchain
            
            # sdl3
            sdl3
            
            # shaderc
            cmake stdenv.cc.cc

            # tools
            just
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath [ sdl3 stdenv.cc.cc ]}";
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          RUSTFLAGS = "-Awarnings";
        };
      }
    );
}
