include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

list(APPEND SOURCE
    crashdmp.c
    pagefile.c
    sminit.c
    smloop.c
    smsbapi.c
    smsessn.c
    smsubsys.c
    smutil.c
    smss.c
    smss.h)

add_executable(smss WIN32 ${SOURCE} smss.rc)
target_link_libraries(smss nt ${PSEH_LIB} smlib)
set_module_type(smss nativecui)
add_importlibs(smss ntdll)
add_pch(smss smss.h SOURCE)
add_cd_file(TARGET smss DESTINATION reactos/system32 FOR all)

# Cohabitation note: a working flip needs the Rust smss/* `_main` flow
# to match C parity for SmpAcquirePrivilege / NtWaitForMultipleObjects
# (see task #18 + memory project_smss_csrss_track). The Rust source is
# in tree (sdk/lib/rust/smss-csrss-rust/src/smss/) and per-feature-set
# `cargo check --no-default-features --features smss_exe,smlib,ntddi_ws03`
# is green; only the runtime BUGCHECK 0xc000021a after Global Flags is
# pending. Until then smss.exe stays cmake-driven C — basesrv.dll flip
# stands on its own.
