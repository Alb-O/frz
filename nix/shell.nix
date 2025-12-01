{ ... }:
{
  perSystem =
    {
      config,
      pkgs,
      self',
      devPackages,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        name = "frz-shell";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell
        ];
        packages = devPackages;
      };
    };
}
