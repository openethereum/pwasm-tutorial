#!/bin/bash

# this script is intended to be used from .travis.yml

curl -sL https://storage.googleapis.com/wasm-llvm/builds/linux/31834/wasm-binaries.tbz2 | tar xvkj
