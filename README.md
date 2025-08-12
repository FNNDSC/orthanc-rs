# Orthanc Plugins in Rust

![Crates.io License](https://img.shields.io/crates/l/orthanc_sdk)
[![Test](https://github.com/FNNDSC/orthanc-rs/actions/workflows/test.yml/badge.svg)](https://github.com/FNNDSC/orthanc-rs/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/FNNDSC/orthanc-rs/graph/badge.svg?token=9RHMEYB2UU)](https://codecov.io/gh/FNNDSC/orthanc-rs)

This [cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) contains:

| Crate                            | Version                                                                                          | Description                                                              |
|----------------------------------|--------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------|
| [orthanc_sdk](./orthanc_sdk)     | [![Crates.io Version](https://img.shields.io/crates/v/orthanc_sdk)][orthanc_sdk]                 | Abstractions for developing a Rust Orthanc plugin                        |
| [orthanc_client_ogen][ogen]      | [![Crates.io Version](https://img.shields.io/crates/v/orthanc_client_ogen)][orthanc_client_ogen] | Automatically generated Orthanc client using OpenAPI                     |
| [orthanc_api](./orthanc_api)     | [![Crates.io Version](https://img.shields.io/crates/v/orthanc_api)][orthanc_api]                 | Hand-written types for the Orthanc API                                   |
| [examples/basic][example]        | N/A                                                                                              | Example Orthanc plugin using [orthanc_sdk][orthanc_sdk]                  |
| [blt](./blt)                     | ![Cargo.toml Version][blt-badge]                                                                 | Orthanc plugin for automating the Boston Children's Hospital BLT project |

[ogen]: ./orthanc_client_ogen_overlay
[example]: ./examples/basic/src/plugin.rs
[orthanc_api]: https://crates.io/crates/orthanc_api
[orthanc_sdk]: https://crates.io/crates/orthanc_sdk
[orthanc_client_ogen]: https://crates.io/crates/orthanc_client_ogen
[blt-badge]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2FFNNDSC%2Forthanc-rs%2Fraw%2Frefs%2Fheads%2Fmaster%2Fblt%2FCargo.toml&query=package.version&label=Cargo.toml

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
- Bindgen: https://rust-lang.github.io/rust-bindgen/requirements.html
- OpenAPI Generator: https://openapi-generator.tech/docs/installation
- Podman: https://podman.io/docs/installation
- Bun (to test the examples): https://bun.com/

</details>

### Codegen

This repository depends on automatic code generation (codegen) for:

- Rust bindings to Orthanc's C plugin header
- Orthanc API models (and client) generated from the [OpenAPI specification](https://orthanc.uclouvain.be/api/)

To run the codegen:

```shell
just
```

### Testing

The [examples/](/examples) directory contains both well-documented example
plugins and tests for [orthanc_sdk](./orthan_sdk).

```shell
# optional: collect test coverage data
source <(cargo llvm-cov show-env --export-prefix)
cargo llvm-cov clean --workspace

cd examples
just up &

just test
just down
just clean

# optional: generate HTML coverage report
cargo llvm-cov report --html --ignore-filename-regex orthanc_client_ogen
```

The tests are written with [Bun](https://bun.com).

<details>
<summary>Why Bun?</summary>

Most of `orthanc_sdk` cannot be tested using `cargo test` because it is FFI
code to be invoked by Orthanc. We use Bun to cause the invocation of our code
via REST API calls made using `fetch`.

</details>
