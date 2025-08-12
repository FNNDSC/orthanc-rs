# Generate required Rust code
codegen: download-header generate-client

# Generate Orthanc model files using OpenAPI.
generate-client: _openapi_generator && _postprocess_client

_openapi_generator:
    openapi-generator-cli batch --clean --fail-fast --threads 1 openapi-generator.yaml

# Post-processing of openapi-generator-cli output.
_postprocess_client: (_fix_rustdoc_all 'orthanc_client_ogen/src') _overlay_client

# Overwrite openapi-generator-cli created files with manually specified content.
_overlay_client:
    cp -rfv orthanc_client_ogen_overlay/* orthanc_client_ogen

# Fix all Rustdoc syntax errors produced by OpenAPI-generator.
_fix_rustdoc_all dir:
    fd --no-ignore-vcs --type f --extension .rs . '{{dir}}' \
       --exec just -q _fix_rustdoc_links '{}'

# Fix Rustdoc syntax errors produced by OpenAPI-generator.
_fix_rustdoc_links file:
    # surround trailing URLs with `<`, `>` characters
    sed -i -e 's#\(///.*\(:\|\.\) \)\(https://[a-zA-Z0-9\./]*\)#\1<\3>#' '{{file}}'
    # escape `[`, `]` in "[0,n] range" snippets
    sed -i -e 's#\(///.*\) \[\([0-9]*,[0-9]*\)\]\( range\.\)#\1 \\[\2\\]\3#' '{{file}}'

# Download Orthanc plugin C header file
download-header:
    mkdir -p 3rdparty
    wget -O 3rdparty/OrthancCPlugin.h 'https://orthanc.uclouvain.be/hg/orthanc/raw-file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h'

