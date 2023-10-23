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
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixgl, crane, ... }:
    let
      myapp = "poe-trade-companion";
      rust-version = "1.72.1";
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ 
          rust-overlay.overlays.default 
          nixgl.overlay 
          (final: prev:  {
            crane = crane;
            craneLib = crane.lib.${system};
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;
        rustBins = pkgs.rust-bin.stable.${rust-version}.default.override {
              extensions =
                [ "rust-src" "llvm-tools-preview" "rust-analysis" ];
              targets = [ "x86_64-pc-windows-gnu" ];
        };
        craneLib = crane.lib.${system};

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
          crate2nix
          yarn
          yarn2nix
        ];
        buildInputs = with pkgs; [
          rustBins
        ] ++ libs;
        nativeBuildInputs = with pkgs; [ pkg-config ];
      in
      {
        packages = {
          # crate2nix can't generate files (panic on array index access)
          # buildRustPackage requires chmod'ing sourceRoot directory and can't do it
          # naersk doesn't work because of this issue https://github.com/nix-community/naersk/issues/310

          # poe-trade-companion = craneLib.buildPackage {
          #   src = craneLib.cleanCargoSource (craneLib.path ./src-tauri);
          # };
          poe-trade-companion = pkgs.callPackage ./default.nix {};
        };
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
