#!/bin/bash

set -e
echo "Installing deduper..."


cargo build --release
mkdir -p ~/.local/bin
cp target/release/deduper ~/.local/bin/deduper
echo "Installed! File location: !/.local/bin/deduper"
