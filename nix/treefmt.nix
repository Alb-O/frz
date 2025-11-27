{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem =
    { pkgs, config, ... }:
    {
      treefmt = {
        projectRootFile = "flake.nix";
        programs = {
          nixfmt.enable = true;
          rustfmt.enable = true;
        };
        settings.formatter = {
          "cargo-sort" = {
            command = "${pkgs.cargo-sort}/bin/cargo-sort";
            args = [ "-w" ];
            includes = [ "**/Cargo.toml" ];
          };
        };
      };
      formatter = config.treefmt.build.wrapper;
    };
}
