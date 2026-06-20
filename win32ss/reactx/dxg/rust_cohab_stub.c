/* Cohabitation placeholder TU for the dxg.sys kernelmodedriver MODULE.
 *
 * The legacy DirectDraw/Direct3D kernel thunk dxg.sys (main.c/ddhmg.c/d3d.c/dd.c/
 * ddraw.c/ddsurf.c/dxgallocator.c/eng.c/historic.c — DriverEntry, the Dd handle
 * manager, the DxDd / DxD3d / DxDvp dispatch thunks, the DXENG_FUNCTIONS table) plus
 * the code-carrying header dxg_driver.h (the ~75 "TRACE(); return 0;" stub thunks +
 * the gaDxgFuncs DRVFN dispatch table) is translated to Rust (sdk/lib/rust/dxg-rust,
 * linked as libdxg_rust.a). The 9 .c bodies are removed from the add_library SOURCES
 * and kept on disk per the never-delete rule; the Rust #[no_mangle] symbols take over.
 */
typedef int dxg_rust_cohab_placeholder;
