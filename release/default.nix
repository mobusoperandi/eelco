{
  pkgs,
  flake-utils,
}: let
  inherit (pkgs) buildNpmPackage cargo-edit writeShellApplication;

  semantic-release = buildNpmPackage {
    pname = "semantic-release-with-plugins";
    version = "1.0.0";
    src = ./semantic-release-with-plugins;
    npmDepsHash = "sha256-IRIXLSmB9yOnvBrxDkU2tJa+7LUquQRBt4YQ+TyKXK8=";
    dontNpmBuild = true;
  };

  bump-version = writeShellApplication {
    name = "bump-version";
    runtimeInputs = [cargo-edit];
    text = ''
      cargo set-version "$@"
    '';
  };

  release = writeShellApplication {
    name = "release-pr-tracker";
    runtimeInputs = [bump-version];
    text = ''
      ${semantic-release}/bin/semantic-release "$@"
    '';
  };
in {
  bump-version = flake-utils.lib.mkApp {drv = bump-version;};
  release = flake-utils.lib.mkApp {drv = release;};
}
