# Build plugin and start Orthanc
up:
    cargo build && docker compose up

# Shut down and delete volumes
down:
    docker compose down -v

# Rebuild plugin and restart Orthanc
restart:
    cargo build
    if docker compose ps --status=exited --format '{{ "{{" }}.Name }}' | grep -q 'dev-1$'; then \
      docker compose restart dev; \
    else \
      curl -sSf -X POST http://localhost:8042/tools/reset; \
    fi

# Generate required Rust code
codegen: generate-bindings generate-openapi-client

# Generate Orthanc model files using OpenAPI
generate-openapi-client: mkdir-3rdparty
    wget -O 3rdparty/orthanc-openapi.json 'https://orthanc.uclouvain.be/api/orthanc-openapi.json'
    podman run --rm --userns=keep-id:uid=100100,gid=100100 -u 100100:100100 \
        -v "$PWD/3rdparty:/3rdparty" -w /3rdparty \
        -v "$PWD/openapi-generator.yaml:/openapi-generator.yaml:ro" \
        docker.io/openapitools/openapi-generator-cli:v7.14.0 \
        batch --clean --fail-fast --root-dir /3rdparty  \
        /openapi-generator.yaml
    mkdir -p src/orthanc/models
    fd --no-ignore-vcs --type f --exact-depth 1 --extension .rs . 3rdparty/client/src/models \
       --exec sh -c 'grep -vF "use crate::models;" {} > src/orthanc/models/{/}'

# Generate Orthanc SDK bindings using bindgen
generate-bindings: mkdir-3rdparty
    wget -O 3rdparty/OrthancCPlugin.h 'https://orthanc.uclouvain.be/hg/orthanc/raw-file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h'
    bindgen 3rdparty/OrthancCPlugin.h -o src/orthanc/bindings.rs

# Create 3rdparty directory
mkdir-3rdparty:
    mkdir -p 3rdparty
    echo '*' > 3rdparty/.gitignore

# Push a directory of DICOM files to PACS.
store dir:
    fd --type f --extension '.dcm' . '{{dir}}' \
        | rust-parallel --progress-bar --jobs 2 --discard-output=all just store-straight

# Upload one DICOM file to PACS.
store-straight file:
    curl -sSfX POST -H 'Expect:' -H 'Content-Type: application/dicom' -T '{{file}}' -o /dev/null \
         http://localhost:8042/modalities/PACS/store-straight