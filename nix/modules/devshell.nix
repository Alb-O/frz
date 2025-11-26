{ ... }:
{
  perSystem =
    { config
    , self'
    , pkgs
    , ...
    }:
    {
      devShells.default = pkgs.mkShell {
        name = "frz-shell";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell # See ./nix/modules/pre-commit.nix
        ];
        packages = with pkgs; [
          just
          nixd
          bacon
          cargo-edit
          cargo-sort
        ];
      };
    };
}
