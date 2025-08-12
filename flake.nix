{
  description = "Development environment for Orthanc plugin SDK in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) { inherit system; };
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            # you are assumed to have `rustup` installed globally
            just
            fd
            xh
            jaq
            openapi-generator-cli
            rustPlatform.bindgenHook

            orthanc
            cargo-llvm-cov
            bun
          ];
        };
      });
}
