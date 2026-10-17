#!/usr/bin/env bash
# Build the oracle for x64 and x86 Windows with llvm-mingw and run it on the Windows host
# (WSL interop). The Rust *-windows-gnu targets want libgcc/libgcc_eh; llvm-mingw has
# libunwind instead, so a stub libgcc.a and libunwind-as-libgcc_eh.a are made in target/.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
mingw=$repo/.build-tools/llvm-mingw
win_tmp=${WIN_TMP:-/mnt/c/Users/$(cmd.exe /c 'echo %USERNAME%' 2>/dev/null | tr -d '\r')/AppData/Local/Temp}/ntdll-gated-oracle
export CARGO_TARGET_DIR=$here/target
mkdir -p "$win_tmp"
for pair in x86_64:x64 i686:x86; do
    arch=${pair%%:*}; tag=${pair##*:}
    fix=$CARGO_TARGET_DIR/linkfix/$tag
    mkdir -p "$fix"
    printf '!<arch>\n' > "$fix/libgcc.a"
    cp "$mingw/$arch-w64-mingw32/lib/libunwind.a" "$fix/libgcc_eh.a"
    triple=$arch-pc-windows-gnu
    var=$(echo "$triple" | tr 'a-z-' 'A-Z_')
    export "CARGO_TARGET_${var}_LINKER=$mingw/bin/$arch-w64-mingw32-clang"
    export "CARGO_TARGET_${var}_RUSTFLAGS=-L native=$fix -C link-arg=-Wl,-Xlink=-force:unresolved"
    (cd "$here" && cargo test --target "$triple" --no-run)
    exe=$(ls -t "$CARGO_TARGET_DIR/$triple/debug/deps/"ntdll_gated_oracle-*.exe | head -1)
    cp "$exe" "$win_tmp/oracle-$tag.exe"
    echo "== $tag"
    (cd "$win_tmp" && "./oracle-$tag.exe" --test-threads=1)
done
