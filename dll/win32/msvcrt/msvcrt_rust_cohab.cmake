# Link msvcrt-crt-rust into a CRT DLL whose C is msvcrt_static.
#
# msvcrt.dll is not the only DLL built from msvcrt_static: msvcrt20, msvcrt40 and
# crtdll (i386) link the same archive, compiled once at _MSVCR_VER=0 with the same
# flags. Until this existed they took none of the Rust, so each shipped a complete
# copy of the Wine C that msvcrt.dll had already retired. Same objects, same C being
# displaced, so the evidence that grades msvcrt.dll's Rust -- msvcrt-difftest against
# the genuine msvcrt C -- is the evidence for them too.
#
# Call BEFORE the target's target_link_libraries(... msvcrt_static ...): the archive
# goes in under --whole-archive so every Rust definition exists before msvcrt_static
# is pulled (plain link order is not enough -- the C monolith is still pulled for its
# un-flipped symbols and drags its own copies of the flipped ones). The C copies are
# then ignored duplicates (--allow-multiple-definition). A DLL's OWN module objects
# still come first and keep their version-specific overrides -- crtdll's dll main
# TU is the one that matters, with its own _strinc, _strdec and stat shims.
#
# Included from directories CMake processes BEFORE dll/win32/msvcrt (crtdll), so it
# names the archive by path and the targets by name; both resolve at generate time.
set(MSVCRT_RUST_COHAB_LIB "${CMAKE_BINARY_DIR}/dll/win32/msvcrt/libmsvcrt_crt_cargo.a")

function(link_msvcrt_rust_cohab target)
    add_dependencies(${target} rust_msvcrt_crt)
    # The `...` entry points are in the archive too (msvcrt-crt-rust src/va_entries.rs,
    # varargs-rust va_entry! thunks); the C msvcrt_rust_shims they replace is not built.
    target_link_libraries(${target}
        -Wl,--whole-archive ${MSVCRT_RUST_COHAB_LIB} -Wl,--no-whole-archive)
endfunction()
