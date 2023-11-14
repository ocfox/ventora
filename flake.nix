{
  description = "A utility to manually adjust AMD graphics card fans";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs@{ flake-parts, crane, nixpkgs, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = import inputs.systems;

    imports = [
      inputs.flake-parts.flakeModules.easyOverlay
    ];

    perSystem = { self', pkgs, lib, ... }:
      let
        packageFor = pkgs:
          let
            inherit (lib)
              importTOML
              sourceByRegex
              ;
            Cargo-toml = importTOML ./Cargo.toml;

            pname = "ventora";
            version = Cargo-toml.package.version;

            craneLib = crane.mkLib pkgs;

            src = sourceByRegex ./. [
              "(src)(/.*)?"
              ''Cargo\.(toml|lock)''
            ];

            commonCraneArgs = {
              inherit src pname version;

              nativeBuildInputs = [ pkgs.installShellFiles ];
            };

            cargoArtifacts = craneLib.buildDepsOnly commonCraneArgs;
          in
          craneLib.buildPackage (commonCraneArgs // {
            inherit cargoArtifacts;

            GEN_ARTIFACTS = "artifacts";

            meta.mainProgram = "ventora";
          });

        ventora = packageFor pkgs;
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        packages = {
          default = ventora;
          typst-dev = self'.packages.default;
        };

        overlayAttrs = builtins.removeAttrs self'.packages [ "default" ];

        apps.default = {
          type = "app";
          program = lib.getExe ventora;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rust-analyzer
            rustfmt
          ];
        };
      };
  };
}
