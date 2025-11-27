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
        opts=()
        files=()
        while [[ $# -gt 0 ]]; do
          case "$1" in
            --*) opts+=("$1"); shift ;;
            *) files+=("$1"); shift ;;
          esac
        done
        for file in "''${files[@]}"; do
          ${lib.getExe pkgs.cargo-sort} "''${opts[@]}" "$(dirname "$file")"
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
