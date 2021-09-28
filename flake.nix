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
          nixfmt
          postgresql_13
          pkgconfig
          rust-analyzer
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
            shellHook = ''
              export PGDATA=$PWD/postgres/data
              export PGHOST=$PWD/postgres/run
              export LOG_PATH=$PWD/postgres/LOG
              export PGDATABASE=entropy
              export DATABASE_URL="postgresql:///$PGDATABASE?host=$PGHOST"
              export ENTROPY_DATABASE_URL=$DATABASE_URL
              if [ ! -d $PGHOST ]; then
                mkdir -p $PGHOST
              fi
              if [ ! -d $PGDATA ]; then
                echo 'Initializing postgresql database...'
                initdb $PGDATA --auth=trust >/dev/null
              fi

              # pg_ctl start -l $LOG_PATH -o "-c listen_addresses= -c unix_socket_directories=$PGHOST" -W
              # createdb $PGDATABASE
            '';
          } // buildEnvVars);
      });
}
