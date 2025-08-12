# Orthanc Plugins in Rust

[![Test](https://github.com/FNNDSC/orthanc-rs/actions/workflows/test.yml/badge.svg)](https://github.com/FNNDSC/orthanc-rs/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/FNNDSC/orthanc-rs/graph/badge.svg?token=9RHMEYB2UU)](https://codecov.io/gh/FNNDSC/orthanc-rs)

This [cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) contains:

| Crate                          |  License                        | Version                         | Description                                                              |
|--------------------------------|---------------------------------|---------------------------------|--------------------------------------------------------------------------|
| [orthanc_sdk][cr:sdk]          | ![Crates.io License][li:sdk]    | ![Crates.io Version][ve:sdk]    | Abstractions for developing a Rust Orthanc plugin                        |
| [orthanc_client_ogen][cr:ogen] | ![Crates.io License][li:ogen]   | ![Crates.io Version][ve:ogen]   | Automatically generated Orthanc client using OpenAPI                     |
| [orthanc_api][cr:api]          | ![Crates.io License][li:api]    | ![Crates.io Version][ve:api]    | Hand-written types for the Orthanc API                                   |
| [include_webdir][cr:webdir]    | ![Crates.io License][li:webdir] | ![Crates.io Version][ve:webdir] | Embed a static webapp bundle in a Rust binary                            |
| [examples/basic][fi:example]   | ![Example License][li:example]  | ![Example Version][ve:example]  | Example Orthanc plugin using [orthanc_sdk][cr:sdk]                       |
| [blt](./blt)                   | ![BLT License][li:blt]          | ![BLT Version][ve:blt]          | Orthanc plugin for automating the Boston Children's Hospital BLT project |

[cr:sdk]: https://crates.io/crates/orthanc_sdk
[li:sdk]: https://img.shields.io/crates/l/orthanc_sdk
[ve:sdk]: https://img.shields.io/crates/v/orthanc_sdk
[cr:ogen]: https://crates.io/crates/orthanc_client_ogen
[li:ogen]: https://img.shields.io/crates/l/orthanc_client_ogen
[ve:ogen]: https://img.shields.io/crates/v/orthanc_client_ogen
[cr:api]: https://crates.io/crates/orthanc_api
[li:api]: https://img.shields.io/crates/l/orthanc_api
[ve:api]: https://img.shields.io/crates/v/orthanc_api
[cr:webdir]: https://crates.io/crates/include_webdir
[li:webdir]: https://img.shields.io/crates/l/include_webdir
[ve:webdir]: https://img.shields.io/crates/v/include_webdir

[fi:example]: ./examples/basic/src/plugin.rs
[li:example]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2FFNNDSC%2Forthanc-rs%2Fraw%2Frefs%2Fheads%2Fmaster%2Fexamples%2Fbasic%2FCargo.toml&query=package.license&label=license
[ve:example]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2FFNNDSC%2Forthanc-rs%2Fraw%2Frefs%2Fheads%2Fmaster%2Fexamples%2Fbasic%2FCargo.toml&query=package.version&label=Cargo.toml

[li:blt]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2FFNNDSC%2Forthanc-rs%2Fraw%2Frefs%2Fheads%2Fmaster%2Fblt%2FCargo.toml&query=package.license&label=license
[ve:blt]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2FFNNDSC%2Forthanc-rs%2Fraw%2Frefs%2Fheads%2Fmaster%2Fblt%2FCargo.toml&query=package.version&label=Cargo.toml

## Development

Dependencies for codegen and testing are listed in [`flake.nix`](flake.nix) and can be setup automatically
using [`nix develop`](https://nix.dev/manual/nix/2.30/command-ref/new-cli/nix3-develop.html).

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
