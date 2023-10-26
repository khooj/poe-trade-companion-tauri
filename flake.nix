{
  description = "rust workspace";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixgl = {
      url = "github:guibou/nixGL";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
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
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        overlays = [ 
          rust-overlay.overlays.default 
          nixgl.overlay 
          (final: prev: {
            xwin = final.callPackage ./xwin.nix {};
            xwin-output = final.callPackage ./xwin-output.nix {};
          })
        ];
        pkgs = import nixpkgs { inherit localSystem overlays; };
        lib = pkgs.lib;
        rustToolchain = pkgs.rust-bin.stable.${rust-version}.default.override {
              extensions =
                [ "rust-src" "llvm-tools-preview" "rust-analysis" ];
              targets = [ "x86_64-pc-windows-gnu" "x86_64-pc-windows-msvc" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        crossSystem = "x86_64-w64-mingw32";
        # crossSystem = "x86_64-windows";
        pkgsCross = import nixpkgs { inherit crossSystem localSystem overlays; };

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
          rustToolchain
        ] ++ libs;
        nativeBuildInputs = with pkgs; [ pkg-config ];
      in
      {
        packages = {
          # crate2nix can't generate files (panic on array index access)
          # buildRustPackage requires chmod'ing sourceRoot directory and can't do it
          # naersk doesn't work because of this issue https://github.com/nix-community/naersk/issues/310

          app = pkgs.callPackage ./default.nix { inherit craneLib; };
          app-win = pkgs.callPackage ./default.nix {
            inherit craneLib;
            target = "windows";
          };
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
