{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem =
    { self'
    , pkgs
    , lib
    , ...
    }:
    {
      rust-project.crates =
        let
          darwinInputs =
            lib.optionals pkgs.stdenv.isDarwin (
              with pkgs.darwin.apple_sdk.frameworks;
              [ IOKit ]
            );
        in
        {
          "frz" = {
            path = ../../crates/frz;
            crane.args.buildInputs = darwinInputs;
          };
          "frz-cli" = {
            path = ../../crates/frz-cli;
            crane.args.buildInputs = darwinInputs;
          };
        };
      packages.default = self'.packages.frz-cli;
    };
}
