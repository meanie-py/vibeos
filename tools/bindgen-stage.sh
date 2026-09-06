#!/usr/bin/env bash
# Shared setup for the kernel/HAL generators. Source after SCRIPT_DIR and SRC.
# Outputs are staging artifacts; this tool never installs into a live crate.
bindgen_stage() {
    local kind=$1
    shift
    if [ "$#" -eq 0 ]; then
        if [ "$(realpath "${SRC}/.build-tools")" != "${SRC}/.build-tools" ] ||
           [ -L "${SRC}/.build-tools/bindgen-runs" ]; then
            echo "bindgen: refusing a symlinked staging root" >&2
            return 1
        fi
        mkdir -p "${SRC}/.build-tools/bindgen-runs"
        BINDGEN_RUN=$(mktemp -d "${SRC}/.build-tools/bindgen-runs/${kind}.XXXXXX")
    elif [ "$#" -eq 2 ] && [ "$1" = "--output-dir" ]; then
        local parent owner
        parent=$(cd "$(dirname "$2")" && pwd -P)
        owner=$parent
        # Inspect ancestry directly: a Git safe-directory error must not be
        # mistaken for permission to write into a different checkout.
        while [ "$owner" != / ] && [ ! -e "$owner/.git" ]; do
            owner=$(dirname "$owner")
        done
        if [ -e "$owner/.git" ] && [ "$owner" != "$SRC" ]; then
            echo "bindgen: refusing output in another worktree: $owner" >&2
            return 1
        fi
        # mkdir is the reservation: concurrent runs cannot share an output tree.
        mkdir -- "$2" || { echo "bindgen: output directory must be new: $2" >&2; return 1; }
        BINDGEN_RUN=$(cd "$2" && pwd -P)
    else
        echo "Usage: $0 [--output-dir NEW_DIRECTORY] (staging only; never installs bindings)" >&2
        return 2
    fi
    mkdir "${BINDGEN_RUN}/raw" "${BINDGEN_RUN}/src"
    # Used by the sourcing generator, not this setup helper.
    # shellcheck disable=SC2034
    OUT_DIR="${BINDGEN_RUN}/src"
    echo "bindgen: staging in ${BINDGEN_RUN}; COMPLETE is written only after all passes succeed"
}

bindgen_inputs() {
    local arch dir headers links
    if [ ! -d "${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" ]; then
        echo "bindgen: missing pinned clang headers; fix: ./vibeos-cli build --arch amd64" >&2
        return 1
    fi
    for arch in "$@"; do
        dir="${SRC}/build-${arch}"
        # Generated inputs must belong to THIS worktree, never a sibling symlink.
        if [ ! -d "$dir" ] || [ "$(realpath "$dir")" != "$dir" ] ||
           [ ! -f "${dir}/sdk/include/ddk/wdm.h" ] ||
           [ ! -f "${dir}/sdk/include/psdk/ntdef.h" ]; then
            echo "bindgen: missing or foreign generated headers; fix: ./vibeos-cli build --arch ${arch}" >&2
            return 1
        fi
        for headers in "${dir}/sdk/include" "${dir}/ntoskrnl"; do
            [ -e "$headers" ] || continue
            links=$(find "$headers" -type l -print -quit) || return 1
            if [ "$(realpath "$headers")" != "$headers" ] || [ -n "$links" ]; then
                echo "bindgen: refusing symlinked generated headers in ${dir}" >&2
                return 1
            fi
        done
    done
}
