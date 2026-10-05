/* Cohabitation placeholder TU for the disk.sys kernelmodedriver MODULE.
 *
 * All 5 disk C sources (data/disk/diskwmi/geometry/pnp) are translated to Rust
 * (sdk/lib/rust/disk-rust, linked as libdisk_rust.a) and kept on disk per the
 * never-delete rule. The PE DriverEntry comes from the linked Rust staticlib;
 * cmake keeps the MODULE link (set_module_type kernelmodedriver + add_importlibs
 * (classpnp ntoskrnl hal)). disk.sys exports NOTHING but DriverEntry (no .spec),
 * so this is simpler than the classpnp kerneldll cohab. Mirrors classpnp-rust.
 */
typedef int disk_rust_cohab_placeholder;

/* REVERSE_BYTES_QUAD / _SHORT — header macros (classpnp.h/scsi.h) the Rust
 * translation calls as functions; C guarantees the symbol (cdecl on i386).
 * Byte-reverse a big-endian SCSI field in place. Plain REVERSE_BYTES is Rust's
 * (disk-rust ddk_shims.rs): a copy here was a duplicate symbol, which
 * tools/link_guard.py now refuses. */
void REVERSE_BYTES_QUAD(void *Dest, const void *Src)
{
    unsigned char *d = (unsigned char *)Dest;
    const unsigned char *s = (const unsigned char *)Src;
    int i;
    for (i = 0; i < 8; i++) d[i] = s[7 - i];
}

void REVERSE_BYTES_SHORT(void *Dest, const void *Src)
{
    unsigned char *d = (unsigned char *)Dest;
    const unsigned char *s = (const unsigned char *)Src;
    d[0] = s[1]; d[1] = s[0];
}

/* _Analysis_assume_ — SAL annotation (a no-op at runtime). The Rust translation
 * emits a call to it; rustc's i386 no_mangle decoration for a leading-underscore
 * name is unreliable, so C provides the exact per-arch symbol (i386
 * `__Analysis_assume_`, amd64/arm64 `_Analysis_assume_`). */
void _Analysis_assume_(int Expr) { (void)Expr; }

/* ntstrsafe RtlStringCch* — these are `static __inline` in ntstrsafe.h, so the Rust
 * translation that calls them as extern functions has no symbol to link. Provide them
 * here, backed by ntoskrnl's _vsnprintf/_vsnwprintf. ABI matches ntstrsafe.h:
 * RtlStringCchPrintfA/W are __cdecl (variadic); RtlStringCchCopyA is NTAPI (stdcall on
 * i386, cdecl on amd64/arm64) — the Rust side declares them extern "C" / extern "system"
 * respectively to match. */
#if defined(__i386__)
#define COHAB_NTAPI __attribute__((stdcall))
#else
#define COHAB_NTAPI
#endif
typedef long cohab_ntstatus;
typedef unsigned short cohab_wchar;
typedef __builtin_va_list cohab_va_list;
int __cdecl _vsnprintf(char *buf, unsigned long count, const char *fmt, cohab_va_list ap);
int __cdecl _vsnwprintf(cohab_wchar *buf, unsigned long count, const cohab_wchar *fmt, cohab_va_list ap);

#define COHAB_STATUS_SUCCESS            ((cohab_ntstatus)0)
#define COHAB_STATUS_BUFFER_OVERFLOW    ((cohab_ntstatus)0x80000005L)
#define COHAB_STATUS_INVALID_PARAMETER  ((cohab_ntstatus)0xC000000DL)

cohab_ntstatus __cdecl RtlStringCchPrintfA(char *dst, unsigned long cch, const char *fmt, ...)
{
    int n;
    cohab_va_list ap;
    if (cch == 0) return COHAB_STATUS_INVALID_PARAMETER;
    __builtin_va_start(ap, fmt);
    n = _vsnprintf(dst, cch, fmt, ap);
    __builtin_va_end(ap);
    dst[cch - 1] = '\0';
    if (n < 0 || (unsigned long)n >= cch) return COHAB_STATUS_BUFFER_OVERFLOW;
    return COHAB_STATUS_SUCCESS;
}

cohab_ntstatus __cdecl RtlStringCchPrintfW(cohab_wchar *dst, unsigned long cch, const cohab_wchar *fmt, ...)
{
    int n;
    cohab_va_list ap;
    if (cch == 0) return COHAB_STATUS_INVALID_PARAMETER;
    __builtin_va_start(ap, fmt);
    n = _vsnwprintf(dst, cch, fmt, ap);
    __builtin_va_end(ap);
    dst[cch - 1] = 0;
    if (n < 0 || (unsigned long)n >= cch) return COHAB_STATUS_BUFFER_OVERFLOW;
    return COHAB_STATUS_SUCCESS;
}

cohab_ntstatus COHAB_NTAPI RtlStringCchCopyA(char *dst, unsigned long cch, const char *src)
{
    unsigned long i = 0;
    if (cch == 0) return COHAB_STATUS_INVALID_PARAMETER;
    for (; i < cch - 1 && src[i]; i++) dst[i] = src[i];
    dst[i] = '\0';
    return src[i] ? COHAB_STATUS_BUFFER_OVERFLOW : COHAB_STATUS_SUCCESS;
}
