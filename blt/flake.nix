{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
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
            # you are assumed to have `rustup` and `podman` installed globally

            podman-compose
            just
            xh
            jaq
            s5cmd
            rust-parallel
            curl
          ];
        };
      });
}
