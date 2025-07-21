#!/usr/bin/env bash

jq=$(which jaq 2> /dev/null)
if [ "$?" != 0 ]; then
  jq=$(which jq 2> /dev/null)
  if [ "$?" != 0 ]; then
    echo "error: please install jaq. https://github.com/01mf02/jaq/blob/main/README.md#installation"
    exit 1
  fi
fi

function jq () {
  "$jq" "$@"
}

set -euxo pipefail

# download sample data
if ! [ -e data/image.dcm ]; then
  mkdir data
  xh -do data/image.dcm GET --ignore-stdin https://storage.googleapis.com/idc-open-data/d478b0bd-1f80-4734-8d81-47f20c36d0ab/1753337a-c390-4698-9d9b-f2a5b5a702a6.dcm
fi

# reset output file
cat /dev/null > output.txt

# reset all Orthanc patients
xh GET :8042/patients | jq -r '.[]' | xargs -I _ xh --ignore-stdin DELETE :8042/patients/_

# upload sample data to Orthanc
xh POST :8042/instances Expect: Content-Type:application/dicom < data/image.dcm

# wait a little bit...
sleep 1

# assert that plugin wrote the MRN to output file
grep -q MSB-01723 output.txt

# test custom REST endpoint
actual="$(xh POST --ignore-stdin :8042/rustexample/add a:=3 b:=5 | jq -r .)"
test "$actual" = '8'
