{
  description = "Footical - football league scraper and fixture website";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
    }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustToolchain = pkgs.rust-bin.stable."1.94.0".default.override {
        targets = [ "wasm32-unknown-unknown" ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      unfilteredRoot = ./.;

      filteredSource = pkgs.lib.fileset.toSource {
        root = unfilteredRoot;
        fileset = pkgs.lib.fileset.unions [
          (craneLib.fileset.commonCargoSources unfilteredRoot)
          (pkgs.lib.fileset.fileFilter (file: file.hasExt "css") unfilteredRoot)
          (pkgs.lib.fileset.fileFilter (file: file.hasExt "sql") unfilteredRoot)
        ];
      };

      commonArgs = {
        src = filteredSource;
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.cargo-leptos
          pkgs.tailwindcss_4
          pkgs.wasm-bindgen-cli
          pkgs.binaryen
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.openssl
        ];
      };

      cargoArtifacts = craneLib.buildDepsOnly (
        commonArgs
        // {
          pname = "footical-deps";
          version = "0.1.0";
        }
      );

      footicalBuild = craneLib.buildPackage (
        commonArgs
        // {
          pname = "footical";
          version = "0.1.0";
          inherit cargoArtifacts;

          doNotPostBuildInstallCargoBinaries = true;

          buildPhaseCargoCommand = ''
            cargo leptos build --release
          '';

          installPhaseCommand = ''
            mkdir -p $out/bin $out/share/footical
            cp target/release/footical-website $out/bin/footical-website
            cp -r target/site/. $out/share/footical/site/
          '';

          doCheck = false;
        }
      );
    in
    {
      packages.${system} = {
        inherit footicalBuild;
        default = footicalBuild;
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [
          rustToolchain
          pkgs.cargo-leptos
          pkgs.tailwindcss_4
          pkgs.wasm-bindgen-cli
          pkgs.binaryen
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.openssl
        ];
      };
    };
}
