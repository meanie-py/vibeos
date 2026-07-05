#!/bin/bash
# Build the vibeos_cov QEMU TCG plugin. Needs glib dev headers only because
# the vendored qemu-plugin.h (v9.0.2, matches the host qemu) includes glib.h;
# the plugin itself is dependency-free C.
set -eu
cd "$(dirname "$0")"
if ! pkg-config --exists glib-2.0; then
    echo "vibeos_cov: glib-2.0 dev headers missing (apt install libglib2.0-dev)" >&2
    exit 1
fi
# shellcheck disable=SC2046
gcc -shared -fPIC -O2 -Wall $(pkg-config --cflags glib-2.0) -o vibeos_cov.so vibeos_cov.c
echo "built $(pwd)/vibeos_cov.so" >&2
