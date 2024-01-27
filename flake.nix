{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {
    self,
    crane,
    flake-utils,
    nixpkgs,
  }: let
    inherit (nixpkgs.lib) optional;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.lib.${system};

      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        inherit (craneLib.crateNameFromCargoToml {cargoToml = ./crates/eelco/Cargo.toml;}) pname;
        buildInputs = optional pkgs.stdenv.isDarwin pkgs.iconv;
        cargoExtraArgs = "--package eelco";
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      packages.default = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          nativeCheckInputs = [pkgs.nix];
          __impureHostDeps = ["/dev/ptmx" "/dev/pts/ptmx"];
          preCheck = ''
            echo "I AM $(whoami)"
            ls -l /dev/
          '';
        }
      );

      devShells.default = craneLib.devShell {
        inputsFrom = [self.packages.${system}.default];
        packages = [pkgs.mob];
      };
    });
}