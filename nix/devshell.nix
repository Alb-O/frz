{ ... }:
{
  perSystem =
    {
      config,
      self',
      pkgs,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        name = "frz-shell";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell
        ];
        packages = with pkgs; [
          openssl
          just
          nixd
          bacon
          cargo-edit
          cargo-sort
        ];
      };
    };
}
