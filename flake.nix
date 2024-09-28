{
  description = "Simfony";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
  { self
  , nixpkgs
  , flake-utils
  , rust-overlay
  , ...
  }:
  flake-utils.lib.eachSystem [
    "x86_64-linux"
    "aarch64-linux"
    "x86_64-darwin"
  ] (system:
    let
      overlays = [
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      mkRust = stable: version: profile: extensions: pkgs.rust-bin.${stable}.${version}.${profile}.override {
        inherit extensions;
      };
      defaultRust = mkRust "stable" "latest" "default" ["rust-src"];
      elementsd-simplicity = pkgs.callPackage ./bitcoind-tests/elementsd-simplicity.nix {};
      CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_16.clang-unwrapped}/bin/clang-16";
      AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_16.libllvm}/bin/llvm-ar";
      CFLAGS_wasm32_unknown_unknown = "-I ${pkgs.llvmPackages_16.libclang.lib}/lib/clang/16/include/";
      default_shell = with_elements: pkgs.mkShell {
          buildInputs = [
            defaultRust
            pkgs.just
            pkgs.gdb
            pkgs.cargo-hack
          ] ++ (
            if with_elements then [ elementsd-simplicity ] else []
          );
          # Constants for IDE
          RUST_TOOLCHAIN = "${defaultRust}/bin";
          RUST_STDLIB = "${defaultRust}/lib/rustlib/src/rust";
          DEBUGGER = "${pkgs.gdb}";
      };
    in
    {
      devShells = {
        default = default_shell false;
        elements = default_shell true;
        # Temporary shells until CI has its nix derivations
        ci = pkgs.mkShell {
          buildInputs = [
            (mkRust "stable" "latest" "default" [])
            pkgs.just
            pkgs.cargo-hack
          ];
        };
        msrv = pkgs.mkShell {
          buildInputs = [
            (mkRust "stable" "1.63.0" "minimal" [])
            pkgs.just
          ];
        };
        fuzz = pkgs.mkShell.override {
          stdenv = pkgs.clang16Stdenv;
        } {
          buildInputs = [
            (mkRust "nightly" "2024-07-01" "minimal" ["llvm-tools-preview"])
            pkgs.just
            pkgs.cargo-fuzz
            pkgs.cargo-binutils
            pkgs.rustfilt
          ];
          # Constants for compiler
          inherit CC_wasm32_unknown_unknown;
          inherit AR_wasm32_unknown_unknown;
          inherit CFLAGS_wasm32_unknown_unknown;
        };
      };
    }
  );
}
