/*
 * Cohabitation stub for the Rust basesrv.dll (PARTIAL flip — task #18).
 *
 * The translated basesrv files (appcompat.c, init.c, nls.c, proc.c,
 * sndsntry.c) are commented out of add_library() in CMakeLists.txt; their
 * symbols (DllMain, ServerDllInitialization, BaseSetProcessCreateNotify,
 * BaseSrvNls*, BaseInitializeStaticServerData, BaseStaticServerData, the
 * BASESRV API dispatch/name tables, …) are now provided by
 * libsmss_csrss_rust.a built with `--features basesrv_dll_only,ntddi_ws03`
 * (sdk/lib/rust/smss-csrss-rust/src/basesrv/).
 *
 * dosdev.c + vdm.c have no Rust equivalent yet, so they stay C and keep
 * defining the VDM/DOS-device helpers the Rust dispatch table extern-refs
 * (BaseSrvCheckVDM, BaseSrvGetNextVDMCommand, BaseSrvGetTempFile, …).
 *
 * CMake's add_library() needs at least one TU it always compiles; with the
 * five C files removed the linker pulls the Rust archive members for the
 * now-undefined symbols (--allow-multiple-definition alone does NOT force
 * archive extraction when a direct .obj already defines a symbol). Keep the
 * five .c files on disk (translation philosophy: never delete the C).
 */
