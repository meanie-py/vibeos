# Outer cargo builds hold their output-directory lock across build.rs and its
# nested compiler/linker work. A shared target/ silently serializes Ninja's DAG.
# Keep each module AND architecture independent; sccache shares immutable work.
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
    endif()
    set(${output_dir} "${workspace}/target/${crate}-${arch}-cargo-link" PARENT_SCOPE)
endfunction()
