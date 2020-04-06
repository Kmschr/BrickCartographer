#!/bin/bash

wasm-pack build src/rust --out-dir ../js/wasm
npm run dev
