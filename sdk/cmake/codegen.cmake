# Track native generator sources/templates in each consuming build graph.
include_guard(GLOBAL)
function(vibeos_codegen_dependencies out)
    get_property(__deps GLOBAL PROPERTY VIBEOS_CODEGEN_DEPENDENCIES)
    if(__deps)
        set(${out} ${__deps} PARENT_SCOPE)
        return()
    endif()
    file(GLOB_RECURSE __sources CONFIGURE_DEPENDS
        "${REACTOS_SOURCE_DIR}/tools/vibeos-codegen/src/*")
    set(__deps ${__sources}
        ${REACTOS_SOURCE_DIR}/tools/codegen
        ${REACTOS_SOURCE_DIR}/tools/vibeos-codegen/Cargo.toml
        ${REACTOS_SOURCE_DIR}/tools/vibeos-codegen/Cargo.lock
        ${REACTOS_SOURCE_DIR}/rust-toolchain.toml)
    set_property(GLOBAL PROPERTY VIBEOS_CODEGEN_DEPENDENCIES "${__deps}")
    set(${out} ${__deps} PARENT_SCOPE)
endfunction()
