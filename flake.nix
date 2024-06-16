{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";
  inputs.fenix.url = "github:nix-community/fenix";
  inputs.flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nix.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nix.url = "github:NixOS/nix/latest-release";

  outputs = {
    self,
    crane,
    fenix,
    flake-compat,
    flake-utils,
    nix,
    nixpkgs,
  }: let
    inherit (nixpkgs.lib) optional;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.completeToolchain;
      craneLib = crane.lib.${system}.overrideToolchain toolchain;
      NIX_BIN_DIR = "${nix.packages.${system}.nix}/bin";

      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = optional pkgs.stdenv.isDarwin pkgs.iconv;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      packages.default = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts NIX_BIN_DIR;
          nativeCheckInputs = [pkgs.nix];
          # 1. integration tests execute `nix`, which fails creating `/nix/var`
          # 2. integration tests require `/dev/ptmx`
          doCheck = false;
        }
      );

      devShells.default = craneLib.devShell {
        inherit NIX_BIN_DIR;
        inputsFrom = [self.packages.${system}.default];
        packages = [
          toolchain
          pkgs.mob
        ];
      };
    });
}
