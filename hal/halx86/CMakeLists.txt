
add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fast486)

function(add_hal _halname)
    cmake_parse_arguments(_haldata "" "" "SOURCES;COMPONENTS" ${ARGN})

    # Handle the spec file for the dll name
    spec2def(${_halname}.dll ../hal.spec ADD_IMPORTLIB)

    # Create the actual target
    foreach(_component ${_haldata_COMPONENTS})
        list(APPEND _haldata_SOURCES "$<TARGET_OBJECTS:lib_hal_${_component}>")
    endforeach()
    add_library(${_halname} MODULE
        ${_haldata_SOURCES}
        ${CMAKE_CURRENT_BINARY_DIR}/hal.def)

    if(${_halname} STREQUAL "hal")
        target_link_libraries(${_halname} libcntpr setjmp arbiter fast486 ${PSEH_LIB})
    else()
        target_link_libraries(${_halname} libcntpr setjmp fast486 ${PSEH_LIB})
    endif()

    add_importlibs(${_halname} ntoskrnl)
    #add_pch(${_halname} include/hal.h)
    add_dependencies(${_halname} psdk asm)
    set_module_type(${_halname} kerneldll ENTRYPOINT HalInitSystem 8)
    add_cd_file(TARGET ${_halname} DESTINATION reactos/system32 NO_CAB FOR all)
    if(MSVC)
        target_link_options(${_halname} PRIVATE /ignore:4216 /SECTION:INIT,ERWD)
    elseif(USE_LLVM_MINGW AND TARGET libclang_rt_builtins)
        target_link_libraries(${_halname} libclang_rt_builtins)
    else()
        target_link_libraries(${_halname} -lgcc)
    endif()
endfunction()

# The components
include(generic.cmake)
include(acpi.cmake)
include(apic.cmake)
include(up.cmake)

if(ARCH STREQUAL "i386")
    include(pcidata.cmake)
    include(legacy.cmake)
    include(pic.cmake)
    add_subdirectory(minihal)

    add_hal(hal SOURCES pic/halpic.rc COMPONENTS generic legacy up pic)

elseif(ARCH STREQUAL "amd64")

    add_hal(hal SOURCES ${HAL_SOURCE} COMPONENTS generic acpi up apic)

endif()
