{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      _module.args.devPackages = with pkgs; [
        # Build libraries
        openssl
        pkg-config
        poppler.dev
        glib.dev
        cairo.dev
        # Cargo extensions
        cargo-edit
        cargo-sort
        cargo-insta
        # Utilities
        (writeShellScriptBin "kitty-runner" ''
          exec cargo run -q -p kitty-macro-tests --bin kitty-runner -- "$@"
        '')
      ];
    };
}
