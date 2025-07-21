# _NeoChRIS_ Orthanc Plugin

This [cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) contains:

| Crate                            | Description                                                              |
|----------------------------------|--------------------------------------------------------------------------|
| [orthanc_api](./orthanc_api)     | Hand-written types for the Orthanc API                                   |
| [orthanc_sdk](./orthanc_sdk)     | Abstractions for developing a Rust Orthanc plugin                        |
| [orthanc_client_ogen_overlay][1] | Automatically generated Orthanc client using OpenAPI                     |
| [blt](./blt)                     | Orthanc plugin for automating the Boston Children's Hospital BLT project |

[1]: ./orthanc_client_ogen_overlay

## Development

This repository depends on automatic code generation (codegen) for:

- Rust bindings to Orthanc's C plugin header
- Orthanc API models (and client) generated from the [OpenAPI specification](https://orthanc.uclouvain.be/api/)

Dependencies for codegen and testing are listed in [./flake.nix](flake.nix) and can be setup automatically
using [nix develop](https://nix.dev/manual/nix/2.30/command-ref/new-cli/nix3-develop.html).
If you don't have [Nix](https://nixos.org), install the packages specified in the
`outputs.devShell.buildInputs` section of `flake.nix` manually.

Run codegen:

```shell
just
```

Or, using `nix develop`:

```shell
nix develop -c just
```

