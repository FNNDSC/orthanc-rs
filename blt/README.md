# BLT Orthanc Plugin

An Orthanc plugin which automates DICOM data query, retrieval, filtering, anonymization, and upload
for the Boston Children's Hospital BLT collaboration project.

## Development

The development environment uses [rustup](https://rustup.rs/)
and [podman-compose](https://github.com/containers/podman-compose).

Additionally, development scripts are powered by [just](https://just.systems/).
The dependencies for development scripts are specified by the `devShell.buildInputs`
section of [flake.nix](./flake.nix). Using [Nix](https://nixos.org/), you can enter
a development shell by running `nix develop`. Otherwise, you can install the
dependencies manually.

To get the project up and running:

```shell
# compile plugin and start Orthanc
just up

# download and store sample data
just get-data

# trigger a test run
just test-run
```
