{ pkgs, lib, config, inputs, ... }:

{
  packages = [ pkgs.git pkgs.lazygit pkgs.rust-analyzer pkgs.cutechess ];

  languages.rust.enable = true;
}
