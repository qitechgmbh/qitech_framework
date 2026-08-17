# Temporary to document dependencies
{
  pkgs ? import <nixpkgs> { },
}:
with pkgs;
mkShell {
  name = "qitech_framework";
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
