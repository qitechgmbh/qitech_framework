# Temporary to document dependencies
{
  pkgs ? import <nixpkgs> { },
}:
with pkgs;
mkShell {
  name = "control-v2";
  buildInputs = [
    rustc
    cargo
    pkg-config
    systemdLibs
    # dev tools
    rust-analyzer
    rustfmt
  ];
}
