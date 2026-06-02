/*
 * Cohabitation stub for the Rust DLL-notification registry.
 *
 * ldr/ldrnotify.c is commented out of the ntdll_vista_static SOURCE list
 * (see ../CMakeLists.txt): LdrRegisterDllNotification /
 * LdrUnregisterDllNotification / LdrpSendDllNotifications are now provided
 * by libntdll_rust.a
 * (sdk/lib/rust/ntdll-rust/src/nt_0600/ldr/ldrnotify.rs, UNGATED so both
 * the ntdll.dll and ntdll_vista.dll builds get them — exactly as the C lib
 * was statically linked into both DLLs).
 *
 * ldrnotify.c was the last remaining source of the ntdll_vista_static
 * OBJECT library; this stub keeps that library target valid (a library
 * needs at least one linkable TU). Mirrors rtl_um_rust_stub.c /
 * etwtrace_rust_stub.c.
 *
 * Keep ldrnotify.c on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
