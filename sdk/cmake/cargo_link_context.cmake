# Outer cargo builds hold their output-directory lock across build.rs and its
# nested compiler/linker work. ONE shared target/ silently serializes Ninja's DAG.
# So each architecture's cells share VIBEOS_CARGO_LINK_SHARDS target dirs (default:
# two per CPU), a cell picking its shard by a stable hash of its crate. A private dir
# per cell (0) compiled the same host build tooling -- the build script, reactos-link
# and cc, ~7.5 MB -- once for each of ~724 cells. Measured 2026-10-16, every cell
# relinked, bodies from the artifact store, 24 CPUs:
#   per cell            11.0 G   i386 4m58s  arm64 7m51s  amd64  9m34s
#   24 per arch (1/CPU)  6.4 G   i386 6m50s  arm64 9m09s  amd64 12m29s
#   48 per arch (2/CPU)  7.0 G   i386 5m18s  arm64 8m04s  amd64  9m48s
# One per CPU left cells waiting on each other's lock; two per CPU costs 2-7%.
function(reactos_cargo_link_context workspace crate arch output_dir)
    get_property(_configured GLOBAL PROPERTY _REACTOS_CARGO_POOL_CONFIGURED)
    if(NOT _configured)
        include(ProcessorCount)
        ProcessorCount(_cpus)
        if(_cpus LESS 1)
            set(_cpus 1)
        endif()
        # These are coordinators, not CPU reservations. The global host-jobs
        # broker admits each actual compiler/linker across all worktrees.
        set(_jobs ${_cpus})
        if(DEFINED ENV{VIBEOS_CARGO_LINK_JOBS})
            set(_jobs "$ENV{VIBEOS_CARGO_LINK_JOBS}")
        endif()
        if(NOT _jobs MATCHES "^[1-9][0-9]*$")
            message(FATAL_ERROR "VIBEOS_CARGO_LINK_JOBS must be a positive integer")
        endif()
        set_property(GLOBAL APPEND PROPERTY JOB_POOLS "vibeos_cargo_links=${_jobs}")
        set_property(GLOBAL PROPERTY _REACTOS_CARGO_POOL_CONFIGURED TRUE)
        math(EXPR _shards "${_jobs} * 2")
        set(VIBEOS_CARGO_LINK_SHARDS "${_shards}" CACHE STRING
            "Cargo cell link target dirs per architecture (0: one per cell)")
        if(NOT VIBEOS_CARGO_LINK_SHARDS MATCHES "^[0-9]+$")
            message(FATAL_ERROR "VIBEOS_CARGO_LINK_SHARDS must be a nonnegative integer")
        endif()
    endif()
    if(VIBEOS_CARGO_LINK_SHARDS EQUAL 0)
        set(${output_dir} "${workspace}/target/${crate}-${arch}-cargo-link" PARENT_SCOPE)
        return()
    endif()
    string(SHA256 _hash "${crate}")
    string(SUBSTRING "${_hash}" 0 7 _hash)
    math(EXPR _shard "0x${_hash} % ${VIBEOS_CARGO_LINK_SHARDS}")
    set(${output_dir} "${workspace}/target/cell-link-v1-${arch}-${_shard}" PARENT_SCOPE)
endfunction()
