
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fmifs)

add_executable(format format.c format.rc)
set_module_type(format win32cui UNICODE)
target_link_libraries(format conutils)
add_importlibs(format fmifs msvcrt kernel32)
set_target_properties(format PROPERTIES SUFFIX ".com")
add_cd_file(TARGET format DESTINATION reactos/system32 FOR all)
