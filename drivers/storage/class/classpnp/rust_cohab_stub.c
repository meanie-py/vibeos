/* Cohabitation placeholder TU for the classpnp kerneldll MODULE.
 *
 * All 18 classpnp C sources (autorun/class/classwmi/clntirp/create/data/debug/
 * dictlib/dispatch/guid/history/lock/obsolete/power/retry/srblib/utils/xferpkt) are
 * translated to Rust (sdk/lib/rust/classpnp-rust, linked as libclasspnp_rust.a) and
 * kept on disk per the never-delete rule. The 60 classpnp.spec exports (Class*) plus
 * the PE DriverEntry come from the linked Rust staticlib; cmake keeps the MODULE link
 * (set_module_type kernelmodedriver + spec2def .def + add_importlibs(ntoskrnl hal)).
 * Mirrors the wmilib-rust / i8042prt-rust cohabitation.
 */
typedef int classpnp_rust_cohab_placeholder;

/* REVERSE_BYTES family — header macros (storduid.h/classpnp.h) that the Rust
 * translation calls as functions. Implemented in C because rustc declines to emit a
 * `#[no_mangle]` body for a name that also has a `#[macro_export]` macro in the crate;
 * C guarantees the symbol (cdecl `_REVERSE_BYTES` on i386). Byte-reverse a big-endian
 * SCSI field in place. */
void REVERSE_BYTES(void *Dest, const void *Src)
{
    unsigned char *d = (unsigned char *)Dest;
    const unsigned char *s = (const unsigned char *)Src;
    d[0] = s[3]; d[1] = s[2]; d[2] = s[1]; d[3] = s[0];
}

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

/* _Analysis_assume_ — SAL annotation (a no-op at runtime). The Rust translation emits a
 * call to it; rustc's i386 no_mangle decoration for a leading-underscore name is unreliable,
 * so C provides the exact per-arch symbol (i386 `__Analysis_assume_`, amd64/arm64
 * `_Analysis_assume_`). */
void _Analysis_assume_(int Expr) { (void)Expr; }
