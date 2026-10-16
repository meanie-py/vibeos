# Share dependency storage only among direct, single-package staticlib builds.
# Outer cargo builds (which invoke nested cargo in build.rs) stay separate.
# Each crate has one owner per build tree, so its unhashed final .a cannot be
# replaced by another invocation between cargo returning and CMake copying it.
# Separate invocations retain Cargo's per-package feature resolution.
set(VIBEOS_RUST_STORAGE_SHARDS "4" CACHE STRING "Staticlib Cargo storage shards (0: legacy; 1-16: shared within this build)")
if(NOT VIBEOS_RUST_STORAGE_SHARDS MATCHES "^([0-9]|1[0-6])$")
    message(FATAL_ERROR "VIBEOS_RUST_STORAGE_SHARDS must be an integer from 0 to 16")
endif()

function(rust_storage_target output legacy crate)
    if(VIBEOS_RUST_STORAGE_SHARDS EQUAL 0)
        set(${output} "${legacy}" PARENT_SCOPE)
        return()
    endif()
    if(crate MATCHES "-cargo$")
        message(FATAL_ERROR "rust_storage_target cannot share an outer cargo build: ${crate}")
    endif()
    string(REPLACE "-" "_" _artifact "${crate}")
    get_property(_owner GLOBAL PROPERTY "RUST_STORAGE_OWNER_${_artifact}")
    if(_owner)
        message(FATAL_ERROR "Rust storage artifact ${_artifact} has multiple owners: ${_owner} and ${legacy}")
    endif()
    set_property(GLOBAL PROPERTY "RUST_STORAGE_OWNER_${_artifact}" "${legacy}")
    string(SHA256 _hash "${crate}")
    string(SUBSTRING "${_hash}" 0 7 _hash)
    math(EXPR _shard "0x${_hash} % ${VIBEOS_RUST_STORAGE_SHARDS}")
    set(${output} "${CMAKE_BINARY_DIR}/rust-shared-${_shard}-target" PARENT_SCOPE)
endfunction()

# Nested body builds use a separate set of shards. Outer link coordinators must
# never acquire a target directory that their own build script needs.
option(VIBEOS_ARTIFACT_STORE_MODE "Use supervised immutable amd64 body archive store" OFF)
set(VIBEOS_ARTIFACT_STORE_ROOT "" CACHE PATH "Supervised immutable body store location")
set(VIBEOS_ARTIFACT_VERIFY_NONCE "" CACHE STRING "Force independent artifact verification requests")
set(VIBEOS_ARTIFACT_TOOLCHAIN_ID "" CACHE STRING "Supervisor-verified immutable worker toolchain identity")
set(VIBEOS_RUST_BODY_SHARDS "4" CACHE STRING "Nested Rust body storage shards (0: private; 1-16: shared)")
if(NOT VIBEOS_RUST_BODY_SHARDS MATCHES "^([0-9]|1[0-6])$")
    message(FATAL_ERROR "VIBEOS_RUST_BODY_SHARDS must be an integer from 0 to 16")
endif()
