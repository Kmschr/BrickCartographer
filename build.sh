#!/bin/bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
npm run build