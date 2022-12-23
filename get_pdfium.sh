#!/usr/bin/env bash

set -o pipefail
set -o nounset
set -e

rm -rf pdfium-linux-x64.tgz
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
mkdir -p ./pdfium
tar zxvf pdfium-linux-x64.tgz -C ./pdfium
cp ./pdfium/lib/libpdfium.so libpdfium.so
rm -rf pdfium-linux-x64.tgz
rm -rf ./pdfium
