#!/usr/bin/env bash
# Download the C header for the Orthanc Plugin SDK and generate Rust bindings

set -ex

mkdir 3rdparty
echo '*' > 3rdparty/.gitignore

wget -O 3rdparty/OrthancCPlugin.h 'https://orthanc.uclouvain.be/hg/orthanc/raw-file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h'

exec bindgen 3rdparty/OrthancCPlugin.h -o src/orthanc/bindings.rs
