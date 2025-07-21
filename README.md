# Orthanc Plugins in Rust

This [cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) contains:

| Crate                            | Description                                                              |
|----------------------------------|--------------------------------------------------------------------------|
| [orthanc_api](./orthanc_api)     | Hand-written types for the Orthanc API                                   |
| [orthanc_sdk](./orthanc_sdk)     | Abstractions for developing a Rust Orthanc plugin                        |
| [orthanc_client_ogen_overlay][1] | Automatically generated Orthanc client using OpenAPI                     |
| [blt](./blt)                     | Orthanc plugin for automating the Boston Children's Hospital BLT project |

[1]: ./orthanc_client_ogen_overlay

## Development

Dependencies for codegen and testing are listed in [./flake.nix](flake.nix) and can be setup automatically
using [nix develop](https://nix.dev/manual/nix/2.30/command-ref/new-cli/nix3-develop.html).

```shell
nix develop -c just
```

<details>
<summary>

### Instructions for Debian, Ubuntu, or Other

</summary>

> [!WARNING]
> I use Nix myself, so these instructions are untested.

If you don't want to use [Nix](https://nixos.org), install the packages specified
in the `outputs.devShell.buildInputs` section of `flake.nix` manually.

On Ubuntu or Debian, some basic dependencies can be installed using `apt`:

```shell
sudo apt update
sudo apt install just fd-find xh podman-compose
```

You will also need these, which are trickier to install:

- Rust: https://rustup.rs
- Bindgen: https://rust-lang.github.io/rust-bindgen/command-line-usage.html
- OpenAPI Generator: https://openapi-generator.tech/docs/installation
- Podman: https://podman.io/docs/installation

</details>

### Codegen

This repository depends on automatic code generation (codegen) for:

- Rust bindings to Orthanc's C plugin header
- Orthanc API models (and client) generated from the [OpenAPI specification](https://orthanc.uclouvain.be/api/)

```shell
just
```

### Testing

[example_plugin](/example_plugin) is both a well-documented example plugin and
a test for the [orthanc_sdk](./orthan_sdk) and [orthanc_api](orthanc_api) crates.

```shell
cd example_plugin
just up -d
sleep 1

just test

just down
```
