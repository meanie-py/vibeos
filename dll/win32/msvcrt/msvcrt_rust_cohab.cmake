# Link msvcrt-crt-rust into a CRT DLL: msvcrt.dll, and on i386 msvcrt20, msvcrt40 and crtdll.
#
# The archive IS the DLL's CRT. It used to sit ahead of msvcrt_static (the Wine C, compiled
# once at _MSVCR_VER=0) with --allow-multiple-definition picking the Rust copies; once the link
# maps showed no DLL pulling a single msvcrt_static object, the C came off every link line
# (2026-10-04), so a symbol the Rust lacks is now an undefined-symbol error, not a silent C
# fallback. msvcrt_static is still compiled as an unlinked REFERENCE: tools read its objects
# (gen_ctype_tables.py's oracle, gen_msvcrt_ref_protos.py, verify_cohab_coverage.py).
#
# The legacy DLLs' own overrides (crtdll's dllmain.c, msvcrt20.c) are in the archive too,
# under CRTDLL_/MSVCRT20_ names their spec files map (src/legacy.rs), so the one archive can
# serve all four without one DLL's quirk becoming another's export.
#
# --whole-archive keeps every definition, the data exports and DllMain included, without
# relying on what the .def happens to reference. Included from directories CMake processes
# BEFORE dll/win32/msvcrt (crtdll), so it names the archive by path and the targets by name;
# both resolve at generate time.
set(MSVCRT_RUST_COHAB_LIB "${CMAKE_BINARY_DIR}/dll/win32/msvcrt/libmsvcrt_crt_cargo.a")

function(link_msvcrt_rust_cohab target)
    add_dependencies(${target} rust_msvcrt_crt)
    # The `...` entry points are in the archive too (msvcrt-crt-rust src/va_entries.rs,
    # varargs-rust va_entry! thunks); the C msvcrt_rust_shims they replace is not built.
    target_link_libraries(${target}
        -Wl,--whole-archive ${MSVCRT_RUST_COHAB_LIB} -Wl,--no-whole-archive)
endfunction()
