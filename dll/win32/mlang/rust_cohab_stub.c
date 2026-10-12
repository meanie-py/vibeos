/* Cohabitation placeholder TU for the mlang.dll win32dll MODULE.
 *
 * mlang.c is translated to Rust (sdk/lib/rust/mlang-rust, linked as
 * libmlang_rust.a). mlang.c is removed from the add_library SOURCES and kept on
 * disk per the never-delete rule. The mlang.spec exports + DllMain + COM entry
 * points come from the linked Rust staticlib; cmake keeps the win32dll MODULE
 * link. Mirrors the wmilib-rust / cabinet-rust cohabitation.
 *
 * Below: the wine debug-channel + helper shims. TRACE/WARN/FIXME and
 * IsEqualGUID / debugstr_* / __wine_*_resources are MACROS / inline helpers in
 * C, never real symbols. The mechanical Rust transliteration lowered the macro
 * USES into function calls, so it references undefined symbols. Provide them
 * here with the decoration each Rust decl expects (cdecl for the variadic
 * TRACE/WARN/FIXME; __stdcall for the rest, matching the _name@N referenced).
 */
typedef int mlang_rust_cohab_placeholder;

void TRACE(const char *f, ...) { (void)f; }
void WARN(const char *f, ...)  { (void)f; }
void FIXME(const char *f, ...) { (void)f; }

const char *__stdcall debugstr_guid(const void *g) { (void)g; return ""; }
const char *__stdcall wine_dbgstr_w(const unsigned short *s) { (void)s; return ""; }

/* IsEqualGUID / IsEqualIID: 16-byte compare (inline in guiddef.h). */
int __stdcall IsEqualGUID(const void *a, const void *b)
{
    const unsigned char *pa = (const unsigned char *)a;
    const unsigned char *pb = (const unsigned char *)b;
    int i;
    for (i = 0; i < 16; i++)
        if (pa[i] != pb[i]) return 0;
    return 1;
}
int __stdcall IsEqualIID(const void *a, const void *b) { return IsEqualGUID(a, b); }

int __stdcall min(int a, int b) { return a < b ? a : b; }

/* wine/list.h's helpers are winelist-rust's (a mlang-rust dependency). */

/* debugstr_a/an/w/wn and __wine_(un)register_resources are mlang-rust's
 * (debug_shims.rs). Copies here were duplicate symbols kept by link order;
 * tools/link_guard.py refuses those. */

/* InterlockedIncrement/Decrement: on i386 these come from kernel32 as _X@4, but
 * on amd64/arm64 they are compiler intrinsics the import lib does not export as
 * functions. Provide them as atomic shims (undecorated symbol on amd64/arm64;
 * on i386 the cdecl _InterlockedIncrement is simply unused, kernel32's @4 wins). */
long InterlockedIncrement(long volatile *addend) { return __atomic_add_fetch(addend, 1, __ATOMIC_SEQ_CST); }
long InterlockedDecrement(long volatile *addend) { return __atomic_sub_fetch(addend, 1, __ATOMIC_SEQ_CST); }

/* IStream_* are wine ICOM_CALL helpers (IStream is opaque in ntdll-sys bindgen,
 * so the Rust side can't dispatch by field). Call through the COM vtable by
 * fixed index: IStream vtable = IUnknown(0..2) + Read(3) Write(4) Seek(5)
 * SetSize(6) CopyTo(7) Commit(8) Revert(9) LockRegion(10) UnlockRegion(11)
 * Stat(12) Clone(13). COM methods are __stdcall. */
typedef long (__stdcall *IStreamRead_t)(void *, void *, unsigned long, unsigned long *);
typedef long (__stdcall *IStreamWrite_t)(void *, const void *, unsigned long, unsigned long *);
typedef long (__stdcall *IStreamStat_t)(void *, void *, unsigned long);

long __stdcall IStream_Read(void *self, void *pv, unsigned long cb, unsigned long *pcbRead)
{
    void ***vtbl = (void ***)self;       /* self->lpVtbl */
    IStreamRead_t fn = (IStreamRead_t)(*vtbl)[3];
    return fn(self, pv, cb, pcbRead);
}
long __stdcall IStream_Write(void *self, const void *pv, unsigned long cb, unsigned long *pcbWritten)
{
    void ***vtbl = (void ***)self;
    IStreamWrite_t fn = (IStreamWrite_t)(*vtbl)[4];
    return fn(self, pv, cb, pcbWritten);
}
long __stdcall IStream_Stat(void *self, void *pstatstg, unsigned long grfStatFlag)
{
    void ***vtbl = (void ***)self;
    IStreamStat_t fn = (IStreamStat_t)(*vtbl)[12];
    return fn(self, pstatstg, grfStatFlag);
}
