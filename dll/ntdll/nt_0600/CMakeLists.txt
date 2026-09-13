
spec2def(ntdll_vista.dll ntdll_vista.spec ADD_IMPORTLIB)

add_definitions(
    -D__NTDLL__
    -D_NTOSKRNL_
    -DCRTDLL)

include_directories(
    BEFORE include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# NOTE: ntdll_vista_static (an empty ldrnotify_rust_stub.c holder) + the cmake
# custom_commands that built libntdll_rust.a (vista) and libsmlib_rust.a for the
# old cmake ntdll_vista link were REMOVED when ntdll_vista.dll graduated to the
# cargo-driven link (add_cargo_module below) — ntdll_vista-cargo/build.rs now
# compiles ntdll-rust(+vista) + smss-csrss-rust(smlib_only) itself via
# reactos-link. ldrnotify_rust_stub.c / DllMain.c / the .c bodies stay on disk.

# --- ntdll_vista.dll: link driven by cargo (graduated from cmake add_library) -
# ntdll_vista-cargo/build.rs (reactos-link) produces the shipping DLL:
# rust_staticlib(ntdll-rust, +vista = DllMain/LdrpHeap/ldrnotify) + smlib_only
# smss-csrss-rust + utf8 + the pseh dummy shim, reusing only the Tier-C
# librtl_vista.a (rtl_vista) and importing ntdll's exports (from ntdll.spec).
# Replaces add_library(ntdll_vista MODULE …) + set_module_type(win32dll DllMain
# 12) + add_importlibs(ntdll_vista ntdll) + add_cd_file. KIND win32dll →
# i386 lld forwarder fix only (win32dll user DLL — no native-pefixup). entry
# (_DllMain@12) / image base / GUI subsystem are baked into the build.rs.
# spec2def(… ADD_IMPORTLIB) above STAYS (kernel32_vista imports libntdll_vista).
# BODY_CRATES is NOT optional here even though the cargo build finds these itself:
# it is what puts the crates' .rs into this custom command's DEPENDS, and without
# it ninja has no edge from their sources to this DLL. Editing one then rebuilding
# says "no work to do" while the binary keeps the old body, and the staleness check
# correctly reports BUILD_INCOMPLETE with a remedy (`run build`) that cannot clear
# it -- only `rm`ing the DLL does, which is a recipe nobody should have to know.
#
# The set is not guessed: cargo's own dep-info for this artifact
# (target/ntdll_vista-cargo-*/*/release/ntdll_vista.d) records 16 .rs from
# ntdll-rust and 3 from smss-csrss-rust, which is the same pair dll/ntdll's own
# add_cargo_module declares. Re-derive it from the .d if this ever looks stale.
add_cargo_module(ntdll_vista ntdll_vista-cargo ntdll_vista.dll
    KIND win32dll
    BODY_CRATES ntdll-rust smss-csrss-rust
    EXTRA_DEPENDS rtl_vista)
