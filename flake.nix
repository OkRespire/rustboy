{
  description = "rustboy";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.stable.toolchain;
        devToolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.stable.rust-src
          fenix.packages.${system}.stable.rust-analyzer
        ];
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        commonArgs = {
          pname = "rustboy";
          version = cargoToml.package.version;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            mold
            clang
          ];
          RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        rustboy = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in
      {
        packages.default = rustboy;
        devShells.default = pkgs.mkShell {
          inputsFrom = [ rustboy ];
          nativeBuildInputs = [ devToolchain ];
          RUST_LOG = "debug";
        };
      }
    );
}
