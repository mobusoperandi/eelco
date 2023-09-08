let
  pkgs = import <nixpkgs> {};
  defaultShellArgs = import ./shell.default.nix {inherit pkgs;};
  myShellArgs = {
    packages = with pkgs; [
      # whatever
    ];
  };
in
  pkgs.mkShell (pkgs.lib.recursiveUpdate defaultShellArgs myShellArgs)
