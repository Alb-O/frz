{ inputs, ... }:
{
  imports = [
    (inputs.git-hooks + /flake-module.nix)
  ];
  perSystem =
    { pkgs, ... }:
    {
      pre-commit.settings = {
        hooks = {
          treefmt.enable = true;
        };
      };
    };
}
