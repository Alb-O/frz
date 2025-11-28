{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem =
    {
      self',
      pkgs,
      lib,
      ...
    }:
    {
      rust-project.crates =
        let
          darwinInputs = lib.optionals pkgs.stdenv.isDarwin (
            with pkgs.darwin.apple_sdk.frameworks; [ IOKit ]
          );
        in
        {
          "frz-core" = {
            path = ../crates/core;
            crane.args.buildInputs = darwinInputs;
          };
          "frz-stream" = {
            path = ../crates/stream;
            crane.args.buildInputs = darwinInputs;
          };
          "frz-tui" = {
            path = ../crates/tui;
            crane.args = {
              buildInputs =
                darwinInputs
                ++ (with pkgs; [
                  poppler.dev
                  glib.dev
                  cairo.dev
                ]);
              nativeBuildInputs = with pkgs; [
                pkg-config
              ];
            };
          };
          "frz-cli" = {
            path = ../crates/cli;
            crane.args.buildInputs = darwinInputs;
          };
        };
      packages.default = self'.packages.frz-cli;
    };
}
