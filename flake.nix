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
        extensions = [
          "rust-analyzer"
          "rust-src"
        ];
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
          pkgs.wasm-bindgen-cli_0_2_121
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

      footicalWatch = pkgs.writeShellApplication {
        name = "footical-watch";

        runtimeInputs = [
          rustToolchain
          pkgs.cargo-leptos
          pkgs.tailwindcss_4
          pkgs.wasm-bindgen-cli_0_2_121
          pkgs.binaryen
          pkgs.pkg-config
          pkgs.mold
          pkgs.clang
        ];

        text = ''
          state_directory="''${FOOTICAL_STATE_DIRECTORY:-''${XDG_DATA_HOME:-$HOME/.local/share}/footical}"
          mkdir -p "$state_directory"

          export DATABASE_URL="''${DATABASE_URL:-sqlite://$state_directory/footical.db}"
          export ADMIN_PASSWORD="''${ADMIN_PASSWORD:-development}"
          export COOKIE_SECRET="''${COOKIE_SECRET:-development-cookie-secret}"
          export RUST_LOG="''${RUST_LOG:-info,footical_website=debug,footical_scraper=debug}"
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"

          exec cargo leptos watch "$@"
        '';
      };
    in
    {
      packages.${system} = {
        inherit footicalBuild footicalWatch;
        default = footicalBuild;
      };

      apps.${system}.watch = {
        type = "app";
        program = "${footicalWatch}/bin/footical-watch";
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [
          rustToolchain
          pkgs.cargo-leptos
          pkgs.tailwindcss_4
          pkgs.wasm-bindgen-cli_0_2_121
          pkgs.binaryen
          pkgs.pkg-config
          pkgs.mold
          pkgs.clang
        ];

        buildInputs = [
          pkgs.openssl
        ];

        packages = [
          pkgs.leptosfmt
          pkgs.just
          pkgs.sqlite
          footicalWatch
        ];

        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
      };
    };
}
