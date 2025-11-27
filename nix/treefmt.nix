{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem =
    {
      pkgs,
      config,
      lib,
      ...
    }:
    let
      cargo-sort-wrapper = pkgs.writeShellScriptBin "cargo-sort-wrapper" ''
        set -euo pipefail
        for file in "$@"; do
          ${lib.getExe pkgs.cargo-sort} "$(dirname "$file")"
        done
      '';
    in
    {
      treefmt = {
        projectRootFile = "flake.nix";
        programs = {
          nixfmt.enable = true;
          rustfmt.enable = true;
        };
        settings.formatter = {
          "cargo-sort" = {
            command = "${cargo-sort-wrapper}/bin/cargo-sort-wrapper";
            options = [ "--workspace" ];
            includes = [
              "Cargo.toml"
              "**/Cargo.toml"
            ];
          };
        };
      };
      formatter = config.treefmt.build.wrapper;
    };
}
