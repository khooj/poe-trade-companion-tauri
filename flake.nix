{
  description = "rust workspace";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixgl = {
      url = "github:guibou/nixGL";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixgl, ... }:
    let
      myapp = "poe-trade-companion";
      rust-version = "1.72.1";
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default nixgl.overlay ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;

        libs = with pkgs; [
          webkitgtk
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl_3
          librsvg
          llvmPackages_15.llvm
          nodejs_20
        ];
        buildInputs = with pkgs; [
          (rust-bin.stable.${rust-version}.default.override {
              extensions =
                [ "rust-src" "llvm-tools-preview" "rust-analysis" ];
          })
        ] ++ libs;
        nativeBuildInputs = with pkgs; [ pkg-config ];
      in
      rec {
        devShell = with pkgs;
          mkShell {
            name = "rust";
            buildInputs = with pkgs; [ pkgs.nixgl.nixGLMesa pkgs.nixgl.nixVulkanMesa ] ++ buildInputs;
            inherit nativeBuildInputs;
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
              vulkan-loader
              xorg.libxcb
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
              libayatana-appindicator
            ];

            shellHook = ''
              export PATH=$PATH:$HOME/.cargo/bin:$PWD/app/node_modules/.bin
            '';
          };
      });
}
