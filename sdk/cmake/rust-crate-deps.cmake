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
