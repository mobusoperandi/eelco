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
  inputs.treefmt-nix.url = "github:numtide/treefmt-nix";
  inputs.treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

  outputs = {
    self,
    crane,
    fenix,
    flake-compat,
    flake-utils,
    nix,
    nixpkgs,
    treefmt-nix,
  }: let
    inherit (nixpkgs.lib) getExe optional mkForce;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.completeToolchain;
      craneLib = crane.lib.${system}.overrideToolchain toolchain;
      nixDrv = nix.packages.${system}.nix;

      isolatedNix = postfix:
        pkgs.symlinkJoin {
          name = "isolated-nix${postfix}";
          paths = [nixDrv];
          nativeBuildInputs = [pkgs.makeWrapper];
          postBuild = ''
            wrapProgram $out/bin/nix${postfix} \
              --set NIX_CONF_DIR /dev/null \
              --set NIX_USER_CONF_FILES /dev/null \
              --unset NIX_CONFIG
          '';
          meta.mainProgram = "nix${postfix}";
        };

      NIX_CMD_PATH = getExe (isolatedNix "");
      NIX_INSTANTIATE_CMD_PATH = getExe (isolatedNix "-instantiate");

      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = optional pkgs.stdenv.isDarwin pkgs.iconv;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      treefmtEval = treefmt-nix.lib.evalModule pkgs {
        projectRootFile = "flake.nix";
        programs.alejandra.enable = true;
        programs.rustfmt.enable = true;
        settings.formatter.rustfmt.command = mkForce "${toolchain}/bin/rustfmt";
      };
    in {
      packages.default = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts NIX_CMD_PATH NIX_INSTANTIATE_CMD_PATH;
          nativeCheckInputs = [pkgs.nix];
          # 1. integration tests execute `nix`, which fails creating `/nix/var`
          # 2. integration tests require `/dev/ptmx`
          doCheck = false;
        }
      );

      devShells.default = craneLib.devShell {
        inherit NIX_CMD_PATH NIX_INSTANTIATE_CMD_PATH;
        inputsFrom = [self.packages.${system}.default];
        packages = [
          toolchain
          pkgs.mob
        ];
      };

      formatter = treefmtEval.config.build.wrapper;

      checks.formatting = treefmtEval.config.build.check self;
    });
}
