
# FULL flip + CARGO-DRIVEN LINK. Both userinit .c files are replaced by Rust
# (userinit-rust), and the userinit.exe LINK is now driven by cargo
# (userinit-cargo → reactos-link ModuleKind::Win32Exe) instead of cmake
# add_executable — the second of OUR translated spine binaries moved cmake→cargo
# (after lsass). userinit.exe is the post-login shell launcher (winlogon →
# userinit → explorer.exe), so `shell ready` proves the cargo-linked userinit
# runs. Keep both .c on disk (never delete the C — translation philosophy).

# userinit.rc is LOAD-BEARING (application manifest + IDB_ROSLOGO logo bitmaps +
# localized dialogs), so unlike lsass it cannot be omitted. Keep cmake compiling
# it via a resource-only OBJECT library; the cargo link reuses the .res through
# reactos-link .resource_built(). (This is the reusable .res pattern for the GUI
# spine binaries.)
add_library(userinit_res OBJECT userinit.rc)
target_compile_definitions(userinit_res PRIVATE UNICODE _UNICODE)
target_include_directories(userinit_res PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

# The CRT statics + libwine.a the cargo link reuses live in sdk/lib,
# which configure after base/, so DEPENDS on their .a FILE paths for
# parallel-build ordering. The 7 import libs are generated in-cargo from .spec.
# The .rc is now compiled IN-cargo (resource_from_rc replays the userinit_res
# windres command), so the .res is no longer reused — but the userinit_res
# OBJECT lib stays (EXTRA_DEPENDS below) to populate compile_commands.json.

add_cargo_module(userinit userinit-cargo userinit.exe
    KIND win32exe
    BODY_CRATE userinit-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS userinit_res)
