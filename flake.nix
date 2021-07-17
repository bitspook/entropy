{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crate2nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        crateName = "entropy";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        project = import
          (generatedCargoNix {
            name = crateName;
            src = ./entropy;
          })
          {
            inherit pkgs;
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              ${crateName} = oldAttrs: {
                inherit buildInputs;
              } // buildEnvVars;
            };
          };

        buildEnvVars = {
          OPENSSL_STATIC = 1;
        };

        buildInputs = with pkgs; [ openssl.dev ];

        nativeBuildInputs = with pkgs; [
          nixpkgs-fmt
          pkgconfig
          rust-analyzer
          sqlite
          diesel-cli
          cargo-edit
          cargo-audit
          cargo-outdated
          (rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "cargo"
              "rustc"
              "rust-analysis"
              "rustfmt"
              "clippy"
            ];
            targets = [ "x86_64-unknown-linux-musl" ];
          })
        ];

        entropy = project.rootCrate.build;
      in
      {
        defaultPackage = entropy;

        defaultApp = flake-utils.lib.mkApp {
          drv = project.rootCrate.build;
          name = crateName;
        };

        devShell = pkgs.mkShell
          ({
            inherit buildInputs nativeBuildInputs;
            RUST_BACKTRACE = 1;
          } // buildEnvVars);
      });
}
