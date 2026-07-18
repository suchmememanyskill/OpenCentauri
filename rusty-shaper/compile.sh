#!/bin/bash
set -euo pipefail

target="${TARGET:-armv7-unknown-linux-musleabihf}"

cross build --locked --release --target "$target"
