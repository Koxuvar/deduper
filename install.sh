#!/bin/bash

echo "Installing deduper..."

cargo build --release

wait $#
