/* Cohabitation TU for the cabinet.dll win32dll MODULE.
 *
 * cabinet_main.c, fci.c, fdi.c and stubs.c are translated to Rust
 * (sdk/lib/rust/cabinet-rust, linked as libcabinet_rust.a). They are removed
 * from the add_library SOURCES and kept on disk per the never-delete rule. All
 * cabinet.spec exports (Extract, DeleteExtractedFiles, DllGetVersion,
 * GetDllVersion plus the FCI and FDI families) come from the linked Rust
 * staticlib; cmake keeps the win32dll MODULE link (spec2def .def + zlib/wine
 * for the deflate/inflate calls the Rust still makes + importlibs). Mirrors the
 * wmilib-rust cohabitation. The precomp.h PCH is dropped: no C TU needs it now.
 *
 * Below: the wine debug-channel shims. In C these are MACROS (WINE_TRACE etc.
 * and debugstr_a / HRESULT_FROM_WIN32 / assert) that expand inline, so they are
 * never real symbols. The mechanical Rust transliteration lowered the macro
 * USES into ordinary function calls, so the Rust references undefined symbols
 * _TRACE / _WARN / _ERR / _FIXME / _debugstr_a / _assert / _HRESULT_FROM_WIN32.
 * Provide them here (cdecl) as faithful no-op / passthrough shims. Variadics
 * live in C so the Rust side needs no unstable c_variadic feature.
 */
typedef int cabinet_rust_cohab_placeholder;

void TRACE(const char *fmt, ...) { (void)fmt; }
void WARN(const char *fmt, ...)  { (void)fmt; }
void ERR(const char *fmt, ...)   { (void)fmt; }
void FIXME(const char *fmt, ...) { (void)fmt; }

const char *debugstr_a(const char *s) { return s; }

void assert(int cond) { (void)cond; }

/* HRESULT_FROM_WIN32(x): x<=0 returns x, else maps the low 16 bits into the
 * FACILITY_WIN32 (7) HRESULT range (0x8007xxxx). */
int HRESULT_FROM_WIN32(unsigned int x)
{
    return (int)x <= 0 ? (int)x : (int)((x & 0x0000FFFFu) | 0x80070000u);
}
