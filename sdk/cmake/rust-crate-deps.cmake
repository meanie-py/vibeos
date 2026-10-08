include(${CMAKE_CURRENT_LIST_DIR}/rust-storage.cmake)

# rust-crate-deps.cmake — the source list a cargo module's ninja edge must depend on.
#
# WHY. Every `add_custom_command(OUTPUT lib<x>_rust.a ... DEPENDS ...)` used to list a
# `file(GLOB_RECURSE ... "<crate>/src/*.rs")` of the crate's OWN sources. Cargo tracks path
# dependencies correctly — but only if it is INVOKED, and ninja never invoked it when the
# change was in a dependency. So editing a shared crate (`ntdll-sys`, `ntkrnl-sys`,
# `win32macros-rust`, `strsafe-rust`, …) left every dependent .a "up to date" while being
# stale, and the build went green on a tree that does not compile.
#
# That is a FALSE GREEN, not a slow rebuild: normalizing the ntdll-sys islands to `extern
# "system"` broke kernel32-rust's `stdcall` fn-pointer typedefs on i386, and a warm build
# reported OK, minted a push stamp, and only the merge queue's fresh worktree — which has no
# stale outputs to be wrong about — caught it. Every one of the ~100 cargo edges had the same
# hole, and shared crates are becoming MORE common, not less.
#
# So: walk the crate's `path = "…"` dependency closure out of the Cargo.toml files and depend
# on all of their sources. Over-triggering here is correct — those crates really are stale.

function(rust_crate_sources out_var workspace crate)
    get_filename_component(_root "${workspace}/${crate}" ABSOLUTE)
    set(_pending "${_root}")
    set(_seen "")
    set(_srcs "")

    while(_pending)
        list(POP_FRONT _pending _dir)
        if("${_dir}" IN_LIST _seen)
            continue()
        endif()
        list(APPEND _seen "${_dir}")

        set(_toml "${_dir}/Cargo.toml")
        if(NOT EXISTS "${_toml}")
            # `path` also names FILES ([lib] path = "src/lib.rs") and crates outside the tree.
            # Neither has a manifest here, and neither is a source we can depend on.
            continue()
        endif()

        # CONFIGURE_DEPENDS: without it the glob is evaluated ONCE, at configure time, so a .rs
        # file ADDED later is never an input of this edge until something else forces a
        # reconfigure — and edits to it then leave the .a "up to date". Same false green as
        # above, one level down: a module added to shell32-rust for the COM-server port was
        # edited, the DLL did not rebuild, and smoke booted the stale media without a word.
        # With it, CMake's VerifyGlobs step (the "Re-checking globbed directories" line every
        # build already prints) re-runs this glob and reconfigures when the file set changes.
        # Project mode only: CMake rejects CONFIGURE_DEPENDS under `cmake -P`, which is how
        # test_cli_units exercises this function, and there is no rebuild to track there.
        if(CMAKE_SCRIPT_MODE_FILE)
            file(GLOB_RECURSE _rs "${_dir}/src/*.rs")
        else()
            file(GLOB_RECURSE _rs CONFIGURE_DEPENDS "${_dir}/src/*.rs")
        endif()
        list(APPEND _srcs ${_rs} "${_toml}")
        # A build script is an input to the crate exactly like its sources are, and the islands
        # some of them generate are what this whole function exists to track.
        if(EXISTS "${_dir}/build.rs")
            list(APPEND _srcs "${_dir}/build.rs")
        endif()

        file(STRINGS "${_toml}" _deplines REGEX "path[ \t]*=[ \t]*\"[^\"]+\"")
        foreach(_line IN LISTS _deplines)
            string(REGEX REPLACE ".*path[ \t]*=[ \t]*\"([^\"]+)\".*" "\\1" _rel "${_line}")
            get_filename_component(_dep "${_dir}/${_rel}" ABSOLUTE)
            if(NOT "${_dep}" IN_LIST _seen)
                list(APPEND _pending "${_dep}")
            endif()
        endforeach()
    endwhile()

    list(REMOVE_DUPLICATES _srcs)
    set(${out_var} ${_srcs} PARENT_SCOPE)
endfunction()

# rust_staticlib_variant(<name> <crate> "<features>" <defaults> [extra DEPENDS...]
#                        [ENV <VAR=value>...])
#
# One user-mode staticlib build of workspace crate <crate> with exactly <features>
# (cargo defaults off unless <defaults>), copied to ${CMAKE_CURRENT_BINARY_DIR}/lib<name>.a,
# with a `rust_<name>` target and `<name>_LIB` set in the caller. Each variant gets its own
# target dir: rust_storage_target allows one owner per crate artifact, and a crate built
# with several feature sets (crt-startup-rust's personalities) needs one per set. Extra
# arguments are extra DEPENDS, for sources the crate compiles from outside itself; after
# ENV, environment for cargo (`RUSTFLAGS=...`: i386 user code that needs SSE2, as the
# workspace config gives i686 -sse,-sse2).
function(rust_staticlib_variant name crate features defaults)
    cmake_parse_arguments(_V "" "" "ENV" ${ARGN})
    set(_ws "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    if(ARCH STREQUAL "i386")
        set(_triple "i686-pc-windows-gnu")
    elseif(ARCH STREQUAL "amd64")
        set(_triple "x86_64-pc-windows-gnu")
    elseif(ARCH STREQUAL "arm64")
        set(_triple "aarch64-pc-windows-gnullvm")
    endif()
    set(_dir "${CMAKE_CURRENT_BINARY_DIR}/rust-${name}-target")
    set(_lib "${CMAKE_CURRENT_BINARY_DIR}/lib${name}.a")
    string(REPLACE "-" "_" _artifact "${crate}")
    rust_crate_sources(_srcs "${_ws}" ${crate})
    set(_feat)
    if(NOT defaults)
        list(APPEND _feat --no-default-features)
    endif()
    if(features)
        list(APPEND _feat --features ${features})
    endif()
    add_custom_command(
        OUTPUT ${_lib}
        COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${_dir} ${_V_ENV}
            cargo build --release --target ${_triple} -p ${crate} ${_feat}
                --manifest-path ${_ws}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy ${_dir}/${_triple}/release/lib${_artifact}.a ${_lib}
        DEPENDS ${_srcs} ${_ws}/${crate}/Cargo.toml ${_V_UNPARSED_ARGUMENTS}
        WORKING_DIRECTORY ${_ws}
        COMMENT "Building ${crate} [${features}] (${_triple})"
        VERBATIM)
    add_custom_target(rust_${name} DEPENDS ${_lib})
    set(${name}_LIB ${_lib} PARENT_SCOPE)
endfunction()

