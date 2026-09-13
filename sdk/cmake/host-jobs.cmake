# Admission wraps actual native tools, never Cargo/Ninja coordinators. Keep the
# existing compile cache inside admission so a cache hit immediately yields its slot.
if(CMAKE_HOST_SYSTEM_NAME STREQUAL "Linux")
    set(_host_jobs "${REACTOS_SOURCE_DIR}/.cache/host-jobs/host-jobs")
    get_property(_compile_launcher GLOBAL PROPERTY RULE_LAUNCH_COMPILE)
    set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE
        "\"${_host_jobs}\" --kind compile -- ${_compile_launcher}")
    set_property(GLOBAL PROPERTY RULE_LAUNCH_LINK
        "\"${_host_jobs}\" --kind link --")
endif()
