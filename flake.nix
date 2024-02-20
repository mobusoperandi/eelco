{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";
  inputs.fenix.url = "github:nix-community/fenix";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {
    self,
    crane,
    fenix,
    flake-utils,
    nixpkgs,
  }: let
    inherit (nixpkgs.lib) optional;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.completeToolchain;
      craneLib = crane.lib.${system}.overrideToolchain toolchain;

      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = optional pkgs.stdenv.isDarwin pkgs.iconv;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      packages.default = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          nativeCheckInputs = [pkgs.nix];
          # 1. integration tests execute `nix`, which fails creating `/nix/var`
          # 2. integration tests require `/dev/ptmx`
          doCheck = false;
        }
      );

      devShells.default = craneLib.devShell {
        inputsFrom = [self.packages.${system}.default];
        packages = [
          toolchain
          pkgs.mob
        ];
      };
    });
}
