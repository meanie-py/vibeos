
list(APPEND UCRT_INTERNAL_SOURCES
    internal/CreateProcessA.cpp
    internal/GetModuleFileNameA.cpp
    internal/LoadLibraryExA.cpp
    internal/OutputDebugStringA.cpp
    internal/peb_access.cpp
    internal/report_runtime_error.cpp
    internal/SetCurrentDirectoryA.cpp
    internal/SetEnvironmentVariableA.cpp
    internal/winapi_thunks.cpp
    internal/win_policies.cpp
)
