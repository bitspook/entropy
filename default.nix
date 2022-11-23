{
  pkgs ? import (<nixpkgs>) {}
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    # sbcl
    openssl
    sqlite
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath([pkgs.openssl])}:${pkgs.lib.makeLibraryPath([pkgs.sqlite])}
  '';
}
