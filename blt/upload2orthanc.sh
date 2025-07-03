#!/bin/bash
# Purpose: upload a directory of DICOMs to Orthanc.

if ! [ -d "$1" ]; then
  echo "Must give a directory of DICOM files."
  exit 1
fi

unset http_proxy

fd --type f --extension .dcm . "$1" \
  | rust-parallel --progress-bar -j 4 "curl -sSfX POST http://localhost:8042/instances -H Expect: -H 'Content-Type: application/dicom' -T {} -o /dev/null"
