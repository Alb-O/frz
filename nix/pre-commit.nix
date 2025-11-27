{ inputs, ... }:
{
  imports = [
    (inputs.git-hooks + /flake-module.nix)
  ];
  perSystem =
    { ... }:
    {
      pre-commit.settings = {
        hooks = {
          treefmt.enable = true;
          cargo-sort.enable = true;
        };
      };
    };
}
