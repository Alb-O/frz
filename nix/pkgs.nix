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
          exec nix run github:Alb-O/kitty-test-harness#kitty-runner -- "$@"
        '')
      ];
    };
}
