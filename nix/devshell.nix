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
          cargo-insta
          pkg-config
          poppler.dev
          glib.dev
          cairo.dev
          # Create a wrapper script for kitty-runner
          (pkgs.writeShellScriptBin "kitty-runner" ''
            exec cargo run -q -p kitty-macro-tests --bin kitty-runner -- "$@"
          '')
        ];
      };
    };
}
