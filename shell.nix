let
  pkgs = import <nixpkgs> {};
  myShellArgs = {
    packages = with pkgs; [
      alejandra
      iconv
      mob
      rustup
    ];
    MOB_TIMER_ROOM = "mobmore";
  };
in
  pkgs.mkShell myShellArgs
