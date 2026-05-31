/*
 * Cohabitation stub for the Rust basesrv.dll (FULL flip — task #18/#41).
 *
 * ALL 7 basesrv .c files (appcompat.c, dosdev.c, init.c, nls.c, proc.c,
 * sndsntry.c, vdm.c) are commented out of add_library() in CMakeLists.txt;
 * their symbols (DllMain, ServerDllInitialization, BaseInitializeStaticServerData,
 * BaseStaticServerData, the BASESRV API dispatch/name tables, every BaseSrv*
 * CSR_API handler incl. the VDM/DOS ones, BaseSrvDefineDosDevice, …) are now
 * provided by libsmss_csrss_rust.a built with
 * `--features basesrv_dll_only,ntddi_ws03` (smss-csrss-rust/src/basesrv/).
 *
 * dosdev.c + vdm.c were translated by Opus 4.8 (task #41) after the rustconv
 * scanner learned the CSR_API() macro. basesrv is now C-free at runtime.
 *
 * CMake's add_library() needs at least one TU it always compiles; with every
 * C file removed the linker pulls the Rust archive members for the
 * now-undefined symbols (--allow-multiple-definition alone does NOT force
 * archive extraction when a direct .obj already defines a symbol). Keep the
 * .c files on disk (translation philosophy: never delete the C).
 */
