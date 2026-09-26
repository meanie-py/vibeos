use win32k_sys::win32k::*;
use ntkrnl_sys::CONTAINING_RECORD;
use ntkrnl_sys::ddk::{ExEnterCriticalRegionAndAcquireFastMutexUnsafe, ExReleaseFastMutexUnsafeAndLeaveCriticalRegion};
use ntkrnl_sys::ddk::{MmCreateSection, MmMapViewInSystemSpace, MmUnmapViewInSystemSpace, ObReferenceObjectByHandle, PsGetCurrentProcessWin32Process, RtlGetDefaultCodePage, ZwClose};
use core::ffi::c_void;

// FT_IS_SFNT / FT_HAS_KERNING come from `ft_sys::ft::*` (hand-impl macros) — not externs.
// CRT string helpers are cdecl symbols from the kernel CRT.
extern "C" {
    fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcslen(s: *const u16) -> usize;
}

// IntLockFreeType()/IntUnLockFreeType() are macros (freetype.c) over the FreeType
// global fast-mutex; inline them (a C shim can't reach g_FreeTypeLock once the C is gone).
#[inline(always)]
unsafe fn IntLockFreeType() {
    ntkrnl_sys::ddk::ExEnterCriticalRegionAndAcquireFastMutexUnsafe(g_FreeTypeLock as _ );
}
#[inline(always)]
unsafe fn IntUnLockFreeType() {
    ntkrnl_sys::ddk::ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(g_FreeTypeLock as _ );
}

// ---- integration preamble (consts/macros/globals the translation needs) ----
use ntkrnl_sys::base::NT_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// NTSTATUS codes (not in the win32k_sys glob).
pub const STATUS_SUCCESS: NTSTATUS = 0;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005u32 as NTSTATUS;

// Pool tags (C multichar = big-endian).
pub const TAG_FONT: u32 = u32::from_be_bytes(*b"ETNF");
pub const GDITAG_TEXT: u32 = u32::from_be_bytes(*b"txtG");
pub const TAG_FINF: u32 = u32::from_be_bytes(*b"FNIF");
pub const TAG_USTR: u32 = u32::from_be_bytes(*b"RTSU");
pub const UNICODE_NULL: u16 = 0;
// FreeType lowercase macro alias for the FT_Sfnt_Tag enum value.
pub const ft_sfnt_os2: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_OS2;

// ObjToGDI!(obj, FONT) = (FONTGDI*)(obj); _countof!(arr) = element count.
macro_rules! ObjToGDI { ($obj:expr, $ty:ident) => { ($obj) as PFONTGDI }; ($obj:expr) => { ($obj) as PFONTGDI }; }
macro_rules! _countof { ($arr:expr) => { ($arr).len() }; }

// CRT abs/labs + InitializeListHead FORCEINLINE.
#[inline(always)] unsafe fn abs(x: core::ffi::c_int) -> core::ffi::c_int { x.wrapping_abs() }
#[inline(always)] unsafe fn labs(x: core::ffi::c_long) -> core::ffi::c_long { x.wrapping_abs() }
#[inline(always)] unsafe fn InitializeListHead(h: PLIST_ENTRY) { (*h).Flink = h; (*h).Blink = h; }
#[inline(always)] unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*gpsi).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// File-static list heads (C: RTL_STATIC_LIST_HEAD self-ref). Null here; InitializeListHead'd
// at the top of InitFontSupport before any font-list access (matches C static self-ref).
static mut g_FontListHead: _LIST_ENTRY = _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
static mut g_FontCacheListHead: _LIST_ENTRY = _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
static mut g_FontLinkCache: _LIST_ENTRY = _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
static mut g_FontSubstListHead: _LIST_ENTRY = _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
static mut s_FontLookupCacheList: _LIST_ENTRY = _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };

// --- long-tail consts / helpers / dropped globals ---
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;
pub const STATUS_NOT_FOUND: NTSTATUS = 0xC0000225u32 as NTSTATUS;
pub const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008u32 as NTSTATUS;
pub const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as NTSTATUS;
pub const GDITAG_PFF: u32 = u32::from_be_bytes(*b"Gpff");   // 'ffpG'
pub const GDITAG_RFONT: u32 = u32::from_be_bytes(*b"Gfnt"); // 'tnfG'
pub const TAG_INTERNAL_SYNC: u32 = u32::from_be_bytes(*b"cnys"); // C literal 'cnys' -- win32ss/user/ntuser/tags.h
pub const TTAG_CFF: FT_ULong = u32::from_be_bytes(*b"CFF ") as FT_ULong;
type LONG_MAX_TYPE = i64;
// FT_Sfnt_Tag enum-value aliases (FreeType has UPPER + lower spellings; bindgen prefixes).
pub const FT_SFNT_OS2: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_OS2;
pub const FT_SFNT_HHEA: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_HHEA;
pub const FT_SFNT_POST: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_POST;
pub const ft_sfnt_hhea: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_HHEA;
pub const ft_sfnt_post: FT_Sfnt_Tag_ = FT_Sfnt_Tag__FT_SFNT_POST;

#[inline(always)] fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)] unsafe fn w_str_space() -> *const u16 { w!(" ").as_ptr() as *const u16 }
#[inline(always)] unsafe fn w_str_amp() -> *const u16 { w!("&").as_ptr() as *const u16 }

// CRT cdecl helpers + the extern-const default matrix (defined in another gdi TU).
extern "C" {
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    fn memcmp(a: *const core::ffi::c_void, b: *const core::ffi::c_void, n: usize) -> i32;
    fn strcmp(a: *const i8, b: *const i8) -> i32;
    fn _wtoi(s: *const u16) -> i32;
    fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    // freetype.c HACK: `#define gmxWorldToDeviceDefault gmxWorldToPageDefault`
    // — use the 1:1 identity page matrix (see dclife.rs DC_vInitDc note).
    static gmxWorldToPageDefault: MATRIX;
}
static mut g_PointZero: POINTL = POINTL { x: 0, y: 0 };

// Dropped file globals (C: static const UNICODE_STRING = RTL_CONSTANT_STRING(L"...")).
static DosPathPrefix_W: [u16; 5] = [0x005c, 0x003f, 0x003f, 0x005c, 0x0000];
static mut DosPathPrefix: _UNICODE_STRING = _UNICODE_STRING { Length: 8, MaximumLength: 10, Buffer: DosPathPrefix_W.as_ptr() as *mut u16 };
static g_FontRegPath_W: [u16; 69] = [0x005c, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052, 0x0059, 0x005c, 0x004d, 0x0061, 0x0063, 0x0068, 0x0069, 0x006e, 0x0065, 0x005c, 0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x0020, 0x004e, 0x0054, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0046, 0x006f, 0x006e, 0x0074, 0x0073, 0x0000];
static mut g_FontRegPath: _UNICODE_STRING = _UNICODE_STRING { Length: 136, MaximumLength: 138, Buffer: g_FontRegPath_W.as_ptr() as *mut u16 };

// usize-accepting wrappers (i386 SIZE_T = u32) — same as font/path/ntbitblt.
#[inline(always)] unsafe fn ExAllocatePoolWithTag(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID { ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag) }
#[inline(always)] unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) { ntkrnl_sys::ddk::ExFreePoolWithTag(p, tag) }
#[inline(always)] unsafe fn RtlZeroMemory(d: *mut c_void, n: usize) { ntkrnl_sys::rtl::RtlZeroMemory(d, n as _) }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) { ntkrnl_sys::rtl::RtlCopyMemory(d, s, n as _) }

// LIST_ENTRY FORCEINLINE ops (NT semantics).
#[inline(always)] unsafe fn IsListEmpty(h: PLIST_ENTRY) -> u8 { ((*h).Flink == h) as u8 }
#[inline(always)] unsafe fn InsertHeadList(h: PLIST_ENTRY, e: PLIST_ENTRY) { let f=(*h).Flink; (*e).Flink=f; (*e).Blink=h; (*f).Blink=e; (*h).Flink=e; }
#[inline(always)] unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) { let b=(*h).Blink; (*e).Flink=h; (*e).Blink=b; (*b).Flink=e; (*h).Blink=e; }
#[inline(always)] unsafe fn RemoveEntryList(e: PLIST_ENTRY) -> u8 { let f=(*e).Flink; let b=(*e).Blink; (*b).Flink=f; (*f).Blink=b; (f==b) as u8 }
#[inline(always)] unsafe fn RemoveHeadList(h: PLIST_ENTRY) -> PLIST_ENTRY { let e=(*h).Flink; RemoveEntryList(e); e }
#[inline(always)] unsafe fn RemoveTailList(h: PLIST_ENTRY) -> PLIST_ENTRY { let e=(*h).Blink; RemoveEntryList(e); e }

// SCALE_X/Y: C macros; the translator passed the scale as a 2nd arg.
#[inline(always)] unsafe fn SCALE_X(value: FT_Fixed, scale: FT_Fixed) -> FT_Long { (FT_MulFix(value, scale) + 32) >> 6 }
#[inline(always)] unsafe fn SCALE_Y(value: FT_Fixed, scale: FT_Fixed) -> FT_Long { (FT_MulFix(value, scale) + 32) >> 6 }

// Per-process private-font lock (text.h FORCEINLINE over W32PROCESS.PrivateFontListLock).
#[inline(always)] unsafe fn IntLockProcessPrivateFonts(w32: *mut c_void) {
    ntkrnl_sys::ddk::ExEnterCriticalRegionAndAcquireFastMutexUnsafe(core::ptr::addr_of_mut!((*(w32 as *mut _PROCESSINFO)).PrivateFontListLock) as _);
}
#[inline(always)] unsafe fn IntUnLockProcessPrivateFonts(w32: *mut c_void) {
    ntkrnl_sys::ddk::ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*(w32 as *mut _PROCESSINFO)).PrivateFontListLock) as _);
}

// win32k registry helpers (stdcall).
extern "system" {
    fn RegOpenKey(pwszKeyName: *const u16, phkResult: *mut HKEY) -> NTSTATUS;
    fn RegQueryValue(hkey: HKEY, pwszValueName: *const u16, dwType: ULONG, pvData: PVOID, pcbData: *mut ULONG) -> NTSTATUS;
}

// RTL string functions — declared locally so they use win32k_sys::_UNICODE_STRING/_STRING.
extern "system" {
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16);
    fn RtlFreeUnicodeString(UnicodeString: *mut _UNICODE_STRING);
    fn RtlCreateUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16) -> BOOLEAN;
    fn RtlEqualUnicodeString(String1: *const _UNICODE_STRING, String2: *const _UNICODE_STRING, CaseInSensitive: BOOLEAN) -> BOOLEAN;
    fn RtlCompareUnicodeString(String1: *const _UNICODE_STRING, String2: *const _UNICODE_STRING, CaseInSensitive: BOOLEAN) -> LONG;
    fn RtlAppendUnicodeStringToString(Destination: *mut _UNICODE_STRING, Source: *const _UNICODE_STRING) -> NTSTATUS;
    fn RtlAppendUnicodeToString(Destination: *mut _UNICODE_STRING, Source: *const u16) -> NTSTATUS;
    fn RtlInitAnsiString(DestinationString: *mut _STRING, SourceString: *const i8);
    fn RtlAnsiStringToUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const _STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
    fn ZwOpenFile(FileHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut OBJECT_ATTRIBUTES, IoStatusBlock: *mut IO_STATUS_BLOCK, ShareAccess: ULONG, OpenOptions: ULONG) -> NTSTATUS;
    fn ZwOpenKey(KeyHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: *mut OBJECT_ATTRIBUTES) -> NTSTATUS;
    fn ZwQueryKey(KeyHandle: HANDLE, KeyInformationClass: ULONG, KeyInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG) -> NTSTATUS;
    fn ZwQueryValueKey(KeyHandle: HANDLE, ValueName: *const _UNICODE_STRING, KeyValueInformationClass: ULONG, KeyValueInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG) -> NTSTATUS;
    fn ZwSetValueKey(KeyHandle: HANDLE, ValueName: *const _UNICODE_STRING, TitleIndex: ULONG, Type: ULONG, Data: PVOID, DataSize: ULONG) -> NTSTATUS;
    fn ZwEnumerateValueKey(KeyHandle: HANDLE, Index: ULONG, KeyValueInformationClass: ULONG, KeyValueInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG) -> NTSTATUS;
    fn ZwQueryObject(Handle: HANDLE, ObjectInformationClass: ULONG, ObjectInformation: PVOID, ObjectInformationLength: ULONG, ReturnLength: *mut ULONG) -> NTSTATUS;
}

// FLOATOBJ soft-float ops are FORCEINLINE/arch-divergent — reuse path_rust_shims.c's
// cdecl w32r_FLOATOBJ_* wrappers (+ 3 added there for GetLong/Mul/Equal). Calls below
// are renamed FLOATOBJ_* -> w32r_FLOATOBJ_*.
extern "C" {
    fn w32r_FLOATOBJ_GetLong(pf: *mut FLOATOBJ) -> LONG;
    fn w32r_FLOATOBJ_MulLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_SetLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_Mul(pf: *mut FLOATOBJ, pf2: *mut FLOATOBJ);
    fn w32r_FLOATOBJ_AddLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_SetFloat(pf: *mut FLOATOBJ, f: FLOATL);
    fn w32r_FLOATOBJ_DivLong(pf: *mut FLOATOBJ, l: LONG);
    fn w32r_FLOATOBJ_Equal(pf: *mut FLOATOBJ, pf2: *mut FLOATOBJ) -> BOOL;
}

// ---- Group A: FORCEINLINE kernel helpers with no exported symbol — inline them ----

// ExInitializeFastMutex: Count=FM_LOCK_BIT(1), Owner=null, Contention=0, init Event.
#[inline(always)]
unsafe fn ExInitializeFastMutex(fm: *mut FAST_MUTEX) {
    (*fm).Count = 1;
    (*fm).Owner = core::ptr::null_mut();
    (*fm).Contention = 0;
    ntkrnl_sys::ddk::KeInitializeEvent(core::ptr::addr_of_mut!((*fm).Event) as *mut _, 1, 0);
}

// ExInitializePushLock: set the Ptr field (first field) to 0.
#[inline(always)]
unsafe fn ExInitializePushLock<T>(push_lock: *mut T) {
    *(push_lock as *mut usize) = 0;
}

// ExAllocatePoolZero: allocate + zero (FORCEINLINE = ExAllocatePoolWithTag + RtlZeroMemory).
#[inline(always)]
unsafe fn ExAllocatePoolZero(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    let p = ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag);
    if !p.is_null() {
        ntkrnl_sys::rtl::RtlZeroMemory(p, n as _);
    }
    p
}

// RtlInitEmptyUnicodeString: Buffer=buf, Length=0, MaximumLength=bufsize_bytes.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(dst: *mut _UNICODE_STRING, buf: *mut u16, len: USHORT) {
    (*dst).Buffer = buf;
    (*dst).Length = 0;
    (*dst).MaximumLength = len;
}

// ---- Group B: shim externs (font_rust_shims.c / ntbitblt_rust_shims.c) ----
extern "C" {
    fn w32r_DC_pmxWorldToDevice(pdc: PDC) -> PMATRIX;
    fn w32r_TEXTOBJ_LockText(hfont: HFONT) -> PTEXTOBJ;
    fn w32r_TEXTOBJ_UnlockText(pto: PTEXTOBJ);
}

// ---- Group C: ntstrsafe header-inline wrappers (freetype_rust_shims.c) ----
extern "C" {
    fn w32r_RtlStringCbCopyW(d: *mut u16, cb: usize, s: *const u16) -> NTSTATUS;
    fn w32r_RtlStringCchCatW(d: *mut u16, cch: usize, s: *const u16) -> NTSTATUS;
    fn w32r_RtlStringCchCopyNW(d: *mut u16, cch: usize, s: *const u16, n: usize) -> NTSTATUS;
    fn w32r_RtlStringCchCopyW(d: *mut u16, cch: usize, s: *const u16) -> NTSTATUS;
    fn w32r_RtlULongMult(a: ULONG, b: ULONG, out: *mut ULONG) -> NTSTATUS;
    fn w32r_RtlStringCbPrintfW_2(d: *mut u16, cb: usize, fmt: *const u16, a1: *const u16) -> NTSTATUS;
}

// Simple C macros expressed as inline fns.
#[inline(always)] unsafe fn PRIMARYLANGID(l: u32) -> u16 { (l as u16) & 0x3ff }
#[inline(always)] unsafe fn RtlEqualMemory(d: *const c_void, s: *const c_void, n: usize) -> bool { memcmp(d, s, n) == 0 }
#[inline(always)] unsafe fn IS_HIGH_SURROGATE(ch: u32) -> bool { ch >= 0xD800 && ch <= 0xDBFF }
#[inline(always)] unsafe fn IS_LOW_SURROGATE(ch: u32) -> bool { ch >= 0xDC00 && ch <= 0xDFFF }
#[inline(always)] unsafe fn Utf32FromSurrogatePair(high: u32, low: u32) -> u32 { 0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00) }
#[inline(always)] unsafe fn ASSERT_FREETYPE_LOCK_NOT_HELD() {}
#[inline(always)] unsafe fn ObDereferenceObject(o: *mut c_void) { ntkrnl_sys::ddk::ObfDereferenceObject(o as _); }
macro_rules! RTL_NUMBER_OF { ($a:expr) => { ($a).len() }; }

// wide_str!("ascii"): const-encode an ASCII string literal to a NUL-terminated
// static [u16; N] and yield a *const u16 (PCWSTR). Used by the arm64 font path.
macro_rules! wide_str {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        static W: [u16; N] = {
            let b = $s.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() { w[i] = b[i] as u16; i += 1; }
            w
        };
        W.as_ptr()
    }};
}

// IntEngFillBox = FASTCALL (@IntEngFillBox@24); IntEngFillPolygon = STDCALL.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn IntEngFillBox(dc: PDC, x: INT, y: INT, w: INT, h: INT, pbo: *mut BRUSHOBJ); }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn IntEngFillBox(dc: PDC, x: INT, y: INT, w: INT, h: INT, pbo: *mut BRUSHOBJ); }
extern "system" {
    fn IntEngFillPolygon(dc: PDC, points: *mut POINTL, count: INT, pbo: *mut BRUSHOBJ);
}
extern "C" {
    fn IntDuplicateUnicodeString(src: PUNICODE_STRING, dst: PUNICODE_STRING) -> NTSTATUS;
    fn IntUnicodeStringToBuffer(buf: *mut u16, cb: usize, src: PUNICODE_STRING);
    #[link_name = "_MmCopyToCaller"] fn MmCopyToCaller(dest: *mut c_void, src: *const c_void, size: ULONG) -> NTSTATUS;
}
// RealizeFontInit is FASTCALL, defined in font.rs.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn RealizeFontInit(hFont: HFONT) -> PTEXTOBJ; }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn RealizeFontInit(hFont: HFONT) -> PTEXTOBJ; }

// --- final batch: macros + externs (ABIs from nm) + path-shim reuse ---
const FW_DONTCARE_F: LONG = 0; const FW_MEDIUM_F: LONG = 500; const FW_BOLD_F: LONG = 700;
#[inline(always)] unsafe fn EMUBOLD_NEEDED(original: LONG, request: LONG) -> bool { request != FW_DONTCARE_F && (request - original) >= (FW_BOLD_F - FW_MEDIUM_F) }
#[inline(always)] unsafe fn INT_TO_FIXED(v: INT) -> FT_Fixed { (v as FT_Fixed) << 16 }
#[inline(always)] unsafe fn MAKELONG(lo: u32, hi: u32) -> LONG { ((lo & 0xffff) | ((hi & 0xffff) << 16)) as LONG }
#[inline(always)] unsafe fn ULongToHandle(h: ULONG) -> HANDLE { h as ULONG_PTR as HANDLE }
#[inline(always)] unsafe fn InterlockedCompareExchange(dest: *mut LONG, exch: LONG, comp: LONG) -> LONG {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(comp, exch, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(v) => v, Err(v) => v }
}
macro_rules! GDIToObj { ($obj:expr, $ty:ident) => { ($obj) as *mut FONTOBJ }; }

// LFONT_* macros (font.h) over GDIOBJ_* (same as font.rs).
#[inline(always)] unsafe fn LFONT_AllocFontWithHandle() -> PLFONT { GDIOBJ_AllocObjWithHandle(0x000a0000, core::mem::size_of::<TEXTOBJ>() as ULONG) as PLFONT }
#[inline(always)] unsafe fn LFONT_UnlockFont(p: PLFONT) { GDIOBJ_vUnlockObject(p as POBJ) }

// RTL_CONSTANT_STRING (runtime length scan, same as dclife.rs).
#[inline(always)] unsafe fn RTL_CONSTANT_STRING(s: *const u16) -> _UNICODE_STRING { let mut n=0usize; while core::ptr::read_unaligned(s.add(n)) != 0 { n+=1; } _UNICODE_STRING { Length: (n*2) as USHORT, MaximumLength: ((n+1)*2) as USHORT, Buffer: s as *mut u16 } }

#[cfg(target_arch = "x86")] extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn IntNameFromCharSet(CharSet: u8) -> *const u16;
}
#[cfg(not(target_arch = "x86"))] extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn IntNameFromCharSet(CharSet: u8) -> *const u16;
}
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: usize, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: usize, Alignment: ULONG) -> NTSTATUS;
    fn EXLATEOBJ_vInitialize(pexlo: *mut EXLATEOBJ, ppalSrc: *mut _PALETTE, ppalDst: *mut _PALETTE, crSrc: COLORREF, crBack: COLORREF, crFore: COLORREF);
    fn EXLATEOBJ_vCleanup(pexlo: *mut EXLATEOBJ);
    fn RegEnumValueW(hKey: HANDLE, idx: ULONG, name: *mut u16, pcch: *mut ULONG, ptype: *mut ULONG, data: *mut u8, pcb: *mut ULONG) -> NTSTATUS;
    fn RegDeleteValueW(hKey: HANDLE, name: *const u16) -> NTSTATUS;
}
extern "C" {
    fn DbgPrint(fmt: *const i8, ...) -> u32;
    fn SZZ_GetSize(szz: *const u16) -> usize;
    fn IntRebaseList(newhead: PLIST_ENTRY, oldhead: PLIST_ENTRY);
    fn IntNormalizeAngle(angle: LONG) -> LONG;
    fn IntCharSetFromLangID(lang: USHORT) -> BYTE;
    fn PathFindFileNameW(path: *const u16) -> *mut u16;
    fn PathIsRelativeW(path: *const u16) -> BOOL;
    fn IntSwapEndian(buf: *mut c_void, size: ULONG);
    #[link_name = "_MmCopyFromCaller"] fn MmCopyFromCaller(dest: *mut c_void, src: *const c_void, size: ULONG) -> NTSTATUS;
    fn FT_Done_Library(lib: FT_Library) -> FT_Error;
    // path_rust_shims.c reuse.
    fn w32r_IntLPtoDP(pdc: PDC, ppt: LPPOINT, count: INT) -> BOOL;
    fn w32r_XFORMOBJ_vInit(pxo: *mut XFORMOBJ, pmx: *mut MATRIX);
    fn w32r_XFORMOBJ_iGetXform(pxo: *mut XFORMOBJ, pxform: *mut XFORML) -> ULONG;
    fn w32r_FLOATOBJ_Equal1(pf: *mut FLOATOBJ) -> BOOL;
    fn w32r_FLOATOBJ_Set1(pf: *mut FLOATOBJ);
}

// PATH_IsPathOpen(dclevel) = hPath && (flPath & DCPATH_ACTIVE).
#[inline(always)] unsafe fn PATH_IsPathOpen(dclevel: *mut DCLEVEL) -> bool {
    !(*dclevel).hPath.is_null() && ((*dclevel).flPath & DCPATH_ACTIVE) != 0
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _FONTLINK {
    pub ListEntry: _LIST_ENTRY,
    pub bIgnore: BOOL,
    pub LogFont: LOGFONTW,
    pub SharedFace: PSHARED_FACE,
}
pub type FONTLINK = _FONTLINK;
pub type PFONTLINK = *mut _FONTLINK;

#[repr(C)]
pub struct _FONTLINK_CHAIN {
    pub FontLinkList: _LIST_ENTRY,
    pub LogFont: LOGFONTW,
    pub pszzFontLink: PZZWSTR,
    pub pBaseTextObj: PTEXTOBJ,
    pub pDefFace: FT_Face,
}
pub type FONTLINK_CHAIN = _FONTLINK_CHAIN;
pub type PFONTLINK_CHAIN = *mut _FONTLINK_CHAIN;

#[repr(C)]
pub struct _FONTLINK_CACHE {
    pub ListEntry: _LIST_ENTRY,
    pub LogFont: LOGFONTW,
    pub Chain: FONTLINK_CHAIN,
}
pub type FONTLINK_CACHE = _FONTLINK_CACHE;
pub type PFONTLINK_CACHE = *mut _FONTLINK_CACHE;

pub const FONTLINK_DEFAULT_CHAR: DWORD = 0x30FB;

pub static mut s_chFontLinkDefaultChar: DWORD = FONTLINK_DEFAULT_CHAR;
pub static mut s_szDefFontLinkFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
pub static mut s_szDefFontLinkFontName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
pub static mut s_fFontLinkUseAnsi: BOOL = 0;
pub static mut s_fFontLinkUseOem: BOOL = 0;
pub static mut s_fFontLinkUseSymbol: BOOL = 0;

pub const MAX_FONTLINK_CACHE: i32 = 128;

pub static mut g_nFontLinkCacheCount: LONG = 0;

#[repr(C)]
pub struct _FONT_LOOKUP_CACHE {
    pub ListEntry: _LIST_ENTRY,
    pub LogFont: LOGFONTW,
    pub SharedFace: PSHARED_FACE,
    pub pFontObj: *mut FONTOBJ,
}
pub type FONT_LOOKUP_CACHE = _FONT_LOOKUP_CACHE;
pub type PFONT_LOOKUP_CACHE = *mut _FONT_LOOKUP_CACHE;

pub const MAX_FONT_LOOKUP_CACHE: i32 = 64;

pub static mut s_FontLookupCacheCount: ULONG = 0;

pub static identityMat: FT_Matrix = FT_Matrix {
    xx: (1 << 16) as FT_Fixed,
    xy: 0,
    yx: 0,
    yy: (1 << 16) as FT_Fixed,
};

pub static mut g_FreeTypeLibrary: FT_Library = core::ptr::null_mut();

pub static mut g_FreeTypeLock: PFAST_MUTEX = core::ptr::null_mut();

pub static mut g_RenderingEnabled: BOOL = 1;

pub const MAX_FONT_CACHE: i32 = 256;

pub static mut g_FontCacheNumEntries: UINT = 0;

pub static mut g_ElfScripts: [PWCHAR; 32] = [
    b"W\0e\0s\0t\0e\0r\0n\0\0\0".as_ptr() as PWCHAR,
    b"C\0e\0n\0t\0r\0a\0l\0_\0E\0u\0r\0o\0p\0e\0a\0n\0\0\0".as_ptr() as PWCHAR,
    b"C\0y\0r\0i\0l\0l\0i\0c\0\0\0".as_ptr() as PWCHAR,
    b"G\0r\0e\0e\0k\0\0\0".as_ptr() as PWCHAR,
    b"T\0u\0r\0k\0i\0s\0h\0\0\0".as_ptr() as PWCHAR,
    b"H\0e\0b\0r\0e\0w\0\0\0".as_ptr() as PWCHAR,
    b"A\0r\0a\0b\0i\0c\0\0\0".as_ptr() as PWCHAR,
    b"B\0a\0l\0t\0i\0c\0\0\0".as_ptr() as PWCHAR,
    b"V\0i\0e\0t\0n\0a\0m\0e\0s\0e\0\0\0".as_ptr() as PWCHAR,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    b"T\0h\0a\0i\0\0\0".as_ptr() as PWCHAR,
    b"J\0a\0p\0a\0n\0e\0s\0e\0\0\0".as_ptr() as PWCHAR,
    b"C\0H\0I\0N\0E\0S\0E\0_\0G\0B\02\03\01\02\0\0\0".as_ptr() as PWCHAR,
    b"H\0a\0n\0g\0u\0l\0\0\0".as_ptr() as PWCHAR,
    b"C\0H\0I\0N\0E\0S\0E\0_\0B\0I\0G\05\0\0\0".as_ptr() as PWCHAR,
    b"H\0a\0n\0g\0u\0l\0(\0J\0o\0h\0a\0b\0)\0\0\0".as_ptr() as PWCHAR,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    b"S\0y\0m\0b\0o\0l\0\0\0".as_ptr() as PWCHAR,
];

pub const CP_SYMBOL: u32 = 42;

pub const MAXTCIINDEX: usize = 32;

pub static g_FontTci: [CHARSETINFO; MAXTCIINDEX] = [
    CHARSETINFO { ciCharset: ANSI_CHARSET as UINT, ciACP: 1252, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: EASTEUROPE_CHARSET as UINT, ciACP: 1250, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN2, 0] } },
    CHARSETINFO { ciCharset: RUSSIAN_CHARSET as UINT, ciACP: 1251, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_CYRILLIC, 0] } },
    CHARSETINFO { ciCharset: GREEK_CHARSET as UINT, ciACP: 1253, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_GREEK, 0] } },
    CHARSETINFO { ciCharset: TURKISH_CHARSET as UINT, ciACP: 1254, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_TURKISH, 0] } },
    CHARSETINFO { ciCharset: HEBREW_CHARSET as UINT, ciACP: 1255, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_HEBREW, 0] } },
    CHARSETINFO { ciCharset: ARABIC_CHARSET as UINT, ciACP: 1256, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_ARABIC, 0] } },
    CHARSETINFO { ciCharset: BALTIC_CHARSET as UINT, ciACP: 1257, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_BALTIC, 0] } },
    CHARSETINFO { ciCharset: VIETNAMESE_CHARSET as UINT, ciACP: 1258, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_VIETNAMESE, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: THAI_CHARSET as UINT, ciACP: 874, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_THAI, 0] } },
    CHARSETINFO { ciCharset: SHIFTJIS_CHARSET as UINT, ciACP: 932, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_JISJAPAN, 0] } },
    CHARSETINFO { ciCharset: GB2312_CHARSET as UINT, ciACP: 936, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_CHINESESIMP, 0] } },
    CHARSETINFO { ciCharset: HANGEUL_CHARSET as UINT, ciACP: 949, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_WANSUNG, 0] } },
    CHARSETINFO { ciCharset: CHINESEBIG5_CHARSET as UINT, ciACP: 950, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_CHINESETRAD, 0] } },
    CHARSETINFO { ciCharset: JOHAB_CHARSET as UINT, ciACP: 1361, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_JOHAB, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: DEFAULT_CHARSET as UINT, ciACP: 0, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_LATIN1, 0] } },
    CHARSETINFO { ciCharset: SYMBOL_CHARSET as UINT, ciACP: CP_SYMBOL, fs: FONTSIGNATURE { fsUsb: [0,0,0,0], fsCsb: [FS_SYMBOL, 0] } },
];

pub const CP_OEMCP: u32 = 1;
pub const CP_MACCP: u32 = 2;

pub static s_szzDefFontLink: [WCHAR; 100] = [
    0x74,0x61,0x68,0x6F,0x6D,0x61,0x2E,0x74,0x74,0x66,0x2C,0x54,0x61,0x68,0x6F,0x6D,0x61,0x00,
    0x6D,0x73,0x67,0x6F,0x74,0x68,0x69,0x63,0x2E,0x74,0x74,0x63,0x2C,0x4D,0x53,0x20,0x55,0x49,0x20,0x47,0x6F,0x74,0x68,0x69,0x63,0x00,
    0x6D,0x69,0x6E,0x67,0x6C,0x69,0x75,0x2E,0x74,0x74,0x63,0x2C,0x50,0x4D,0x69,0x6E,0x67,0x4C,0x69,0x55,0x00,
    0x73,0x69,0x6D,0x73,0x75,0x6E,0x2E,0x74,0x74,0x63,0x2C,0x53,0x69,0x6D,0x53,0x75,0x6E,0x00,
    0x67,0x75,0x6C,0x69,0x6D,0x2E,0x74,0x74,0x63,0x2C,0x47,0x75,0x6C,0x69,0x6D,0x00,
    0x00u16,
];

pub static s_szzDefFixedFontLink: [WCHAR; 103] = [
    0x63,0x6F,0x75,0x72,0x2E,0x74,0x74,0x66,0x2C,0x43,0x6F,0x75,0x72,0x69,0x65,0x72,0x20,0x4E,0x65,0x77,0x00,
    0x6D,0x73,0x67,0x6F,0x74,0x68,0x69,0x63,0x2E,0x74,0x74,0x63,0x2C,0x4D,0x53,0x20,0x47,0x6F,0x74,0x68,0x69,0x63,0x00,
    0x6D,0x69,0x6E,0x67,0x6C,0x69,0x75,0x2E,0x74,0x74,0x63,0x2C,0x4D,0x69,0x6E,0x67,0x4C,0x69,0x55,0x00,
    0x73,0x69,0x6D,0x73,0x75,0x6E,0x2E,0x74,0x74,0x63,0x2C,0x4E,0x53,0x69,0x6D,0x53,0x75,0x6E,0x00,
    0x67,0x75,0x6C,0x69,0x6D,0x2E,0x74,0x74,0x63,0x2C,0x47,0x75,0x6C,0x69,0x6D,0x43,0x68,0x65,0x00,
    0x00u16,
];

#[repr(C)]
pub struct FONT_NAMES {
    pub FamilyNameW: _UNICODE_STRING,
    pub FaceNameW: _UNICODE_STRING,
    pub StyleNameW: _UNICODE_STRING,
    pub FullNameW: _UNICODE_STRING,
    pub OtmSize: ULONG,
}
pub type LPFONT_NAMES = *mut FONT_NAMES;

pub const STACK_TEXT_BUFFER_SIZE: usize = 512;

unsafe fn FontLink_LoadSettings() -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut cbData: DWORD;
    let mut dwValue: DWORD = 0;

    s_chFontLinkDefaultChar = FONTLINK_DEFAULT_CHAR;

    Status = RegOpenKey(
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\FontLink").as_ptr() as *const u16,
        &mut hKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    cbData = core::mem::size_of::<DWORD>() as DWORD;
    Status = RegQueryValue(
        hKey,
        w!("FontLinkDefaultChar").as_ptr() as *const u16,
        REG_DWORD,
        core::ptr::addr_of_mut!(dwValue) as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) && cbData == core::mem::size_of::<DWORD>() as DWORD {
        s_chFontLinkDefaultChar = dwValue;
    }

    ZwClose(hKey as _ );
    STATUS_SUCCESS
}

unsafe fn FontLink_LoadDefaultFonts() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut cbData: DWORD;
    let mut szValue: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // Set the default values
    s_szDefFontLinkFileName[0] = 0;
    s_szDefFontLinkFontName[0] = 0;

    // Open the registry key
    Status = RegOpenKey(
        b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0F\0o\0n\0t\0A\0s\0s\0o\0c\0\\\0A\0s\0s\0o\0c\0i\0a\0t\0e\0d\0 \0D\0e\0f\0a\0u\0l\0t\0F\0o\0n\0t\0s\0\0\0".as_ptr() as *const WCHAR,
        &mut hKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    cbData = core::mem::size_of_val(&szValue) as DWORD;
    Status = RegQueryValue(
        hKey,
        b"A\0s\0s\0o\0c\0S\0y\0s\0t\0e\0m\0F\0o\0n\0t\0\0\0".as_ptr() as *const WCHAR,
        REG_SZ,
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) {
        szValue[szValue.len() - 1] = 0;
        w32r_RtlStringCchCopyW(
            (*(&raw mut s_szDefFontLinkFileName)).as_mut_ptr(),
            (*(&raw const s_szDefFontLinkFileName)).len(),
            szValue.as_ptr(),
        );
    }

    cbData = core::mem::size_of_val(&szValue) as DWORD;
    Status = RegQueryValue(
        hKey,
        b"F\0o\0n\0t\0P\0a\0c\0k\0a\0g\0e\0\0\0".as_ptr() as *const WCHAR,
        REG_SZ,
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) {
        szValue[szValue.len() - 1] = 0;
        w32r_RtlStringCchCopyW(
            (*(&raw mut s_szDefFontLinkFontName)).as_mut_ptr(),
            (*(&raw const s_szDefFontLinkFontName)).len(),
            szValue.as_ptr(),
        );
    }

    ZwClose(hKey as _ );
    STATUS_SUCCESS
}

unsafe fn FontLink_LoadDefaultCharset() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut cbData: DWORD;
    let mut szValue: [WCHAR; 8] = [0; 8];

    s_fFontLinkUseSymbol = 0;
    s_fFontLinkUseOem = 0;
    s_fFontLinkUseAnsi = 0;

    Status = RegOpenKey(
        b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0F\0o\0n\0t\0A\0s\0s\0o\0c\0\\\0A\0s\0s\0o\0c\0i\0a\0t\0e\0d\0 \0C\0h\0a\0r\0s\0e\0t\0\0\0".as_ptr() as *const WCHAR,
        &mut hKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    cbData = core::mem::size_of_val(&szValue) as DWORD;
    Status = RegQueryValue(
        hKey,
        b"A\0N\0S\0I\0(\00\00\0)\0\0\0".as_ptr() as *const WCHAR,
        REG_SZ,
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) {
        szValue[szValue.len() - 1] = 0;
        s_fFontLinkUseAnsi = (_wcsicmp(szValue.as_ptr(), b"Y\0E\0S\0\0\0".as_ptr() as *const WCHAR) == 0) as BOOL;
    }

    cbData = core::mem::size_of_val(&szValue) as DWORD;
    Status = RegQueryValue(
        hKey,
        b"O\0E\0M\0(\0F\0F\0)\0\0\0".as_ptr() as *const WCHAR,
        REG_SZ,
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) {
        szValue[szValue.len() - 1] = 0;
        s_fFontLinkUseOem = (_wcsicmp(szValue.as_ptr(), b"Y\0E\0S\0\0\0".as_ptr() as *const WCHAR) == 0) as BOOL;
    }

    cbData = core::mem::size_of_val(&szValue) as DWORD;
    Status = RegQueryValue(
        hKey,
        b"S\0Y\0M\0B\0O\0L\0(\00\02\0)\0\0\0".as_ptr() as *const WCHAR,
        REG_SZ,
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if NT_SUCCESS(Status) {
        szValue[szValue.len() - 1] = 0;
        s_fFontLinkUseSymbol = (_wcsicmp(szValue.as_ptr(), b"Y\0E\0S\0\0\0".as_ptr() as *const WCHAR) == 0) as BOOL;
    }

    ZwClose(hKey as *mut core::ffi::c_void);
    STATUS_SUCCESS
}

unsafe fn FontLink_Destroy(pLink: PFONTLINK) {
    ExFreePoolWithTag(pLink as *mut core::ffi::c_void, TAG_FONT);
}

unsafe fn FontLink_Chain_IsPopulated(pChain: *const FONTLINK_CHAIN) -> BOOL {
    ((*pChain).LogFont.lfFaceName[0] != 0) as BOOL
}

unsafe fn FontLink_Chain_Free(pChain: PFONTLINK_CHAIN) {
    if FontLink_Chain_IsPopulated(pChain) == 0 {
        return;
    }

    if !(*pChain).pszzFontLink.is_null() {
        ExFreePoolWithTag((*pChain).pszzFontLink as *mut core::ffi::c_void, TAG_FONT);
    }

    while IsListEmpty(core::ptr::addr_of_mut!((*pChain).FontLinkList)) == 0 {
        let Entry = RemoveHeadList(core::ptr::addr_of_mut!((*pChain).FontLinkList));
        let pLink = (Entry as *mut u8).offset(-(core::mem::offset_of!(_FONTLINK, ListEntry) as isize)) as PFONTLINK;
        FontLink_Destroy(pLink);
    }
}

unsafe fn FontLink_AddCache(mut pCache: PFONTLINK_CACHE) {
    g_nFontLinkCacheCount += 1;
    InsertHeadList(core::ptr::addr_of_mut!(g_FontLinkCache), core::ptr::addr_of_mut!((*pCache).ListEntry));

    if g_nFontLinkCacheCount > MAX_FONTLINK_CACHE as LONG {
        let Entry = RemoveTailList(core::ptr::addr_of_mut!(g_FontLinkCache));
        g_nFontLinkCacheCount -= 1;
        pCache = (Entry as *mut u8).offset(-(core::mem::offset_of!(_FONTLINK_CACHE, ListEntry) as isize)) as PFONTLINK_CACHE;
        FontLink_Chain_Free(core::ptr::addr_of_mut!((*pCache).Chain));
        ExFreePoolWithTag(pCache as *mut core::ffi::c_void, TAG_FONT);
    }
}

unsafe fn FontLink_Chain_Finish(pChain: PFONTLINK_CHAIN) {
    let pCache: PFONTLINK_CACHE;

    if FontLink_Chain_IsPopulated(pChain) == 0 {
        return;
    }

    pCache = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<FONTLINK_CACHE>() as SIZE_T as _ , TAG_FONT) as PFONTLINK_CACHE;
    if pCache.is_null() {
        return;
    }

    (*pCache).LogFont = (*pChain).LogFont;
    (*pCache).Chain = core::ptr::read(pChain);
    IntRebaseList(core::ptr::addr_of_mut!((*pCache).Chain.FontLinkList), core::ptr::addr_of_mut!((*pChain).FontLinkList));

    FontLink_AddCache(pCache);
}

unsafe fn FontLink_FindCache(pLogFont: *const LOGFONTW) -> PFONTLINK_CACHE {
    let mut Entry: PLIST_ENTRY = g_FontLinkCache.Flink;
    while Entry != core::ptr::addr_of_mut!(g_FontLinkCache) {
        let pLinkCache = (Entry as *mut u8)
            .sub(core::mem::offset_of!(_FONTLINK_CACHE, ListEntry)) as PFONTLINK_CACHE;
        if RtlEqualMemory(
            core::ptr::addr_of!((*pLinkCache).LogFont) as *const _,
            pLogFont as *const _,
            core::mem::size_of::<LOGFONTW>(),
        ) {
            return pLinkCache;
        }
        Entry = (*Entry).Flink;
    }
    core::ptr::null_mut()
}

unsafe fn FontLink_CleanupCache() {
    while IsListEmpty(core::ptr::addr_of_mut!(g_FontLinkCache)) == 0 {
        let Entry = RemoveHeadList(core::ptr::addr_of_mut!(g_FontLinkCache));
        let pLinkCache = (Entry as *mut u8).offset(-(core::mem::offset_of!(_FONTLINK_CACHE, ListEntry) as isize)) as PFONTLINK_CACHE;
        FontLink_Chain_Free(core::ptr::addr_of_mut!((*pLinkCache).Chain));
        ExFreePoolWithTag(pLinkCache as *mut core::ffi::c_void, TAG_FONT);
    }

    g_nFontLinkCacheCount = 0;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCharSetFromCodePage(uCodePage: UINT) -> BYTE {
    IntCharSetFromCodePage_impl(uCodePage)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCharSetFromCodePage(uCodePage: UINT) -> BYTE {
    IntCharSetFromCodePage_impl(uCodePage)
}

unsafe fn IntCharSetFromCodePage_impl(uCodePage: UINT) -> BYTE {
    if uCodePage == CP_OEMCP {
        return OEM_CHARSET as BYTE;
    }

    if uCodePage == CP_MACCP {
        return MAC_CHARSET as BYTE;
    }

    let mut i: UINT = 0;
    while i < (MAXTCIINDEX as _) {
        if g_FontTci[i as usize].ciACP == 0 {
            i += 1;
            continue;
        }

        if g_FontTci[i as usize].ciACP == uCodePage {
            return g_FontTci[i as usize].ciCharset as BYTE;
        }

        i += 1;
    }

    DEFAULT_CHARSET as BYTE
}

unsafe fn FontLink_PrepareFontInfo(pFontLink: PFONTLINK) -> BOOL {
    if (*pFontLink).bIgnore != 0 {
        return 0;
    }
    if !(*pFontLink).SharedFace.is_null() {
        return 1;
    }

    let SharedFace: PSHARED_FACE = IntRealizeFont(
        core::ptr::addr_of!((*pFontLink).LogFont),
        core::ptr::null_mut(),
    );
    if SharedFace.is_null() {
        (*pFontLink).bIgnore = 1;
        return 0;
    }

    (*pFontLink).SharedFace = SharedFace;
    1
}

unsafe fn SharedMem_AddRef(Ptr: PSHARED_MEM) {
    (*Ptr).RefCount += 1;
}

unsafe fn SharedFaceCache_Init(Cache: PSHARED_FACE_CACHE) {
    (*Cache).OutlineRequiredSize = 0;
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Cache).FontFamily) as _ , core::ptr::null_mut());
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Cache).FullName) as _ , core::ptr::null_mut());
}

unsafe fn SharedFace_Create(Face: FT_Face, Memory: PSHARED_MEM) -> PSHARED_FACE {
    let Ptr: PSHARED_FACE = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<SHARED_FACE>() as u64 as _ , TAG_FONT) as PSHARED_FACE;
    if !Ptr.is_null() {
        (*Ptr).Face = Face;
        (*Ptr).RefCount = 1;
        (*Ptr).Memory = Memory;
        SharedFaceCache_Init(core::ptr::addr_of_mut!((*Ptr).EnglishUS));
        SharedFaceCache_Init(core::ptr::addr_of_mut!((*Ptr).UserLanguage));

        SharedMem_AddRef(Memory);
    }
    Ptr
}

unsafe fn SharedMem_Create(Buffer: PBYTE, BufferSize: ULONG, IsMapping: BOOL) -> PSHARED_MEM {
    let Ptr: PSHARED_MEM = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<SHARED_MEM>() as SIZE_T as _ , TAG_FONT) as PSHARED_MEM;
    if !Ptr.is_null() {
        (*Ptr).Buffer = Buffer as _ ;
        (*Ptr).BufferSize = BufferSize;
        (*Ptr).RefCount = 1;
        (*Ptr).IsMapping = IsMapping;
    }
    Ptr
}

unsafe fn SharedFace_AddRef(Ptr: PSHARED_FACE) {
    (*Ptr).RefCount += 1;
}

unsafe fn RemoveCachedEntry(Entry: PFONT_CACHE_ENTRY) {
    FT_Done_Glyph((*Entry).BitmapGlyph as FT_Glyph);
    RemoveEntryList(core::ptr::addr_of_mut!((*Entry).ListEntry));
    ExFreePoolWithTag(Entry as *mut core::ffi::c_void, TAG_FONT);
    g_FontCacheNumEntries -= 1;
}

unsafe fn RemoveCacheEntries(Face: FT_Face) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut FontEntry: PFONT_CACHE_ENTRY;

    CurrentEntry = g_FontCacheListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(g_FontCacheListHead) {
        FontEntry = CONTAINING_RECORD!(CurrentEntry, _FONT_CACHE_ENTRY, ListEntry);
        NextEntry = (*CurrentEntry).Flink;

        if (*FontEntry).Hashed.Face == Face {
            RemoveCachedEntry(FontEntry);
        }

        CurrentEntry = NextEntry;
    }
}

unsafe fn SharedMem_Release(Ptr: PSHARED_MEM) {
    if (*Ptr).RefCount <= 0 {
        return;
    }

    (*Ptr).RefCount -= 1;
    if (*Ptr).RefCount == 0 {
        if (*Ptr).IsMapping != 0 {
            MmUnmapViewInSystemSpace((*Ptr).Buffer);
        } else {
            ExFreePoolWithTag((*Ptr).Buffer, TAG_FONT);
        }
        ExFreePoolWithTag(Ptr as *mut core::ffi::c_void, TAG_FONT);
    }
}

unsafe fn SharedFaceCache_Release(Cache: PSHARED_FACE_CACHE) {
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Cache).FontFamily) as _ );
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Cache).FullName) as _ );
}

unsafe fn SharedFace_Release(Ptr: PSHARED_FACE, bDoLock: BOOL) {
    if bDoLock != 0 {
        IntLockFreeType();
    }

    if (*Ptr).RefCount <= 0 {
        return;
    }

    (*Ptr).RefCount -= 1;
    if (*Ptr).RefCount == 0 {
        RemoveCacheEntries((*Ptr).Face);
        FT_Done_Face((*Ptr).Face);
        SharedMem_Release((*Ptr).Memory);
        SharedFaceCache_Release(core::ptr::addr_of_mut!((*Ptr).EnglishUS));
        SharedFaceCache_Release(core::ptr::addr_of_mut!((*Ptr).UserLanguage));
        ExFreePoolWithTag(Ptr as *mut core::ffi::c_void, TAG_FONT);
    }

    if bDoLock != 0 {
        IntUnLockFreeType();
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CleanupFontEntryEx(FontEntry: PFONT_ENTRY, FontGDI: PFONTGDI) {
    let SharedFace: PSHARED_FACE = (*FontGDI).SharedFace;

    if !(*FontGDI).Filename.is_null() {
        ExFreePoolWithTag((*FontGDI).Filename as *mut _, GDITAG_PFF);
    }

    if !(*FontEntry).StyleName.Buffer.is_null() {
        RtlFreeUnicodeString((core::ptr::addr_of_mut!((*FontEntry).StyleName) as _));
    }

    if !(*FontEntry).FaceName.Buffer.is_null() {
        RtlFreeUnicodeString((core::ptr::addr_of_mut!((*FontEntry).FaceName) as _));
    }

    EngFreeMem(FontGDI as *mut _);
    SharedFace_Release(SharedFace, 1);
    ExFreePoolWithTag(FontEntry as *mut _, TAG_FONT);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CleanupFontEntryEx(FontEntry: PFONT_ENTRY, FontGDI: PFONTGDI) {
    let SharedFace: PSHARED_FACE = (*FontGDI).SharedFace;

    if !(*FontGDI).Filename.is_null() {
        ExFreePoolWithTag((*FontGDI).Filename as *mut _, GDITAG_PFF);
    }

    if !(*FontEntry).StyleName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*FontEntry).StyleName));
    }

    if !(*FontEntry).FaceName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*FontEntry).FaceName));
    }

    EngFreeMem(FontGDI as *mut _);
    SharedFace_Release(SharedFace, 1);
    ExFreePoolWithTag(FontEntry as *mut _, TAG_FONT);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CleanupFontEntry(FontEntry: PFONT_ENTRY) {
    CleanupFontEntryEx(FontEntry, (*FontEntry).Font);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CleanupFontEntry(FontEntry: PFONT_ENTRY) {
    CleanupFontEntryEx(FontEntry, (*FontEntry).Font);
}

unsafe fn FTVectorToPOINTFX(vec: *mut FT_Vector, pt: *mut POINTFX) {
    (*pt).x.value = ((*vec).x >> 6) as i16;
    (*pt).x.fract = (((*vec).x & 0x3f) << 10) as u16;
    (*pt).x.fract |= ((*pt).x.fract >> 6) | ((*pt).x.fract >> 12);
    (*pt).y.value = ((*vec).y >> 6) as i16;
    (*pt).y.fract = (((*vec).y & 0x3f) << 10) as u16;
    (*pt).y.fract |= ((*pt).y.fract >> 6) | ((*pt).y.fract >> 12);
}

unsafe fn FT_FixedFromFIXED(f: FIXED) -> FT_Fixed {
    ((f.value as core::ffi::c_long) << 16 | (f.fract as core::ffi::c_ulong) as core::ffi::c_long) as FT_Fixed
}

unsafe fn DumpFontEntry(FontEntry: PFONT_ENTRY) {
    let _family_name: *const core::ffi::c_char;
    let _style_name: *const core::ffi::c_char;
    let Face: FT_Face;
    let FontGDI: PFONTGDI = (*FontEntry).Font;

    if FontGDI.is_null() {
        return;
    }

    Face = if !(*FontGDI).SharedFace.is_null() {
        (*(*FontGDI).SharedFace).Face
    } else {
        core::ptr::null_mut()
    };

    if !Face.is_null() {
        _family_name = (*Face).family_name;
        _style_name = (*Face).style_name;
    } else {
        _family_name = b"<invalid>\0".as_ptr() as *const core::ffi::c_char;
        _style_name = b"<invalid>\0".as_ptr() as *const core::ffi::c_char;
    }
}

unsafe fn DumpFontList(Head: PLIST_ENTRY) {
    let mut Entry: PLIST_ENTRY;
    let CurrentEntry: PFONT_ENTRY;

    Entry = (*Head).Flink;
    while Entry != Head {
        let CurrentEntry = (Entry as *mut u8)
            .sub(core::mem::offset_of!(_FONT_ENTRY, ListEntry)) as PFONT_ENTRY;
        DumpFontEntry(CurrentEntry);
        Entry = (*Entry).Flink;
    }
    let _ = CurrentEntry;
}

unsafe fn DumpFontSubstEntry(pSubstEntry: PFONTSUBST_ENTRY) {
    let _ = pSubstEntry;
}

unsafe fn DumpFontSubstList() {
    let pHead: PLIST_ENTRY = core::ptr::addr_of_mut!(g_FontSubstListHead);
    let mut pListEntry: PLIST_ENTRY;
    let pSubstEntry: PFONTSUBST_ENTRY;

    pListEntry = (*pHead).Flink;
    while pListEntry != pHead {
        let pSubstEntry = (pListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(FONTSUBST_ENTRY, ListEntry) as isize))
            as PFONTSUBST_ENTRY;
        DumpFontSubstEntry(pSubstEntry);
        pListEntry = (*pListEntry).Flink;
    }
    let _ = pSubstEntry;
}

unsafe fn DumpPrivateFontList(bDoLock: BOOL) {
    let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    if Win32Process.is_null() {
        return;
    }

    if bDoLock != 0 {
        IntLockFreeType();
        IntLockProcessPrivateFonts(Win32Process as _ );
    }

    DumpFontList(core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead));

    if bDoLock != 0 {
        IntUnLockProcessPrivateFonts(Win32Process as _ );
        IntUnLockFreeType();
    }
}

unsafe fn DumpGlobalFontList(bDoLock: BOOL) {
    if bDoLock != 0 {
        ExEnterCriticalRegionAndAcquireFastMutexUnsafe(g_FreeTypeLock as _ );
    }

    DumpFontList(core::ptr::addr_of_mut!(g_FontListHead));

    if bDoLock != 0 {
        ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(g_FreeTypeLock as _ );
    }
}

unsafe fn DumpFontInfo(bDoLock: BOOL) {
    DumpGlobalFontList(bDoLock);
    DumpPrivateFontList(bDoLock);
    DumpFontSubstList();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntLoadFontSubstList(pHead: PLIST_ENTRY) -> BOOL {
    IntLoadFontSubstList_impl(pHead)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntLoadFontSubstList(pHead: PLIST_ENTRY) -> BOOL {
    IntLoadFontSubstList_impl(pHead)
}

unsafe fn IntLoadFontSubstList_impl(pHead: PLIST_ENTRY) -> BOOL {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyFullInfo: KEY_FULL_INFORMATION = core::mem::zeroed();
    let mut i: ULONG;
    let mut Length: ULONG = 0;
    let mut FromW: _UNICODE_STRING = core::mem::zeroed();
    let mut ToW: _UNICODE_STRING = core::mem::zeroed();
    let mut InfoBuffer: [u8; 128] = [0; 128];
    let mut pInfo: PKEY_VALUE_FULL_INFORMATION;
    let mut CharSets: [u8; FONTSUBST_FROM_AND_TO as usize] = [0; FONTSUBST_FROM_AND_TO as usize];
    let mut pch: LPWSTR;
    let mut pEntry: *mut FONTSUBST_ENTRY;

    let mut FontSubstKey: _UNICODE_STRING = RTL_CONSTANT_STRING(
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\FontSubstitutes").as_ptr() as LPWSTR,
    );

    /* InitializeObjectAttributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FontSubstKey);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        &mut KeyHandle,
        KEY_READ,
        core::ptr::addr_of_mut!(ObjectAttributes) as _ ,
    );
    if !NT_SUCCESS(Status) {
        return 0u8 as BOOL;
    }

    Status = ZwQueryKey(
        KeyHandle,
        KeyFullInformation,
        core::ptr::addr_of_mut!(KeyFullInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG,
        &mut Length,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return 0u8 as BOOL;
    }

    i = 0;
    while i < KeyFullInfo.Values {
        Status = ZwEnumerateValueKey(
            KeyHandle,
            i,
            KeyValueFullInformation,
            InfoBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&InfoBuffer) as ULONG,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        pInfo = InfoBuffer.as_mut_ptr() as PKEY_VALUE_FULL_INFORMATION;
        Length = (*pInfo).NameLength / core::mem::size_of::<u16>() as ULONG;
        *(*pInfo).Name.as_mut_ptr().add(Length as usize) = 0;
        if RtlCreateUnicodeString(core::ptr::addr_of_mut!(FromW) as _ , (*pInfo).Name.as_ptr()) == 0 {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(FromW) as _ ,
            KeyValueFullInformation,
            InfoBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&InfoBuffer) as ULONG,
            &mut Length,
        );
        pInfo = InfoBuffer.as_mut_ptr() as PKEY_VALUE_FULL_INFORMATION;
        if !NT_SUCCESS(Status) || (*pInfo).DataLength == 0 {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FromW) as _ );
            break;
        }

        pch = ((pInfo as *mut u8).add((*pInfo).DataOffset as usize)) as LPWSTR;
        Length = (*pInfo).DataLength / core::mem::size_of::<u16>() as ULONG;
        *pch.add(Length as usize) = 0;
        if RtlCreateUnicodeString(core::ptr::addr_of_mut!(ToW) as _ , pch) == 0 {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FromW) as _ );
            break;
        }

        CharSets[FONTSUBST_FROM as usize] = DEFAULT_CHARSET as u8;
        pch = wcsrchr(FromW.Buffer, ',' as u16);
        if !pch.is_null() {
            *pch = 0;
            FromW.Length = (pch.offset_from(FromW.Buffer) as usize * core::mem::size_of::<u16>()) as u16;
            CharSets[FONTSUBST_FROM as usize] = _wtoi(pch.add(1)) as u8;
        }

        CharSets[FONTSUBST_TO as usize] = DEFAULT_CHARSET as u8;
        pch = wcsrchr(ToW.Buffer, ',' as u16);
        if !pch.is_null() {
            *pch = 0;
            ToW.Length = (pch.offset_from(ToW.Buffer) as usize * core::mem::size_of::<u16>()) as u16;
            CharSets[FONTSUBST_TO as usize] = _wtoi(pch.add(1)) as u8;
        }

        if RtlEqualUnicodeString(&FromW, &ToW, 1u8) != 0
            && CharSets[FONTSUBST_FROM as usize] == CharSets[FONTSUBST_TO as usize]
        {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FromW) as _ );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(ToW) as _ );
            i += 1;
            continue;
        }

        pEntry = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<FONTSUBST_ENTRY>(),
            TAG_FONT,
        ) as *mut FONTSUBST_ENTRY;
        if pEntry.is_null() {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FromW) as _ );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(ToW) as _ );
            break;
        }

        (*pEntry).FontNames[FONTSUBST_FROM as usize] = FromW;
        (*pEntry).FontNames[FONTSUBST_TO as usize] = ToW;
        (*pEntry).CharSets[FONTSUBST_FROM as usize] = CharSets[FONTSUBST_FROM as usize];
        (*pEntry).CharSets[FONTSUBST_TO as usize] = CharSets[FONTSUBST_TO as usize];

        InsertTailList(pHead, core::ptr::addr_of_mut!((*pEntry).ListEntry));

        i += 1;
    }

    ZwClose(KeyHandle);

    NT_SUCCESS(Status) as BOOL
}

unsafe fn FontLookUp_Destroy(pCache: PFONT_LOOKUP_CACHE) {
    SharedFace_Release((*pCache).SharedFace, 0);
    ExFreePoolWithTag(pCache as *mut core::ffi::c_void, TAG_FONT);
}

unsafe fn FontLookUp_Cleanup(bDoLock: BOOL, SharedFace: PSHARED_FACE) {
    if bDoLock != 0 {
        IntLockFreeType();
    }

    let pHead: PLIST_ENTRY = core::ptr::addr_of_mut!(s_FontLookupCacheList);
    let mut pEntry: PLIST_ENTRY;
    let mut pCache: PFONT_LOOKUP_CACHE;

    if !SharedFace.is_null() {
        pEntry = (*pHead).Flink;
        while pEntry != pHead {
            pCache = (pEntry as *mut u8)
                .sub(core::mem::offset_of!(_FONT_LOOKUP_CACHE, ListEntry))
                as PFONT_LOOKUP_CACHE;
            if (*pCache).SharedFace == SharedFace {
                RemoveEntryList(core::ptr::addr_of_mut!((*pCache).ListEntry));
                FontLookUp_Destroy(pCache);
                s_FontLookupCacheCount -= 1;
            }
            pEntry = (*pEntry).Flink;
        }
    } else {
        while IsListEmpty(pHead) == 0 {
            pEntry = RemoveHeadList(pHead);
            pCache = (pEntry as *mut u8)
                .sub(core::mem::offset_of!(_FONT_LOOKUP_CACHE, ListEntry))
                as PFONT_LOOKUP_CACHE;
            FontLookUp_Destroy(pCache);
        }
        s_FontLookupCacheCount = 0;
    }

    if bDoLock != 0 {
        IntUnLockFreeType();
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InitFontSupport() -> BOOL {
    InitFontSupport_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InitFontSupport() -> BOOL {
    InitFontSupport_impl()
}

unsafe fn InitFontSupport_impl() -> BOOL {
    let ulError: ULONG;

    // C had these as RTL_STATIC_LIST_HEAD (statically self-ref); init them here.
    InitializeListHead(core::ptr::addr_of_mut!(g_FontListHead));
    InitializeListHead(core::ptr::addr_of_mut!(g_FontCacheListHead));
    InitializeListHead(core::ptr::addr_of_mut!(g_FontLinkCache));
    InitializeListHead(core::ptr::addr_of_mut!(g_FontSubstListHead));
    InitializeListHead(core::ptr::addr_of_mut!(s_FontLookupCacheList));

    g_FontCacheNumEntries = 0;

    g_FreeTypeLock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<FAST_MUTEX>() as SIZE_T as _ ,
        TAG_INTERNAL_SYNC,
    ) as PFAST_MUTEX;
    if g_FreeTypeLock.is_null() {
        return 0;
    }
    ExInitializeFastMutex(g_FreeTypeLock as _ );

    ulError = FT_Init_FreeType(&raw mut g_FreeTypeLibrary) as ULONG;
    if ulError != 0 {
        return 0;
    }

    if IntLoadFontsInRegistry() == 0 {
        DbgPrint(b"InitFontSupport: loading system fonts\n\0".as_ptr() as *const i8);
        IntLoadSystemFonts();
        DbgPrint(b"InitFontSupport: font loading done\n\0".as_ptr() as *const i8);
    }

    IntLoadFontSubstList(core::ptr::addr_of_mut!(g_FontSubstListHead));

    FontLink_LoadSettings();
    FontLink_LoadDefaultFonts();
    FontLink_LoadDefaultCharset();

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FreeFontSupport() {
    FreeFontSupport_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FreeFontSupport() {
    FreeFontSupport_impl();
}

#[inline(always)]
unsafe fn FreeFontSupport_impl() {
    let mut pHead: PLIST_ENTRY;
    let mut pEntry: PLIST_ENTRY;
    let pFontCache: PFONT_CACHE_ENTRY;
    let pSubstEntry: PFONTSUBST_ENTRY;
    let pFontEntry: PFONT_ENTRY;

    // Cleanup the FontLink cache
    FontLink_CleanupCache();

    // Cleanup FONT_LOOKUP_CACHE
    FontLookUp_Cleanup(0, core::ptr::null_mut());

    // Free font cache list
    pHead = core::ptr::addr_of_mut!(g_FontCacheListHead);
    while IsListEmpty(pHead) == 0 {
        pEntry = RemoveHeadList(pHead);
        let pFontCache = (pEntry as *mut u8)
            .sub(core::mem::offset_of!(_FONT_CACHE_ENTRY, ListEntry))
            as PFONT_CACHE_ENTRY;
        RemoveCachedEntry(pFontCache);
    }

    // Free font subst list
    pHead = core::ptr::addr_of_mut!(g_FontSubstListHead);
    while IsListEmpty(pHead) == 0 {
        pEntry = RemoveHeadList(pHead);
        let pSubstEntry = (pEntry as *mut u8)
            .sub(core::mem::offset_of!(FONTSUBST_ENTRY, ListEntry))
            as PFONTSUBST_ENTRY;
        ExFreePoolWithTag(pSubstEntry as *mut core::ffi::c_void, TAG_FONT);
    }

    // Free font list
    pHead = core::ptr::addr_of_mut!(g_FontListHead);
    while IsListEmpty(pHead) == 0 {
        pEntry = RemoveHeadList(pHead);
        let pFontEntry = (pEntry as *mut u8)
            .sub(core::mem::offset_of!(_FONT_ENTRY, ListEntry))
            as PFONT_ENTRY;
        CleanupFontEntry(pFontEntry);
    }

    if !g_FreeTypeLibrary.is_null() {
        FT_Done_Library(g_FreeTypeLibrary);
        g_FreeTypeLibrary = core::ptr::null_mut();
    }

    ExFreePoolWithTag(g_FreeTypeLock as *mut core::ffi::c_void, TAG_INTERNAL_SYNC);
    g_FreeTypeLock = core::ptr::null_mut();

    let _ = pFontCache;
    let _ = pSubstEntry;
    let _ = pFontEntry;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEscapeMatrix(pmat: *mut FT_Matrix, lfEscapement: LONG) {
    let mut vecAngle: FT_Vector = core::mem::zeroed();
    let angle: FT_Angle = INT_TO_FIXED(lfEscapement) / 10;
    FT_Vector_Unit(&mut vecAngle, angle);
    (*pmat).xx = vecAngle.x;
    (*pmat).xy = -vecAngle.y;
    (*pmat).yx = -(*pmat).xy;
    (*pmat).yy = (*pmat).xx;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEscapeMatrix(pmat: *mut FT_Matrix, lfEscapement: LONG) {
    let mut vecAngle: FT_Vector = core::mem::zeroed();
    let angle: FT_Angle = INT_TO_FIXED(lfEscapement) / 10;
    FT_Vector_Unit(&mut vecAngle, angle);
    (*pmat).xx = vecAngle.x;
    (*pmat).xy = -vecAngle.y;
    (*pmat).yx = -(*pmat).xy;
    (*pmat).yy = (*pmat).xx;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntMatrixFromMx(pmat: *mut FT_Matrix, pmx: *const MATRIX) {
    let mut ef: FLOATOBJ;

    ef = (*pmx).efM11;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).xx = w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM21;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).xy = -w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM12;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).yx = -w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM22;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).yy = w32r_FLOATOBJ_GetLong(&mut ef);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntMatrixFromMx(pmat: *mut FT_Matrix, pmx: *const MATRIX) {
    let mut ef: FLOATOBJ;

    ef = (*pmx).efM11;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).xx = w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM21;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).xy = -w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM12;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).yx = -w32r_FLOATOBJ_GetLong(&mut ef);

    ef = (*pmx).efM22;
    w32r_FLOATOBJ_MulLong(&mut ef, 0x00010000);
    (*pmat).yy = w32r_FLOATOBJ_GetLong(&mut ef);
}

unsafe fn SubstituteFontByList(
    pHead: PLIST_ENTRY,
    pOutputName: PUNICODE_STRING,
    pInputName: PUNICODE_STRING,
    RequestedCharSet: BYTE,
    CharSetMap: *mut BYTE,
) -> BOOL {
    let mut CharSets: [BYTE; FONTSUBST_FROM_AND_TO as usize] = [0; FONTSUBST_FROM_AND_TO as usize];

    *CharSetMap.add(FONTSUBST_FROM as usize) = DEFAULT_CHARSET as BYTE;
    *CharSetMap.add(FONTSUBST_TO as usize) = RequestedCharSet;

    let mut pListEntry = (*pHead).Flink;
    while pListEntry != pHead {
        let pSubstEntry = (pListEntry as *mut u8)
            .sub(core::mem::offset_of!(FONTSUBST_ENTRY, ListEntry))
            as *mut FONTSUBST_ENTRY;

        CharSets[FONTSUBST_FROM as usize] = (*pSubstEntry).CharSets[FONTSUBST_FROM as usize];

        if CharSets[FONTSUBST_FROM as usize] != DEFAULT_CHARSET as BYTE
            && CharSets[FONTSUBST_FROM as usize] != RequestedCharSet
        {
            pListEntry = (*pListEntry).Flink;
            continue;
        }

        if (*pSubstEntry).CharSets[FONTSUBST_TO as usize] != DEFAULT_CHARSET as BYTE {
            CharSets[FONTSUBST_TO as usize] = (*pSubstEntry).CharSets[FONTSUBST_TO as usize];
        } else {
            CharSets[FONTSUBST_TO as usize] = RequestedCharSet;
        }

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*pSubstEntry).FontNames[FONTSUBST_FROM as usize]),
            pInputName,
            1u8,
        ) == 0
        {
            pListEntry = (*pListEntry).Flink;
            continue;
        }

        *pOutputName = (*pSubstEntry).FontNames[FONTSUBST_TO as usize];

        if *CharSetMap.add(FONTSUBST_FROM as usize) == DEFAULT_CHARSET as BYTE {
            *CharSetMap.add(FONTSUBST_FROM as usize) = CharSets[FONTSUBST_FROM as usize];
            *CharSetMap.add(FONTSUBST_TO as usize) = CharSets[FONTSUBST_TO as usize];
        }
        return 1u8 as BOOL;
    }

    0u8 as BOOL
}

unsafe fn SubstituteFontRecurse(pLogFont: PLOGFONTW) -> BOOL {
    let mut RecurseCount: UINT = 5;
    let mut OutputNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut CharSetMap: [BYTE; FONTSUBST_FROM_AND_TO as usize] = [0; FONTSUBST_FROM_AND_TO as usize];
    let mut Found: BOOL;
    let mut InputNameW: _UNICODE_STRING = core::mem::zeroed();

    if (*pLogFont).lfFaceName[0] == UNICODE_NULL as u16 {
        return FALSE as BOOL;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(InputNameW) as _ , (*pLogFont).lfFaceName.as_ptr());

    while {
        let old = RecurseCount;
        RecurseCount = RecurseCount.wrapping_sub(1);
        old > 0
    } {
        Found = SubstituteFontByList(
            core::ptr::addr_of_mut!(g_FontSubstListHead),
            &mut OutputNameW,
            &mut InputNameW,
            (*pLogFont).lfCharSet,
            CharSetMap.as_mut_ptr(),
        );
        if Found == 0 {
            break;
        }

        IntUnicodeStringToBuffer(
            (*pLogFont).lfFaceName.as_mut_ptr(),
            core::mem::size_of_val(&(*pLogFont).lfFaceName),
            &mut OutputNameW,
        );
        RtlInitUnicodeString(core::ptr::addr_of_mut!(InputNameW) as _ , (*pLogFont).lfFaceName.as_ptr());

        if CharSetMap[FONTSUBST_FROM as usize] == DEFAULT_CHARSET as BYTE
            || CharSetMap[FONTSUBST_FROM as usize] == (*pLogFont).lfCharSet
        {
            (*pLogFont).lfCharSet = CharSetMap[FONTSUBST_TO as usize];
        }
    }

    TRUE as BOOL
}

unsafe fn FontLink_Chain_Init(
    pChain: PFONTLINK_CHAIN,
    pTextObj: PTEXTOBJ,
    face: FT_Face,
) {
    RtlZeroMemory(pChain as *mut _, core::mem::size_of::<FONTLINK_CHAIN>() as _);
    (*pChain).pBaseTextObj = pTextObj;
    (*pChain).pDefFace = face;
}

unsafe fn FontLink_Chain_LoadReg(
    pChain: PFONTLINK_CHAIN,
    pLF: PLOGFONTW,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut cbData: DWORD;
    let mut szzFontLink: [WCHAR; 512] = [0; 512];
    let mut FontLinkSize: SIZE_T = 0;
    let mut pszzFontLink: PZZWSTR = core::ptr::null_mut();

    Status = RegOpenKey(
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\FontLink\\SystemLink").as_ptr() as *const u16,
        &mut hKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    cbData = core::mem::size_of_val(&szzFontLink) as DWORD;
    Status = RegQueryValue(
        hKey,
        (*pLF).lfFaceName.as_ptr(),
        REG_MULTI_SZ,
        szzFontLink.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbData,
    );
    if !NT_SUCCESS(Status)
        && Status != STATUS_BUFFER_OVERFLOW
        && Status != STATUS_BUFFER_TOO_SMALL
    {
        SubstituteFontRecurse(pLF);
        cbData = core::mem::size_of_val(&szzFontLink) as DWORD;
        Status = RegQueryValue(
            hKey,
            (*pLF).lfFaceName.as_ptr(),
            REG_MULTI_SZ,
            szzFontLink.as_mut_ptr() as *mut core::ffi::c_void,
            &mut cbData,
        );
    }

    if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
        if cbData as usize >= 2 * core::mem::size_of::<WCHAR>() {
            FontLinkSize = cbData as SIZE_T;
            pszzFontLink = ExAllocatePoolWithTag(PagedPool, FontLinkSize as _ , TAG_FONT) as PZZWSTR;
            if pszzFontLink.is_null() {
                ZwClose(hKey as _ );
                return STATUS_NO_MEMORY;
            }
            Status = RegQueryValue(
                hKey,
                (*pLF).lfFaceName.as_ptr(),
                REG_MULTI_SZ,
                pszzFontLink as *mut core::ffi::c_void,
                &mut cbData,
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(pszzFontLink as *mut core::ffi::c_void, TAG_FONT);
                pszzFontLink = core::ptr::null_mut();
            }
        }
    }

    ZwClose(hKey as _ );

    if !NT_SUCCESS(Status) {
        if (*pLF).lfPitchAndFamily as u32 & FIXED_PITCH == 0 {
            RtlCopyMemory(
                szzFontLink.as_mut_ptr() as *mut core::ffi::c_void,
                s_szzDefFontLink.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&s_szzDefFontLink),
            );
        } else {
            RtlCopyMemory(
                szzFontLink.as_mut_ptr() as *mut core::ffi::c_void,
                s_szzDefFixedFontLink.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&s_szzDefFixedFontLink),
            );
        }
    }

    if !pszzFontLink.is_null() {
        let count = (FontLinkSize as usize / core::mem::size_of::<WCHAR>()) as u32;
        *pszzFontLink.add((count - 1) as usize) = UNICODE_NULL as u16;
        *pszzFontLink.add((count - 2) as usize) = UNICODE_NULL as u16;
    } else {
        let count = szzFontLink.len();
        szzFontLink[count - 1] = UNICODE_NULL as u16;
        szzFontLink[count - 2] = UNICODE_NULL as u16;

        FontLinkSize = SZZ_GetSize(szzFontLink.as_ptr()) as _ ;
        pszzFontLink = ExAllocatePoolWithTag(PagedPool, FontLinkSize as _ , TAG_FONT) as PZZWSTR;
        if pszzFontLink.is_null() {
            return STATUS_NO_MEMORY;
        }
        RtlCopyMemory(
            pszzFontLink as *mut core::ffi::c_void,
            szzFontLink.as_ptr() as *const core::ffi::c_void,
            FontLinkSize as _ ,
        );
    }
    (*pChain).pszzFontLink = pszzFontLink;

    STATUS_SUCCESS
}

unsafe fn FontLink_Chain_FindLink(
    pChain: PFONTLINK_CHAIN,
    plf: PLOGFONTW,
) -> PFONTLINK {
    let Head: PLIST_ENTRY = core::ptr::addr_of_mut!((*pChain).FontLinkList);
    let mut Entry: PLIST_ENTRY = (*Head).Flink;

    while Entry != Head {
        let pLink = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONTLINK, ListEntry) as isize))
            as *mut _FONTLINK;
        if RtlEqualMemory(
            core::ptr::addr_of!((*pLink).LogFont) as *const core::ffi::c_void,
            plf as *const core::ffi::c_void,
            core::mem::size_of::<LOGFONTW>(),
        ) {
            return pLink;
        }
        Entry = (*Entry).Flink;
    }

    core::ptr::null_mut()
}

unsafe fn FontLink_Create(
    pChain: PFONTLINK_CHAIN,
    plfBase: *const LOGFONTW,
    pszLink: LPCWSTR,
) -> PFONTLINK {
    let mut lf: LOGFONTW = *plfBase;

    let pch0: LPWSTR = wcschr(pszLink, b',' as i32 as u16);
    if pch0.is_null() {
        return core::ptr::null_mut();
    }
    let pch0 = pch0.offset(1);

    let pch1: LPWSTR = wcschr(pch0, b',' as i32 as u16);
    if !pch1.is_null() {
        w32r_RtlStringCchCopyNW(
            lf.lfFaceName.as_mut_ptr(),
            lf.lfFaceName.len(),
            pch0,
            pch1.offset_from(pch0) as usize,
        );
    } else {
        w32r_RtlStringCchCopyW(lf.lfFaceName.as_mut_ptr(), lf.lfFaceName.len(), pch0);
    }

    SubstituteFontRecurse(&mut lf);

    if RtlEqualMemory(
        plfBase as *const core::ffi::c_void,
        core::ptr::addr_of!(lf) as *const core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>(),
    ) || !FontLink_Chain_FindLink(pChain, &mut lf).is_null()
    {
        return core::ptr::null_mut();
    }

    let pLink: PFONTLINK = ExAllocatePoolZero(
        PagedPool,
        core::mem::size_of::<FONTLINK>() as _ ,
        TAG_FONT,
    ) as PFONTLINK;
    if pLink.is_null() {
        return core::ptr::null_mut();
    }

    (*pLink).LogFont = lf;
    pLink
}

unsafe fn FontLink_Chain_Populate(pChain: PFONTLINK_CHAIN) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLink: PFONTLINK;
    let mut lfBase: LOGFONTW;
    let pTextObj: PTEXTOBJ = (*pChain).pBaseTextObj;
    let pFontGDI: PFONTGDI;
    let mut pszLink: PWSTR;
    let pLinkCache: PFONTLINK_CACHE;
    let mut szEntry: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let bFixCharSet: BOOL;

    InitializeListHead(core::ptr::addr_of_mut!((*pChain).FontLinkList));

    lfBase = (*pTextObj).logfont.elfEnumLogfontEx.elfLogFont;
    pFontGDI = ObjToGDI!((*pTextObj).Font, FONT);
    lfBase.lfHeight = (*pFontGDI).lfHeight;
    lfBase.lfWidth = (*pFontGDI).lfWidth;

    // Use pTextObj->TextFace if lfFaceName was empty
    if lfBase.lfFaceName[0] == 0 {
        w32r_RtlStringCchCopyW(
            lfBase.lfFaceName.as_mut_ptr(),
            _countof!(lfBase.lfFaceName),
            (*pTextObj).TextFace.as_ptr(),
        );
    }

    // Fix lfCharSet
    bFixCharSet = match lfBase.lfCharSet as u32 {
        ANSI_CHARSET => (s_fFontLinkUseAnsi == 0) as BOOL,
        OEM_CHARSET => (s_fFontLinkUseOem == 0) as BOOL,
        SYMBOL_CHARSET => (s_fFontLinkUseSymbol == 0) as BOOL,
        _ => 1 as BOOL,
    };
    if bFixCharSet != 0 {
        lfBase.lfCharSet = DEFAULT_CHARSET as u8;
    }

    // Use cache if any
    pLinkCache = FontLink_FindCache(&lfBase);
    if !pLinkCache.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*pLinkCache).ListEntry));
        core::ptr::write(pChain, core::ptr::read(core::ptr::addr_of!((*pLinkCache).Chain)));
        IntRebaseList(
            core::ptr::addr_of_mut!((*pChain).FontLinkList),
            core::ptr::addr_of_mut!((*pLinkCache).Chain.FontLinkList),
        );
        ExFreePoolWithTag(pLinkCache as *mut _, TAG_FONT);
        return STATUS_SUCCESS;
    }

    (*pChain).LogFont = lfBase;

    // Load FontLink entry from registry
    Status = FontLink_Chain_LoadReg(pChain, core::ptr::addr_of_mut!((*pChain).LogFont));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    pszLink = (*pChain).pszzFontLink;
    while *pszLink != 0 {
        pLink = FontLink_Create(pChain, &lfBase, pszLink);
        if !pLink.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!((*pChain).FontLinkList),
                core::ptr::addr_of_mut!((*pLink).ListEntry),
            );
        }
        pszLink = pszLink.add(wcslen(pszLink) + 1);
    }

    // Use default settings (if any)
    if s_szDefFontLinkFileName[0] != 0 && s_szDefFontLinkFontName[0] != 0 {
        w32r_RtlStringCchCopyW(
            szEntry.as_mut_ptr(),
            _countof!(szEntry),
            (*(&raw const s_szDefFontLinkFileName)).as_ptr(),
        );
        w32r_RtlStringCchCatW(
            szEntry.as_mut_ptr(),
            _countof!(szEntry),
            w!(",").as_ptr() as *const u16,
        );
        w32r_RtlStringCchCatW(
            szEntry.as_mut_ptr(),
            _countof!(szEntry),
            (*(&raw const s_szDefFontLinkFontName)).as_ptr(),
        );
        pLink = FontLink_Create(pChain, &lfBase, szEntry.as_ptr());
        if !pLink.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!((*pChain).FontLinkList),
                core::ptr::addr_of_mut!((*pLink).ListEntry),
            );
        }
    }

    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntLoadSystemFonts() {
    IntLoadSystemFonts_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntLoadSystemFonts() {
    IntLoadSystemFonts_impl();
}

extern "system" {
    fn ZwQueryDirectoryFile(
        FileHandle: HANDLE,
        Event: HANDLE,
        ApcRoutine: PVOID,
        ApcContext: PVOID,
        IoStatusBlock: *mut IO_STATUS_BLOCK,
        FileInformation: PVOID,
        Length: ULONG,
        FileInformationClass: ULONG,
        ReturnSingleEntry: BOOLEAN,
        FileName: *mut _UNICODE_STRING,
        RestartScan: BOOLEAN,
    ) -> NTSTATUS;
    fn RtlCopyUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const _UNICODE_STRING);
}

/// Load every font file in `\SystemRoot\Fonts`, as upstream does.
///
/// This used to load nothing on x86 ("deferred to winlogon", which then added 8 fonts
/// by name) and a fixed 6 on arm64. Neither list held the console's default face
/// "VGA" (UniVGA16.ttf) or its "Terminal" fallback, so no console window could be
/// created on any arch: consrv's WM_NCCREATE failed in InitFonts, and its teardown
/// then waited forever on a term event the input thread never sets for a window
/// that does not exist. That wedged the csrss API thread serving the CUI process.
unsafe fn IntLoadSystemFonts_impl() {
    const FILE_DIRECTORY_INFORMATION_CLASS: ULONG = 1; // FileDirectoryInformation
    const STATUS_NO_MORE_FILES: NTSTATUS = 0x8000_0006u32 as NTSTATUS;
    const DIR_BUFFER_SIZE: ULONG = 0x4000;
    // Upstream queries once per pattern (*.ttf, *.ttc, *.otf, *.otc, *.fon, *.fnt).
    // Our FAT driver ignores the pattern and returns every file each time, which
    // loaded each font six times, so enumerate once and match the extension here.
    const EXTENSIONS: [[u16; 4]; 6] = [
        [b'.' as u16, b't' as u16, b't' as u16, b'f' as u16],
        [b'.' as u16, b't' as u16, b't' as u16, b'c' as u16],
        [b'.' as u16, b'o' as u16, b't' as u16, b'f' as u16],
        [b'.' as u16, b'o' as u16, b't' as u16, b'c' as u16],
        [b'.' as u16, b'f' as u16, b'o' as u16, b'n' as u16],
        [b'.' as u16, b'f' as u16, b'n' as u16, b't' as u16],
    ];
    unsafe fn has_font_extension(name: *const u16, len_chars: usize) -> bool {
        if len_chars < 4 {
            return false;
        }
        let tail = core::slice::from_raw_parts(name.add(len_chars - 4), 4);
        EXTENSIONS.iter().any(|ext| {
            tail.iter().zip(ext.iter()).all(|(&c, &e)| {
                let lc = if (b'A' as u16..=b'Z' as u16).contains(&c) { c + 32 } else { c };
                lc == e
            })
        })
    }
    let mut Directory: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Directory, wide_str!("\\SystemRoot\\Fonts\\"));
    let mut Dot: _UNICODE_STRING = core::mem::zeroed();
    let mut DotDot: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Dot, wide_str!("."));
    RtlInitUnicodeString(&mut DotDot, wide_str!(".."));

    let DirInfoBuffer = ExAllocatePoolWithTag(PagedPool, DIR_BUFFER_SIZE as usize, TAG_FONT) as *mut u8;
    if DirInfoBuffer.is_null() {
        return;
    }
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    FileName.Buffer = ExAllocatePoolWithTag(PagedPool, MAX_PATH as usize * 2, TAG_FONT) as *mut u16;
    if FileName.Buffer.is_null() {
        ExFreePoolWithTag(DirInfoBuffer as _, TAG_FONT);
        return;
    }
    FileName.MaximumLength = (MAX_PATH * 2) as USHORT;

    let mut nLoaded: INT = 0;
    {
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.ObjectName = &mut Directory;
        let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut hDirectory: HANDLE = core::ptr::null_mut();
        let Status = ZwOpenFile(
            &mut hDirectory,
            SYNCHRONIZE | FILE_LIST_DIRECTORY,
            &mut ObjectAttributes,
            &mut Iosb,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_DIRECTORY_FILE,
        );
        if !NT_SUCCESS(Status) {
            DbgPrint(b"IntLoadSystemFonts: cannot open fonts directory (0x%08lx)\n\0".as_ptr() as *const i8, Status);
            ExFreePoolWithTag(FileName.Buffer as _, TAG_FONT);
            ExFreePoolWithTag(DirInfoBuffer as _, TAG_FONT);
            return;
        }

        let mut bRestartScan: BOOLEAN = 1;
        loop {
            let Status = ZwQueryDirectoryFile(
                hDirectory,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut Iosb,
                DirInfoBuffer as PVOID,
                DIR_BUFFER_SIZE,
                FILE_DIRECTORY_INFORMATION_CLASS,
                0,
                core::ptr::null_mut(),
                bRestartScan,
            );
            if !NT_SUCCESS(Status) || Status == STATUS_NO_MORE_FILES {
                break;
            }

            let mut DirInfo = DirInfoBuffer as *mut _FILE_DIRECTORY_INFORMATION;
            loop {
                let mut TempString: _UNICODE_STRING = core::mem::zeroed();
                TempString.Buffer = core::ptr::addr_of_mut!((*DirInfo).FileName) as *mut u16;
                TempString.Length = (*DirInfo).FileNameLength as USHORT;
                TempString.MaximumLength = TempString.Length;

                if RtlEqualUnicodeString(&Dot, &TempString, 0) == 0
                    && RtlEqualUnicodeString(&DotDot, &TempString, 0) == 0
                    && has_font_extension(TempString.Buffer, TempString.Length as usize / 2)
                {
                    RtlCopyUnicodeString(&mut FileName, &Directory);
                    RtlAppendUnicodeStringToString(&mut FileName, &TempString);
                    if IntGdiAddFontResourceEx(&FileName, 1, 0, AFRX_WRITE_REGISTRY) != 0 {
                        nLoaded += 1;
                    } else {
                        DbgPrint(b"IntLoadSystemFonts: failed to load %wZ\n\0".as_ptr() as *const i8, &FileName as *const _UNICODE_STRING);
                    }
                }

                if (*DirInfo).NextEntryOffset == 0 {
                    break;
                }
                DirInfo = (DirInfo as *mut u8).add((*DirInfo).NextEntryOffset as usize) as *mut _FILE_DIRECTORY_INFORMATION;
            }
            bRestartScan = 0;
        }
        ZwClose(hDirectory);
    }

    DbgPrint(b"IntLoadSystemFonts: loaded %d font files from \\SystemRoot\\Fonts\n\0".as_ptr() as *const i8, nLoaded);
    ExFreePoolWithTag(FileName.Buffer as _, TAG_FONT);
    ExFreePoolWithTag(DirInfoBuffer as _, TAG_FONT);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCharSet(nIndex: INT, CodePageRange1: FT_ULong) -> UINT {
    IntGetCharSetImpl(nIndex, CodePageRange1)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCharSet(nIndex: INT, CodePageRange1: FT_ULong) -> UINT {
    IntGetCharSetImpl(nIndex, CodePageRange1)
}

unsafe fn IntGetCharSetImpl(nIndex: INT, CodePageRange1: FT_ULong) -> UINT {
    let mut CharSet: UINT;
    let mut nCount: UINT = 0;

    if CodePageRange1 == 0 {
        return if nIndex < 0 { 1 } else { DEFAULT_CHARSET as UINT };
    }

    let mut BitIndex: UINT = 0;
    while BitIndex < MAXTCIINDEX as UINT {
        if CodePageRange1 & (1u32 as FT_ULong) << BitIndex != 0 {
            CharSet = g_FontTci[BitIndex as usize].ciCharset as UINT;
            if nIndex >= 0 && nCount == nIndex as UINT {
                return CharSet;
            }
            nCount += 1;
        }
        BitIndex += 1;
    }

    if nIndex < 0 { nCount } else { ANSI_CHARSET as UINT }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiLoadFontsFromMemory(
    pLoadFont: PGDI_LOAD_FONT,
    SharedFace: PSHARED_FACE,
    FontIndex: FT_Long,
    CharSetIndex: INT,
) -> INT {
    IntGdiLoadFontsFromMemory_impl(pLoadFont, SharedFace, FontIndex, CharSetIndex)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiLoadFontsFromMemory(
    pLoadFont: PGDI_LOAD_FONT,
    SharedFace: PSHARED_FACE,
    FontIndex: FT_Long,
    CharSetIndex: INT,
) -> INT {
    IntGdiLoadFontsFromMemory_impl(pLoadFont, SharedFace, FontIndex, CharSetIndex)
}

unsafe fn IntGdiLoadFontsFromMemory_impl(
    pLoadFont: PGDI_LOAD_FONT,
    SharedFace: PSHARED_FACE,
    _FontIndex: FT_Long,
    CharSetIndex: INT,
) -> INT {
    let mut Error: FT_Error = 0;
    let Entry: PFONT_ENTRY;
    let mut PrivateEntry: *mut FONT_ENTRY_MEM = core::ptr::null_mut();
    let FontGDI: *mut FONTGDI;
    let mut Status: NTSTATUS;
    let Face: FT_Face;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut WinFNT: FT_WinFNT_HeaderRec = core::mem::zeroed();
    let mut FaceCount: INT = 0;
    let mut CharSetCount: INT = 0;
    let pFileName: PUNICODE_STRING = (*pLoadFont).pFileName;
    let Characteristics: DWORD = (*pLoadFont).Characteristics;
    let pValueName: PUNICODE_STRING = core::ptr::addr_of_mut!((*pLoadFont).RegValueName);
    let mut pOS2: *mut TT_OS2;
    let mut BitIndex: INT;
    let os2_version: FT_UShort;
    let mut os2_ulCodePageRange1: FT_ULong = 0;
    let mut os2_usWeightClass: FT_UShort = 0;

    IntLockFreeType();
    Face = (*SharedFace).Face;
    SharedFace_AddRef(SharedFace);
    IntUnLockFreeType();

    Entry = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<_FONT_ENTRY>() as _, TAG_FONT) as PFONT_ENTRY;
    if Entry.is_null() {
        SharedFace_Release(SharedFace, 1u8 as BOOL);
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    FontGDI = EngAllocMem(FL_ZERO_MEMORY as _, core::mem::size_of::<FONTGDI>() as _, GDITAG_RFONT) as *mut FONTGDI;
    if FontGDI.is_null() {
        SharedFace_Release(SharedFace, 1u8 as BOOL);
        ExFreePoolWithTag(Entry as _, TAG_FONT);
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if !pFileName.is_null() {
        (*FontGDI).Filename = ExAllocatePoolWithTag(
            PagedPool,
            ((*pFileName).Length as usize + core::mem::size_of::<u16>()) as _,
            GDITAG_PFF,
        ) as _;
        if (*FontGDI).Filename.is_null() {
            EngFreeMem(FontGDI as _);
            SharedFace_Release(SharedFace, 1u8 as BOOL);
            ExFreePoolWithTag(Entry as _, TAG_FONT);
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            (*pFileName).Buffer as *const u8,
            (*FontGDI).Filename as *mut u8,
            (*pFileName).Length as usize,
        );
        *(*FontGDI).Filename.add((*pFileName).Length as usize / core::mem::size_of::<u16>()) = 0;
    } else {
        (*FontGDI).Filename = core::ptr::null_mut();

        PrivateEntry = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<FONT_ENTRY_MEM>() as _, TAG_FONT) as *mut FONT_ENTRY_MEM;
        if PrivateEntry.is_null() {
            if !(*FontGDI).Filename.is_null() {
                ExFreePoolWithTag((*FontGDI).Filename as _, GDITAG_PFF);
            }
            EngFreeMem(FontGDI as _);
            SharedFace_Release(SharedFace, 1u8 as BOOL);
            ExFreePoolWithTag(Entry as _, TAG_FONT);
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }

        (*PrivateEntry).Entry = Entry;
        if !(*pLoadFont).PrivateEntry.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!((*(*pLoadFont).PrivateEntry).ListEntry),
                core::ptr::addr_of_mut!((*PrivateEntry).ListEntry),
            );
        } else {
            InitializeListHead(core::ptr::addr_of_mut!((*PrivateEntry).ListEntry));
            (*pLoadFont).PrivateEntry = PrivateEntry;
        }
    }

    (*FontGDI).SharedFace = SharedFace;
    (*FontGDI).CharSet = ANSI_CHARSET as _;
    (*FontGDI).OriginalItalic = 0u8 as _;
    (*FontGDI).RequestItalic = 0u8 as _;
    (*FontGDI).OriginalWeight = FW_DONTCARE as _;
    (*FontGDI).RequestWeight = FW_NORMAL as _;

    IntLockFreeType();
    pOS2 = FT_Get_Sfnt_Table(Face, FT_SFNT_OS2) as *mut TT_OS2;
    if !pOS2.is_null() {
        (*FontGDI).OriginalItalic = (((*pOS2).fsSelection & 0x1) != 0) as _;
        (*FontGDI).OriginalWeight = (*pOS2).usWeightClass as _;
    } else {
        Error = FT_Get_WinFNT_Header(Face, &mut WinFNT);
        if Error == 0 {
            (*FontGDI).OriginalItalic = (WinFNT.italic != 0) as _;
            (*FontGDI).OriginalWeight = WinFNT.weight as _;
        }
    }
    IntUnLockFreeType();

    RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString) as _ , (*Face).family_name);
    Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!((*Entry).FaceName), &mut AnsiString, 1u8);
    if NT_SUCCESS(Status) {
        if !(*Face).style_name.is_null()
            && *(*Face).style_name != 0
            && strcmp((*Face).style_name, b"Regular\0".as_ptr() as PCHAR) != 0
        {
            RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString) as _ , (*Face).style_name);
            Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!((*Entry).StyleName), &mut AnsiString, 1u8);
            if !NT_SUCCESS(Status) {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Entry).FaceName) as _ );
            }
        } else {
            RtlInitUnicodeString(core::ptr::addr_of_mut!((*Entry).StyleName) as _ , core::ptr::null_mut());
        }
    }
    if !NT_SUCCESS(Status) {
        if !PrivateEntry.is_null() {
            if (*pLoadFont).PrivateEntry == PrivateEntry {
                (*pLoadFont).PrivateEntry = core::ptr::null_mut();
            } else {
                RemoveEntryList(core::ptr::addr_of_mut!((*PrivateEntry).ListEntry));
            }
            ExFreePoolWithTag(PrivateEntry as _, TAG_FONT);
        }
        if !(*FontGDI).Filename.is_null() {
            ExFreePoolWithTag((*FontGDI).Filename as _, GDITAG_PFF);
        }
        EngFreeMem(FontGDI as _);
        SharedFace_Release(SharedFace, 1u8 as BOOL);
        ExFreePoolWithTag(Entry as _, TAG_FONT);
        return 0;
    }

    os2_version = 0;
    IntLockFreeType();
    pOS2 = FT_Get_Sfnt_Table(Face, FT_SFNT_OS2) as *mut TT_OS2;
    let os2_version = if !pOS2.is_null() {
        os2_ulCodePageRange1 = (*pOS2).ulCodePageRange1;
        os2_usWeightClass = (*pOS2).usWeightClass;
        (*pOS2).version
    } else {
        os2_version
    };
    IntUnLockFreeType();

    if !pOS2.is_null() && os2_version >= 1 {
        pOS2 = core::ptr::null_mut();

        BitIndex = 0;
        while BitIndex < (MAXTCIINDEX as _) {
            if (os2_ulCodePageRange1 & (1u32 << BitIndex) as FT_ULong) != 0 {
                if g_FontTci[BitIndex as usize].ciCharset as u32 == DEFAULT_CHARSET {
                    BitIndex += 1;
                    continue;
                }

                if (CharSetIndex == -1 && CharSetCount == 0) || CharSetIndex == CharSetCount {
                    (*FontGDI).CharSet = g_FontTci[BitIndex as usize].ciCharset as _;
                }

                CharSetCount += 1;
            }
            BitIndex += 1;
        }

        (*FontGDI).OriginalWeight = os2_usWeightClass as _;
    } else {
        IntLockFreeType();
        Error = FT_Get_WinFNT_Header(Face, &mut WinFNT);
        if Error == 0 {
            (*FontGDI).CharSet = WinFNT.charset as _;
        }
        IntUnLockFreeType();
    }

    FaceCount += 1;

    (*Entry).Font = FontGDI;
    (*Entry).NotEnum = (Characteristics & FR_NOT_ENUM) as _;

    IntLockFreeType();
    if (Characteristics & FR_PRIVATE) != 0 {
        let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;
        IntLockProcessPrivateFonts(Win32Process as _ );
        InsertTailList(
            core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead),
            core::ptr::addr_of_mut!((*Entry).ListEntry),
        );
        IntUnLockProcessPrivateFonts(Win32Process as _ );
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!(g_FontListHead),
            core::ptr::addr_of_mut!((*Entry).ListEntry),
        );
    }
    IntUnLockFreeType();

    if CharSetIndex == -1 {
        let mut i: INT;
        let mut NameLength: USHORT = (*Entry).FaceName.Length;

        if (*Entry).StyleName.Length != 0 {
            NameLength += (*Entry).StyleName.Length + core::mem::size_of::<u16>() as USHORT;
        }

        if (*pLoadFont).RegValueName.Length == 0 {
            (*pValueName).Length = 0;
            (*pValueName).MaximumLength = NameLength + core::mem::size_of::<u16>() as USHORT;
            (*pValueName).Buffer = ExAllocatePoolWithTag(PagedPool, (*pValueName).MaximumLength as _, TAG_USTR) as _;
            *(*pValueName).Buffer = 0;
            RtlAppendUnicodeStringToString(pValueName, core::ptr::addr_of_mut!((*Entry).FaceName));
        } else {
            let mut NewString: _UNICODE_STRING = core::mem::zeroed();
            let Length: USHORT = (*pValueName).Length + 3 * core::mem::size_of::<u16>() as USHORT + NameLength;
            NewString.Length = 0;
            NewString.MaximumLength = Length + core::mem::size_of::<u16>() as USHORT;
            NewString.Buffer = ExAllocatePoolWithTag(PagedPool, NewString.MaximumLength as _, TAG_USTR) as _;
            *NewString.Buffer = 0;

            RtlAppendUnicodeStringToString(&mut NewString, pValueName);
            RtlAppendUnicodeToString(core::ptr::addr_of_mut!(NewString) as _ , w_str_amp());
            RtlAppendUnicodeStringToString(&mut NewString, core::ptr::addr_of_mut!((*Entry).FaceName));

            RtlFreeUnicodeString(pValueName as _ );
            *pValueName = NewString;
        }
        if (*Entry).StyleName.Length != 0 {
            RtlAppendUnicodeToString(pValueName as _ , w_str_space());
            RtlAppendUnicodeStringToString(pValueName, core::ptr::addr_of_mut!((*Entry).StyleName));
        }

        i = 1;
        while i < CharSetCount {
            IntGdiLoadFontsFromMemory(pLoadFont, SharedFace, _FontIndex, i);
            i += 1;
        }
    }

    FaceCount
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiLoadFontByIndexFromMemory(
    pLoadFont: PGDI_LOAD_FONT,
    FontIndex: FT_Long,
) -> INT {
    IntGdiLoadFontByIndexFromMemory_impl(pLoadFont, FontIndex)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiLoadFontByIndexFromMemory(
    pLoadFont: PGDI_LOAD_FONT,
    FontIndex: FT_Long,
) -> INT {
    IntGdiLoadFontByIndexFromMemory_impl(pLoadFont, FontIndex)
}

unsafe fn IntGdiLoadFontByIndexFromMemory_impl(
    pLoadFont: PGDI_LOAD_FONT,
    mut FontIndex: FT_Long,
) -> INT {
    let Error: FT_Error;
    let mut Face: FT_Face = core::ptr::null_mut();
    let mut iFace: FT_Long;
    let num_faces: FT_Long;
    let SharedFace: PSHARED_FACE;
    let mut FaceCount: INT = 0;

    IntLockFreeType();

    Error = FT_New_Memory_Face(
        g_FreeTypeLibrary,
        (*(*pLoadFont).Memory).Buffer as _ ,
        (*(*pLoadFont).Memory).BufferSize as FT_Long,
        if FontIndex == -1 { 0 } else { FontIndex },
        core::ptr::addr_of_mut!(Face),
    );
    if Error != 0 {
        IntUnLockFreeType();
        return 0;
    }

    (*pLoadFont).IsTrueType = FT_IS_SFNT(Face);
    num_faces = (*Face).num_faces;
    SharedFace = SharedFace_Create(Face, (*pLoadFont).Memory);

    IntUnLockFreeType();

    if SharedFace.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if FontIndex == -1 {
        iFace = 1;
        while iFace < num_faces {
            FaceCount += IntGdiLoadFontByIndexFromMemory(pLoadFont, iFace);
            iFace += 1;
        }
        FontIndex = 0;
    }

    FaceCount += IntGdiLoadFontsFromMemory(pLoadFont, SharedFace, FontIndex, -1);
    FaceCount
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiAddFontResourceSingle(
    FileName: PCUNICODE_STRING,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    IntGdiAddFontResourceSingle_impl(FileName, Characteristics, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiAddFontResourceSingle(
    FileName: PCUNICODE_STRING,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    IntGdiAddFontResourceSingle_impl(FileName, Characteristics, dwFlags)
}

unsafe fn IntGdiAddFontResourceSingle_impl(
    FileName: PCUNICODE_STRING,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut SectionObject: PVOID = core::ptr::null_mut();
    let mut ViewSize: SIZE_T = 0;
    let Length: SIZE_T;
    let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LoadFont: GDI_LOAD_FONT = core::mem::zeroed();
    let FontCount: INT;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut PathName: _UNICODE_STRING = core::mem::zeroed();
    let pszBuffer: LPWSTR;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let TrueTypePostfix: _UNICODE_STRING = RTL_CONSTANT_STRING(
        w!(" (TrueType)").as_ptr() as LPWSTR,
    );

    /* Build PathName */
    if (dwFlags & AFRX_DOS_DEVICE_PATH) != 0 {
        Length = (DosPathPrefix.Length as SIZE_T)
            + ((*FileName).Length as SIZE_T)
            + core::mem::size_of::<u16>() as SIZE_T;
        pszBuffer = ExAllocatePoolWithTag(PagedPool, Length as _ , TAG_USTR) as LPWSTR;
        if pszBuffer.is_null() {
            return 0;
        }

        RtlInitEmptyUnicodeString(&mut PathName, pszBuffer, Length as USHORT);
        RtlAppendUnicodeStringToString(&mut PathName, &raw const DosPathPrefix);
        RtlAppendUnicodeStringToString(&mut PathName, FileName);
    } else {
        Status = IntDuplicateUnicodeString(FileName as _ , &mut PathName);
        if !NT_SUCCESS(Status) {
            return 0;
        }
    }

    /* Open the font file */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut PathName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenFile(
        &mut FileHandle,
        FILE_GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut Iosb,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS(Status) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
        return 0;
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        FILE_READ_DATA,
        core::ptr::null_mut(),
        0i32 as i8,
        &mut FileObject as *mut PFILE_OBJECT as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ZwClose(FileHandle);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
        return 0;
    }

    SectionSize.QuadPart = 0;
    Status = MmCreateSection(
        &mut SectionObject,
        STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SectionSize) as _,
        PAGE_READONLY,
        SEC_COMMIT,
        FileHandle,
        FileObject as *mut _,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(FileHandle);
        ObDereferenceObject(FileObject as PVOID);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
        return 0;
    }
    ZwClose(FileHandle);

    Status = MmMapViewInSystemSpace(SectionObject, &mut Buffer, &mut ViewSize);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(SectionObject);
        ObDereferenceObject(FileObject as PVOID);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
        return 0;
    }

    RtlZeroMemory(
        &mut LoadFont as *mut GDI_LOAD_FONT as PVOID,
        core::mem::size_of::<GDI_LOAD_FONT>(),
    );
    LoadFont.pFileName = &mut PathName;
    LoadFont.Memory = SharedMem_Create(Buffer as PBYTE, ViewSize as ULONG, 1);
    LoadFont.Characteristics = Characteristics;
    RtlInitUnicodeString(core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ , core::ptr::null_mut());
    LoadFont.CharSet = DEFAULT_CHARSET as u8;
    FontCount = IntGdiLoadFontByIndexFromMemory(&mut LoadFont, -1);

    /* Release our copy */
    IntLockFreeType();
    SharedMem_Release(LoadFont.Memory);
    IntUnLockFreeType();

    ObDereferenceObject(SectionObject);

    ObDereferenceObject(FileObject as PVOID);

    /* Save the loaded font name into the registry */
    if FontCount > 0 && (dwFlags & AFRX_WRITE_REGISTRY) != 0 {
        let mut NewString: _UNICODE_STRING = core::mem::zeroed();
        let Length: SIZE_T;
        let pszBuffer: PWCHAR;
        let CharSetName: LPCWSTR;
        if LoadFont.IsTrueType != 0 {
            /* Append " (TrueType)" */
            let Length2 = (LoadFont.RegValueName.Length as SIZE_T)
                + (TrueTypePostfix.Length as SIZE_T)
                + core::mem::size_of::<u16>() as SIZE_T;
            let pszBuffer2 = ExAllocatePoolWithTag(PagedPool, Length2 as _ , TAG_USTR) as PWCHAR;
            if !pszBuffer2.is_null() {
                RtlInitEmptyUnicodeString(&mut NewString, pszBuffer2, Length2 as USHORT);
                *NewString.Buffer.offset(0) = 0;
                RtlAppendUnicodeStringToString(&mut NewString, &LoadFont.RegValueName);
                RtlAppendUnicodeStringToString(&mut NewString, &TrueTypePostfix);
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ );
                LoadFont.RegValueName = NewString;
            }
        } else if LoadFont.CharSet != DEFAULT_CHARSET as u8 {
            /* Append " (CharSetName)" */
            CharSetName = IntNameFromCharSet((LoadFont.CharSet as INT) as _);
            Length = (LoadFont.RegValueName.Length as SIZE_T)
                + ((wcslen(CharSetName) + 3) * core::mem::size_of::<u16>()) as SIZE_T
                + core::mem::size_of::<u16>() as SIZE_T;

            pszBuffer = ExAllocatePoolWithTag(PagedPool, Length as _ , TAG_USTR) as PWCHAR;
            if !pszBuffer.is_null() {
                RtlInitEmptyUnicodeString(&mut NewString, pszBuffer, Length as USHORT);
                *NewString.Buffer.offset(0) = 0;
                RtlAppendUnicodeStringToString(&mut NewString, &LoadFont.RegValueName);
                RtlAppendUnicodeToString(
                    core::ptr::addr_of_mut!(NewString) as _ ,
                    w!(" (").as_ptr() as PCWSTR,
                );
                RtlAppendUnicodeToString(core::ptr::addr_of_mut!(NewString) as _ , CharSetName);
                RtlAppendUnicodeToString(core::ptr::addr_of_mut!(NewString) as _ , w!(")").as_ptr() as PCWSTR);
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ );
                LoadFont.RegValueName = NewString;
            }
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.ObjectName = &raw mut g_FontRegPath;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(&mut KeyHandle, KEY_WRITE, core::ptr::addr_of_mut!(ObjectAttributes) as _ );
        if NT_SUCCESS(Status) {
            let DataSize: SIZE_T;
            let mut pFileName: LPWSTR;

            if (dwFlags & AFRX_ALTERNATIVE_PATH) != 0 {
                pFileName = PathName.Buffer;
            } else {
                pFileName = wcsrchr(PathName.Buffer, '\\' as u16 as i32 as _ );
            }

            if !pFileName.is_null() {
                if (dwFlags & AFRX_ALTERNATIVE_PATH) == 0 {
                    pFileName = pFileName.offset(1);
                }
                DataSize = ((wcslen(pFileName) + 1) * core::mem::size_of::<u16>()) as SIZE_T;
                ZwSetValueKey(
                    KeyHandle,
                    core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ ,
                    0,
                    REG_SZ,
                    pFileName as PVOID,
                    DataSize as ULONG,
                );
            }
            ZwClose(KeyHandle);
        }
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
    FontCount
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiAddFontResourceEx(
    FileName: PCUNICODE_STRING,
    cFiles: DWORD,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    IntGdiAddFontResourceEx_impl(FileName, cFiles, Characteristics, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiAddFontResourceEx(
    FileName: PCUNICODE_STRING,
    cFiles: DWORD,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    IntGdiAddFontResourceEx_impl(FileName, cFiles, Characteristics, dwFlags)
}

unsafe fn IntGdiAddFontResourceEx_impl(
    FileName: PCUNICODE_STRING,
    mut cFiles: DWORD,
    Characteristics: DWORD,
    dwFlags: DWORD,
) -> INT {
    let mut pchFile: PWSTR = (*FileName).Buffer;
    let mut cchFile: SIZE_T;
    let mut ret: INT = 0;

    while cFiles != 0 {
        cFiles -= 1;

        cchFile = wcslen(pchFile) as _ ;

        let mut ustrPathName: _UNICODE_STRING = core::mem::zeroed();
        ustrPathName.Length = ((cchFile as usize) * core::mem::size_of::<WCHAR>()) as USHORT;
        ustrPathName.MaximumLength = ustrPathName.Length + core::mem::size_of::<WCHAR>() as USHORT;
        ustrPathName.Buffer = pchFile;

        let count: INT = IntGdiAddFontResourceSingle(
            core::ptr::addr_of_mut!(ustrPathName),
            Characteristics,
            dwFlags,
        );
        if count == 0 {
            return 0;
        }
        ret += count;

        pchFile = pchFile.add((cchFile + 1) as usize);
    }

    ret
}

unsafe fn IntDeleteRegFontEntries(pszFileName: PCWSTR, dwFlags: DWORD) {
    let mut Status: NTSTATUS;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szValue: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwIndex: ULONG;
    let mut NameLength: ULONG;
    let mut ValueSize: ULONG;
    let mut dwType: ULONG = 0;

    Status = RegOpenKey(g_FontRegPath.Buffer, &mut hKey);
    if !NT_SUCCESS(Status) {
        return;
    }

    dwIndex = 0;
    loop {
        NameLength = RTL_NUMBER_OF!(szName) as ULONG;
        ValueSize = core::mem::size_of_val(&szValue) as ULONG;
        Status = RegEnumValueW(
            hKey as _ ,
            dwIndex,
            szName.as_mut_ptr(),
            &mut NameLength,
            &mut dwType,
            szValue.as_mut_ptr() as *mut u8,
            &mut ValueSize,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        if dwType != REG_SZ || _wcsicmp(szValue.as_ptr(), pszFileName) != 0 {
            dwIndex += 1;
            continue;
        }

        Status = RegDeleteValueW(hKey as _ , szName.as_ptr());
        if !NT_SUCCESS(Status) {
            break;
        }
    }

    ZwClose(hKey as *mut core::ffi::c_void);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiRemoveFontResourceSingle(
    FileName: PCUNICODE_STRING,
    dwFlags: DWORD,
) -> BOOL {
    IntGdiRemoveFontResourceSingle_impl(FileName, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiRemoveFontResourceSingle(
    FileName: PCUNICODE_STRING,
    dwFlags: DWORD,
) -> BOOL {
    IntGdiRemoveFontResourceSingle_impl(FileName, dwFlags)
}

unsafe fn IntGdiRemoveFontResourceSingle_impl(
    FileName: PCUNICODE_STRING,
    dwFlags: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut PathName: _UNICODE_STRING = core::mem::zeroed();
    let mut CurrentEntry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut FontEntry: PFONT_ENTRY;
    let mut FontGDI: PFONTGDI;
    let mut pszBuffer: PWSTR;
    let mut pszFileTitle: PWSTR;
    let mut Length: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    if dwFlags & AFRX_DOS_DEVICE_PATH != 0 {
        Length = (DosPathPrefix.Length as SIZE_T)
            + ((*FileName).Length as SIZE_T)
            + core::mem::size_of::<u16>() as SIZE_T;
        pszBuffer = ExAllocatePoolWithTag(PagedPool, Length as _ , TAG_USTR) as PWSTR;
        if pszBuffer.is_null() {
            return 0;
        }

        RtlInitEmptyUnicodeString(&mut PathName, pszBuffer, Length as USHORT);
        RtlAppendUnicodeStringToString(&mut PathName, &raw const DosPathPrefix);
        RtlAppendUnicodeStringToString(&mut PathName, FileName);
    } else {
        Status = IntDuplicateUnicodeString(FileName as _ , &mut PathName);
        if !NT_SUCCESS(Status) {
            return 0;
        }
    }

    pszFileTitle = PathName.Buffer;
    if dwFlags & AFRX_ALTERNATIVE_PATH == 0 {
        pszFileTitle = PathFindFileNameW(PathName.Buffer);
    }

    if pszFileTitle.is_null() || *pszFileTitle == 0 {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
        return 0;
    }

    IntLockFreeType();
    CurrentEntry = g_FontListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(g_FontListHead) {
        FontEntry = (CurrentEntry as *mut u8)
            .sub(core::mem::offset_of!(_FONT_ENTRY, ListEntry)) as PFONT_ENTRY;
        NextEntry = (*CurrentEntry).Flink;

        FontGDI = (*FontEntry).Font;
        if !(*FontGDI).Filename.is_null()
            && _wcsicmp((*FontGDI).Filename, pszFileTitle) == 0
        {
            FontLookUp_Cleanup(1, (*FontGDI).SharedFace);
            RemoveEntryList(core::ptr::addr_of_mut!((*FontEntry).ListEntry));
            CleanupFontEntry(FontEntry);
            if dwFlags & AFRX_WRITE_REGISTRY != 0 {
                IntDeleteRegFontEntries(pszFileTitle, dwFlags);
            }
            ret = 1;
        }

        CurrentEntry = NextEntry;
    }
    IntUnLockFreeType();

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(PathName) as _ );
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiRemoveFontResource(
    FileName: PCUNICODE_STRING,
    cFiles: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    IntGdiRemoveFontResource_impl(FileName, cFiles, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiRemoveFontResource(
    FileName: PCUNICODE_STRING,
    cFiles: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    IntGdiRemoveFontResource_impl(FileName, cFiles, dwFlags)
}

unsafe fn IntGdiRemoveFontResource_impl(
    FileName: PCUNICODE_STRING,
    mut cFiles: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut pchFile: PWSTR = (*FileName).Buffer;
    let mut cchFile: SIZE_T;

    loop {
        let old = cFiles;
        cFiles = cFiles.wrapping_sub(1);
        if old == 0 {
            break;
        }

        cchFile = wcslen(pchFile) as SIZE_T;

        let mut ustrPathName: _UNICODE_STRING = core::mem::zeroed();
        ustrPathName.Length = (cchFile * core::mem::size_of::<WCHAR>() as SIZE_T) as USHORT;
        ustrPathName.MaximumLength = ustrPathName.Length + core::mem::size_of::<WCHAR>() as USHORT;
        ustrPathName.Buffer = pchFile;

        let ret = IntGdiRemoveFontResourceSingle(&mut ustrPathName, dwFlags);
        if ret == 0 {
            return 0;
        }

        pchFile = pchFile.add((cchFile + 1) as usize);
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntLoadFontsInRegistry() -> BOOL {
    IntLoadFontsInRegistry_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntLoadFontsInRegistry() -> BOOL {
    IntLoadFontsInRegistry_impl()
}

unsafe fn IntLoadFontsInRegistry_impl() -> BOOL {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyFullInfo: KEY_FULL_INFORMATION = core::mem::zeroed();
    let mut i: ULONG;
    let mut Length: ULONG = 0;
    let mut FontTitleW: _UNICODE_STRING = core::mem::zeroed();
    let mut FileNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut InfoSize: SIZE_T;
    let mut InfoBuffer: LPBYTE;
    let mut pInfo: PKEY_VALUE_FULL_INFORMATION;
    let mut pchPath: LPWSTR;
    let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut nFontCount: INT = 0;
    let mut dwFlags: DWORD;

    /* InitializeObjectAttributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(g_FontRegPath);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&mut KeyHandle, KEY_READ, core::ptr::addr_of_mut!(ObjectAttributes) as _ );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    Status = ZwQueryKey(
        KeyHandle,
        KeyFullInformation,
        core::ptr::addr_of_mut!(KeyFullInfo) as *mut _,
        core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG,
        &mut Length,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return 0;
    }

    InfoSize = ((MAX_PATH + 256) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T;
    InfoBuffer = ExAllocatePoolWithTag(PagedPool, InfoSize as _ , TAG_FONT) as LPBYTE;
    if InfoBuffer.is_null() {
        ZwClose(KeyHandle);
        return 0;
    }

    i = 0;
    while i < KeyFullInfo.Values {
        Status = ZwEnumerateValueKey(
            KeyHandle,
            i,
            KeyValueFullInformation,
            InfoBuffer as *mut _,
            InfoSize as ULONG,
            &mut Length,
        );
        if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            ExFreePoolWithTag(InfoBuffer as *mut _, TAG_FONT);
            InfoSize *= 2;
            InfoBuffer = ExAllocatePoolWithTag(PagedPool, InfoSize as _ , TAG_FONT) as LPBYTE;
            if InfoBuffer.is_null() {
                break;
            }
            Status = ZwEnumerateValueKey(
                KeyHandle,
                i,
                KeyValueFullInformation,
                InfoBuffer as *mut _,
                InfoSize as ULONG,
                &mut Length,
            );
        }
        if !NT_SUCCESS(Status) {
            break;
        }

        pInfo = InfoBuffer as PKEY_VALUE_FULL_INFORMATION;
        Length = (*pInfo).NameLength / core::mem::size_of::<WCHAR>() as ULONG;
        *(*pInfo).Name.as_mut_ptr().add(Length as usize) = 0;
        if RtlCreateUnicodeString(core::ptr::addr_of_mut!(FontTitleW) as _ , (*pInfo).Name.as_ptr()) == 0 {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            let _ = Status;
            break;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(FontTitleW) as _ ,
            KeyValueFullInformation,
            InfoBuffer as *mut _,
            InfoSize as ULONG,
            &mut Length,
        );
        if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            ExFreePoolWithTag(InfoBuffer as *mut _, TAG_FONT);
            InfoSize *= 2;
            InfoBuffer = ExAllocatePoolWithTag(PagedPool, InfoSize as _ , TAG_FONT) as LPBYTE;
            if InfoBuffer.is_null() {
                break;
            }
            Status = ZwQueryValueKey(
                KeyHandle,
                core::ptr::addr_of_mut!(FontTitleW) as _ ,
                KeyValueFullInformation,
                InfoBuffer as *mut _,
                InfoSize as ULONG,
                &mut Length,
            );
        }
        pInfo = InfoBuffer as PKEY_VALUE_FULL_INFORMATION;
        if !NT_SUCCESS(Status) || (*pInfo).DataLength == 0 {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FontTitleW) as _ );
            break;
        }

        pchPath = ((pInfo as PUCHAR).add((*pInfo).DataOffset as usize)) as LPWSTR;
        Length = (*pInfo).DataLength / core::mem::size_of::<WCHAR>() as ULONG;
        *pchPath.add(Length as usize) = 0;

        if PathIsRelativeW(pchPath) != 0 {
            dwFlags = 0;
            Status = w32r_RtlStringCbPrintfW_2(
                szPath.as_mut_ptr(),
                core::mem::size_of_val(&szPath),
                b"\\\0S\0y\0s\0t\0e\0m\0R\0o\0o\0t\0\\\0F\0o\0n\0t\0s\0\\\0%\0s\0\0\0".as_ptr() as *const WCHAR,
                pchPath,
            );
        } else {
            dwFlags = (AFRX_ALTERNATIVE_PATH | AFRX_DOS_DEVICE_PATH) as DWORD;
            Status = w32r_RtlStringCbCopyW(
                szPath.as_mut_ptr(),
                core::mem::size_of_val(&szPath),
                pchPath,
            );
        }

        if NT_SUCCESS(Status) {
            RtlCreateUnicodeString(core::ptr::addr_of_mut!(FileNameW) as _ , szPath.as_ptr());
            nFontCount += IntGdiAddFontResourceEx(&FileNameW, 1, 0, dwFlags);
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(FileNameW) as _ );
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(FontTitleW) as _ );

        i += 1;
    }

    ZwClose(KeyHandle);

    if !InfoBuffer.is_null() {
        ExFreePoolWithTag(InfoBuffer as *mut _, TAG_FONT);
    }

    if KeyFullInfo.Values != 0 && nFontCount != 0 {
        1
    } else {
        0
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiAddFontMemResource(
    Buffer: PVOID,
    dwSize: DWORD,
    pNumAdded: PDWORD,
) -> HANDLE {
    IntGdiAddFontMemResource_impl(Buffer, dwSize, pNumAdded)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiAddFontMemResource(
    Buffer: PVOID,
    dwSize: DWORD,
    pNumAdded: PDWORD,
) -> HANDLE {
    IntGdiAddFontMemResource_impl(Buffer, dwSize, pNumAdded)
}

unsafe fn IntGdiAddFontMemResource_impl(
    Buffer: PVOID,
    dwSize: DWORD,
    pNumAdded: PDWORD,
) -> HANDLE {
    let mut Ret: HANDLE = core::ptr::null_mut();
    let mut LoadFont: GDI_LOAD_FONT = core::mem::zeroed();
    let EntryCollection: PFONT_ENTRY_COLL_MEM;
    let FaceCount: INT;

    let BufferCopy: PVOID = ExAllocatePoolWithTag(PagedPool, dwSize as SIZE_T as _ , TAG_FONT);
    if BufferCopy.is_null() {
        *pNumAdded = 0;
        return core::ptr::null_mut();
    }
    RtlCopyMemory(BufferCopy, Buffer, dwSize as SIZE_T as _ );

    RtlZeroMemory(
        core::ptr::addr_of_mut!(LoadFont) as PVOID,
        core::mem::size_of::<GDI_LOAD_FONT>() as SIZE_T as _ ,
    );
    LoadFont.Memory = SharedMem_Create(BufferCopy as PBYTE, dwSize, 0);
    LoadFont.Characteristics = FR_PRIVATE | FR_NOT_ENUM;
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ ,
        core::ptr::null_mut(),
    );
    FaceCount = IntGdiLoadFontByIndexFromMemory(core::ptr::addr_of_mut!(LoadFont), -1);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(LoadFont.RegValueName) as _ );

    IntLockFreeType();
    SharedMem_Release(LoadFont.Memory);
    IntUnLockFreeType();

    if FaceCount > 0 {
        EntryCollection = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<FONT_ENTRY_COLL_MEM>() as SIZE_T as _ ,
            TAG_FONT,
        ) as PFONT_ENTRY_COLL_MEM;
        if !EntryCollection.is_null() {
            let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;
            (*EntryCollection).Entry = LoadFont.PrivateEntry;
            IntLockFreeType();
            IntLockProcessPrivateFonts(Win32Process as _ );
            (*Win32Process).PrivateMemFontHandleCount += 1;
            (*EntryCollection).Handle = ULongToHandle((*Win32Process).PrivateMemFontHandleCount);
            InsertTailList(
                core::ptr::addr_of_mut!((*Win32Process).PrivateMemFontListHead),
                core::ptr::addr_of_mut!((*EntryCollection).ListEntry),
            );
            IntUnLockProcessPrivateFonts(Win32Process as _ );
            IntUnLockFreeType();
            Ret = (*EntryCollection).Handle;
        }
    }
    *pNumAdded = FaceCount as DWORD;

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiCleanupMemEntry(Head: PFONT_ENTRY_MEM) {
    while IsListEmpty(core::ptr::addr_of_mut!((*Head).ListEntry)) == 0 {
        let Entry = RemoveHeadList(core::ptr::addr_of_mut!((*Head).ListEntry));
        let FontEntry = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONT_ENTRY_MEM, ListEntry) as isize))
            as PFONT_ENTRY_MEM;

        CleanupFontEntry((*FontEntry).Entry);
        ExFreePoolWithTag(FontEntry as *mut core::ffi::c_void, TAG_FONT);
    }

    CleanupFontEntry((*Head).Entry);
    ExFreePoolWithTag(Head as *mut core::ffi::c_void, TAG_FONT);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiCleanupMemEntry(Head: PFONT_ENTRY_MEM) {
    while IsListEmpty(core::ptr::addr_of_mut!((*Head).ListEntry)) == 0 {
        let Entry = RemoveHeadList(core::ptr::addr_of_mut!((*Head).ListEntry));
        let FontEntry = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONT_ENTRY_MEM, ListEntry) as isize))
            as PFONT_ENTRY_MEM;

        CleanupFontEntry((*FontEntry).Entry);
        ExFreePoolWithTag(FontEntry as *mut core::ffi::c_void, TAG_FONT);
    }

    CleanupFontEntry((*Head).Entry);
    ExFreePoolWithTag(Head as *mut core::ffi::c_void, TAG_FONT);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UnlinkFontMemCollection(Collection: PFONT_ENTRY_COLL_MEM) {
    UnlinkFontMemCollectionImpl(Collection)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UnlinkFontMemCollection(Collection: PFONT_ENTRY_COLL_MEM) {
    UnlinkFontMemCollectionImpl(Collection)
}

unsafe fn UnlinkFontMemCollectionImpl(Collection: PFONT_ENTRY_COLL_MEM) {
    let mut FontMemEntry: PFONT_ENTRY_MEM = (*Collection).Entry;
    let mut ListEntry: PLIST_ENTRY;
    RemoveEntryList(core::ptr::addr_of_mut!((*Collection).ListEntry));

    loop {
        RemoveEntryList(core::ptr::addr_of_mut!((*(*FontMemEntry).Entry).ListEntry));

        ListEntry = (*FontMemEntry).ListEntry.Flink;
        FontMemEntry = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONT_ENTRY_MEM, ListEntry) as isize))
            as PFONT_ENTRY_MEM;

        if FontMemEntry == (*Collection).Entry {
            break;
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiRemoveFontMemResource(hMMFont: HANDLE) -> BOOL {
    IntGdiRemoveFontMemResourceImpl(hMMFont)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiRemoveFontMemResource(hMMFont: HANDLE) -> BOOL {
    IntGdiRemoveFontMemResourceImpl(hMMFont)
}

unsafe fn IntGdiRemoveFontMemResourceImpl(hMMFont: HANDLE) -> BOOL {
    let mut Entry: PLIST_ENTRY;
    let mut CurrentEntry: PFONT_ENTRY_COLL_MEM;
    let mut EntryCollection: PFONT_ENTRY_COLL_MEM = core::ptr::null_mut();
    let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;

    IntLockFreeType();
    IntLockProcessPrivateFonts(Win32Process as _ );

    Entry = (*Win32Process).PrivateMemFontListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*Win32Process).PrivateMemFontListHead) {
        CurrentEntry = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONT_ENTRY_COLL_MEM, ListEntry) as isize))
            as PFONT_ENTRY_COLL_MEM;

        if (*CurrentEntry).Handle == hMMFont {
            EntryCollection = CurrentEntry;
            UnlinkFontMemCollection(CurrentEntry);
            break;
        }

        Entry = (*Entry).Flink;
    }

    IntUnLockProcessPrivateFonts(Win32Process as _ );
    IntUnLockFreeType();

    if !EntryCollection.is_null() {
        IntGdiCleanupMemEntry((*EntryCollection).Entry);
        ExFreePoolWithTag(EntryCollection as *mut core::ffi::c_void, TAG_FONT);
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiCleanupPrivateFontsForProcess() {
    IntGdiCleanupPrivateFontsForProcessImpl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiCleanupPrivateFontsForProcess() {
    IntGdiCleanupPrivateFontsForProcessImpl()
}

unsafe fn IntGdiCleanupPrivateFontsForProcessImpl() {
    let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;
    let mut Entry: PLIST_ENTRY;
    let mut EntryCollection: PFONT_ENTRY_COLL_MEM;

    if g_FreeTypeLock.is_null() {
        return;
    }

    loop {
        Entry = core::ptr::null_mut();
        EntryCollection = core::ptr::null_mut();

        IntLockFreeType();
        IntLockProcessPrivateFonts(Win32Process as _ );
        if IsListEmpty(core::ptr::addr_of_mut!((*Win32Process).PrivateMemFontListHead)) == 0 {
            Entry = (*Win32Process).PrivateMemFontListHead.Flink;
            EntryCollection = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(_FONT_ENTRY_COLL_MEM, ListEntry) as isize))
                as PFONT_ENTRY_COLL_MEM;
            UnlinkFontMemCollection(EntryCollection);
        }
        IntUnLockProcessPrivateFonts(Win32Process as _ );
        IntUnLockFreeType();

        if !EntryCollection.is_null() {
            IntGdiCleanupMemEntry((*EntryCollection).Entry);
            ExFreePoolWithTag(EntryCollection as *mut core::ffi::c_void, TAG_FONT);
        } else {
            Entry = core::ptr::null_mut();
            IntLockFreeType();
            IntLockProcessPrivateFonts(Win32Process as _ );
            if IsListEmpty(core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead)) == 0 {
                Entry = RemoveHeadList(core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead));
            }
            IntUnLockProcessPrivateFonts(Win32Process as _ );
            IntUnLockFreeType();

            if !Entry.is_null() {
                let FontEntry = (Entry as *mut u8)
                    .offset(-(core::mem::offset_of!(_FONT_ENTRY, ListEntry) as isize))
                    as PFONT_ENTRY;
                CleanupFontEntry(FontEntry);
            }
        }

        if Entry.is_null() {
            break;
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntIsFontRenderingEnabled() -> BOOL {
    ((*gpsi_per_user()).BitsPixel > 8 && g_RenderingEnabled != 0) as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntIsFontRenderingEnabled() -> BOOL {
    ((*gpsi_per_user()).BitsPixel > 8 && g_RenderingEnabled != 0) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEnableFontRendering(Enable: BOOL) {
    g_RenderingEnabled = Enable;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEnableFontRendering(Enable: BOOL) {
    g_RenderingEnabled = Enable;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetFontRenderMode(logfont: *mut LOGFONTW) -> FT_Render_Mode {
    IntGetFontRenderModeImpl(logfont)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetFontRenderMode(logfont: *mut LOGFONTW) -> FT_Render_Mode {
    IntGetFontRenderModeImpl(logfont)
}

unsafe fn IntGetFontRenderModeImpl(logfont: *mut LOGFONTW) -> FT_Render_Mode {
    match (*logfont).lfQuality as u32 {
        ANTIALIASED_QUALITY => {}
        NONANTIALIASED_QUALITY => return FT_RENDER_MODE_MONO,
        DRAFT_QUALITY => return FT_RENDER_MODE_LIGHT,
        CLEARTYPE_QUALITY => {
            if gspv.bFontSmoothing == 0 {
            } else if gspv.uiFontSmoothingType == 0 {
            } else {
                return FT_RENDER_MODE_LCD;
            }
        }
        _ => {}
    }
    FT_RENDER_MODE_NORMAL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn TextIntCreateFontIndirect(lf: LPLOGFONTW, NewFont: *mut HFONT) -> NTSTATUS {
    TextIntCreateFontIndirect_impl(lf, NewFont)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn TextIntCreateFontIndirect(lf: LPLOGFONTW, NewFont: *mut HFONT) -> NTSTATUS {
    TextIntCreateFontIndirect_impl(lf, NewFont)
}

unsafe fn TextIntCreateFontIndirect_impl(lf: LPLOGFONTW, NewFont: *mut HFONT) -> NTSTATUS {
    let plfont: PLFONT = LFONT_AllocFontWithHandle();
    if plfont.is_null() {
        return STATUS_NO_MEMORY;
    }

    ExInitializePushLock(core::ptr::addr_of_mut!((*plfont).lock) as _ );
    *NewFont = (*plfont).BaseObject.hHmgr as HFONT;
    let plf: *mut LOGFONTW = core::ptr::addr_of_mut!((*plfont).logfont.elfEnumLogfontEx.elfLogFont);
    RtlCopyMemory(plf as *mut _, lf as *const _, core::mem::size_of::<LOGFONTW>() as _);
    if (*lf).lfEscapement != (*lf).lfOrientation {
        (*plf).lfOrientation = (*plf).lfEscapement;
    }
    LFONT_UnlockFont(plfont);

    STATUS_SUCCESS
}

unsafe fn IntTranslateCharsetInfo(Src: PDWORD, Cs: LPCHARSETINFO, Flags: DWORD) -> BOOLEAN {
    let mut Index: i32 = 0;

    match Flags {
        TCI_SRCFONTSIG => {
            while Index < MAXTCIINDEX as i32 && 0 == (*Src >> Index & 0x0001) {
                Index += 1;
            }
        }
        TCI_SRCCODEPAGE => {
            while Index < MAXTCIINDEX as i32 && *Src != g_FontTci[Index as usize].ciACP {
                Index += 1;
            }
        }
        TCI_SRCCHARSET => {
            while Index < MAXTCIINDEX as i32 && *Src != g_FontTci[Index as usize].ciCharset as DWORD {
                Index += 1;
            }
        }
        TCI_SRCLOCALE => {
            return 0u8;
        }
        _ => {
            return 0u8;
        }
    }

    if Index >= MAXTCIINDEX as i32 || DEFAULT_CHARSET == g_FontTci[Index as usize].ciCharset as u32 {
        return 0u8;
    }

    RtlCopyMemory(
        Cs as *mut core::ffi::c_void,
        core::ptr::addr_of!(g_FontTci[Index as usize]) as *const core::ffi::c_void,
        core::mem::size_of::<CHARSETINFO>() as u64 as _ ,
    );

    1u8
}

unsafe fn face_has_symbol_charmap(ft_face: FT_Face) -> BOOL {
    let mut i: i32 = 0;
    while i < (*ft_face).num_charmaps {
        let charmap = *(*ft_face).charmaps.offset(i as isize);
        if (*charmap).platform_id as u32 == TT_PLATFORM_MICROSOFT as u32
            && (*charmap).encoding == FT_ENCODING_MS_SYMBOL
        {
            return 1;
        }
        i += 1;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FillTM(
    TM: *mut TEXTMETRICW,
    FontGDI: PFONTGDI,
    pOS2: *mut TT_OS2,
    pHori: *mut TT_HoriHeader,
    pFNT: *mut FT_WinFNT_HeaderRec,
) {
    FillTM_impl(TM, FontGDI, pOS2, pHori, pFNT)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FillTM(
    TM: *mut TEXTMETRICW,
    FontGDI: PFONTGDI,
    pOS2: *mut TT_OS2,
    pHori: *mut TT_HoriHeader,
    pFNT: *mut FT_WinFNT_HeaderRec,
) {
    FillTM_impl(TM, FontGDI, pOS2, pHori, pFNT)
}

unsafe fn FillTM_impl(
    TM: *mut TEXTMETRICW,
    FontGDI: PFONTGDI,
    pOS2: *mut TT_OS2,
    pHori: *mut TT_HoriHeader,
    pFNT: *mut FT_WinFNT_HeaderRec,
) {
    let XScale: FT_Fixed;
    let YScale: FT_Fixed;
    let Ascent: i32;
    let Descent: i32;
    let Face: FT_Face = (*(*FontGDI).SharedFace).Face;

    XScale = (*(*Face).size).metrics.x_scale;
    YScale = (*(*Face).size).metrics.y_scale;

    if !pFNT.is_null() {
        (*TM).tmHeight = (*pFNT).pixel_height as i32;
        (*TM).tmAscent = (*pFNT).ascent as i32;
        (*TM).tmDescent = (*TM).tmHeight - (*TM).tmAscent;
        (*TM).tmInternalLeading = (*pFNT).internal_leading as i32;
        (*TM).tmExternalLeading = (*pFNT).external_leading as i32;
        (*TM).tmAveCharWidth = (*pFNT).avg_width as i32;
        (*TM).tmMaxCharWidth = (*pFNT).max_width as i32;
        (*TM).tmOverhang = 0;
        (*TM).tmDigitizedAspectX = (*pFNT).horizontal_resolution as i32;
        (*TM).tmDigitizedAspectY = (*pFNT).vertical_resolution as i32;
        (*TM).tmFirstChar = (*pFNT).first_char as u16;
        (*TM).tmLastChar = (*pFNT).last_char as u16;
        (*TM).tmDefaultChar = ((*pFNT).default_char as i32 + (*pFNT).first_char as i32) as u16;
        (*TM).tmBreakChar = ((*pFNT).break_char as i32 + (*pFNT).first_char as i32) as u16;
        (*TM).tmPitchAndFamily = (*pFNT).pitch_and_family;
        (*TM).tmWeight = (*FontGDI).RequestWeight as i32;
        (*TM).tmItalic = (*FontGDI).RequestItalic;
        (*TM).tmUnderlined = (*FontGDI).RequestUnderline;
        (*TM).tmStruckOut = (*FontGDI).RequestStrikeOut;
        (*TM).tmCharSet = (*FontGDI).CharSet;
        return;
    }

    if pOS2.is_null() {
        return;
    }

    if (*pOS2).usWinAscent as FT_Short as i32 + (*pOS2).usWinDescent as FT_Short as i32 == 0 {
        Ascent = (*pHori).Ascender as i32;
        Descent = -((*pHori).Descender as i32);
    } else {
        Ascent = (*pOS2).usWinAscent as FT_Short as i32;
        Descent = (*pOS2).usWinDescent as FT_Short as i32;
    }

    (*TM).tmAscent = (*FontGDI).tmAscent;
    (*TM).tmDescent = (*FontGDI).tmDescent;
    (*TM).tmHeight = (*TM).tmAscent + (*TM).tmDescent;
    (*TM).tmInternalLeading = (*FontGDI).tmInternalLeading;

    let el = (FT_MulFix(
        (*pHori).Line_Gap as FT_Fixed
            - ((Ascent + Descent) as FT_Fixed - ((*pHori).Ascender as FT_Fixed - (*pHori).Descender as FT_Fixed)),
        YScale,
    ) + 32) >> 6;
    (*TM).tmExternalLeading = if 0 > el { 0 } else { el };

    if (*FontGDI).lfWidth != 0 {
        (*TM).tmAveCharWidth = (*FontGDI).lfWidth;
    } else {
        (*TM).tmAveCharWidth = (FT_MulFix((*pOS2).xAvgCharWidth as FT_Fixed, XScale) + 32) >> 6;
    }

    if (*TM).tmAveCharWidth == 0 {
        (*TM).tmAveCharWidth = 1;
    }

    (*TM).tmMaxCharWidth = (FT_MulFix((*Face).max_advance_width as FT_Fixed, XScale) + 32) >> 6;

    if (*FontGDI).OriginalWeight != FW_DONTCARE as i32
        && (*FontGDI).OriginalWeight > (*FontGDI).RequestWeight
    {
        (*TM).tmWeight = (*FontGDI).OriginalWeight;
    } else {
        (*TM).tmWeight = (*FontGDI).RequestWeight;
    }

    (*TM).tmOverhang = 0;
    (*TM).tmDigitizedAspectX = 96;
    (*TM).tmDigitizedAspectY = 96;
    if face_has_symbol_charmap(Face) != 0
        || ((*pOS2).usFirstCharIndex >= 0xf000 && (*pOS2).usFirstCharIndex < 0xf100)
    {
        let mut cpOEM: u16 = 0;
        let mut cpAnsi: u16 = 0;

        EngGetCurrentCodePage(&mut cpOEM, &mut cpAnsi);
        (*TM).tmFirstChar = 0;
        match cpAnsi {
            1257 => {
                (*TM).tmLastChar = 0xf8fd;
            }
            _ => {
                (*TM).tmLastChar = 0xf0ff;
            }
        }
        (*TM).tmBreakChar = 0x20;
        (*TM).tmDefaultChar = 0x1f;
    } else {
        (*TM).tmFirstChar = (*pOS2).usFirstCharIndex;
        (*TM).tmLastChar = (*pOS2).usLastCharIndex;

        if (*pOS2).usFirstCharIndex <= 1 {
            (*TM).tmBreakChar = (*pOS2).usFirstCharIndex + 2;
        } else if (*pOS2).usFirstCharIndex > 0xff {
            (*TM).tmBreakChar = 0x20;
        } else {
            (*TM).tmBreakChar = (*pOS2).usFirstCharIndex;
        }
        (*TM).tmDefaultChar = (*TM).tmBreakChar - 1;
    }

    if (*FontGDI).OriginalItalic != 0 || (*FontGDI).RequestItalic != 0 {
        (*TM).tmItalic = 0xFF;
    } else {
        (*TM).tmItalic = 0;
    }
    (*TM).tmUnderlined = if (*FontGDI).RequestUnderline != 0 { 0xFF } else { 0 };
    (*TM).tmStruckOut = if (*FontGDI).RequestStrikeOut != 0 { 0xFF } else { 0 };

    if FT_IS_FIXED_WIDTH(Face) == 0 {
        match (*pOS2).panose[PAN_PROPORTION_INDEX as usize] as u32 {
            x if x == PAN_PROP_MONOSPACED as u32 => {
                (*TM).tmPitchAndFamily = 0;
            }
            _ => {
                (*TM).tmPitchAndFamily = TMPF_FIXED_PITCH as u8;
            }
        }
    } else {
        (*TM).tmPitchAndFamily = 0;
    }

    match (*pOS2).panose[PAN_FAMILYTYPE_INDEX as usize] as u32 {
        x if x == PAN_FAMILY_SCRIPT as u32 => {
            (*TM).tmPitchAndFamily |= FF_SCRIPT as u8;
        }
        x if x == PAN_FAMILY_DECORATIVE as u32 => {
            (*TM).tmPitchAndFamily |= FF_DECORATIVE as u8;
        }
        x if x == PAN_ANY as u32
            || x == PAN_NO_FIT as u32
            || x == PAN_FAMILY_TEXT_DISPLAY as u32
            || x == PAN_FAMILY_PICTORIAL as u32 =>
        {
            if (*TM).tmPitchAndFamily == 0 {
                (*TM).tmPitchAndFamily = FF_MODERN as u8;
            } else {
                match (*pOS2).panose[PAN_SERIFSTYLE_INDEX as usize] as u32 {
                    x if x == PAN_SERIF_COVE as u32
                        || x == PAN_SERIF_OBTUSE_COVE as u32
                        || x == PAN_SERIF_SQUARE_COVE as u32
                        || x == PAN_SERIF_OBTUSE_SQUARE_COVE as u32
                        || x == PAN_SERIF_SQUARE as u32
                        || x == PAN_SERIF_THIN as u32
                        || x == PAN_SERIF_BONE as u32
                        || x == PAN_SERIF_EXAGGERATED as u32
                        || x == PAN_SERIF_TRIANGLE as u32 =>
                    {
                        (*TM).tmPitchAndFamily |= FF_ROMAN as u8;
                    }
                    x if x == PAN_SERIF_NORMAL_SANS as u32
                        || x == PAN_SERIF_OBTUSE_SANS as u32
                        || x == PAN_SERIF_PERP_SANS as u32
                        || x == PAN_SERIF_FLARED as u32
                        || x == PAN_SERIF_ROUNDED as u32 =>
                    {
                        (*TM).tmPitchAndFamily |= FF_SWISS as u8;
                    }
                    _ => {
                        (*TM).tmPitchAndFamily |= FF_DONTCARE as u8;
                    }
                }
            }
        }
        _ => {
            (*TM).tmPitchAndFamily |= FF_DONTCARE as u8;
        }
    }

    if FT_IS_SCALABLE(Face) != 0 {
        (*TM).tmPitchAndFamily |= TMPF_VECTOR as u8;
    }
    if FT_IS_SFNT(Face) != 0 {
        (*TM).tmPitchAndFamily |= TMPF_TRUETYPE as u8;
    }

    (*TM).tmCharSet = (*FontGDI).CharSet;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntInitFontNames(Names: *mut FONT_NAMES, SharedFace: PSHARED_FACE) {
    IntInitFontNames_impl(Names, SharedFace)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntInitFontNames(Names: *mut FONT_NAMES, SharedFace: PSHARED_FACE) {
    IntInitFontNames_impl(Names, SharedFace)
}

#[inline]
unsafe fn IntInitFontNames_impl(Names: *mut FONT_NAMES, SharedFace: PSHARED_FACE) {
    let OtmSize: ULONG;

    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Names).FamilyNameW) as _ , core::ptr::null_mut());
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Names).FaceNameW) as _ , core::ptr::null_mut());
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Names).StyleNameW) as _ , core::ptr::null_mut());
    RtlInitUnicodeString(core::ptr::addr_of_mut!((*Names).FullNameW) as _ , core::ptr::null_mut());

    IntGetFontLocalizedName(core::ptr::addr_of_mut!((*Names).FamilyNameW), SharedFace, TT_NAME_ID_FONT_FAMILY as FT_UShort, gusLanguageID);
    IntGetFontLocalizedName(core::ptr::addr_of_mut!((*Names).FaceNameW), SharedFace, TT_NAME_ID_FULL_NAME as FT_UShort, gusLanguageID);
    IntGetFontLocalizedName(core::ptr::addr_of_mut!((*Names).StyleNameW), SharedFace, TT_NAME_ID_FONT_SUBFAMILY as FT_UShort, gusLanguageID);
    IntGetFontLocalizedName(core::ptr::addr_of_mut!((*Names).FullNameW), SharedFace, TT_NAME_ID_UNIQUE_ID as FT_UShort, gusLanguageID);

    OtmSize = core::mem::size_of::<OUTLINETEXTMETRICW>() as ULONG +
              (*Names).FamilyNameW.Length as ULONG + core::mem::size_of::<u16>() as ULONG +
              (*Names).FaceNameW.Length as ULONG + core::mem::size_of::<u16>() as ULONG +
              (*Names).StyleNameW.Length as ULONG + core::mem::size_of::<u16>() as ULONG +
              (*Names).FullNameW.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
    (*Names).OtmSize = OtmSize;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntStoreName(pName: *const _UNICODE_STRING, pb: PBYTE) -> SIZE_T {
    RtlCopyMemory(pb as *mut _, (*pName).Buffer as *const _, (*pName).Length as usize);
    *(pb.add((*pName).Length as usize) as *mut WCHAR) = UNICODE_NULL as WCHAR;
    (*pName).Length as SIZE_T + core::mem::size_of::<WCHAR>() as SIZE_T
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntStoreName(pName: *const _UNICODE_STRING, pb: PBYTE) -> SIZE_T {
    RtlCopyMemory(pb as *mut _, (*pName).Buffer as *const _, (*pName).Length as usize);
    *(pb.add((*pName).Length as usize) as *mut WCHAR) = UNICODE_NULL as WCHAR;
    (*pName).Length as SIZE_T + core::mem::size_of::<WCHAR>() as SIZE_T
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntStoreFontNames(Names: *const FONT_NAMES, Otm: *mut OUTLINETEXTMETRICW) -> PBYTE {
    let mut pb = (Otm as PBYTE).add(core::mem::size_of::<OUTLINETEXTMETRICW>());

    (*Otm).otmpFamilyName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FamilyNameW), pb) as usize);

    (*Otm).otmpFaceName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FaceNameW), pb) as usize);

    (*Otm).otmpStyleName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).StyleNameW), pb) as usize);

    (*Otm).otmpFullName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FullNameW), pb) as usize);

    pb
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntStoreFontNames(Names: *const FONT_NAMES, Otm: *mut OUTLINETEXTMETRICW) -> PBYTE {
    let mut pb = (Otm as PBYTE).add(core::mem::size_of::<OUTLINETEXTMETRICW>());

    (*Otm).otmpFamilyName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FamilyNameW), pb) as usize);

    (*Otm).otmpFaceName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FaceNameW), pb) as usize);

    (*Otm).otmpStyleName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).StyleNameW), pb) as usize);

    (*Otm).otmpFullName = pb.offset_from(Otm as PBYTE) as LPSTR;
    pb = pb.add(IntStoreName(core::ptr::addr_of!((*Names).FullNameW), pb) as usize);

    pb
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFreeFontNames(Names: *mut FONT_NAMES) {
    RtlFreeUnicodeString((core::ptr::addr_of_mut!((*Names).FamilyNameW) as _));
    RtlFreeUnicodeString((core::ptr::addr_of_mut!((*Names).FaceNameW) as _));
    RtlFreeUnicodeString((core::ptr::addr_of_mut!((*Names).StyleNameW) as _));
    RtlFreeUnicodeString((core::ptr::addr_of_mut!((*Names).FullNameW) as _));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFreeFontNames(Names: *mut FONT_NAMES) {
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Names).FamilyNameW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Names).FaceNameW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Names).StyleNameW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*Names).FullNameW));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetOutlineTextMetrics(
    FontGDI: PFONTGDI,
    Size: UINT,
    Otm: *mut OUTLINETEXTMETRICW,
    bLocked: BOOL,
) -> INT {
    IntGetOutlineTextMetrics_impl(FontGDI, Size, Otm, bLocked)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetOutlineTextMetrics(
    FontGDI: PFONTGDI,
    Size: UINT,
    Otm: *mut OUTLINETEXTMETRICW,
    bLocked: BOOL,
) -> INT {
    IntGetOutlineTextMetrics_impl(FontGDI, Size, Otm, bLocked)
}

#[inline]
unsafe fn IntGetOutlineTextMetrics_impl(
    FontGDI: PFONTGDI,
    Size: UINT,
    Otm: *mut OUTLINETEXTMETRICW,
    bLocked: BOOL,
) -> INT {
    let pOS2: *mut TT_OS2;
    let pHori: *mut TT_HoriHeader;
    let pPost: *mut TT_Postscript;
    let XScale: FT_Fixed;
    let YScale: FT_Fixed;
    let mut WinFNT: FT_WinFNT_HeaderRec = core::mem::zeroed();
    let Error: FT_Error;
    let pb: *mut BYTE;
    let mut FontNames: FONT_NAMES = core::mem::zeroed();
    let SharedFace: PSHARED_FACE = (*FontGDI).SharedFace;
    let Cache: PSHARED_FACE_CACHE;
    let Face: FT_Face = (*SharedFace).Face;

    if PRIMARYLANGID(gusLanguageID as _ ) as i32 == LANG_ENGLISH as i32 {
        Cache = core::ptr::addr_of_mut!((*SharedFace).EnglishUS);
    } else {
        Cache = core::ptr::addr_of_mut!((*SharedFace).UserLanguage);
    }

    if Size == 0 && (*Cache).OutlineRequiredSize > 0 {
        return (*Cache).OutlineRequiredSize as INT;
    }

    if bLocked == 0 {
        IntLockFreeType();
    }

    IntInitFontNames(core::ptr::addr_of_mut!(FontNames), SharedFace);
    (*Cache).OutlineRequiredSize = FontNames.OtmSize;

    if Size == 0 {
        IntFreeFontNames(core::ptr::addr_of_mut!(FontNames));
        if bLocked == 0 {
            IntUnLockFreeType();
        }
        return (*Cache).OutlineRequiredSize as INT;
    }

    if Size < (*Cache).OutlineRequiredSize {
        IntFreeFontNames(core::ptr::addr_of_mut!(FontNames));
        if bLocked == 0 {
            IntUnLockFreeType();
        }
        return 0;
    }

    XScale = (*(*Face).size).metrics.x_scale;
    YScale = (*(*Face).size).metrics.y_scale;

    pOS2 = FT_Get_Sfnt_Table(Face, FT_SFNT_OS2) as *mut TT_OS2;
    pHori = FT_Get_Sfnt_Table(Face, FT_SFNT_HHEA) as *mut TT_HoriHeader;
    pPost = FT_Get_Sfnt_Table(Face, FT_SFNT_POST) as *mut TT_Postscript;
    Error = FT_Get_WinFNT_Header(Face, core::ptr::addr_of_mut!(WinFNT));

    if pOS2.is_null() && Error != 0 {
        if bLocked == 0 {
            IntUnLockFreeType();
        }
        IntFreeFontNames(core::ptr::addr_of_mut!(FontNames));
        return 0;
    }

    if pHori.is_null() && Error != 0 {
        if bLocked == 0 {
            IntUnLockFreeType();
        }
        IntFreeFontNames(core::ptr::addr_of_mut!(FontNames));
        return 0;
    }

    (*Otm).otmSize = (*Cache).OutlineRequiredSize;

    FillTM(
        core::ptr::addr_of_mut!((*Otm).otmTextMetrics),
        FontGDI,
        pOS2,
        pHori,
        if Error != 0 { core::ptr::null_mut() } else { core::ptr::addr_of_mut!(WinFNT) },
    );

    'skip_os2: {
        if pOS2.is_null() {
            break 'skip_os2;
        }

        (*Otm).otmFiller = 0;
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Otm).otmPanoseNumber) as PVOID,
            core::ptr::addr_of!((*pOS2).panose) as PVOID,
            PANOSE_COUNT as usize,
        );
        (*Otm).otmfsSelection = (*pOS2).fsSelection as UINT;
        (*Otm).otmfsType = (*pOS2).fsType as UINT;
        (*Otm).otmsCharSlopeRise = (*pHori).caret_Slope_Rise as INT;
        (*Otm).otmsCharSlopeRun = (*pHori).caret_Slope_Run as INT;
        (*Otm).otmItalicAngle = 0;
        (*Otm).otmEMSquare = (*Face).units_per_EM as UINT;

        (*Otm).otmAscent = SCALE_Y((*pOS2).sTypoAscender as FT_Fixed, YScale) as INT;
        (*Otm).otmDescent = SCALE_Y((*pOS2).sTypoDescender as FT_Fixed, YScale) as INT;
        (*Otm).otmLineGap = SCALE_Y((*pOS2).sTypoLineGap as FT_Fixed, YScale) as UINT;
        (*Otm).otmsCapEmHeight = SCALE_Y((*pOS2).sCapHeight as FT_Fixed, YScale) as UINT;
        (*Otm).otmsXHeight = SCALE_Y((*pOS2).sxHeight as FT_Fixed, YScale) as UINT;
        (*Otm).otmrcFontBox.left = SCALE_X((*Face).bbox.xMin as FT_Fixed, XScale) as LONG;
        (*Otm).otmrcFontBox.right = SCALE_X((*Face).bbox.xMax as FT_Fixed, XScale) as LONG;
        (*Otm).otmrcFontBox.top = SCALE_Y((*Face).bbox.yMax as FT_Fixed, YScale) as LONG;
        (*Otm).otmrcFontBox.bottom = SCALE_Y((*Face).bbox.yMin as FT_Fixed, YScale) as LONG;
        (*Otm).otmMacAscent = (*Otm).otmTextMetrics.tmAscent;
        (*Otm).otmMacDescent = -(*Otm).otmTextMetrics.tmDescent;
        (*Otm).otmMacLineGap = (*Otm).otmLineGap;
        (*Otm).otmusMinimumPPEM = 0;
        (*Otm).otmptSubscriptSize.x = SCALE_X((*pOS2).ySubscriptXSize as FT_Fixed, XScale) as LONG;
        (*Otm).otmptSubscriptSize.y = SCALE_Y((*pOS2).ySubscriptYSize as FT_Fixed, YScale) as LONG;
        (*Otm).otmptSubscriptOffset.x = SCALE_X((*pOS2).ySubscriptXOffset as FT_Fixed, XScale) as LONG;
        (*Otm).otmptSubscriptOffset.y = SCALE_Y((*pOS2).ySubscriptYOffset as FT_Fixed, YScale) as LONG;
        (*Otm).otmptSuperscriptSize.x = SCALE_X((*pOS2).ySuperscriptXSize as FT_Fixed, XScale) as LONG;
        (*Otm).otmptSuperscriptSize.y = SCALE_Y((*pOS2).ySuperscriptYSize as FT_Fixed, YScale) as LONG;
        (*Otm).otmptSuperscriptOffset.x = SCALE_X((*pOS2).ySuperscriptXOffset as FT_Fixed, XScale) as LONG;
        (*Otm).otmptSuperscriptOffset.y = SCALE_Y((*pOS2).ySuperscriptYOffset as FT_Fixed, YScale) as LONG;
        (*Otm).otmsStrikeoutSize = SCALE_Y((*pOS2).yStrikeoutSize as FT_Fixed, YScale) as UINT;
        (*Otm).otmsStrikeoutPosition = SCALE_Y((*pOS2).yStrikeoutPosition as FT_Fixed, YScale) as INT;

        if pPost.is_null() {
            (*Otm).otmsUnderscoreSize = 0;
            (*Otm).otmsUnderscorePosition = 0;
        } else {
            (*Otm).otmsUnderscoreSize = SCALE_Y((*pPost).underlineThickness as FT_Fixed, YScale) as INT;
            (*Otm).otmsUnderscorePosition = SCALE_Y((*pPost).underlinePosition as FT_Fixed, YScale) as INT;
        }
    }

    if bLocked == 0 {
        IntUnLockFreeType();
    }

    pb = IntStoreFontNames(core::ptr::addr_of!(FontNames), Otm);
    let _ = pb;

    IntFreeFontNames(core::ptr::addr_of_mut!(FontNames));

    (*Cache).OutlineRequiredSize as INT
}

unsafe fn IntGetFontLocalizedName(
    pNameW: PUNICODE_STRING,
    SharedFace: PSHARED_FACE,
    NameID: FT_UShort,
    LangID: FT_UShort,
) -> NTSTATUS {
    let mut Name: FT_SfntName = core::mem::zeroed();
    let mut i: INT;
    let Count: INT;
    let mut BestIndex: INT;
    let mut Score: INT = 0;
    let mut BestScore: INT;
    let mut Error: FT_Error;
    let mut Status: NTSTATUS = STATUS_NOT_FOUND;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    let Cache: PSHARED_FACE_CACHE;
    let Face: FT_Face = (*SharedFace).Face;

    RtlFreeUnicodeString(pNameW as _ );

    /* select cache */
    if PRIMARYLANGID(LangID as _ ) == LANG_ENGLISH as FT_UShort {
        Cache = core::ptr::addr_of_mut!((*SharedFace).EnglishUS);
    } else {
        Cache = core::ptr::addr_of_mut!((*SharedFace).UserLanguage);
    }

    /* use cache if available */
    if NameID == TT_NAME_ID_FONT_FAMILY as FT_UShort && !(*Cache).FontFamily.Buffer.is_null() {
        return IntDuplicateUnicodeString(core::ptr::addr_of_mut!((*Cache).FontFamily), pNameW);
    }
    if NameID == TT_NAME_ID_FULL_NAME as FT_UShort && !(*Cache).FullName.Buffer.is_null() {
        return IntDuplicateUnicodeString(core::ptr::addr_of_mut!((*Cache).FullName), pNameW);
    }

    BestIndex = -1;
    BestScore = 0;

    Count = FT_Get_Sfnt_Name_Count(Face) as INT;
    i = 0;
    while i < Count {
        Error = FT_Get_Sfnt_Name(Face, i as FT_UInt, &mut Name);
        if Error != 0 {
            i += 1;
            continue;
        }

        if Name.name_id != NameID {
            i += 1;
            continue;
        }

        if Name.platform_id != TT_PLATFORM_MICROSOFT as FT_UShort
            || (Name.encoding_id != TT_MS_ID_UNICODE_CS as FT_UShort
                && Name.encoding_id != TT_MS_ID_SYMBOL_CS as FT_UShort)
        {
            i += 1;
            continue;
        }

        if Name.string.is_null()
            || Name.string_len == 0
            || (*Name.string.offset(0) == 0 && *Name.string.offset(1) == 0)
        {
            i += 1;
            continue;
        }

        if Name.language_id == LangID {
            Score = 30;
            BestIndex = i;
            break;
        } else if PRIMARYLANGID(Name.language_id as _ ) == PRIMARYLANGID(LangID as _ ) {
            Score = 20;
        } else if PRIMARYLANGID(Name.language_id as _ ) == LANG_ENGLISH as FT_UShort {
            Score = 10;
        } else {
            Score = 0;
        }

        if Score > BestScore {
            BestScore = Score;
            BestIndex = i;
        }

        i += 1;
    }

    if BestIndex >= 0 {
        Error = if Score == 30 {
            0
        } else {
            FT_Get_Sfnt_Name(Face, BestIndex as FT_UInt, &mut Name)
        };
        if Error == 0 {
            let mut Tmp: _UNICODE_STRING = core::mem::zeroed();
            Tmp.Buffer = Name.string as PWCH;
            Tmp.Length = Name.string_len as u16;
            Tmp.MaximumLength = Name.string_len as u16;

            (*pNameW).Length = 0;
            (*pNameW).MaximumLength =
                (Name.string_len as usize + core::mem::size_of::<WCHAR>()) as u16;
            (*pNameW).Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (*pNameW).MaximumLength as usize,
                TAG_USTR,
            ) as PWCH;

            if !(*pNameW).Buffer.is_null() {
                Status = RtlAppendUnicodeStringToString(pNameW, &mut Tmp);
                if Status == STATUS_SUCCESS {
                    IntSwapEndian((*pNameW).Buffer as *mut core::ffi::c_void, (*pNameW).Length as usize as _ );
                }
            } else {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if NameID == TT_NAME_ID_FONT_SUBFAMILY as FT_UShort {
            RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiName) as _ , (*Face).style_name);
            Status = RtlAnsiStringToUnicodeString(pNameW, &mut AnsiName, 1u8);
        } else {
            RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiName) as _ , (*Face).family_name);
            Status = RtlAnsiStringToUnicodeString(pNameW, &mut AnsiName, 1u8);
        }
    }

    if NT_SUCCESS(Status) {
        if NameID == TT_NAME_ID_FONT_FAMILY as FT_UShort {
            if (*Cache).FontFamily.Buffer.is_null() {
                IntDuplicateUnicodeString(pNameW, core::ptr::addr_of_mut!((*Cache).FontFamily));
            }
        } else if NameID == TT_NAME_ID_FULL_NAME as FT_UShort {
            if (*Cache).FullName.Buffer.is_null() {
                IntDuplicateUnicodeString(pNameW, core::ptr::addr_of_mut!((*Cache).FullName));
            }
        }
    }

    Status
}


fastcall_fn! {
#[no_mangle]
pub unsafe fn FontFamilyFillInfo(
    Info: PFONTFAMILYINFO,
    mut FaceName: LPCWSTR,
    mut FullName: LPCWSTR,
    FontGDI: PFONTGDI,
) {
    let mut StyleA: ANSI_STRING = core::mem::zeroed();
    let mut StyleW: _UNICODE_STRING = core::mem::zeroed();
    let pOS2: *mut TT_OS2;
    let mut fs: FONTSIGNATURE = core::mem::zeroed();
    let mut CharSetInfo: CHARSETINFO = core::mem::zeroed();
    let mut i: u32;
    let mut Size: u32;
    let Otm: *mut OUTLINETEXTMETRICW;
    let Lf: *mut LOGFONTW;
    let TM: *mut TEXTMETRICW;
    let Ntm: *mut NEWTEXTMETRICW;
    let mut fs0: DWORD;
    let status: NTSTATUS;
    let SharedFace: PSHARED_FACE = (*FontGDI).SharedFace;
    let Face: FT_Face = (*SharedFace).Face;
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(NameW) as _ , core::ptr::null_mut());
    RtlZeroMemory(Info as *mut _, core::mem::size_of::<FONTFAMILYINFO>() as _);

    Size = IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 1) as u32;
    if Size == 0 {
        return;
    }
    Otm = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as *mut OUTLINETEXTMETRICW;
    if Otm.is_null() {
        return;
    }
    Size = IntGetOutlineTextMetrics(FontGDI, Size, Otm, 1) as u32;
    if Size == 0 {
        ExFreePoolWithTag(Otm as *mut _, GDITAG_TEXT);
        return;
    }

    Lf = core::ptr::addr_of_mut!((*Info).EnumLogFontEx.elfLogFont);
    TM = core::ptr::addr_of_mut!((*Otm).otmTextMetrics);

    (*Lf).lfHeight = (*TM).tmHeight;
    (*Lf).lfWidth = (*TM).tmAveCharWidth;
    (*Lf).lfWeight = (*TM).tmWeight;
    (*Lf).lfItalic = (*TM).tmItalic;
    (*Lf).lfPitchAndFamily = ((*TM).tmPitchAndFamily & 0xf1) + 1;
    (*Lf).lfCharSet = (*TM).tmCharSet;
    (*Lf).lfOutPrecision = OUT_OUTLINE_PRECIS as u8;
    (*Lf).lfClipPrecision = CLIP_DEFAULT_PRECIS as u8;
    (*Lf).lfQuality = PROOF_QUALITY as u8;

    Ntm = core::ptr::addr_of_mut!((*Info).NewTextMetricEx.ntmTm);
    (*Ntm).tmHeight = (*TM).tmHeight;
    (*Ntm).tmAscent = (*TM).tmAscent;
    (*Ntm).tmDescent = (*TM).tmDescent;
    (*Ntm).tmInternalLeading = (*TM).tmInternalLeading;
    (*Ntm).tmExternalLeading = (*TM).tmExternalLeading;
    (*Ntm).tmAveCharWidth = (*TM).tmAveCharWidth;
    (*Ntm).tmMaxCharWidth = (*TM).tmMaxCharWidth;
    (*Ntm).tmWeight = (*TM).tmWeight;
    (*Ntm).tmOverhang = (*TM).tmOverhang;
    (*Ntm).tmDigitizedAspectX = (*TM).tmDigitizedAspectX;
    (*Ntm).tmDigitizedAspectY = (*TM).tmDigitizedAspectY;
    (*Ntm).tmFirstChar = (*TM).tmFirstChar;
    (*Ntm).tmLastChar = (*TM).tmLastChar;
    (*Ntm).tmDefaultChar = (*TM).tmDefaultChar;
    (*Ntm).tmBreakChar = (*TM).tmBreakChar;
    (*Ntm).tmItalic = (*TM).tmItalic;
    (*Ntm).tmUnderlined = (*TM).tmUnderlined;
    (*Ntm).tmStruckOut = (*TM).tmStruckOut;
    (*Ntm).tmPitchAndFamily = (*TM).tmPitchAndFamily;
    (*Ntm).tmCharSet = (*TM).tmCharSet;
    (*Ntm).ntmFlags = if (*TM).tmItalic != 0 { NTM_ITALIC as DWORD } else { 0 };

    if 550 < (*TM).tmWeight {
        (*Ntm).ntmFlags |= NTM_BOLD as DWORD;
    }

    if 0 == (*Ntm).ntmFlags {
        (*Ntm).ntmFlags = NTM_REGULAR as DWORD;
    }

    (*Info).FontType = if 0 != ((*TM).tmPitchAndFamily & TMPF_TRUETYPE as u8) {
        TRUETYPE_FONTTYPE as DWORD
    } else {
        0
    };

    if 0 == ((*TM).tmPitchAndFamily & TMPF_VECTOR as u8) {
        (*Info).FontType |= RASTER_FONTTYPE as DWORD;
    }

    /* face name */
    if FaceName.is_null() {
        FaceName = (Otm as usize + (*Otm).otmpFamilyName as usize) as LPCWSTR;
    }

    w32r_RtlStringCbCopyW(
        (*Lf).lfFaceName.as_mut_ptr(),
        core::mem::size_of_val(&(*Lf).lfFaceName),
        FaceName,
    );

    /* full name */
    if FullName.is_null() {
        FullName = (Otm as usize + (*Otm).otmpFaceName as usize) as LPCWSTR;
    }

    w32r_RtlStringCbCopyW(
        (*Info).EnumLogFontEx.elfFullName.as_mut_ptr(),
        core::mem::size_of_val(&(*Info).EnumLogFontEx.elfFullName),
        FullName,
    );

    RtlInitAnsiString(core::ptr::addr_of_mut!(StyleA) as _ , (*Face).style_name);
    StyleW.Buffer = (*Info).EnumLogFontEx.elfStyle.as_mut_ptr();
    StyleW.MaximumLength = core::mem::size_of_val(&(*Info).EnumLogFontEx.elfStyle) as u16;
    status = RtlAnsiStringToUnicodeString(&mut StyleW, &mut StyleA, 0);
    if !NT_SUCCESS(status) {
        ExFreePoolWithTag(Otm as *mut _, GDITAG_TEXT);
        return;
    }
    (*Info).EnumLogFontEx.elfScript[0] = 0;

    pOS2 = FT_Get_Sfnt_Table(Face, ft_sfnt_os2) as *mut TT_OS2;

    if pOS2.is_null() {
        ExFreePoolWithTag(Otm as *mut _, GDITAG_TEXT);
        return;
    }

    (*Ntm).ntmSizeEM = (*Otm).otmEMSquare;
    (*Ntm).ntmCellHeight =
        ((*pOS2).usWinAscent as FT_Short as i32 + (*pOS2).usWinDescent as FT_Short as i32) as _;
    (*Ntm).ntmAvgWidth = 0;

    ExFreePoolWithTag(Otm as *mut _, GDITAG_TEXT);

    fs.fsCsb[0] = (*pOS2).ulCodePageRange1;
    fs.fsCsb[1] = (*pOS2).ulCodePageRange2;
    fs.fsUsb[0] = (*pOS2).ulUnicodeRange1;
    fs.fsUsb[1] = (*pOS2).ulUnicodeRange2;
    fs.fsUsb[2] = (*pOS2).ulUnicodeRange3;
    fs.fsUsb[3] = (*pOS2).ulUnicodeRange4;

    if 0 == (*pOS2).version {
        let mut Dummy: FT_UInt = 0;

        if FT_Get_First_Char(Face, &mut Dummy) < 0x100 {
            fs.fsCsb[0] |= FS_LATIN1 as DWORD;
        } else {
            fs.fsCsb[0] |= FS_SYMBOL as DWORD;
        }
    }

    if fs.fsCsb[0] == 0 {
        i = 0;
        while i < (*Face).num_charmaps as u32 {
            let encoding = (*(*(*Face).charmaps.add(i as usize))).encoding;
            if encoding == FT_ENCODING_UNICODE || encoding == FT_ENCODING_APPLE_ROMAN {
                fs.fsCsb[0] |= FS_LATIN1 as DWORD;
            } else if encoding == FT_ENCODING_MS_SYMBOL {
                fs.fsCsb[0] |= FS_SYMBOL as DWORD;
            }
            i += 1;
        }
    }

    i = 0;
    while i < (MAXTCIINDEX as _) {
        fs0 = 1u32 << i;
        if fs.fsCsb[0] & fs0 != 0 {
            if IntTranslateCharsetInfo(&mut fs0, &mut CharSetInfo, TCI_SRCFONTSIG) == 0 {
                CharSetInfo.ciCharset = DEFAULT_CHARSET as u32;
            }
            if DEFAULT_CHARSET as u32 != CharSetInfo.ciCharset {
                if !g_ElfScripts[i as usize].is_null() {
                    wcscpy((*Info).EnumLogFontEx.elfScript.as_mut_ptr(), g_ElfScripts[i as usize]);
                }
            }
        }
        i += 1;
    }
    (*Info).NewTextMetricEx.ntmFontSig = fs;
}
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn GetFontFamilyInfoForList(
    LogFont: *const LOGFONTW,
    Info: PFONTFAMILYINFO,
    NominalName: LPCWSTR,
    pCount: *mut LONG,
    MaxCount: LONG,
    Head: PLIST_ENTRY,
) -> BOOLEAN {
    let mut Entry: PLIST_ENTRY;
    let mut CurrentEntry: *mut FONT_ENTRY = core::ptr::null_mut();
    let mut FontGDI: *mut FONTGDI = core::ptr::null_mut();
    let mut InfoEntry: FONTFAMILYINFO = core::mem::zeroed();
    let mut Count: LONG = *pCount;

    Entry = (*Head).Flink;
    while Entry != Head {
        CurrentEntry = CONTAINING_RECORD!(Entry, FONT_ENTRY, ListEntry);
        FontGDI = (*CurrentEntry).Font;

        if (*LogFont).lfCharSet != (DEFAULT_CHARSET as i8 as _) &&
            (*LogFont).lfCharSet != ((*FontGDI).CharSet as i8 as _)
        {
            Entry = (*Entry).Flink;
            continue;
        }

        FontFamilyFillInfo(
            core::ptr::addr_of_mut!(InfoEntry),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            FontGDI,
        );

        if (*LogFont).lfFaceName[0] != 0 {
            if _wcsnicmp(
                (*LogFont).lfFaceName.as_ptr(),
                InfoEntry.EnumLogFontEx.elfLogFont.lfFaceName.as_ptr(),
                RTL_NUMBER_OF!((*LogFont).lfFaceName) - 1,
            ) != 0 &&
                _wcsnicmp(
                    (*LogFont).lfFaceName.as_ptr(),
                    InfoEntry.EnumLogFontEx.elfFullName.as_ptr(),
                    RTL_NUMBER_OF!((*LogFont).lfFaceName) - 1,
                ) != 0
            {
                Entry = (*Entry).Flink;
                continue;
            }
        }

        if !NominalName.is_null() {
            w32r_RtlStringCbCopyW(
                InfoEntry.EnumLogFontEx.elfLogFont.lfFaceName.as_mut_ptr(),
                core::mem::size_of_val(&InfoEntry.EnumLogFontEx.elfLogFont.lfFaceName),
                NominalName,
            );
        }

        if 0 <= Count && Count < MaxCount {
            RtlCopyMemory(
                core::ptr::addr_of_mut!(*Info.offset(Count as isize)) as *mut core::ffi::c_void,
                core::ptr::addr_of!(InfoEntry) as *const core::ffi::c_void,
                core::mem::size_of_val(&InfoEntry),
            );
        }
        Count += 1;

        Entry = (*Entry).Flink;
    }

    *pCount = Count;

    1u8
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GetFontFamilyInfoForSubstitutes(
    LogFont: *const LOGFONTW,
    Info: PFONTFAMILYINFO,
    pCount: *mut LONG,
    MaxCount: LONG,
) -> BOOLEAN {
    GetFontFamilyInfoForSubstitutes_impl(LogFont, Info, pCount, MaxCount)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GetFontFamilyInfoForSubstitutes(
    LogFont: *const LOGFONTW,
    Info: PFONTFAMILYINFO,
    pCount: *mut LONG,
    MaxCount: LONG,
) -> BOOLEAN {
    GetFontFamilyInfoForSubstitutes_impl(LogFont, Info, pCount, MaxCount)
}

unsafe fn GetFontFamilyInfoForSubstitutes_impl(
    LogFont: *const LOGFONTW,
    Info: PFONTFAMILYINFO,
    pCount: *mut LONG,
    MaxCount: LONG,
) -> BOOLEAN {
    let mut pEntry: PLIST_ENTRY;
    let pHead: PLIST_ENTRY = core::ptr::addr_of_mut!(g_FontSubstListHead);
    let mut pCurrentEntry: *mut FONTSUBST_ENTRY = core::ptr::null_mut();
    let pFromW: PUNICODE_STRING;
    let pToW: PUNICODE_STRING;
    let mut lf: LOGFONTW = *LogFont;
    let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;

    pEntry = (*pHead).Flink;
    while pEntry != pHead {
        pCurrentEntry = (pEntry as *mut u8)
            .offset(-(core::mem::offset_of!(FONTSUBST_ENTRY, ListEntry) as isize))
            as *mut FONTSUBST_ENTRY;

        let pFromW_local = core::ptr::addr_of_mut!((*pCurrentEntry).FontNames[FONTSUBST_FROM as usize]);
        let _ = &pFromW_local;

        if (*LogFont).lfFaceName[0] != UNICODE_NULL as u16 {
            /* check name */
            if _wcsicmp((*LogFont).lfFaceName.as_ptr(), (*pFromW_local).Buffer) != 0 {
                pEntry = (*pEntry).Flink;
                continue;
            }
        }

        let pToW_local = core::ptr::addr_of_mut!((*pCurrentEntry).FontNames[FONTSUBST_TO as usize]);

        if RtlEqualUnicodeString(pFromW_local, pToW_local, TRUE as u8) != 0
            && (*pCurrentEntry).CharSets[FONTSUBST_FROM as usize]
                == (*pCurrentEntry).CharSets[FONTSUBST_TO as usize]
        {
            /* identical mapping */
            pEntry = (*pEntry).Flink;
            continue;
        }

        /* substitute and get the real name */
        IntUnicodeStringToBuffer(
            lf.lfFaceName.as_mut_ptr(),
            core::mem::size_of_val(&lf.lfFaceName),
            pFromW_local,
        );
        SubstituteFontRecurse(&mut lf);
        if (*LogFont).lfCharSet != DEFAULT_CHARSET as u8 && (*LogFont).lfCharSet != lf.lfCharSet {
            pEntry = (*pEntry).Flink;
            continue;
        }

        /* search in global fonts */
        IntLockFreeType();
        GetFontFamilyInfoForList(
            &lf,
            Info,
            (*pFromW_local).Buffer,
            pCount,
            MaxCount,
            core::ptr::addr_of_mut!(g_FontListHead),
        );

        /* search in private fonts */
        IntLockProcessPrivateFonts(Win32Process as _ );
        GetFontFamilyInfoForList(
            &lf,
            Info,
            (*pFromW_local).Buffer,
            pCount,
            MaxCount,
            core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead),
        );
        IntUnLockProcessPrivateFonts(Win32Process as _ );
        IntUnLockFreeType();

        if (*LogFont).lfFaceName[0] != UNICODE_NULL as u16 {
            /* it's already matched to the exact name and charset if the name
               was specified at here, then so don't scan more for another name */
            break;
        }

        let _ = pFromW;
        let _ = pToW;

        pEntry = (*pEntry).Flink;
    }

    TRUE as BOOLEAN
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetRasterizerCaps(lprs: LPRASTERIZER_STATUS) -> u8 {
    ftGdiGetRasterizerCaps_impl(lprs)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetRasterizerCaps(lprs: LPRASTERIZER_STATUS) -> u8 {
    ftGdiGetRasterizerCaps_impl(lprs)
}

unsafe fn ftGdiGetRasterizerCaps_impl(lprs: LPRASTERIZER_STATUS) -> u8 {
    if !lprs.is_null() {
        (*lprs).nSize = core::mem::size_of::<RASTERIZER_STATUS>() as u32 as _ ;
        (*lprs).wFlags = (TT_AVAILABLE | TT_ENABLED) as i16;
        (*lprs).nLanguageID = gusLanguageID as i16;
        return 1u8;
    }
    EngSetLastError(ERROR_INVALID_PARAMETER as u32);
    0u8
}

unsafe fn IntGetHash(pv: LPCVOID, cdw: DWORD) -> DWORD {
    let mut cdw = cdw;
    let mut dwHash: DWORD = cdw;
    let mut pdw = pv as *const DWORD;

    while {
        let cont = cdw > 0;
        cdw = cdw.wrapping_sub(1);
        cont
    } {
        dwHash = dwHash.wrapping_mul(3);
        dwHash ^= *pdw;
        pdw = pdw.add(1);
    }

    dwHash
}

unsafe fn IntFindGlyphCache(pCache: *const FONT_CACHE_ENTRY) -> FT_BitmapGlyph {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut FontEntry: PFONT_CACHE_ENTRY = core::ptr::null_mut();
    let dwHash = (*pCache).dwHash;

    CurrentEntry = g_FontCacheListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(g_FontCacheListHead) {
        FontEntry = CONTAINING_RECORD!(CurrentEntry, FONT_CACHE_ENTRY, ListEntry);
        if (*FontEntry).dwHash == dwHash
            && (*FontEntry).Hashed.GlyphIndex == (*pCache).Hashed.GlyphIndex
            && (*FontEntry).Hashed.Face == (*pCache).Hashed.Face
            && (*FontEntry).Hashed.lfHeight == (*pCache).Hashed.lfHeight
            && (*FontEntry).Hashed.lfWidth == (*pCache).Hashed.lfWidth
            && (*FontEntry).Hashed.__bindgen_anon_1.AspectValue == (*pCache).Hashed.__bindgen_anon_1.AspectValue
            && memcmp(
                core::ptr::addr_of!((*FontEntry).Hashed.matTransform) as *const core::ffi::c_void,
                core::ptr::addr_of!((*pCache).Hashed.matTransform) as *const core::ffi::c_void,
                core::mem::size_of::<FT_Matrix>(),
            ) == 0
        {
            break;
        }
        CurrentEntry = (*CurrentEntry).Flink;
    }

    if CurrentEntry == core::ptr::addr_of_mut!(g_FontCacheListHead) {
        return core::ptr::null_mut();
    }

    RemoveEntryList(CurrentEntry);
    InsertHeadList(core::ptr::addr_of_mut!(g_FontCacheListHead), CurrentEntry);
    (*FontEntry).BitmapGlyph
}

unsafe fn IntGetBitmapGlyphWithCache(
    Cache: PFONT_CACHE_ENTRY,
    GlyphSlot: FT_GlyphSlot,
) -> FT_BitmapGlyph {
    let mut GlyphCopy: FT_Glyph = core::ptr::null_mut();
    let mut error: INT;
    let mut NewEntry: PFONT_CACHE_ENTRY;
    let mut AlignedBitmap: FT_Bitmap = core::mem::zeroed();
    let BitmapGlyph: FT_BitmapGlyph;

    error = FT_Get_Glyph(GlyphSlot, core::ptr::addr_of_mut!(GlyphCopy));
    if error != 0 {
        return core::ptr::null_mut();
    }

    error = FT_Glyph_To_Bitmap(
        core::ptr::addr_of_mut!(GlyphCopy),
        (*Cache).Hashed.__bindgen_anon_1.Aspect.RenderMode as _,
        core::ptr::null_mut(),
        1,
    );
    if error != 0 {
        FT_Done_Glyph(GlyphCopy);
        return core::ptr::null_mut();
    }

    NewEntry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<FONT_CACHE_ENTRY>(),
        TAG_FONT,
    ) as PFONT_CACHE_ENTRY;
    if NewEntry.is_null() {
        FT_Done_Glyph(GlyphCopy);
        return core::ptr::null_mut();
    }

    BitmapGlyph = GlyphCopy as FT_BitmapGlyph;
    FT_Bitmap_New(core::ptr::addr_of_mut!(AlignedBitmap));
    if FT_Bitmap_Convert_ReactOS_Hack(
        (*GlyphSlot).library,
        core::ptr::addr_of_mut!((*BitmapGlyph).bitmap),
        core::ptr::addr_of_mut!(AlignedBitmap),
        4,
        1u8,
    ) != 0
    {
        ExFreePoolWithTag(NewEntry as *mut core::ffi::c_void, TAG_FONT);
        FT_Bitmap_Done((*GlyphSlot).library, core::ptr::addr_of_mut!(AlignedBitmap));
        FT_Done_Glyph(BitmapGlyph as FT_Glyph);
        return core::ptr::null_mut();
    }

    FT_Bitmap_Done(
        (*GlyphSlot).library,
        core::ptr::addr_of_mut!((*BitmapGlyph).bitmap),
    );
    (*BitmapGlyph).bitmap = AlignedBitmap;

    (*NewEntry).BitmapGlyph = BitmapGlyph;
    (*NewEntry).dwHash = (*Cache).dwHash;
    (*NewEntry).Hashed = (*Cache).Hashed;

    InsertHeadList(
        core::ptr::addr_of_mut!(g_FontCacheListHead),
        core::ptr::addr_of_mut!((*NewEntry).ListEntry),
    );
    g_FontCacheNumEntries += 1;
    if g_FontCacheNumEntries > (MAX_FONT_CACHE as _) {
        NewEntry = (g_FontCacheListHead.Blink as *mut u8)
            .offset(-(core::mem::offset_of!(FONT_CACHE_ENTRY, ListEntry) as isize))
            as PFONT_CACHE_ENTRY;
        RemoveCachedEntry(NewEntry);
    }

    BitmapGlyph
}

unsafe fn get_native_glyph_outline(outline: *mut FT_Outline, buflen: core::ffi::c_uint, buf: *mut core::ffi::c_char) -> core::ffi::c_uint {
    let mut pph: *mut TTPOLYGONHEADER;
    let mut ppc: *mut TTPOLYCURVE;
    let mut needed: i32 = 0;
    let mut point: i32 = 0;
    let mut contour: i32;
    let mut first_pt: i32;
    let mut pph_start: core::ffi::c_uint;
    let mut cpfx: core::ffi::c_uint;
    let mut type_: DWORD;

    contour = 0;
    while contour < (*outline).n_contours as i32 {
        if point == *(*outline).contours.offset(contour as isize) as i32 {
            point += 1;
            contour += 1;
            continue;
        }

        pph_start = needed as core::ffi::c_uint;
        pph = buf.offset(needed as isize) as *mut TTPOLYGONHEADER;
        first_pt = point;
        if !buf.is_null() {
            (*pph).dwType = TT_POLYGON_TYPE as DWORD;
            FTVectorToPOINTFX(
                (*outline).points.offset(point as isize),
                core::ptr::addr_of_mut!((*pph).pfxStart),
            );
        }
        needed += core::mem::size_of::<TTPOLYGONHEADER>() as i32;
        point += 1;
        while point <= *(*outline).contours.offset(contour as isize) as i32 {
            ppc = buf.offset(needed as isize) as *mut TTPOLYCURVE;
            type_ = if (*(*outline).tags.offset(point as isize) as i32 & FT_Curve_Tag_On as i32) != 0 {
                TT_PRIM_LINE as DWORD
            } else {
                TT_PRIM_QSPLINE as DWORD
            };
            cpfx = 0;
            loop {
                if !buf.is_null() {
                    FTVectorToPOINTFX(
                        (*outline).points.offset(point as isize),
                        core::ptr::addr_of_mut!((*ppc).apfx[cpfx as usize]),
                    );
                }
                cpfx += 1;
                point += 1;
                if !(point <= *(*outline).contours.offset(contour as isize) as i32
                    && ((*(*outline).tags.offset(point as isize) as i32 & FT_Curve_Tag_On as i32)
                        == (*(*outline).tags.offset((point - 1) as isize) as i32 & FT_Curve_Tag_On as i32)))
                {
                    break;
                }
            }
            if point > *(*outline).contours.offset(contour as isize) as i32
                && (*(*outline).tags.offset((point - 1) as isize) as i32 & FT_Curve_Tag_On as i32) == 0
            {
                if !buf.is_null() {
                    FTVectorToPOINTFX(
                        (*outline).points.offset(first_pt as isize),
                        core::ptr::addr_of_mut!((*ppc).apfx[cpfx as usize]),
                    );
                }
                cpfx += 1;
            } else if point <= *(*outline).contours.offset(contour as isize) as i32
                && (*(*outline).tags.offset(point as isize) as i32 & FT_Curve_Tag_On as i32) != 0
            {
                if !buf.is_null() {
                    FTVectorToPOINTFX(
                        (*outline).points.offset(point as isize),
                        core::ptr::addr_of_mut!((*ppc).apfx[cpfx as usize]),
                    );
                }
                cpfx += 1;
                point += 1;
            }
            if !buf.is_null() {
                (*ppc).wType = type_ as u16;
                (*ppc).cpfx = cpfx as u16;
            }
            needed += core::mem::size_of::<TTPOLYCURVE>() as i32
                + (cpfx - 1) as i32 * core::mem::size_of::<POINTFX>() as i32;
        }
        if !buf.is_null() {
            (*pph).cb = (needed - pph_start as i32) as DWORD;
        }
        contour += 1;
    }
    needed as core::ffi::c_uint
}

unsafe fn get_bezier_glyph_outline(outline: *mut FT_Outline, _buflen: u32, buf: *mut i8) -> u32 {
    let mut point: i32 = 0;
    let mut first_pt: i32;
    let mut pph: *mut TTPOLYGONHEADER;
    let mut ppc: *mut TTPOLYCURVE;
    let mut pph_start: u32;
    let mut cpfx: u32;
    let mut type_: u32;
    let mut cubic_control: [FT_Vector; 4] = core::mem::zeroed();
    let mut needed: u32 = 0;

    let mut contour: i32 = 0;
    while contour < (*outline).n_contours as i32 {
        pph_start = needed;
        pph = buf.add(needed as usize) as *mut TTPOLYGONHEADER;
        first_pt = point;
        if !buf.is_null() {
            (*pph).dwType = TT_POLYGON_TYPE as u32;
            FTVectorToPOINTFX(
                core::ptr::addr_of_mut!(*(*outline).points.add(point as usize)),
                core::ptr::addr_of_mut!((*pph).pfxStart),
            );
        }
        needed += core::mem::size_of::<TTPOLYGONHEADER>() as u32;
        point += 1;

        while point <= *(*outline).contours.add(contour as usize) as i32 {
            ppc = buf.add(needed as usize) as *mut TTPOLYCURVE;
            type_ = if (*(*outline).tags.add(point as usize) as i32 & FT_Curve_Tag_On as i32) != 0 {
                TT_PRIM_LINE as u32
            } else {
                TT_PRIM_CSPLINE as u32
            };
            cpfx = 0;
            loop {
                if type_ == TT_PRIM_LINE as u32 {
                    if !buf.is_null() {
                        FTVectorToPOINTFX(
                            core::ptr::addr_of_mut!(*(*outline).points.add(point as usize)),
                            core::ptr::addr_of_mut!((*ppc).apfx[cpfx as usize]),
                        );
                    }
                    cpfx += 1;
                    point += 1;
                } else {
                    cubic_control[0] = *(*outline).points.add((point - 1) as usize);
                    if (*(*outline).tags.add((point - 1) as usize) as i32 & FT_Curve_Tag_On as i32) == 0 {
                        cubic_control[0].x += (*(*outline).points.add(point as usize)).x + 1;
                        cubic_control[0].y += (*(*outline).points.add(point as usize)).y + 1;
                        cubic_control[0].x >>= 1;
                        cubic_control[0].y >>= 1;
                    }
                    if point + 1 > *(*outline).contours.add(contour as usize) as i32 {
                        cubic_control[3] = *(*outline).points.add(first_pt as usize);
                    } else {
                        cubic_control[3] = *(*outline).points.add((point + 1) as usize);
                        if (*(*outline).tags.add((point + 1) as usize) as i32 & FT_Curve_Tag_On as i32) == 0 {
                            cubic_control[3].x += (*(*outline).points.add(point as usize)).x + 1;
                            cubic_control[3].y += (*(*outline).points.add(point as usize)).y + 1;
                            cubic_control[3].x >>= 1;
                            cubic_control[3].y >>= 1;
                        }
                    }
                    cubic_control[1].x = (2 * (*(*outline).points.add(point as usize)).x + 1) / 3;
                    cubic_control[1].y = (2 * (*(*outline).points.add(point as usize)).y + 1) / 3;
                    cubic_control[2] = cubic_control[1];
                    cubic_control[1].x += (cubic_control[0].x + 1) / 3;
                    cubic_control[1].y += (cubic_control[0].y + 1) / 3;
                    cubic_control[2].x += (cubic_control[3].x + 1) / 3;
                    cubic_control[2].y += (cubic_control[3].y + 1) / 3;
                    if !buf.is_null() {
                        FTVectorToPOINTFX(
                            core::ptr::addr_of_mut!(cubic_control[1]),
                            core::ptr::addr_of_mut!((*ppc).apfx[cpfx as usize]),
                        );
                        FTVectorToPOINTFX(
                            core::ptr::addr_of_mut!(cubic_control[2]),
                            core::ptr::addr_of_mut!((*ppc).apfx[(cpfx + 1) as usize]),
                        );
                        FTVectorToPOINTFX(
                            core::ptr::addr_of_mut!(cubic_control[3]),
                            core::ptr::addr_of_mut!((*ppc).apfx[(cpfx + 2) as usize]),
                        );
                    }
                    cpfx += 3;
                    point += 1;
                }

                if !(point <= *(*outline).contours.add(contour as usize) as i32
                    && (*(*outline).tags.add(point as usize) as i32 & FT_Curve_Tag_On as i32)
                        == (*(*outline).tags.add((point - 1) as usize) as i32 & FT_Curve_Tag_On as i32))
                {
                    break;
                }
            }

            if point <= *(*outline).contours.add(contour as usize) as i32
                && (*(*outline).tags.add(point as usize) as i32 & FT_Curve_Tag_On as i32) != 0
            {
                point += 1;
            }
            if !buf.is_null() {
                (*ppc).wType = type_ as u16;
                (*ppc).cpfx = cpfx as u16;
            }
            needed += core::mem::size_of::<TTPOLYCURVE>() as u32
                + (cpfx - 1) * core::mem::size_of::<POINTFX>() as u32;
        }
        if !buf.is_null() {
            (*pph).cb = needed - pph_start;
        }
        contour += 1;
    }
    needed
}

unsafe fn IntRequestFontSize(dc: PDC, FontGDI: PFONTGDI, mut lfWidth: LONG, mut lfHeight: LONG) -> FT_Error {
    let error: FT_Error;
    let mut req: FT_Size_RequestRec = core::mem::zeroed();
    let face: FT_Face = (*(*FontGDI).SharedFace).Face;
    let pOS2: *mut TT_OS2;
    let pHori: *mut TT_HoriHeader;
    let mut WinFNT: FT_WinFNT_HeaderRec = core::mem::zeroed();
    let mut Ascent: LONG;
    let mut Descent: LONG;
    let mut Sum: LONG;
    let mut EmHeight: LONG;

    lfWidth = lfWidth.abs();
    if lfHeight == 0 {
        if lfWidth == 0 {
            lfHeight = -16;
        } else {
            lfHeight = lfWidth;
        }
    }

    if lfHeight == -1 {
        lfHeight = -2;
    }

    if (*FontGDI).Magic == (FONTGDI_MAGIC as _)
        && (*FontGDI).lfHeight == lfHeight
        && (*FontGDI).lfWidth == lfWidth
    {
        return 0;
    }

    pOS2 = FT_Get_Sfnt_Table(face, FT_SFNT_OS2) as *mut TT_OS2;
    pHori = FT_Get_Sfnt_Table(face, FT_SFNT_HHEA) as *mut TT_HoriHeader;

    if pOS2.is_null() || pHori.is_null() {
        error = FT_Get_WinFNT_Header(face, &mut WinFNT);
        if error != 0 {
            return error;
        }

        (*FontGDI).tmHeight = WinFNT.pixel_height as LONG;
        (*FontGDI).tmAscent = WinFNT.ascent as LONG;
        (*FontGDI).tmDescent = (*FontGDI).tmHeight - (*FontGDI).tmAscent;
        (*FontGDI).tmInternalLeading = WinFNT.internal_leading as LONG;
        (*FontGDI).Magic = FONTGDI_MAGIC as _ ;
        (*FontGDI).lfHeight = lfHeight;
        (*FontGDI).lfWidth = lfWidth;
        return 0;
    }

    const FM_SEL_USE_TYPO_METRICS: u16 = 0x80;
    if lfHeight > 0 {
        Sum = (*pOS2).usWinAscent as FT_Short as LONG + (*pOS2).usWinDescent as FT_Short as LONG;
        if Sum == 0 || ((*pOS2).fsSelection & FM_SEL_USE_TYPO_METRICS) != 0 {
            Ascent = (*pHori).Ascender as LONG;
            Descent = -((*pHori).Descender as LONG);
            Sum = Ascent + Descent;
        } else {
            Ascent = (*pOS2).usWinAscent as FT_Short as LONG;
            Descent = (*pOS2).usWinDescent as FT_Short as LONG;
        }

        (*FontGDI).tmAscent = FT_MulDiv(lfHeight, Ascent, Sum);
        (*FontGDI).tmDescent = FT_MulDiv(lfHeight, Descent, Sum);
        (*FontGDI).tmHeight = (*FontGDI).tmAscent + (*FontGDI).tmDescent;
        (*FontGDI).tmInternalLeading =
            (*FontGDI).tmHeight - FT_MulDiv(lfHeight, (*face).units_per_EM as LONG, Sum);
    } else if lfHeight < 0 {
        if ((*pOS2).fsSelection & FM_SEL_USE_TYPO_METRICS) != 0 {
            (*FontGDI).tmAscent =
                FT_MulDiv(-lfHeight, (*pHori).Ascender as LONG, (*face).units_per_EM as LONG);
            (*FontGDI).tmDescent =
                FT_MulDiv(-lfHeight, -((*pHori).Descender as LONG), (*face).units_per_EM as LONG);
        } else {
            (*FontGDI).tmAscent = FT_MulDiv(
                -lfHeight,
                (*pOS2).usWinAscent as FT_Short as LONG,
                (*face).units_per_EM as LONG,
            );
            (*FontGDI).tmDescent = FT_MulDiv(
                -lfHeight,
                (*pOS2).usWinDescent as FT_Short as LONG,
                (*face).units_per_EM as LONG,
            );
        }
        (*FontGDI).tmHeight = (*FontGDI).tmAscent + (*FontGDI).tmDescent;
        (*FontGDI).tmInternalLeading = (*FontGDI).tmHeight + lfHeight;
    }

    (*FontGDI).Magic = FONTGDI_MAGIC as _ ;
    (*FontGDI).lfHeight = lfHeight;
    (*FontGDI).lfWidth = lfWidth;

    EmHeight = (*FontGDI).tmHeight - (*FontGDI).tmInternalLeading;
    EmHeight = core::cmp::max(EmHeight, 1);
    EmHeight = core::cmp::min(EmHeight, USHORT_MAX as LONG);

    if lfWidth != 0 {
        req.width =
            (FT_MulDiv(lfWidth, (*face).units_per_EM as LONG, (*pOS2).xAvgCharWidth as LONG) << 6)
                as _;
    } else {
        req.width = 0;
    }

    if lfWidth < 10 {
        req.width = 0;
    }

    req.type_ = FT_SIZE_REQUEST_TYPE_NOMINAL;
    req.height = (EmHeight << 6) as _;
    req.horiResolution = 0;
    req.vertResolution = 0;
    FT_Request_Size(face, &mut req)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn TextIntUpdateSize(dc: PDC, TextObj: PTEXTOBJ, FontGDI: PFONTGDI, bDoLock: BOOL) -> BOOL {
    TextIntUpdateSize_impl(dc, TextObj, FontGDI, bDoLock)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn TextIntUpdateSize(dc: PDC, TextObj: PTEXTOBJ, FontGDI: PFONTGDI, bDoLock: BOOL) -> BOOL {
    TextIntUpdateSize_impl(dc, TextObj, FontGDI, bDoLock)
}

unsafe fn TextIntUpdateSize_impl(dc: PDC, TextObj: PTEXTOBJ, FontGDI: PFONTGDI, bDoLock: BOOL) -> BOOL {
    let face: FT_Face;
    let mut error: INT;
    let mut n: INT;
    let mut charmap: FT_CharMap;
    let mut found: FT_CharMap;
    let plf: *mut LOGFONTW;

    if bDoLock != 0 {
        IntLockFreeType();
    }

    face = (*(*FontGDI).SharedFace).Face;
    if (*face).charmap.is_null() {
        found = core::ptr::null_mut();
        n = 0;
        while n < (*face).num_charmaps {
            charmap = *(*face).charmaps.offset(n as isize);
            if (*charmap).encoding == FT_ENCODING_UNICODE {
                found = charmap;
                break;
            }
            n += 1;
        }
        if found.is_null() {
            n = 0;
            while n < (*face).num_charmaps {
                charmap = *(*face).charmaps.offset(n as isize);
                if (*charmap).platform_id as u32 == TT_PLATFORM_APPLE_UNICODE {
                    found = charmap;
                    break;
                }
                n += 1;
            }
        }
        if found.is_null() {
            n = 0;
            while n < (*face).num_charmaps {
                charmap = *(*face).charmaps.offset(n as isize);
                if (*charmap).encoding == FT_ENCODING_MS_SYMBOL {
                    found = charmap;
                    break;
                }
                n += 1;
            }
        }
        if found.is_null() && (*face).num_charmaps > 0 {
            found = *(*face).charmaps.offset(0);
        }
        if !found.is_null() {
            error = FT_Set_Charmap(face, found);
            let _ = error;
        }
    }

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);

    error = IntRequestFontSize(dc, FontGDI, (*plf).lfWidth, (*plf).lfHeight);

    if bDoLock != 0 {
        IntUnLockFreeType();
    }

    if error != 0 {
        return 0;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn get_glyph_index_symbol(ft_face: FT_Face, mut glyph: UINT) -> FT_UInt {
    let mut ret: FT_UInt;

    if glyph < 0x100 {
        glyph += 0xf000;
    }

    ret = FT_Get_Char_Index(ft_face, glyph);
    if ret == 0 {
        ret = FT_Get_Char_Index(ft_face, glyph - 0xf000);
    }

    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn get_glyph_index_symbol(ft_face: FT_Face, mut glyph: UINT) -> FT_UInt {
    let mut ret: FT_UInt;

    if glyph < 0x100 {
        glyph += 0xf000;
    }

    ret = FT_Get_Char_Index(ft_face, glyph);
    if ret == 0 {
        ret = FT_Get_Char_Index(ft_face, glyph - 0xf000);
    }

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn get_glyph_index(ft_face: FT_Face, glyph: UINT) -> FT_UInt {
    let mut ret: FT_UInt;

    if face_has_symbol_charmap(ft_face) != 0 {
        ret = get_glyph_index_symbol(ft_face, glyph);
        if ret != 0 {
            return ret;
        }
    }

    FT_Get_Char_Index(ft_face, glyph)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn get_glyph_index(ft_face: FT_Face, glyph: UINT) -> FT_UInt {
    let mut ret: FT_UInt;

    if face_has_symbol_charmap(ft_face) != 0 {
        ret = get_glyph_index_symbol(ft_face, glyph);
        if ret != 0 {
            return ret;
        }
    }

    FT_Get_Char_Index(ft_face, glyph)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn get_glyph_index_flagged(face: FT_Face, code: FT_ULong, fCodeAsIndex: BOOL) -> FT_UInt {
    if fCodeAsIndex != 0 {
        code as FT_UInt
    } else {
        get_glyph_index(face, code as UINT)
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn get_glyph_index_flagged(face: FT_Face, code: FT_ULong, fCodeAsIndex: BOOL) -> FT_UInt {
    if fCodeAsIndex != 0 {
        code as FT_UInt
    } else {
        get_glyph_index(face, code as UINT)
    }
}

unsafe fn FontLink_Chain_Dump(pChain: PFONTLINK_CHAIN) {
}

unsafe fn FontLink_Chain_FindGlyph(
    pChain: PFONTLINK_CHAIN,
    pCache: PFONT_CACHE_ENTRY,
    pFace: *mut FT_Face,
    mut code: UINT,
    fCodeAsIndex: BOOL,
) -> UINT {
    let mut pFontLink: PFONTLINK;
    let mut Entry: PLIST_ENTRY;
    let Head: PLIST_ENTRY;
    let mut index: UINT;
    let mut face: FT_Face;

    // Try the default font at first
    index = get_glyph_index_flagged((*pChain).pDefFace, code as FT_ULong, fCodeAsIndex);
    if index != 0 {
        *pFace = (*pChain).pDefFace;
        (*pCache).Hashed.Face = (*pChain).pDefFace;
        return index;
    }

    if FontLink_Chain_IsPopulated(pChain) == 0 {
        FontLink_Chain_Populate(pChain);
        FontLink_Chain_Dump(pChain);
    }

    // Now the chain is populated. Looking for the target glyph...
    Head = core::ptr::addr_of_mut!((*pChain).FontLinkList);
    Entry = (*Head).Flink;
    while Entry != Head {
        pFontLink = CONTAINING_RECORD!(Entry, FONTLINK, ListEntry);
        if FontLink_PrepareFontInfo(pFontLink) == 0 {
            Entry = (*Entry).Flink;
            continue;
        }

        face = (*(*pFontLink).SharedFace).Face;
        index = get_glyph_index(face, code);
        if index == 0 {
            Entry = (*Entry).Flink;
            continue;
        }

        *pFace = face;
        (*pCache).Hashed.Face = face;
        FT_Set_Transform(face, core::ptr::addr_of_mut!((*pCache).Hashed.matTransform), core::ptr::null_mut());
        return index;
    }

    // No target glyph found in the chain: use default glyph
    code = s_chFontLinkDefaultChar;
    index = get_glyph_index(*pFace, code);
    *pFace = (*pChain).pDefFace;
    (*pCache).Hashed.Face = (*pChain).pDefFace;
    index
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetGlyphOutline(
    dc: PDC,
    wch: WCHAR,
    iFormat: UINT,
    pgm: LPGLYPHMETRICS,
    cjBuf: ULONG,
    pvBuf: PVOID,
    pmat2: *const MAT2,
    bIgnoreRotation: BOOL,
) -> ULONG {
    ftGdiGetGlyphOutline_impl(dc, wch, iFormat, pgm, cjBuf, pvBuf, pmat2, bIgnoreRotation)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetGlyphOutline(
    dc: PDC,
    wch: WCHAR,
    iFormat: UINT,
    pgm: LPGLYPHMETRICS,
    cjBuf: ULONG,
    pvBuf: PVOID,
    pmat2: *const MAT2,
    bIgnoreRotation: BOOL,
) -> ULONG {
    ftGdiGetGlyphOutline_impl(dc, wch, iFormat, pgm, cjBuf, pvBuf, pmat2, bIgnoreRotation)
}

unsafe fn ftGdiGetGlyphOutline_impl(
    dc: PDC,
    wch: WCHAR,
    mut iFormat: UINT,
    pgm: LPGLYPHMETRICS,
    cjBuf: ULONG,
    mut pvBuf: PVOID,
    pmat2: *const MAT2,
    _bIgnoreRotation: BOOL,
) -> ULONG {
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut hFont: HFONT = 0 as HFONT;
    let mut gm: GLYPHMETRICS = core::mem::zeroed();
    let mut Size: ULONG;
    let ft_face: FT_Face;
    let glyph_index: FT_UInt;
    let mut width: DWORD;
    let mut height: DWORD;
    let mut pitch: DWORD;
    let mut needed: DWORD = 0;
    let mut ft_bitmap: FT_Bitmap = core::mem::zeroed();
    let error: FT_Error;
    let mut left: INT;
    let mut right: INT;
    let mut top: INT = 0;
    let mut bottom: INT = 0;
    let mut load_flags: FT_Int = (FT_LOAD_DEFAULT | FT_LOAD_IGNORE_GLOBAL_ADVANCE_WIDTH) as FT_Int;
    let mut eM11: FLOATOBJ = core::mem::zeroed();
    let mut widthRatio: FLOATOBJ = core::mem::zeroed();
    let mut eTemp: FLOATOBJ = core::mem::zeroed();
    let mut mat: FT_Matrix = core::mem::zeroed();
    let mut transMat: FT_Matrix = identityMat;
    let mut needsTransform: BOOL = 0;
    let orientation: INT;
    let aveWidth: LONG;
    let adv: INT;
    let lsb: INT;
    let bbx: INT;
    let mut potm: *mut OUTLINETEXTMETRICW;
    let mut xo: XFORMOBJ = core::mem::zeroed();
    let mut xform: XFORML = core::mem::zeroed();
    let plf: *mut LOGFONTW;

    pdcattr = (*dc).pdcattr;

    w32r_XFORMOBJ_vInit(&mut xo, core::ptr::addr_of_mut!((*(*dc).pdcattr).mxWorldToDevice));
    w32r_XFORMOBJ_iGetXform(&mut xo, &mut xform);
    w32r_FLOATOBJ_SetFloat(&mut eM11, xform.eM11);

    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }
    FontGDI = ObjToGDI!((*TextObj).Font, FONT);
    ft_face = (*(*FontGDI).SharedFace).Face;

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);
    aveWidth = if FT_IS_SCALABLE(ft_face) != 0 { abs((*plf).lfWidth) } else { 0 };
    orientation = if FT_IS_SCALABLE(ft_face) != 0 { (*plf).lfOrientation } else { 0 };

    ASSERT_FREETYPE_LOCK_NOT_HELD();
    Size = IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 0) as _ ;
    if Size == 0 {
        w32r_TEXTOBJ_UnlockText(TextObj);
        EngSetLastError(ERROR_GEN_FAILURE);
        return GDI_ERROR;
    }
    potm = ExAllocatePoolWithTag(PagedPool, Size as usize, GDITAG_TEXT) as *mut OUTLINETEXTMETRICW;
    if potm.is_null() {
        w32r_TEXTOBJ_UnlockText(TextObj);
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return GDI_ERROR;
    }
    ASSERT_FREETYPE_LOCK_NOT_HELD();
    Size = IntGetOutlineTextMetrics(FontGDI, Size, potm, 0) as _ ;
    if Size == 0 {
        ExFreePoolWithTag(potm as PVOID, GDITAG_TEXT);
        w32r_TEXTOBJ_UnlockText(TextObj);
        EngSetLastError(ERROR_GEN_FAILURE);
        return GDI_ERROR;
    }

    IntLockFreeType();
    TextIntUpdateSize(dc, TextObj, FontGDI, 0);
    IntMatrixFromMx(&mut mat, w32r_DC_pmxWorldToDevice(dc));
    FT_Set_Transform(ft_face, &mut mat, core::ptr::null_mut());

    w32r_TEXTOBJ_UnlockText(TextObj);

    glyph_index = get_glyph_index_flagged(ft_face, wch as FT_ULong, (iFormat & GGO_GLYPH_INDEX) as BOOL);
    iFormat &= !GGO_GLYPH_INDEX;

    if orientation != 0 || (iFormat != GGO_METRICS && iFormat != GGO_BITMAP) || aveWidth != 0 || !pmat2.is_null() {
        load_flags |= FT_LOAD_NO_BITMAP as FT_Int;
    }

    if iFormat & GGO_UNHINTED != 0 {
        load_flags |= FT_LOAD_NO_HINTING as FT_Int;
        iFormat &= !GGO_UNHINTED;
    }

    error = FT_Load_Glyph(ft_face, glyph_index, load_flags);
    if error != 0 {
        IntUnLockFreeType();
        if !potm.is_null() {
            ExFreePoolWithTag(potm as PVOID, GDITAG_TEXT);
        }
        return GDI_ERROR;
    }
    IntUnLockFreeType();

    w32r_FLOATOBJ_Set1(&mut widthRatio);
    if aveWidth != 0 && !potm.is_null() {
        w32r_FLOATOBJ_SetLong(&mut widthRatio, aveWidth);
        w32r_FLOATOBJ_Mul(&mut widthRatio, &mut eM11);
        w32r_FLOATOBJ_DivLong(&mut widthRatio, (*potm).otmTextMetrics.tmAveCharWidth);
    }

    w32r_FLOATOBJ_SetLong(&mut eTemp, (*(*ft_face).glyph).metrics.horiBearingX);
    w32r_FLOATOBJ_Mul(&mut eTemp, &mut widthRatio);
    left = w32r_FLOATOBJ_GetLong(&mut eTemp) & -64;

    w32r_FLOATOBJ_SetLong(&mut eTemp, (*(*ft_face).glyph).metrics.horiBearingX * (*(*ft_face).glyph).metrics.width);
    w32r_FLOATOBJ_Mul(&mut eTemp, &mut widthRatio);
    w32r_FLOATOBJ_AddLong(&mut eTemp, 63);
    right = w32r_FLOATOBJ_GetLong(&mut eTemp) & -64;

    w32r_FLOATOBJ_SetLong(&mut eTemp, (*(*ft_face).glyph).metrics.horiAdvance);
    w32r_FLOATOBJ_Mul(&mut eTemp, &mut widthRatio);
    w32r_FLOATOBJ_AddLong(&mut eTemp, 63);
    adv = w32r_FLOATOBJ_GetLong(&mut eTemp) >> 6;

    lsb = left >> 6;
    bbx = (right - left) >> 6;
    let _ = (lsb, bbx);

    IntLockFreeType();

    if w32r_FLOATOBJ_Equal1(&mut widthRatio) == 0 {
        let mut scaleMat: FT_Matrix = core::mem::zeroed();

        eTemp = widthRatio;
        w32r_FLOATOBJ_MulLong(&mut eTemp, 1 << 16);

        scaleMat.xx = w32r_FLOATOBJ_GetLong(&mut eTemp) as _;
        scaleMat.xy = 0;
        scaleMat.yx = 0;
        scaleMat.yy = INT_TO_FIXED(1) as _;
        FT_Matrix_Multiply(&mut scaleMat, &mut transMat);
        needsTransform = 1;
    }

    {
        let mut ftmatrix: FT_Matrix = core::mem::zeroed();
        let pmx: PMATRIX = w32r_DC_pmxWorldToDevice(dc);

        IntMatrixFromMx(&mut ftmatrix, pmx);

        if memcmp(
            &ftmatrix as *const _ as *const core::ffi::c_void,
            &identityMat as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<FT_Matrix>(),
        ) != 0
        {
            FT_Matrix_Multiply(&mut ftmatrix, &mut transMat);
            needsTransform = 1;
        }
    }

    if orientation != 0 {
        let mut rotationMat: FT_Matrix = core::mem::zeroed();
        IntEscapeMatrix(&mut rotationMat, orientation);
        FT_Matrix_Multiply(&mut rotationMat, &mut transMat);
        needsTransform = 1;
    }

    if !pmat2.is_null() {
        let mut extraMat: FT_Matrix = core::mem::zeroed();
        extraMat.xx = FT_FixedFromFIXED((*pmat2).eM11);
        extraMat.xy = FT_FixedFromFIXED((*pmat2).eM21);
        extraMat.yx = FT_FixedFromFIXED((*pmat2).eM12);
        extraMat.yy = FT_FixedFromFIXED((*pmat2).eM22);
        FT_Matrix_Multiply(&mut extraMat, &mut transMat);
        needsTransform = 1;
    }

    if !potm.is_null() {
        ExFreePoolWithTag(potm as PVOID, GDITAG_TEXT);
        potm = core::ptr::null_mut();
    }

    if needsTransform == 0 {
        top = ((*(*ft_face).glyph).metrics.horiBearingY + 63) & -64;
        bottom = ((*(*ft_face).glyph).metrics.horiBearingY - (*(*ft_face).glyph).metrics.height) & -64;
        gm.gmCellIncX = adv as _;
        gm.gmCellIncY = 0;
    } else {
        let mut xc: INT;
        let mut yc: INT;
        let mut vec: FT_Vector = core::mem::zeroed();
        xc = 0;
        while xc < 2 {
            yc = 0;
            while yc < 2 {
                vec.x = (*(*ft_face).glyph).metrics.horiBearingX
                    + xc as core::ffi::c_long * (*(*ft_face).glyph).metrics.width;
                vec.y = (*(*ft_face).glyph).metrics.horiBearingY
                    - yc as core::ffi::c_long * (*(*ft_face).glyph).metrics.height;
                FT_Vector_Transform(&mut vec, &mut transMat);
                if xc == 0 && yc == 0 {
                    left = vec.x as INT;
                    right = vec.x as INT;
                    top = vec.y as INT;
                    bottom = vec.y as INT;
                } else {
                    if (vec.x as INT) < left {
                        left = vec.x as INT;
                    } else if (vec.x as INT) > right {
                        right = vec.x as INT;
                    }
                    if (vec.y as INT) < bottom {
                        bottom = vec.y as INT;
                    } else if (vec.y as INT) > top {
                        top = vec.y as INT;
                    }
                }
                yc += 1;
            }
            xc += 1;
        }
        left = left & -64;
        right = (right + 63) & -64;
        bottom = bottom & -64;
        top = (top + 63) & -64;

        vec.x = (*(*ft_face).glyph).metrics.horiAdvance;
        vec.y = 0;
        FT_Vector_Transform(&mut vec, &mut transMat);
        gm.gmCellIncX = ((vec.x + 63) >> 6) as _;
        gm.gmCellIncY = (-((vec.y + 63) >> 6)) as _;
    }
    gm.gmBlackBoxX = ((right - left) >> 6) as _;
    gm.gmBlackBoxY = ((top - bottom) >> 6) as _;
    gm.gmptGlyphOrigin.x = left >> 6;
    gm.gmptGlyphOrigin.y = top >> 6;

    IntUnLockFreeType();

    if iFormat == GGO_METRICS {
        *pgm = gm;
        return 1;
    }

    if (*(*ft_face).glyph).format != 1 /* ft_glyph_format_outline */ && iFormat != GGO_BITMAP {
        return GDI_ERROR;
    }

    let ft_glyph_format_bitmap: core::ffi::c_uint = 0x62697473; /* 'bits' */
    let ft_glyph_format_outline: core::ffi::c_uint = 0x6f75746c; /* 'outl' */

    'switch_block: {
        if iFormat == GGO_BITMAP {
            width = gm.gmBlackBoxX;
            height = gm.gmBlackBoxY;
            pitch = ((width + 31) >> 5) << 2;
            needed = pitch * height;

            if pvBuf.is_null() || cjBuf == 0 {
                break 'switch_block;
            }
            if needed == 0 {
                return GDI_ERROR;
            }
            if needed > cjBuf {
                return GDI_ERROR;
            }

            if (*(*ft_face).glyph).format == ft_glyph_format_bitmap {
                let mut src: *mut BYTE = (*(*ft_face).glyph).bitmap.buffer;
                let mut dst: *mut BYTE = pvBuf as *mut BYTE;
                let w: INT = min(pitch as INT, (((*(*ft_face).glyph).bitmap.width + 7) >> 3) as INT);
                let mut h: INT = min(height as INT, (*(*ft_face).glyph).bitmap.rows as INT);
                while h != 0 {
                    h -= 1;
                    RtlCopyMemory(dst as PVOID, src as PVOID, w as usize);
                    src = src.add((*(*ft_face).glyph).bitmap.pitch as usize);
                    dst = dst.add(pitch as usize);
                }
            } else if (*(*ft_face).glyph).format == ft_glyph_format_outline {
                ft_bitmap.width = width as _;
                ft_bitmap.rows = height as _;
                ft_bitmap.pitch = pitch as _;
                ft_bitmap.pixel_mode = FT_PIXEL_MODE_GRAY as _; /* mono placeholder */
                ft_bitmap.pixel_mode = FT_PIXEL_MODE_MONO as _;
                ft_bitmap.buffer = pvBuf as *mut core::ffi::c_uchar;

                IntLockFreeType();
                if needsTransform != 0 {
                    FT_Outline_Transform(core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), &mut transMat);
                }
                FT_Outline_Translate(core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), -(left as core::ffi::c_long), -(bottom as core::ffi::c_long));
                RtlZeroMemory(pvBuf, needed as usize);
                FT_Outline_Get_Bitmap(g_FreeTypeLibrary, core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), &mut ft_bitmap);
                IntUnLockFreeType();
            } else {
                return GDI_ERROR;
            }
        } else if iFormat == GGO_GRAY2_BITMAP || iFormat == GGO_GRAY4_BITMAP || iFormat == GGO_GRAY8_BITMAP {
            let mult: core::ffi::c_uint;
            let mut row: core::ffi::c_uint;
            let mut col: core::ffi::c_uint;
            let mut start: *mut BYTE;
            let mut ptr: *mut BYTE;

            width = gm.gmBlackBoxX;
            height = gm.gmBlackBoxY;
            pitch = (width + 3) / 4 * 4;
            needed = pitch * height;

            if pvBuf.is_null() || cjBuf == 0 {
                break 'switch_block;
            }
            if needed == 0 {
                return GDI_ERROR;
            }
            if needed > cjBuf {
                return GDI_ERROR;
            }

            if (*(*ft_face).glyph).format == ft_glyph_format_bitmap {
                let mut src: *mut BYTE = (*(*ft_face).glyph).bitmap.buffer;
                let mut dst: *mut BYTE = pvBuf as *mut BYTE;
                let mut h: INT = min(height as INT, (*(*ft_face).glyph).bitmap.rows as INT);
                let mut x: INT;
                while h != 0 {
                    h -= 1;
                    x = 0;
                    while (x as UINT) < pitch {
                        if (x as core::ffi::c_uint) < (*(*ft_face).glyph).bitmap.width as core::ffi::c_uint {
                            let sb = *src.add((x / 8) as usize);
                            *dst.add(x as usize) = if sb & (1 << (7 - (x % 8))) != 0 { 0xff } else { 0 };
                        } else {
                            *dst.add(x as usize) = 0;
                        }
                        x += 1;
                    }
                    src = src.add((*(*ft_face).glyph).bitmap.pitch as usize);
                    dst = dst.add(pitch as usize);
                }
            } else if (*(*ft_face).glyph).format == ft_glyph_format_outline {
                ft_bitmap.width = width as _;
                ft_bitmap.rows = height as _;
                ft_bitmap.pitch = pitch as _;
                ft_bitmap.pixel_mode = FT_PIXEL_MODE_GRAY as _;
                ft_bitmap.buffer = pvBuf as *mut core::ffi::c_uchar;

                IntLockFreeType();
                if needsTransform != 0 {
                    FT_Outline_Transform(core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), &mut transMat);
                }
                FT_Outline_Translate(core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), -(left as core::ffi::c_long), -(bottom as core::ffi::c_long));
                RtlZeroMemory(ft_bitmap.buffer as PVOID, cjBuf as usize);
                FT_Outline_Get_Bitmap(g_FreeTypeLibrary, core::ptr::addr_of_mut!((*(*ft_face).glyph).outline), &mut ft_bitmap);
                IntUnLockFreeType();

                if iFormat == GGO_GRAY2_BITMAP {
                    mult = 4;
                } else if iFormat == GGO_GRAY4_BITMAP {
                    mult = 16;
                } else if iFormat == GGO_GRAY8_BITMAP {
                    mult = 64;
                } else {
                    return GDI_ERROR;
                }

                start = pvBuf as *mut BYTE;
                row = 0;
                while row < height {
                    ptr = start;
                    col = 0;
                    while col < width {
                        *ptr = (((*ptr as core::ffi::c_int) * mult as core::ffi::c_int + 128) / 256) as BYTE;
                        ptr = ptr.add(1);
                        col += 1;
                    }
                    start = start.add(pitch as usize);
                    row += 1;
                }
            } else {
                return GDI_ERROR;
            }
        } else if iFormat == GGO_NATIVE {
            let outline: *mut FT_Outline = core::ptr::addr_of_mut!((*(*ft_face).glyph).outline);

            if cjBuf == 0 {
                pvBuf = core::ptr::null_mut();
            }

            IntLockFreeType();
            if needsTransform != 0 && !pvBuf.is_null() {
                FT_Outline_Transform(outline, &mut transMat);
            }

            needed = get_native_glyph_outline(outline, cjBuf, core::ptr::null_mut());

            if pvBuf.is_null() || cjBuf == 0 {
                IntUnLockFreeType();
                break 'switch_block;
            }
            if needed > cjBuf {
                IntUnLockFreeType();
                return GDI_ERROR;
            }
            get_native_glyph_outline(outline, cjBuf, pvBuf as *mut i8);
            IntUnLockFreeType();
        } else if iFormat == GGO_BEZIER {
            let outline: *mut FT_Outline = core::ptr::addr_of_mut!((*(*ft_face).glyph).outline);
            if cjBuf == 0 {
                pvBuf = core::ptr::null_mut();
            }

            if needsTransform != 0 && !pvBuf.is_null() {
                IntLockFreeType();
                FT_Outline_Transform(outline, &mut transMat);
                IntUnLockFreeType();
            }
            needed = get_bezier_glyph_outline(outline, cjBuf, core::ptr::null_mut());

            if pvBuf.is_null() || cjBuf == 0 {
                break 'switch_block;
            }
            if needed > cjBuf {
                return GDI_ERROR;
            }

            get_bezier_glyph_outline(outline, cjBuf, pvBuf as *mut i8);
        } else {
            return GDI_ERROR;
        }
    }

    if gm.gmBlackBoxX == 0 {
        gm.gmBlackBoxX = 1;
    }
    if gm.gmBlackBoxY == 0 {
        gm.gmBlackBoxY = 1;
    }

    *pgm = gm;
    needed
}

unsafe fn IntGetRealGlyph(Cache: PFONT_CACHE_ENTRY) -> FT_BitmapGlyph {
    let error: INT;
    let glyph: FT_GlyphSlot;
    let mut realglyph: FT_BitmapGlyph;

    (*Cache).dwHash = IntGetHash(
        core::ptr::addr_of!((*Cache).Hashed) as LPCVOID,
        (core::mem::size_of_val(&(*Cache).Hashed) / core::mem::size_of::<DWORD>()) as DWORD,
    );

    realglyph = IntFindGlyphCache(Cache);
    if !realglyph.is_null() {
        return realglyph;
    }

    error = FT_Load_Glyph(
        (*Cache).Hashed.Face,
        (*Cache).Hashed.GlyphIndex as _ ,
        FT_LOAD_DEFAULT as FT_Int32,
    );
    if error != 0 {
        return core::ptr::null_mut();
    }

    glyph = (*(*Cache).Hashed.Face).glyph;

    if (*Cache).Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Bold != 0 {
        FT_GlyphSlot_Embolden(glyph);
    }

    if (*Cache).Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Italic != 0 {
        FT_GlyphSlot_Oblique(glyph);
    }

    realglyph = IntGetBitmapGlyphWithCache(Cache, glyph);

    realglyph
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn TextIntGetTextExtentPoint(
    dc: PDC,
    TextObj: PTEXTOBJ,
    String: PCWCH,
    Count: INT,
    MaxExtent: ULONG,
    Fit: PINT,
    Dx: PINT,
    Size: PSIZE,
    fl: FLONG,
) -> BOOL {
    TextIntGetTextExtentPoint_impl(dc, TextObj, String, Count, MaxExtent, Fit, Dx, Size, fl)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn TextIntGetTextExtentPoint(
    dc: PDC,
    TextObj: PTEXTOBJ,
    String: PCWCH,
    Count: INT,
    MaxExtent: ULONG,
    Fit: PINT,
    Dx: PINT,
    Size: PSIZE,
    fl: FLONG,
) -> BOOL {
    TextIntGetTextExtentPoint_impl(dc, TextObj, String, Count, MaxExtent, Fit, Dx, Size, fl)
}

unsafe fn TextIntGetTextExtentPoint_impl(
    dc: PDC,
    TextObj: PTEXTOBJ,
    mut String: PCWCH,
    Count: INT,
    MaxExtent: ULONG,
    Fit: PINT,
    Dx: PINT,
    Size: PSIZE,
    fl: FLONG,
) -> BOOL {
    let FontGDI: PFONTGDI;
    let mut realglyph: FT_BitmapGlyph;
    let mut glyph_index: INT;
    let mut i: INT;
    let mut previous: INT;
    let nTenthsOfDegrees: INT;
    let mut TotalWidth64: ULONGLONG = 0;
    let plf: *mut LOGFONTW;
    let use_kerning: BOOL;
    let bVerticalWriting: bool;
    let ascender: LONG;
    let descender: LONG;
    let mut Cache: FONT_CACHE_ENTRY = core::mem::zeroed();
    let mut ch0: DWORD;
    let mut ch1: DWORD;
    let mut Chain: FONTLINK_CHAIN = core::mem::zeroed();

    FontGDI = ObjToGDI!((*TextObj).Font, FONT);

    Cache.Hashed.Face = (*(*FontGDI).SharedFace).Face;
    if !Fit.is_null() {
        *Fit = 0;
    }

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);
    Cache.Hashed.lfHeight = (*plf).lfHeight;
    Cache.Hashed.lfWidth = (*plf).lfWidth;
    Cache.Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Bold = EMUBOLD_NEEDED((*FontGDI).OriginalWeight, (*plf).lfWeight) as BYTE ;
    Cache.Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Italic = (((*plf).lfItalic != 0) && ((*FontGDI).OriginalItalic == 0)) as BYTE ;

    nTenthsOfDegrees = IntNormalizeAngle((*plf).lfEscapement - (*plf).lfOrientation);
    bVerticalWriting = (nTenthsOfDegrees == 90 * 10) || (nTenthsOfDegrees == 270 * 10);

    if IntIsFontRenderingEnabled() != 0 {
        Cache.Hashed.__bindgen_anon_1.Aspect.RenderMode = IntGetFontRenderMode(plf) as BYTE as _ ;
    } else {
        Cache.Hashed.__bindgen_anon_1.Aspect.RenderMode = FT_RENDER_MODE_MONO as BYTE as _ ;
    }

    IntLockFreeType();
    TextIntUpdateSize(dc, TextObj, FontGDI, 0u8 as BOOL);
    Cache.Hashed.matTransform = identityMat;
    FT_Set_Transform(Cache.Hashed.Face, core::ptr::null_mut(), core::ptr::null_mut());

    FontLink_Chain_Init(core::ptr::addr_of_mut!(Chain), TextObj, Cache.Hashed.Face);

    use_kerning = FT_HAS_KERNING(Cache.Hashed.Face);
    previous = 0;

    i = 0;
    while i < Count {
        ch0 = *String as DWORD;
        String = String.add(1);
        if IS_HIGH_SURROGATE(ch0) {
            i += 1;
            if i >= Count {
                break;
            }

            ch1 = *String as DWORD;
            String = String.add(1);
            if IS_LOW_SURROGATE(ch1) {
                ch0 = Utf32FromSurrogatePair(ch0, ch1);
            }
        }

        glyph_index = FontLink_Chain_FindGlyph(
            core::ptr::addr_of_mut!(Chain),
            core::ptr::addr_of_mut!(Cache),
            core::ptr::addr_of_mut!(Cache.Hashed.Face),
            ch0 as UINT,
            (fl & GTEF_INDICES as FLONG) as BOOL,
        ) as INT;
        Cache.Hashed.GlyphIndex = glyph_index as _;

        realglyph = IntGetRealGlyph(core::ptr::addr_of_mut!(Cache));
        if realglyph.is_null() {
            break;
        }

        if use_kerning != 0 && previous != 0 && glyph_index != 0 {
            let mut delta: FT_Vector = core::mem::zeroed();
            FT_Get_Kerning(Cache.Hashed.Face, previous as FT_UInt, glyph_index as FT_UInt, 0, core::ptr::addr_of_mut!(delta));
            TotalWidth64 += delta.x as ULONGLONG;
        }

        TotalWidth64 += ((*realglyph).root.advance.x >> 10) as ULONGLONG;

        if ((TotalWidth64 + 32) >> 6) <= MaxExtent as ULONGLONG && !Fit.is_null() {
            *Fit = i + 1;
        }
        if !Dx.is_null() {
            *Dx.offset(i as isize) = ((TotalWidth64 + 32) >> 6) as INT;
        }

        previous = glyph_index;

        i += 1;
    }

    ascender = (*FontGDI).tmAscent;
    descender = (*FontGDI).tmDescent;
    IntUnLockFreeType();

    if bVerticalWriting {
        (*Size).cx = ascender + descender;
        (*Size).cy = ((TotalWidth64 + 32) >> 6) as LONG;
    } else {
        (*Size).cx = ((TotalWidth64 + 32) >> 6) as LONG;
        (*Size).cy = ascender + descender;
    }

    FontLink_Chain_Finish(core::ptr::addr_of_mut!(Chain));

    1u8 as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetTextCharsetInfo(
    Dc: PDC,
    lpSig: LPFONTSIGNATURE,
    dwFlags: DWORD,
) -> INT {
    ftGdiGetTextCharsetInfo_impl(Dc, lpSig, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetTextCharsetInfo(
    Dc: PDC,
    lpSig: LPFONTSIGNATURE,
    dwFlags: DWORD,
) -> INT {
    ftGdiGetTextCharsetInfo_impl(Dc, lpSig, dwFlags)
}

unsafe fn ftGdiGetTextCharsetInfo_impl(
    Dc: PDC,
    lpSig: LPFONTSIGNATURE,
    _dwFlags: DWORD,
) -> INT {
    let pdcattr: PDC_ATTR;
    let mut Ret: UINT = DEFAULT_CHARSET as UINT;
    let mut i: INT;
    let hFont: HFONT;
    let TextObj: PTEXTOBJ;
    let FontGdi: PFONTGDI;
    let mut fs: FONTSIGNATURE = core::mem::zeroed();
    let mut pOS2: *mut TT_OS2;
    let Face: FT_Face;
    let mut csi: CHARSETINFO = core::mem::zeroed();
    let mut cp: DWORD;
    let mut fs0: DWORD;
    let mut usACP: USHORT = 0;
    let mut usOEM: USHORT = 0;

    pdcattr = (*Dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);

    if TextObj.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return Ret as INT;
    }
    FontGdi = ObjToGDI!((*TextObj).Font, FONT);
    Face = (*(*FontGdi).SharedFace).Face;
    w32r_TEXTOBJ_UnlockText(TextObj);

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(fs) as *mut u8,
        0,
        core::mem::size_of::<FONTSIGNATURE>(),
    );
    IntLockFreeType();
    pOS2 = FT_Get_Sfnt_Table(Face, ft_sfnt_os2) as *mut TT_OS2;
    if !pOS2.is_null() {
        fs.fsCsb[0] = (*pOS2).ulCodePageRange1;
        fs.fsCsb[1] = (*pOS2).ulCodePageRange2;
        fs.fsUsb[0] = (*pOS2).ulUnicodeRange1;
        fs.fsUsb[1] = (*pOS2).ulUnicodeRange2;
        fs.fsUsb[2] = (*pOS2).ulUnicodeRange3;
        fs.fsUsb[3] = (*pOS2).ulUnicodeRange4;
        if (*pOS2).version == 0 {
            let mut dummy: FT_UInt = 0;

            if FT_Get_First_Char(Face, &mut dummy) < 0x100 {
                fs.fsCsb[0] |= FS_LATIN1;
            } else {
                fs.fsCsb[0] |= FS_SYMBOL;
            }
        }
    }
    pOS2 = core::ptr::null_mut();
    let _ = pOS2;
    IntUnLockFreeType();
    if fs.fsCsb[0] == 0 {
        i = 0;
        while i < (*Face).num_charmaps {
            match (*(*(*Face).charmaps.offset(i as isize))).encoding {
                FT_ENCODING_UNICODE | FT_ENCODING_APPLE_ROMAN => {
                    fs.fsCsb[0] |= FS_LATIN1;
                }
                FT_ENCODING_MS_SYMBOL => {
                    fs.fsCsb[0] |= FS_SYMBOL;
                }
                _ => {}
            }
            i += 1;
        }
    }
    if !lpSig.is_null() {
        RtlCopyMemory(
            lpSig as *mut core::ffi::c_void,
            core::ptr::addr_of!(fs) as *const core::ffi::c_void,
            core::mem::size_of::<FONTSIGNATURE>() as SIZE_T as _ ,
        );
    }

    RtlGetDefaultCodePage(&mut usACP, &mut usOEM);
    cp = usACP as DWORD;

    'exit: {
        if IntTranslateCharsetInfo(&mut cp, &mut csi, TCI_SRCCODEPAGE) != 0 {
            if csi.fs.fsCsb[0] & fs.fsCsb[0] != 0 {
                Ret = csi.ciCharset;
                break 'exit;
            }
        }

        i = 0;
        while i < MAXTCIINDEX as INT {
            fs0 = 1u32 << i;
            if fs.fsCsb[0] & fs0 != 0 {
                if IntTranslateCharsetInfo(&mut fs0, &mut csi, TCI_SRCFONTSIG) != 0 {
                    Ret = csi.ciCharset;
                    break 'exit;
                }
            }
            i += 1;
        }
    }

    let _ = Ret;
    MAKELONG(csi.ciACP, csi.ciCharset) as INT
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGetFontUnicodeRanges(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD {
    ftGetFontUnicodeRanges_impl(Font, glyphset)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGetFontUnicodeRanges(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD {
    ftGetFontUnicodeRanges_impl(Font, glyphset)
}

unsafe fn ftGetFontUnicodeRanges_impl(Font: PFONTGDI, glyphset: PGLYPHSET) -> DWORD {
    let mut size: DWORD;
    let mut num_ranges: DWORD = 0;
    let face: FT_Face = (*(*Font).SharedFace).Face;

    if (*face).charmap.is_null() {
        return 0;
    }

    /* Lock FreeType for face lookup (upstream e1bf662d9a) */
    IntLockFreeType();

    if (*(*face).charmap).encoding == FT_ENCODING_UNICODE {
        let mut glyph_code: FT_UInt = 0;
        let mut char_code: FT_ULong;
        let mut char_code_prev: FT_ULong;

        char_code = FT_Get_First_Char(face, &mut glyph_code);
        char_code_prev = char_code;

        if glyph_code == 0 {
            IntUnLockFreeType();
            return 0;
        }

        if !glyphset.is_null() {
            (*glyphset).ranges[0].wcLow = char_code as USHORT;
            (*glyphset).ranges[0].cGlyphs = 0;
            (*glyphset).cGlyphsSupported = 0;
        }

        num_ranges = 1;
        while glyph_code != 0 {
            if char_code < char_code_prev {
                IntUnLockFreeType();
                return 0;
            }
            if char_code - char_code_prev > 1 {
                num_ranges += 1;
                if !glyphset.is_null() {
                    (*glyphset).ranges[(num_ranges - 1) as usize].wcLow = char_code as USHORT;
                    (*glyphset).ranges[(num_ranges - 1) as usize].cGlyphs = 1;
                    (*glyphset).cGlyphsSupported += 1;
                }
            } else if !glyphset.is_null() {
                (*glyphset).ranges[(num_ranges - 1) as usize].cGlyphs += 1;
                (*glyphset).cGlyphsSupported += 1;
            }
            char_code_prev = char_code;
            char_code = FT_Get_Next_Char(face, char_code, &mut glyph_code);
        }
    }

    size = (core::mem::size_of::<GLYPHSET>() as DWORD)
        + (core::mem::size_of::<WCRANGE>() as DWORD) * (num_ranges - 1);
    if !glyphset.is_null() {
        (*glyphset).cbThis = size;
        (*glyphset).cRanges = num_ranges;
        (*glyphset).flAccel = 0;
    }

    /* Unlock FreeType */
    IntUnLockFreeType();

    size
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetTextMetricsW(hDC: HDC, ptmwi: PTMW_INTERNAL) -> BOOL {
    ftGdiGetTextMetricsW_impl(hDC, ptmwi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetTextMetricsW(hDC: HDC, ptmwi: PTMW_INTERNAL) -> BOOL {
    ftGdiGetTextMetricsW_impl(hDC, ptmwi)
}

unsafe fn ftGdiGetTextMetricsW_impl(hDC: HDC, ptmwi: PTMW_INTERNAL) -> BOOL {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let Face: FT_Face;
    let pOS2: *mut TT_OS2;
    let pHori: *mut TT_HoriHeader;
    let mut Win: FT_WinFNT_HeaderRec = core::mem::zeroed();
    let mut Error: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let plf: *mut LOGFONTW;

    if ptmwi.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    RtlZeroMemory(ptmwi as *mut _, core::mem::size_of::<TMW_INTERNAL>() as ULONG as _ );

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    TextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
    if !TextObj.is_null() {
        plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);
        FontGDI = ObjToGDI!((*TextObj).Font);

        Face = (*(*FontGDI).SharedFace).Face;

        IntLockFreeType();
        Error = IntRequestFontSize(dc, FontGDI, (*plf).lfWidth, (*plf).lfHeight) as ULONG;
        FT_Set_Transform(Face, core::ptr::null_mut(), core::ptr::null_mut());

        IntUnLockFreeType();

        if Error != 0 {
            Status = STATUS_UNSUCCESSFUL;
        } else {
            Status = STATUS_SUCCESS;

            IntLockFreeType();

            Error = FT_Get_WinFNT_Header(Face, &mut Win) as ULONG;
            pOS2 = FT_Get_Sfnt_Table(Face, ft_sfnt_os2) as *mut TT_OS2;
            pHori = FT_Get_Sfnt_Table(Face, ft_sfnt_hhea) as *mut TT_HoriHeader;

            if pOS2.is_null() && Error != 0 {
                Status = STATUS_INTERNAL_ERROR;
            }

            if pHori.is_null() && Error != 0 {
                Status = STATUS_INTERNAL_ERROR;
            }

            if NT_SUCCESS(Status) {
                FillTM(
                    core::ptr::addr_of_mut!((*ptmwi).TextMetric),
                    FontGDI,
                    pOS2,
                    pHori,
                    if Error != 0 { core::ptr::null_mut() } else { &mut Win },
                );
            }

            IntUnLockFreeType();
        }
        w32r_TEXTOBJ_UnlockText(TextObj);
    } else {
        Status = STATUS_INVALID_HANDLE;
    }
    DC_UnlockDc(dc);

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetFontData(
    FontGdi: PFONTGDI,
    Table: DWORD,
    Offset: DWORD,
    Buffer: PVOID,
    Size: DWORD,
) -> DWORD {
    ftGdiGetFontData_impl(FontGdi, Table, Offset, Buffer, Size)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetFontData(
    FontGdi: PFONTGDI,
    Table: DWORD,
    Offset: DWORD,
    Buffer: PVOID,
    Size: DWORD,
) -> DWORD {
    ftGdiGetFontData_impl(FontGdi, Table, Offset, Buffer, Size)
}

unsafe fn ftGdiGetFontData_impl(
    FontGdi: PFONTGDI,
    mut Table: DWORD,
    Offset: DWORD,
    Buffer: PVOID,
    mut Size: DWORD,
) -> DWORD {
    let mut Result: DWORD = GDI_ERROR;
    let Face: FT_Face = (*(*FontGdi).SharedFace).Face;

    IntLockFreeType();

    if FT_IS_SFNT(Face) != 0 {
        if Table != 0 {
            Table = Table >> 24
                | Table << 24
                | (Table >> 8 & 0xFF00)
                | (Table << 8 & 0xFF0000);
        }

        if Buffer.is_null() {
            Size = 0;
        }

        if !Buffer.is_null() && Size != 0 {
            let mut Needed: FT_ULong = 0;

            let Error: FT_Error = FT_Load_Sfnt_Table(
                Face,
                Table as FT_ULong,
                Offset as _,
                core::ptr::null_mut(),
                &mut Needed,
            );

            if Error == 0 && Needed < Size as FT_ULong {
                Size = Needed as DWORD;
            }
        }

        let mut SizeUlong: FT_ULong = Size as FT_ULong;
        if FT_Load_Sfnt_Table(
            Face,
            Table as FT_ULong,
            Offset as _,
            Buffer as *mut u8,
            &mut SizeUlong,
        ) == 0
        {
            Size = SizeUlong as DWORD;
            Result = Size;
        }
    }

    IntUnLockFreeType();

    Result
}

unsafe fn GetFontPenalty(
    LogFont: *const LOGFONTW,
    Otm: *const OUTLINETEXTMETRICW,
    _style_name: *const core::ffi::c_char,
) -> UINT {
    let mut Penalty: ULONG = 0;
    let mut Byte: BYTE;
    let Long: LONG;
    let mut fNeedScaling: BOOL = 0;
    let UserCharSet: BYTE = IntCharSetFromLangID(gusLanguageID);
    let TM: *const TEXTMETRICW = core::ptr::addr_of!((*Otm).otmTextMetrics);
    let mut ActualNameW: *mut WCHAR;

    Byte = (*LogFont).lfCharSet;

    if Byte != (*TM).tmCharSet {
        if Byte != DEFAULT_CHARSET as BYTE && Byte != ANSI_CHARSET as BYTE {
            Penalty += 65000;
        } else {
            if UserCharSet != (*TM).tmCharSet {
                Penalty += 100;

                if ANSI_CHARSET as BYTE != (*TM).tmCharSet {
                    Penalty += 100;
                }
            }
        }
    }

    Byte = (*LogFont).lfOutPrecision;
    match Byte as u32 {
        OUT_DEFAULT_PRECIS => {}
        OUT_DEVICE_PRECIS => {
            if ((*TM).tmPitchAndFamily as u32 & TMPF_DEVICE) == 0
                || ((*TM).tmPitchAndFamily as u32 & (TMPF_VECTOR | TMPF_TRUETYPE)) == 0
            {
                Penalty += 19000;
            }
        }
        _ => {
            if ((*TM).tmPitchAndFamily as u32 & (TMPF_VECTOR | TMPF_TRUETYPE)) != 0 {
                Penalty += 19000;
            }
        }
    }

    Byte = (*LogFont).lfPitchAndFamily & 0x0F;
    if Byte as u32 == DEFAULT_PITCH {
        Byte = VARIABLE_PITCH as BYTE;
    }
    if Byte as u32 == FIXED_PITCH {
        if ((*TM).tmPitchAndFamily as u32 & TMPF_FIXED_PITCH) != 0 {
            Penalty += 15000;
        }
    }
    if Byte as u32 == VARIABLE_PITCH {
        if ((*TM).tmPitchAndFamily as u32 & TMPF_FIXED_PITCH) == 0 {
            Penalty += 350;
        }
    }

    Byte = (*LogFont).lfPitchAndFamily & 0x0F;
    if Byte as u32 == DEFAULT_PITCH {
        if ((*TM).tmPitchAndFamily as u32 & TMPF_FIXED_PITCH) == 0 {
            Penalty += 1;
        }
    }

    ActualNameW = (Otm as ULONG_PTR + (*Otm).otmpFamilyName as ULONG_PTR) as *mut WCHAR;

    if (*LogFont).lfFaceName[0] != 0 {
        let mut Found: BOOL = 0;

        if Found == 0 {
            Found = (_wcsicmp((*LogFont).lfFaceName.as_ptr(), ActualNameW) == 0) as BOOL;
        }
        if Found == 0 {
            ActualNameW = (Otm as ULONG_PTR + (*Otm).otmpFaceName as ULONG_PTR) as *mut WCHAR;
            Found = (_wcsicmp((*LogFont).lfFaceName.as_ptr(), ActualNameW) == 0) as BOOL;
        }
        if Found == 0 {
            Penalty += 10000;
        }
    }

    Byte = (*LogFont).lfPitchAndFamily & 0xF0;
    if Byte as u32 != FF_DONTCARE {
        if Byte != ((*TM).tmPitchAndFamily & 0xF0) {
            Penalty += 9000;
        }
    }

    if ((*TM).tmPitchAndFamily as u32 & 0xF0) == FF_DONTCARE {
        Penalty += 8000;
    }

    if ((*TM).tmPitchAndFamily as u32 & (TMPF_TRUETYPE | TMPF_VECTOR)) == 0 {
        if (*LogFont).lfHeight != 0 {
            if labs((*LogFont).lfHeight as _) < (*TM).tmHeight as _ {
                Penalty += 600;
                Penalty += (150 * labs(((*TM).tmHeight as core::ffi::c_long) - labs((*LogFont).lfHeight as _))) as ULONG;
                fNeedScaling = 1;
            }
            if ((*TM).tmHeight as core::ffi::c_long) < labs((*LogFont).lfHeight as _) {
                Penalty += (150 * labs(((*TM).tmHeight as core::ffi::c_long) - labs((*LogFont).lfHeight as _))) as ULONG;
                fNeedScaling = 1;
            }
        }
    }

    match (*LogFont).lfPitchAndFamily as u32 & 0xF0 {
        FF_ROMAN | FF_MODERN | FF_SWISS => match (*TM).tmPitchAndFamily as u32 & 0xF0 {
            FF_DECORATIVE | FF_SCRIPT => {
                Penalty += 50;
            }
            _ => {}
        },
        FF_DECORATIVE | FF_SCRIPT => {
            match (*TM).tmPitchAndFamily as u32 & 0xF0 {
                FF_ROMAN | FF_MODERN | FF_SWISS => {
                    Penalty += 50;
                }
                _ => {}
            }
        }
        _ => {}
    }

    if (*LogFont).lfWidth != 0 {
        if (*LogFont).lfWidth != (*TM).tmAveCharWidth {
            Penalty += (50 * labs(((*LogFont).lfWidth - (*TM).tmAveCharWidth) as _)) as ULONG;

            if ((*TM).tmPitchAndFamily as u32 & (TMPF_TRUETYPE | TMPF_VECTOR)) == 0 {
                fNeedScaling = 1;
            }
        }
    }

    if fNeedScaling != 0 {
        Penalty += 50;
    }

    if (*LogFont).lfItalic == 0 && (*TM).tmItalic != 0 {
        Penalty += 40;
    } else if (*LogFont).lfItalic != 0 && (*TM).tmItalic == 0 {
        Penalty += 1;
    }

    if (*LogFont).lfOutPrecision as u32 == OUT_TT_PRECIS {
        if ((*TM).tmPitchAndFamily as u32 & TMPF_TRUETYPE) == 0 {
            Penalty += 4;
        }
    }

    Long = if (*LogFont).lfWeight == FW_DONTCARE as LONG {
        FW_NORMAL as LONG
    } else {
        (*LogFont).lfWeight
    };
    if Long != (*TM).tmWeight {
        Penalty += (3 * (labs((Long - (*TM).tmWeight) as _) / 10)) as ULONG;
    }

    if (*LogFont).lfUnderline == 0 && (*TM).tmUnderlined != 0 {
        Penalty += 3;
    }

    if (*LogFont).lfStrikeOut == 0 && (*TM).tmStruckOut != 0 {
        Penalty += 3;
    }

    if ((*TM).tmPitchAndFamily as u32 & (TMPF_TRUETYPE | TMPF_VECTOR)) == 0 {
        if (*LogFont).lfHeight != 0 && ((*TM).tmHeight as LONG) < (*LogFont).lfHeight {
            Penalty += (2 * labs(((*TM).tmHeight as LONG - (*LogFont).lfHeight) as _)) as ULONG;
        }
        if (*LogFont).lfHeight != 0 && ((*TM).tmHeight as LONG) > (*LogFont).lfHeight {
            Penalty += (1 * labs(((*TM).tmHeight as LONG - (*LogFont).lfHeight) as _)) as ULONG;
        }
    }

    if ((*TM).tmPitchAndFamily as u32 & TMPF_DEVICE) == 0 {
        Penalty += 2;
    }

    if (*TM).tmAveCharWidth >= 5 && (*TM).tmHeight >= 5 {
        if (*TM).tmAveCharWidth / (*TM).tmHeight >= 3 {
            Penalty += (((*TM).tmAveCharWidth / (*TM).tmHeight) - 2) as ULONG * 30;
        } else if (*TM).tmHeight / (*TM).tmAveCharWidth >= 3 {
            Penalty += (((*TM).tmHeight / (*TM).tmAveCharWidth) - 2) as ULONG * 30;
        }
    }

    Penalty
}

unsafe fn FindBestFontFromList(
    FontObj: *mut *mut FONTOBJ,
    MatchPenalty: *mut ULONG,
    LogFont: *const LOGFONTW,
    Head: PLIST_ENTRY,
) {
    let mut Penalty: ULONG;
    let mut Entry: PLIST_ENTRY;
    let mut CurrentEntry: *mut FONT_ENTRY;
    let mut FontGDI: *mut FONTGDI;
    let mut Otm: *mut OUTLINETEXTMETRICW = core::ptr::null_mut();
    let mut OtmSize: UINT;
    let mut OldOtmSize: UINT;
    let mut Face: FT_Face;

    OldOtmSize = 0x200;
    Otm = ExAllocatePoolWithTag(PagedPool, OldOtmSize as usize, GDITAG_TEXT) as *mut OUTLINETEXTMETRICW;

    Entry = (*Head).Flink;
    while Entry != Head {
        CurrentEntry = CONTAINING_RECORD!(Entry, FONT_ENTRY, ListEntry);

        FontGDI = (*CurrentEntry).Font;
        Face = (*(*FontGDI).SharedFace).Face;

        OtmSize = IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 1) as UINT;
        if OtmSize > OldOtmSize {
            if !Otm.is_null() {
                ExFreePoolWithTag(Otm as *mut core::ffi::c_void, GDITAG_TEXT);
            }
            Otm = ExAllocatePoolWithTag(PagedPool, OtmSize as usize, GDITAG_TEXT) as *mut OUTLINETEXTMETRICW;
        }

        if !Otm.is_null() {
            IntRequestFontSize(core::ptr::null_mut(), FontGDI, (*LogFont).lfWidth, (*LogFont).lfHeight);

            OtmSize = IntGetOutlineTextMetrics(FontGDI, OtmSize, Otm, 1) as UINT;
            if OtmSize == 0 {
                Entry = (*Entry).Flink;
                continue;
            }

            OldOtmSize = OtmSize;

            Penalty = GetFontPenalty(LogFont, Otm, (*Face).style_name);
            if *MatchPenalty == MAXULONG || Penalty < *MatchPenalty {
                *FontObj = GDIToObj!(FontGDI, FONT);
                *MatchPenalty = Penalty;
            }
        }

        Entry = (*Entry).Flink;
    }

    if !Otm.is_null() {
        ExFreePoolWithTag(Otm as *mut core::ffi::c_void, GDITAG_TEXT);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntFontType(Font: PFONTGDI) {
    let mut psfInfo: PS_FontInfoRec = core::mem::zeroed();
    let mut tmp_size: FT_ULong = 0;
    let Face: FT_Face = (*(*Font).SharedFace).Face;

    if FT_HAS_MULTIPLE_MASTERS(Face) != 0 {
        (*Font).FontObj.flFontType |= FO_MULTIPLEMASTER as FLONG ;
    }
    if FT_HAS_VERTICAL(Face) != 0 {
        (*Font).FontObj.flFontType |= FO_VERT_FACE as FLONG ;
    }
    if FT_IS_SCALABLE(Face) == 0 {
        (*Font).FontObj.flFontType |= FO_TYPE_RASTER as FLONG ;
    }
    if FT_IS_SFNT(Face) != 0 {
        (*Font).FontObj.flFontType |= FO_TYPE_TRUETYPE as FLONG ;
        if !FT_Get_Sfnt_Table(Face, ft_sfnt_post).is_null() {
            (*Font).FontObj.flFontType |= FO_POSTSCRIPT as FLONG ;
        }
    }
    if FT_Get_PS_Font_Info(Face, &mut psfInfo) == 0 {
        (*Font).FontObj.flFontType |= FO_POSTSCRIPT as FLONG ;
    }
    if FT_Load_Sfnt_Table(Face, TTAG_CFF, 0, core::ptr::null_mut(), &mut tmp_size) == 0 {
        (*Font).FontObj.flFontType |= (FO_CFF | FO_POSTSCRIPT) as FLONG ;
    }
}

unsafe fn MatchFontName(
    SharedFace: PSHARED_FACE,
    Name1: PUNICODE_STRING,
    NameID: FT_UShort,
    LangID: FT_UShort,
) -> BOOL {
    let mut Name2: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name2) as _ , core::ptr::null_mut());
    let Status = IntGetFontLocalizedName(&mut Name2, SharedFace, NameID, LangID);

    if NT_SUCCESS(Status) {
        if RtlCompareUnicodeString(Name1, &mut Name2, 1u8) == 0 {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(Name2) as _ );
            return 1;
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(Name2) as _ );
    }

    0
}

unsafe fn MatchFontNames(SharedFace: PSHARED_FACE, lfFaceName: LPCWSTR) -> BOOL {
    let mut Name1: _UNICODE_STRING = core::mem::zeroed();

    if *lfFaceName == 0 {
        return 0;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name1) as _ , lfFaceName);

    if MatchFontName(SharedFace, &mut Name1, TT_NAME_ID_FONT_FAMILY as FT_UShort, LANG_ENGLISH as FT_UShort) != 0
        || MatchFontName(SharedFace, &mut Name1, TT_NAME_ID_FULL_NAME as FT_UShort, LANG_ENGLISH as FT_UShort) != 0
    {
        return 1;
    }

    if PRIMARYLANGID(gusLanguageID as _ ) as u32 != LANG_ENGLISH as u32 {
        if MatchFontName(SharedFace, &mut Name1, TT_NAME_ID_FONT_FAMILY as FT_UShort, gusLanguageID) != 0
            || MatchFontName(SharedFace, &mut Name1, TT_NAME_ID_FULL_NAME as FT_UShort, gusLanguageID) != 0
        {
            return 1;
        }
    }

    0
}

unsafe fn IntPopulateTextObj(
    TextObj: PTEXTOBJ,
    FontGdi: PFONTGDI,
    pFontObj: *mut FONTOBJ,
    pLogFont: *const LOGFONTW,
    SubstitutedLogFont: PLOGFONTW,
) {
    (*TextObj).Font = pFontObj;
    (*TextObj).TextFace[0] = 0;

    let SharedFace: PSHARED_FACE = (*FontGdi).SharedFace;
    if MatchFontNames(SharedFace, (*SubstitutedLogFont).lfFaceName.as_ptr()) != 0 {
        w32r_RtlStringCchCopyW(
            (*TextObj).TextFace.as_mut_ptr(),
            (*TextObj).TextFace.len(),
            (*pLogFont).lfFaceName.as_ptr(),
        );
    } else {
        let mut Name: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(core::ptr::addr_of_mut!(Name) as _ , core::ptr::null_mut());
        let Status: NTSTATUS = IntGetFontLocalizedName(
            &mut Name,
            SharedFace,
            TT_NAME_ID_FONT_FAMILY as FT_UShort,
            gusLanguageID,
        );
        if NT_SUCCESS(Status) {
            IntUnicodeStringToBuffer(
                (*TextObj).TextFace.as_mut_ptr(),
                core::mem::size_of_val(&(*TextObj).TextFace),
                &mut Name,
            );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(Name) as _ );
        }
    }

    (*(*TextObj).Font).iUniq = 1;
    (*TextObj).fl |= TEXTOBJECT_INIT as u32;
}

#[no_mangle]
pub unsafe extern "C" fn IntPopulateFontGdi(
    FontGdi: PFONTGDI,
    pFontObj: *const FONTOBJ,
    pLogFont: *const LOGFONTW,
) {
    IntFontType(FontGdi);
    (*FontGdi).flType = (*pFontObj).flFontType;
    (*FontGdi).RequestUnderline = if (*pLogFont).lfUnderline != 0 { 0xFF } else { 0 };
    (*FontGdi).RequestStrikeOut = if (*pLogFont).lfStrikeOut != 0 { 0xFF } else { 0 };
    (*FontGdi).RequestItalic = if (*pLogFont).lfItalic != 0 { 0xFF } else { 0 };
    if (*pLogFont).lfWeight != FW_DONTCARE as i32 {
        (*FontGdi).RequestWeight = (*pLogFont).lfWeight;
    } else {
        (*FontGdi).RequestWeight = FW_NORMAL as i32;
    }
}

unsafe fn FontLookUp_LookUp(pLogFont: *const LOGFONTW) -> PFONT_LOOKUP_CACHE {
    let pHead: PLIST_ENTRY = core::ptr::addr_of_mut!(s_FontLookupCacheList);
    let mut Entry: PLIST_ENTRY = (*pHead).Flink;
    while Entry != pHead {
        let pEntry = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_FONT_LOOKUP_CACHE, ListEntry) as isize))
            as *mut _FONT_LOOKUP_CACHE;
        if RtlEqualMemory(
            core::ptr::addr_of!((*pEntry).LogFont) as *const core::ffi::c_void,
            pLogFont as *const core::ffi::c_void,
            core::mem::size_of::<LOGFONTW>(),
        ) {
            RemoveEntryList(core::ptr::addr_of_mut!((*pEntry).ListEntry));
            InsertHeadList(
                core::ptr::addr_of_mut!(s_FontLookupCacheList),
                core::ptr::addr_of_mut!((*pEntry).ListEntry),
            );
            return pEntry;
        }
        Entry = (*Entry).Flink;
    }
    core::ptr::null_mut()
}

unsafe fn FontLookUp_Add(LogFont: *const LOGFONTW, SharedFace: PSHARED_FACE, pFontObj: *mut FONTOBJ) {
    if s_FontLookupCacheCount >= (MAX_FONT_LOOKUP_CACHE as _) {
        let OldestEntry = RemoveTailList(core::ptr::addr_of_mut!(s_FontLookupCacheList));
        let pOldCache = (OldestEntry as *mut u8)
            .sub(core::mem::offset_of!(_FONT_LOOKUP_CACHE, ListEntry)) as PFONT_LOOKUP_CACHE;
        FontLookUp_Destroy(pOldCache);
        s_FontLookupCacheCount -= 1;
    }

    let pEntry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_FONT_LOOKUP_CACHE>() as SIZE_T as _ ,
        TAG_FONT,
    ) as PFONT_LOOKUP_CACHE;
    if !pEntry.is_null() {
        core::ptr::copy_nonoverlapping(
            LogFont,
            core::ptr::addr_of_mut!((*pEntry).LogFont),
            1,
        );
        (*pEntry).SharedFace = SharedFace;
        SharedFace_AddRef(SharedFace);
        (*pEntry).pFontObj = pFontObj;
        InsertHeadList(
            core::ptr::addr_of_mut!(s_FontLookupCacheList),
            core::ptr::addr_of_mut!((*pEntry).ListEntry),
        );
        s_FontLookupCacheCount += 1;
    }
}

unsafe fn IntRealizeFont(mut pLogFont: *const LOGFONTW, TextObj: PTEXTOBJ) -> PSHARED_FACE {
    let mut LogFont: LOGFONTW = *pLogFont;
    RtlZeroMemory(
        core::ptr::addr_of_mut!(LogFont.lfFaceName) as *mut core::ffi::c_void,
        core::mem::size_of_val(&LogFont.lfFaceName),
    );
    w32r_RtlStringCchCopyW(
        LogFont.lfFaceName.as_mut_ptr(),
        LogFont.lfFaceName.len(),
        (*pLogFont).lfFaceName.as_ptr(),
    );
    pLogFont = core::ptr::addr_of!(LogFont);

    let mut SubstitutedLogFont: LOGFONTW = *pLogFont;
    SubstituteFontRecurse(&mut SubstitutedLogFont);

    let mut pFontObj: *mut FONTOBJ = core::ptr::null_mut();
    let pLookUp: PFONT_LOOKUP_CACHE = FontLookUp_LookUp(pLogFont);
    if !pLookUp.is_null() {
        pFontObj = (*pLookUp).pFontObj;
    } else {
        let mut MatchPenalty: ULONG = 0xFFFFFFFF;
        let Win32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _ ;

        IntLockProcessPrivateFonts(Win32Process as _ );
        FindBestFontFromList(
            &mut pFontObj,
            &mut MatchPenalty,
            &SubstitutedLogFont,
            core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead),
        );
        IntUnLockProcessPrivateFonts(Win32Process as _ );

        FindBestFontFromList(
            &mut pFontObj,
            &mut MatchPenalty,
            &SubstitutedLogFont,
            core::ptr::addr_of_mut!(g_FontListHead),
        );
    }

    if pFontObj.is_null() {
        static mut s_NoFontWarned: LONG = 0;
        let _ = InterlockedCompareExchange(core::ptr::addr_of_mut!(s_NoFontWarned), 1, 0);
        return core::ptr::null_mut();
    }

    let pFontGDI: PFONTGDI = ObjToGDI!(pFontObj, FONT);

    if !TextObj.is_null() {
        IntPopulateTextObj(TextObj, pFontGDI, pFontObj, pLogFont, &mut SubstitutedLogFont);
    }

    IntPopulateFontGdi(pFontGDI, pFontObj, pLogFont);

    if pLookUp.is_null() {
        FontLookUp_Add(pLogFont, (*pFontGDI).SharedFace, pFontObj);
    }

    (*pFontGDI).SharedFace
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn TextIntRealizeFont(FontHandle: HFONT, pTextObj: PTEXTOBJ) -> NTSTATUS {
    TextIntRealizeFont_impl(FontHandle, pTextObj)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn TextIntRealizeFont(FontHandle: HFONT, pTextObj: PTEXTOBJ) -> NTSTATUS {
    TextIntRealizeFont_impl(FontHandle, pTextObj)
}

unsafe fn TextIntRealizeFont_impl(FontHandle: HFONT, pTextObj: PTEXTOBJ) -> NTSTATUS {
    let TextObj: PTEXTOBJ;
    let SharedFace: PSHARED_FACE;

    if pTextObj.is_null() {
        TextObj = w32r_TEXTOBJ_LockText(FontHandle);
        if TextObj.is_null() {
            return STATUS_INVALID_HANDLE;
        }

        if (*TextObj).fl & TEXTOBJECT_INIT != 0 {
            w32r_TEXTOBJ_UnlockText(TextObj);
            return STATUS_SUCCESS;
        }
    } else {
        TextObj = pTextObj;
    }

    IntLockFreeType();
    SharedFace = IntRealizeFont(
        core::ptr::addr_of!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont),
        TextObj,
    );
    IntUnLockFreeType();

    if pTextObj.is_null() {
        w32r_TEXTOBJ_UnlockText(TextObj);
    }

    if !SharedFace.is_null() {
        STATUS_SUCCESS
    } else {
        STATUS_NOT_FOUND
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetFullFileName(
    NameInfo: POBJECT_NAME_INFORMATION,
    Size: ULONG,
    FileName: PUNICODE_STRING,
) -> BOOL {
    IntGetFullFileNameImpl(NameInfo, Size, FileName)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetFullFileName(
    NameInfo: POBJECT_NAME_INFORMATION,
    Size: ULONG,
    FileName: PUNICODE_STRING,
) -> BOOL {
    IntGetFullFileNameImpl(NameInfo, Size, FileName)
}

unsafe fn IntGetFullFileNameImpl(
    NameInfo: POBJECT_NAME_INFORMATION,
    Size: ULONG,
    FileName: PUNICODE_STRING,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Desired: ULONG = 0;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.ObjectName = FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenFile(
        &mut hFile,
        0,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        0,
    );

    if !NT_SUCCESS(Status) {
        return 0u8 as BOOL;
    }

    Status = ZwQueryObject(
        hFile,
        _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
        NameInfo as *mut core::ffi::c_void,
        Size,
        &mut Desired,
    );
    ZwClose(hFile);
    if !NT_SUCCESS(Status) {
        return 0u8 as BOOL;
    }

    1u8 as BOOL
}

unsafe fn EqualFamilyInfo(pInfo1: *const FONTFAMILYINFO, pInfo2: *const FONTFAMILYINFO) -> BOOL {
    let pLog1: *const ENUMLOGFONTEXW = core::ptr::addr_of!((*pInfo1).EnumLogFontEx);
    let pLog2: *const ENUMLOGFONTEXW = core::ptr::addr_of!((*pInfo2).EnumLogFontEx);
    let plf1: *const LOGFONTW = core::ptr::addr_of!((*pLog1).elfLogFont);
    let plf2: *const LOGFONTW = core::ptr::addr_of!((*pLog2).elfLogFont);

    if _wcsicmp(
        core::ptr::addr_of!((*plf1).lfFaceName) as *const u16,
        core::ptr::addr_of!((*plf2).lfFaceName) as *const u16,
    ) != 0
    {
        return 0u8 as BOOL;
    }

    if _wcsicmp(
        core::ptr::addr_of!((*pLog1).elfStyle) as *const u16,
        core::ptr::addr_of!((*pLog2).elfStyle) as *const u16,
    ) != 0
    {
        return 0u8 as BOOL;
    }

    1u8 as BOOL
}

unsafe fn IntAddNameFromFamInfo(psz: LPWSTR, FamInfo: *mut FONTFAMILYINFO) {

    wcscat(psz, core::ptr::addr_of!((*FamInfo).EnumLogFontEx.elfLogFont.lfFaceName) as *const u16);
    if (*FamInfo).EnumLogFontEx.elfStyle[0] != 0 &&
        _wcsicmp(core::ptr::addr_of!((*FamInfo).EnumLogFontEx.elfStyle) as *const u16,
                 w!("Regular").as_ptr() as *const u16) != 0
    {
        wcscat(psz, w!(" ").as_ptr() as *const u16);
        wcscat(psz, core::ptr::addr_of!((*FamInfo).EnumLogFontEx.elfStyle) as *const u16);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiGetFontResourceInfo(
    FileName: PUNICODE_STRING,
    pBuffer: PVOID,
    pdwBytes: *mut DWORD,
    dwType: DWORD,
) -> BOOL {
    IntGdiGetFontResourceInfoImpl(FileName, pBuffer, pdwBytes, dwType)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiGetFontResourceInfo(
    FileName: PUNICODE_STRING,
    pBuffer: PVOID,
    pdwBytes: *mut DWORD,
    dwType: DWORD,
) -> BOOL {
    IntGdiGetFontResourceInfoImpl(FileName, pBuffer, pdwBytes, dwType)
}

unsafe fn IntGdiGetFontResourceInfoImpl(
    FileName: PUNICODE_STRING,
    pBuffer: PVOID,
    pdwBytes: *mut DWORD,
    dwType: DWORD,
) -> BOOL {

    let mut EntryFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut NameInfo1: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut NameInfo2: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY;
    let mut FontEntry: *mut FONT_ENTRY;
    let mut Size: ULONG;
    let mut i: ULONG;
    let mut Count: ULONG;
    let mut pbBuffer: LPBYTE;
    let mut IsEqual: BOOL;
    let mut FamInfo: *mut FONTFAMILYINFO = core::ptr::null_mut();
    let MaxFamInfo: ULONG = 64;
    let MAX_FAM_INFO_BYTES: ULONG =
        core::mem::size_of::<FONTFAMILYINFO>() as ULONG * MaxFamInfo;
    let mut bSuccess: BOOL;
    let NAMEINFO_SIZE: ULONG = core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG
        + MAX_PATH * core::mem::size_of::<u16>() as ULONG;

    loop {
        NameInfo1 = ExAllocatePoolWithTag(PagedPool, NAMEINFO_SIZE as usize, TAG_FINF)
            as POBJECT_NAME_INFORMATION;
        if NameInfo1.is_null() {
            break;
        }

        if IntGetFullFileName(NameInfo1, NAMEINFO_SIZE, FileName) == 0 {
            break;
        }

        NameInfo2 = ExAllocatePoolWithTag(PagedPool, NAMEINFO_SIZE as usize, TAG_FINF)
            as POBJECT_NAME_INFORMATION;
        if NameInfo2.is_null() {
            break;
        }

        FamInfo = ExAllocatePoolWithTag(PagedPool, MAX_FAM_INFO_BYTES as usize, TAG_FINF)
            as *mut FONTFAMILYINFO;
        break;
    }

    if NameInfo1.is_null() || NameInfo2.is_null() || FamInfo.is_null() {
        if !NameInfo2.is_null() {
            ExFreePoolWithTag(NameInfo2 as PVOID, TAG_FINF);
        }
        if !NameInfo1.is_null() {
            ExFreePoolWithTag(NameInfo1 as PVOID, TAG_FINF);
        }
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0u8 as BOOL;
    }

    Count = 0;

    IntLockFreeType();
    ListEntry = g_FontListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!(g_FontListHead) {
        FontEntry = (ListEntry as LPBYTE)
            .offset(-(core::mem::offset_of!(FONT_ENTRY, ListEntry) as isize))
            as *mut FONT_ENTRY;

        if (*(*FontEntry).Font).Filename.is_null() {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(EntryFileName) as _ ,
            (*(*FontEntry).Font).Filename,
        );
        if IntGetFullFileName(NameInfo2, NAMEINFO_SIZE, core::ptr::addr_of_mut!(EntryFileName)) == 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*NameInfo1).Name),
            core::ptr::addr_of_mut!((*NameInfo2).Name),
            0u8,
        ) == 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        IsEqual = 0u8 as BOOL;
        FontFamilyFillInfo(
            FamInfo.offset(Count as isize),
            (*FontEntry).FaceName.Buffer,
            core::ptr::null_mut(),
            (*FontEntry).Font,
        );
        i = 0;
        while i < Count {
            if EqualFamilyInfo(FamInfo.offset(i as isize), FamInfo.offset(Count as isize)) != 0 {
                IsEqual = 1u8 as BOOL;
                break;
            }
            i += 1;
        }
        if IsEqual == 0 {
            Count += 1;
            if Count >= MaxFamInfo {
                break;
            }
        }

        ListEntry = (*ListEntry).Flink;
    }
    IntUnLockFreeType();

    ExFreePoolWithTag(NameInfo1 as PVOID, TAG_FINF);
    ExFreePoolWithTag(NameInfo2 as PVOID, TAG_FINF);

    if Count == 0 && dwType != 5 {
        ExFreePoolWithTag(FamInfo as PVOID, TAG_FINF);
        return 0u8 as BOOL;
    }

    bSuccess = 0u8 as BOOL;
    match dwType {
        0 => {
            Size = core::mem::size_of::<DWORD>() as ULONG;
            if *pdwBytes == 0 {
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else if !pBuffer.is_null() {
                if *pdwBytes >= Size {
                    *(pBuffer as *mut DWORD) = Count;
                }
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            }
        }
        1 => {
            Size = 0;
            i = 0;
            while i < Count {
                if i > 0 {
                    Size += 3;
                }
                Size += wcslen(
                    core::ptr::addr_of!(
                        (*FamInfo.offset(i as isize)).EnumLogFontEx.elfLogFont.lfFaceName
                    ) as *const u16,
                ) as ULONG;
                if (*FamInfo.offset(i as isize)).EnumLogFontEx.elfStyle[0] != 0
                    && _wcsicmp(
                        core::ptr::addr_of!(
                            (*FamInfo.offset(i as isize)).EnumLogFontEx.elfStyle
                        ) as *const u16,
                        w!("Regular").as_ptr() as *const u16,
                    ) != 0
                {
                    Size += 1 + wcslen(
                        core::ptr::addr_of!(
                            (*FamInfo.offset(i as isize)).EnumLogFontEx.elfStyle
                        ) as *const u16,
                    ) as ULONG;
                }
                i += 1;
            }
            Size += 2;
            Size *= core::mem::size_of::<u16>() as ULONG;

            if *pdwBytes == 0 {
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else if !pBuffer.is_null() {
                if *pdwBytes >= Size {
                    let psz: *mut u16 = pBuffer as *mut u16;
                    *psz = 0;
                    i = 0;
                    while i < Count {
                        if i > 0 {
                            wcscat(psz, w!(" & ").as_ptr() as *const u16);
                        }
                        IntAddNameFromFamInfo(psz, FamInfo.offset(i as isize));
                        i += 1;
                    }
                    *psz.offset(wcslen(psz) as isize + 1) = 0;
                    *pdwBytes = Size;
                    bSuccess = 1u8 as BOOL;
                } else {
                    *pdwBytes = 1024;
                }
            }
        }
        2 => {
            Size = Count * core::mem::size_of::<LOGFONTW>() as ULONG;
            if *pdwBytes == 0 {
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else if !pBuffer.is_null() {
                if *pdwBytes >= Size {
                    pbBuffer = pBuffer as LPBYTE;
                    i = 0;
                    while i < Count {
                        (*FamInfo.offset(i as isize)).EnumLogFontEx.elfLogFont.lfWidth = 0;
                        RtlCopyMemory(
                            pbBuffer as PVOID,
                            core::ptr::addr_of!(
                                (*FamInfo.offset(i as isize)).EnumLogFontEx.elfLogFont
                            ) as PVOID,
                            core::mem::size_of::<LOGFONTW>(),
                        );
                        pbBuffer = pbBuffer.add(core::mem::size_of::<LOGFONTW>());
                        i += 1;
                    }
                }
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else {
                *pdwBytes = 1024;
            }
        }
        3 => {
            Size = core::mem::size_of::<DWORD>() as ULONG;
            if *pdwBytes == 0 {
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else if !pBuffer.is_null() {
                if *pdwBytes >= Size {
                    *(pBuffer as *mut DWORD) = 1;
                }
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            }
        }
        4 => {
            if (*FileName).Length as ULONG >= 4 * core::mem::size_of::<u16>() as ULONG {
                let pch: LPWSTR = (*FileName).Buffer.offset(4);
                let Length: DWORD =
                    (*FileName).Length as DWORD - 4 * core::mem::size_of::<u16>() as DWORD;

                Size = Length + core::mem::size_of::<u16>() as ULONG;
                if *pdwBytes == 0 {
                    *pdwBytes = Size;
                    bSuccess = 1u8 as BOOL;
                } else if !pBuffer.is_null() {
                    if *pdwBytes >= Size {
                        RtlCopyMemory(pBuffer, pch as PVOID, Size as usize);
                    }
                    *pdwBytes = Size;
                    bSuccess = 1u8 as BOOL;
                }
            }
        }
        5 => {
            Size = core::mem::size_of::<BOOL>() as ULONG;
            if *pdwBytes == 0 {
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            } else if !pBuffer.is_null() {
                if *pdwBytes >= Size {
                    *(pBuffer as *mut BOOL) = if Count == 0 { 1u8 as BOOL } else { 0u8 as BOOL };
                }
                *pdwBytes = Size;
                bSuccess = 1u8 as BOOL;
            }
        }
        _ => {}
    }
    ExFreePoolWithTag(FamInfo as PVOID, TAG_FINF);

    bSuccess
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiRealizationInfo(Font: PFONTGDI, Info: PREALIZATION_INFO) -> BOOL {
    ftGdiRealizationInfo_impl(Font, Info)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiRealizationInfo(Font: PFONTGDI, Info: PREALIZATION_INFO) -> BOOL {
    ftGdiRealizationInfo_impl(Font, Info)
}

#[inline]
unsafe fn ftGdiRealizationInfo_impl(Font: PFONTGDI, Info: PREALIZATION_INFO) -> BOOL {
    let Face = (*(*Font).SharedFace).Face;

    if FT_HAS_FIXED_SIZES(Face) != 0 {
        (*Info).iTechnology = RI_TECH_BITMAP as _;
    } else {
        if FT_IS_SCALABLE(Face) != 0 {
            (*Info).iTechnology = RI_TECH_SCALABLE as _;
        } else {
            (*Info).iTechnology = RI_TECH_FIXED as _;
        }
    }

    (*Info).iUniq = (*Font).FontObj.iUniq;
    (*Info).dwUnknown = -1i32 as _;
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ftGdiGetKerningPairs(
    Font: PFONTGDI,
    cPairs: DWORD,
    pKerningPair: LPKERNINGPAIR,
) -> DWORD {
    ftGdiGetKerningPairs_impl(Font, cPairs, pKerningPair)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ftGdiGetKerningPairs(
    Font: PFONTGDI,
    cPairs: DWORD,
    pKerningPair: LPKERNINGPAIR,
) -> DWORD {
    ftGdiGetKerningPairs_impl(Font, cPairs, pKerningPair)
}

unsafe fn ftGdiGetKerningPairs_impl(
    Font: PFONTGDI,
    cPairs: DWORD,
    pKerningPair: LPKERNINGPAIR,
) -> DWORD {
    let mut Count: DWORD = 0;
    let mut i: INT = 0;
    let face: FT_Face = (*(*Font).SharedFace).Face;

    if FT_HAS_KERNING(face) != 0 && (*(*face).charmap).encoding == FT_ENCODING_UNICODE {
        let mut previous_index: FT_UInt = 0;
        let mut glyph_index: FT_UInt = 0;
        let mut char_code: FT_ULong;
        let mut char_previous: FT_ULong;
        let mut delta: FT_Vector = core::mem::zeroed();

        char_code = FT_Get_First_Char(face, &mut glyph_index);
        char_previous = char_code;

        IntLockFreeType();

        while glyph_index != 0 {
            if previous_index != 0 && glyph_index != 0 {
                FT_Get_Kerning(
                    face,
                    previous_index,
                    glyph_index,
                    FT_KERNING_DEFAULT,
                    &mut delta,
                );

                if !pKerningPair.is_null() && cPairs != 0 {
                    (*pKerningPair.offset(i as isize)).wFirst = char_previous as u16;
                    (*pKerningPair.offset(i as isize)).wSecond = char_code as u16;
                    (*pKerningPair.offset(i as isize)).iKernAmount = delta.x as i32;
                    i += 1;
                    if i as DWORD == cPairs {
                        break;
                    }
                }
                Count += 1;
            }
            previous_index = glyph_index;
            char_previous = char_code;
            char_code = FT_Get_Next_Char(face, char_code, &mut glyph_index);
        }
        IntUnLockFreeType();
    }
    Count
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetFontFamilyInfo(
    Dc: HDC,
    SafeLogFont: *const LOGFONTW,
    SafeInfo: PFONTFAMILYINFO,
    InfoCount: LONG,
) -> LONG {
    IntGetFontFamilyInfo_impl(Dc, SafeLogFont, SafeInfo, InfoCount)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetFontFamilyInfo(
    Dc: HDC,
    SafeLogFont: *const LOGFONTW,
    SafeInfo: PFONTFAMILYINFO,
    InfoCount: LONG,
) -> LONG {
    IntGetFontFamilyInfo_impl(Dc, SafeLogFont, SafeInfo, InfoCount)
}

unsafe fn IntGetFontFamilyInfo_impl(
    Dc: HDC,
    SafeLogFont: *const LOGFONTW,
    SafeInfo: PFONTFAMILYINFO,
    InfoCount: LONG,
) -> LONG {
    let mut AvailCount: LONG = 0;
    let Win32Process: PPROCESSINFO;

    IntLockFreeType();
    if GetFontFamilyInfoForList(
        SafeLogFont,
        SafeInfo,
        core::ptr::null_mut(),
        &mut AvailCount,
        InfoCount,
        core::ptr::addr_of_mut!(g_FontListHead),
    ) == 0
    {
        IntUnLockFreeType();
        return -1;
    }

    Win32Process = PsGetCurrentProcessWin32Process() as _ ;
    IntLockProcessPrivateFonts(Win32Process as _ );
    if GetFontFamilyInfoForList(
        SafeLogFont,
        SafeInfo,
        core::ptr::null_mut(),
        &mut AvailCount,
        InfoCount,
        core::ptr::addr_of_mut!((*Win32Process).PrivateFontListHead),
    ) == 0
    {
        IntUnLockProcessPrivateFonts(Win32Process as _ );
        IntUnLockFreeType();
        return -1;
    }
    IntUnLockProcessPrivateFonts(Win32Process as _ );
    IntUnLockFreeType();

    if GetFontFamilyInfoForSubstitutes(SafeLogFont, SafeInfo, &mut AvailCount, InfoCount) == 0 {
        return -1;
    }

    AvailCount
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetFontFamilyInfo(
    Dc: HDC,
    UnsafeLogFont: *const LOGFONTW,
    UnsafeInfo: PFONTFAMILYINFO,
    UnsafeInfoCount: *mut LONG,
) -> LONG {
    let mut Status: NTSTATUS;
    let mut LogFont: LOGFONTW = core::mem::zeroed();
    let Info: PFONTFAMILYINFO;
    let mut GotCount: LONG;
    let AvailCount: LONG;
    let mut SafeInfoCount: LONG = 0;
    let mut DataSize: ULONG = 0;

    if UnsafeLogFont.is_null() || UnsafeInfo.is_null() || UnsafeInfoCount.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(SafeInfoCount) as *mut core::ffi::c_void,
        UnsafeInfoCount as *const core::ffi::c_void,
        core::mem::size_of::<LONG>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }
    GotCount = 0;
    Status = MmCopyToCaller(
        UnsafeInfoCount as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(GotCount) as *const core::ffi::c_void,
        core::mem::size_of::<LONG>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }
    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(LogFont) as *mut core::ffi::c_void,
        UnsafeLogFont as *const core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }
    if SafeInfoCount <= 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    Status = w32r_RtlULongMult(
        SafeInfoCount as ULONG,
        core::mem::size_of::<FONTFAMILYINFO>() as ULONG,
        core::ptr::addr_of_mut!(DataSize),
    );
    if !NT_SUCCESS(Status) || DataSize as LONG_MAX_TYPE > LONG_MAX as LONG_MAX_TYPE {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }
    Info = ExAllocatePoolWithTag(PagedPool, DataSize as usize, GDITAG_TEXT) as PFONTFAMILYINFO;
    if Info.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return -1;
    }

    AvailCount = IntGetFontFamilyInfo(Dc, core::ptr::addr_of!(LogFont), Info, SafeInfoCount);
    GotCount = if AvailCount < SafeInfoCount { AvailCount } else { SafeInfoCount };
    SafeInfoCount = AvailCount;

    if GotCount > 0 {
        Status = w32r_RtlULongMult(
            GotCount as ULONG,
            core::mem::size_of::<FONTFAMILYINFO>() as ULONG,
            core::ptr::addr_of_mut!(DataSize),
        );
        if !NT_SUCCESS(Status) || DataSize as LONG_MAX_TYPE > LONG_MAX as LONG_MAX_TYPE {
            ExFreePoolWithTag(Info as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return -1;
        }
        Status = MmCopyToCaller(
            UnsafeInfo as *mut core::ffi::c_void,
            Info as *const core::ffi::c_void,
            DataSize,
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(Info as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return -1;
        }
        Status = MmCopyToCaller(
            UnsafeInfoCount as *mut core::ffi::c_void,
            core::ptr::addr_of!(SafeInfoCount) as *const core::ffi::c_void,
            core::mem::size_of::<LONG>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(Info as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return -1;
        }
    }

    ExFreePoolWithTag(Info as *mut core::ffi::c_void, GDITAG_TEXT);

    GotCount
}

unsafe fn IntGetTextDisposition(
    pX64: *mut LONGLONG,
    pY64: *mut LONGLONG,
    mut String: PCWCH,
    Count: INT,
    Dx: *const INT,
    Cache: PFONT_CACHE_ENTRY,
    fuOptions: UINT,
    bNoTransform: BOOL,
    pChain: PFONTLINK_CHAIN,
) -> BOOL {
    let mut X64: LONGLONG = 0;
    let mut Y64: LONGLONG = 0;
    let mut i: INT;
    let mut glyph_index: INT;
    let mut realglyph: FT_BitmapGlyph;
    let mut face: FT_Face = (*Cache).Hashed.Face;
    let use_kerning: BOOL = FT_HAS_KERNING(face);
    let mut previous: ULONG = 0;
    let mut delta: FT_Vector = core::mem::zeroed();
    let mut vec: FT_Vector = core::mem::zeroed();
    let mut ch0: DWORD;
    let mut ch1: DWORD;

    i = 0;
    while i < Count {
        ch0 = *String as DWORD;
        String = String.add(1);
        if IS_HIGH_SURROGATE(ch0) {
            i += 1;
            if i >= Count {
                return 1;
            }

            ch1 = *String as DWORD;
            String = String.add(1);
            if IS_LOW_SURROGATE(ch1) {
                ch0 = Utf32FromSurrogatePair(ch0, ch1);
            }
        }

        glyph_index = FontLink_Chain_FindGlyph(
            pChain,
            Cache,
            core::ptr::addr_of_mut!(face),
            ch0,
            (fuOptions & ETO_GLYPH_INDEX) as BOOL,
        ) as INT;
        (*Cache).Hashed.GlyphIndex = glyph_index;

        realglyph = IntGetRealGlyph(Cache);
        if realglyph.is_null() {
            return 0;
        }

        if use_kerning != 0 && previous != 0 && glyph_index != 0 {
            FT_Get_Kerning(face, previous, glyph_index as FT_UInt, 0, core::ptr::addr_of_mut!(delta));
            X64 += delta.x as LONGLONG;
            Y64 -= delta.y as LONGLONG;
        }

        if Dx.is_null() {
            X64 += ((*realglyph).root.advance.x >> 10) as LONGLONG;
            Y64 -= ((*realglyph).root.advance.y >> 10) as LONGLONG;
        } else if (fuOptions & ETO_PDY) != 0 {
            vec.x = ((*Dx.add((2 * i + 0) as usize)) << 6) as FT_Pos;
            vec.y = ((*Dx.add((2 * i + 1) as usize)) << 6) as FT_Pos;
            if bNoTransform == 0 {
                FT_Vector_Transform(core::ptr::addr_of_mut!(vec), core::ptr::addr_of!((*Cache).Hashed.matTransform));
            }
            X64 += vec.x as LONGLONG;
            Y64 -= vec.y as LONGLONG;
        } else {
            vec.x = ((*Dx.add(i as usize)) << 6) as FT_Pos;
            vec.y = 0;
            if bNoTransform == 0 {
                FT_Vector_Transform(core::ptr::addr_of_mut!(vec), core::ptr::addr_of!((*Cache).Hashed.matTransform));
            }
            X64 += vec.x as LONGLONG;
            Y64 -= vec.y as LONGLONG;
        }

        previous = glyph_index as ULONG;

        i += 1;
    }

    *pX64 = X64;
    *pY64 = Y64;
    1
}

#[no_mangle]
pub unsafe extern "system" fn IntExtTextOutW(
    dc: PDC,
    XStart: INT,
    YStart: INT,
    mut fuOptions: UINT,
    lprc: PRECTL,
    mut String: PCWCH,
    Count: INT,
    Dx: *const INT,
    _dwCodePage: DWORD,
) -> BOOL {
    let pdcattr: PDC_ATTR;
    let psoDest: *mut SURFOBJ;
    let mut psoGlyph: *mut SURFOBJ;
    let psurf: *mut SURFACE;
    let mut glyph_index: INT;
    let mut i: INT;
    let mut face: FT_Face;
    let mut realglyph: FT_BitmapGlyph;
    let mut X64: LONGLONG;
    let mut Y64: LONGLONG;
    let mut RealXStart64: LONGLONG;
    let mut RealYStart64: LONGLONG;
    let mut DeltaX64: LONGLONG = 0;
    let mut DeltaY64: LONGLONG = 0;
    let mut previous: ULONG;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut MaskRect: RECTL = core::mem::zeroed();
    let mut hbmGlyph: HBITMAP = core::ptr::null_mut();
    let mut glyphSize: SIZEL = core::mem::zeroed();
    let FontObj: *mut FONTOBJ;
    let FontGDI: PFONTGDI;
    let mut TextObj: PTEXTOBJ = core::ptr::null_mut();
    let mut exloRGB2Dst: EXLATEOBJ = core::mem::zeroed();
    let mut exloDst2RGB: EXLATEOBJ = core::mem::zeroed();
    let mut Start: POINT = core::mem::zeroed();
    let pmxWorldToDevice: PMATRIX;
    let mut delta: FT_Vector = core::mem::zeroed();
    let mut vecAscent64: FT_Vector = core::mem::zeroed();
    let mut vecDescent64: FT_Vector = core::mem::zeroed();
    let mut vec: FT_Vector = core::mem::zeroed();
    let plf: *mut LOGFONTW;
    let use_kerning: BOOL;
    let mut bResult: BOOL;
    let mut DoBreak: BOOL;
    let mut Cache: FONT_CACHE_ENTRY = core::mem::zeroed();
    let mut mat: FT_Matrix = core::mem::zeroed();
    let bNoTransform: BOOL;
    let mut ch0: DWORD;
    let mut ch1: DWORD;
    let del: DWORD = 0x7f;
    let nbsp: DWORD = 0xa0;
    let mut Chain: FONTLINK_CHAIN = core::mem::zeroed();
    let mut spaceWidth: SIZE = core::mem::zeroed();

    if Count > 0xFFFF || (Count > 0 && String.is_null()) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel)) {
        return PATH_ExtTextOut(dc, XStart, YStart, fuOptions, lprc, String, Count as _ , Dx);
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    'cleanup: {
        if (*dc).dclevel.pSurface.is_null() {
            bResult = 1;
            break 'cleanup;
        }

        pdcattr = (*dc).pdcattr;
        if (*pdcattr).flTextAlign & TA_UPDATECP != 0 {
            Start.x = (*pdcattr).ptlCurrent.x;
            Start.y = (*pdcattr).ptlCurrent.y;
        } else {
            Start.x = XStart;
            Start.y = YStart;
        }

        w32r_IntLPtoDP(dc, &mut Start, 1);
        RealXStart64 = ((Start.x as LONGLONG + (*dc).ptlDCOrig.x as LONGLONG)) << 6;
        RealYStart64 = ((Start.y as LONGLONG + (*dc).ptlDCOrig.y as LONGLONG)) << 6;

        MaskRect.left = 0;
        MaskRect.top = 0;

        psurf = (*dc).dclevel.pSurface;
        psoDest = core::ptr::addr_of_mut!((*psurf).SurfObj);

        if (*pdcattr).iGraphicsMode == GM_ADVANCED as i32 {
            pmxWorldToDevice = w32r_DC_pmxWorldToDevice(dc);
        } else {
            pmxWorldToDevice = core::ptr::addr_of!(gmxWorldToPageDefault) as PMATRIX;
        }

        if (*pdcattr).ulDirty_ & DIRTY_BACKGROUND != 0 {
            DC_vUpdateBackgroundBrush(dc);
        }

        if !lprc.is_null() && (fuOptions & (ETO_CLIPPED | ETO_OPAQUE)) != 0 {
            w32r_IntLPtoDP(dc, lprc as *mut POINT, 2);
            (*lprc).left += (*dc).ptlDCOrig.x;
            (*lprc).top += (*dc).ptlDCOrig.y;
            (*lprc).right += (*dc).ptlDCOrig.x;
            (*lprc).bottom += (*dc).ptlDCOrig.y;
        }

        if !lprc.is_null() && (fuOptions & ETO_OPAQUE) != 0 {
            IntEngFillBox(
                dc,
                (*lprc).left,
                (*lprc).top,
                (*lprc).right - (*lprc).left,
                (*lprc).bottom - (*lprc).top,
                core::ptr::addr_of_mut!((*dc).eboBackground.BrushObject),
            );
            fuOptions &= !ETO_OPAQUE;
        } else if (*pdcattr).jBkMode == (OPAQUE as i32 as _) {
            fuOptions |= ETO_OPAQUE;
        }

        TextObj = RealizeFontInit((*pdcattr).hlfntNew as _ );
        if TextObj.is_null() {
            bResult = 0;
            break 'cleanup;
        }

        FontObj = (*TextObj).Font;
        FontGDI = ObjToGDI!(FontObj, FONT);

        IntLockFreeType();
        face = (*(*FontGDI).SharedFace).Face;
        Cache.Hashed.Face = face;

        plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);
        Cache.Hashed.lfHeight = (*plf).lfHeight;
        Cache.Hashed.lfWidth = (*plf).lfWidth;
        Cache.Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Bold = EMUBOLD_NEEDED((*FontGDI).OriginalWeight, (*plf).lfWeight) as BYTE ;
        Cache.Hashed.__bindgen_anon_1.Aspect.__bindgen_anon_1.Emu.Italic = (((*plf).lfItalic != 0) && ((*FontGDI).OriginalItalic == 0)) as BYTE ;

        if IntIsFontRenderingEnabled() != 0 {
            Cache.Hashed.__bindgen_anon_1.Aspect.RenderMode = IntGetFontRenderMode(plf) as BYTE as _ ;
        } else {
            Cache.Hashed.__bindgen_anon_1.Aspect.RenderMode = FT_RENDER_MODE_MONO as BYTE as _ ;
        }

        if TextIntUpdateSize(dc, TextObj, FontGDI, 0) == 0 {
            IntUnLockFreeType();
            bResult = 0;
            break 'cleanup;
        }

        FontLink_Chain_Init(&mut Chain, TextObj, face);

        let mut mat_transform: FT_Matrix;
        if FT_IS_SCALABLE(face) != 0 && (*plf).lfEscapement != 0 {
            mat_transform = core::mem::zeroed();
            IntEscapeMatrix(&mut mat_transform, (*plf).lfEscapement);
        } else {
            mat_transform = identityMat;
        }
        core::ptr::write(core::ptr::addr_of_mut!(Cache.Hashed.matTransform), mat_transform);

        IntMatrixFromMx(&mut mat, pmxWorldToDevice);
        FT_Matrix_Multiply(&mut mat, &mut mat_transform);
        FT_Set_Transform(face, &mut mat_transform, core::ptr::null_mut());
        core::ptr::write(core::ptr::addr_of_mut!(Cache.Hashed.matTransform), mat_transform);

        bNoTransform = ((mat.xy == 0) && (mat.yx == 0) && (mat.xx == (1 << 16)) && (mat.yy == (1 << 16))) as BOOL;

        vecAscent64.x = 0;
        vecAscent64.y = ((*FontGDI).tmAscent << 6) as _;
        FT_Vector_Transform(&mut vecAscent64, &mut mat_transform);
        vecDescent64.x = 0;
        vecDescent64.y = -(((*FontGDI).tmDescent << 6) as FT_Pos);
        FT_Vector_Transform(&mut vecDescent64, &mut mat_transform);

        let valign_mask = TA_TOP | TA_BASELINE | TA_BOTTOM;
        if ((*pdcattr).flTextAlign & valign_mask) == TA_BASELINE {
        } else if ((*pdcattr).flTextAlign & valign_mask) == TA_BOTTOM {
            RealXStart64 -= vecDescent64.x as LONGLONG;
            RealYStart64 += vecDescent64.y as LONGLONG;
        } else {
            RealXStart64 -= vecAscent64.x as LONGLONG;
            RealYStart64 += vecAscent64.y as LONGLONG;
        }

        use_kerning = FT_HAS_KERNING(face);

        if (fuOptions & ETO_OPAQUE) != 0 || ((*pdcattr).flTextAlign & (TA_CENTER | TA_RIGHT)) != 0 {
            if IntGetTextDisposition(
                &mut DeltaX64,
                &mut DeltaY64,
                String,
                Count,
                Dx,
                &mut Cache,
                fuOptions,
                bNoTransform,
                &mut Chain,
            ) == 0
            {
                FontLink_Chain_Finish(&mut Chain);
                IntUnLockFreeType();
                bResult = 0;
                break 'cleanup;
            }

            if ((*pdcattr).flTextAlign & TA_CENTER) == TA_CENTER {
                RealXStart64 -= DeltaX64 / 2;
                RealYStart64 -= DeltaY64 / 2;
            } else if ((*pdcattr).flTextAlign & TA_RIGHT) == TA_RIGHT {
                RealXStart64 -= DeltaX64;
                RealYStart64 -= DeltaY64;
            }

            if (fuOptions & ETO_OPAQUE) != 0 {
                let X0: INT = ((RealXStart64 + vecAscent64.x as LONGLONG + 32) >> 6) as INT;
                let Y0: INT = ((RealYStart64 - vecAscent64.y as LONGLONG + 32) >> 6) as INT;
                let DX: INT = (DeltaX64 >> 6) as INT;
                if mat_transform.xy == 0 && mat_transform.yx == 0 {
                    let CY: INT = ((vecAscent64.y - vecDescent64.y) as LONGLONG + 32 >> 6) as INT;
                    IntEngFillBox(dc, X0, Y0, DX, CY, core::ptr::addr_of_mut!((*dc).eboBackground.BrushObject));
                } else {
                    let DY: INT = (DeltaY64 >> 6) as INT;
                    let X1: INT = ((RealXStart64 + vecDescent64.x as LONGLONG + 32) >> 6) as INT;
                    let Y1: INT = ((RealYStart64 - vecDescent64.y as LONGLONG + 32) >> 6) as INT;
                    let mut Points: [POINT; 4] = [
                        POINT { x: X0, y: Y0 },
                        POINT { x: X0 + DX, y: Y0 + DY },
                        POINT { x: X1 + DX, y: Y1 + DY },
                        POINT { x: X1, y: Y1 },
                    ];
                    IntEngFillPolygon(dc, Points.as_mut_ptr() as _ , 4, core::ptr::addr_of_mut!((*dc).eboBackground.BrushObject));
                }
            }
        }

        EXLATEOBJ_vInitialize(&mut exloRGB2Dst, &raw mut gpalRGB, (*psurf).ppal, 0, 0, 0);
        EXLATEOBJ_vInitialize(&mut exloDst2RGB, (*psurf).ppal, &raw mut gpalRGB, 0, 0, 0);

        if (*pdcattr).ulDirty_ & DIRTY_TEXT != 0 {
            DC_vUpdateTextBrush(dc);
        }

        X64 = RealXStart64;
        Y64 = RealYStart64;
        previous = 0;
        DoBreak = 0;
        bResult = 1;
        i = 0;
        while i < Count {
            ch0 = *String as DWORD;
            String = String.add(1);
            if IS_HIGH_SURROGATE(ch0) {
                i += 1;
                if i >= Count {
                    break;
                }
                ch1 = *String as DWORD;
                String = String.add(1);
                if IS_LOW_SURROGATE(ch1) {
                    ch0 = Utf32FromSurrogatePair(ch0, ch1);
                }
            }

            glyph_index = FontLink_Chain_FindGlyph(&mut Chain, &mut Cache, &mut face, ch0, (fuOptions & ETO_GLYPH_INDEX) as BOOL) as INT;
            Cache.Hashed.GlyphIndex = glyph_index as _;

            realglyph = IntGetRealGlyph(&mut Cache);
            if realglyph.is_null() {
                bResult = 0;
                break;
            }

            if use_kerning != 0 && previous != 0 && glyph_index != 0 && Dx.is_null() {
                FT_Get_Kerning(face, previous, glyph_index as FT_UInt, 0, &mut delta);
                X64 += delta.x as LONGLONG;
                Y64 -= delta.y as LONGLONG;
            }

            glyphSize.cx = (*realglyph).bitmap.width as LONG;
            glyphSize.cy = (*realglyph).bitmap.rows as LONG;

            if ((*pdcattr).flTextAlign & TA_UPDATECP) != 0
                && glyphSize.cx == 0
                && (ch0 == ' ' as DWORD || ch0 == nbsp)
            {
                IntUnLockFreeType();
                let space: [WCHAR; 2] = [' ' as WCHAR, 0];
                TextIntGetTextExtentPoint(
                    dc,
                    TextObj,
                    space.as_ptr(),
                    1,
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut spaceWidth,
                    0,
                );
                IntLockFreeType();
                glyphSize.cx = spaceWidth.cx;
                (*realglyph).left = 0;
            }

            MaskRect.right = (*realglyph).bitmap.width as LONG;
            MaskRect.bottom = (*realglyph).bitmap.rows as LONG;

            DestRect.left = ((X64 + 32) >> 6) as LONG + (*realglyph).left;
            DestRect.right = DestRect.left + glyphSize.cx;
            DestRect.top = ((Y64 + 32) >> 6) as LONG - (*realglyph).top;
            DestRect.bottom = DestRect.top + glyphSize.cy;

            if glyphSize.cx != 0 && glyphSize.cy != 0 {
                hbmGlyph = EngCreateBitmap(
                    glyphSize,
                    (*realglyph).bitmap.pitch,
                    BMF_8BPP,
                    BMF_TOPDOWN,
                    (*realglyph).bitmap.buffer as *mut _,
                );
                if hbmGlyph.is_null() {
                    bResult = 0;
                    break;
                }

                psoGlyph = EngLockSurface(hbmGlyph as HSURF);
                if psoGlyph.is_null() {
                    EngDeleteSurface(hbmGlyph as HSURF);
                    bResult = 0;
                    break;
                }

                if !lprc.is_null() && (fuOptions & ETO_CLIPPED) != 0 {
                    if DestRect.right >= (*lprc).right {
                        DestRect.right = (*lprc).right;
                        DoBreak = 1;
                    }
                    if DestRect.bottom >= (*lprc).bottom {
                        DestRect.bottom = (*lprc).bottom;
                    }
                }

                IntEngMaskBlt(
                    psoDest,
                    psoGlyph,
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    &mut exloRGB2Dst.xlo,
                    &mut exloDst2RGB.xlo,
                    &mut DestRect,
                    core::ptr::addr_of_mut!(MaskRect) as PPOINTL,
                    core::ptr::addr_of_mut!((*dc).eboText.BrushObject),
                    &raw mut g_PointZero,
                );

                EngUnlockSurface(psoGlyph);
                EngDeleteSurface(hbmGlyph as HSURF);
            }

            if DoBreak != 0 {
                break;
            }

            if Dx.is_null() {
                X64 += ((*realglyph).root.advance.x >> 10) as LONGLONG;
                Y64 -= ((*realglyph).root.advance.y >> 10) as LONGLONG;
            } else if (fuOptions & ETO_PDY) != 0 {
                vec.x = ((*Dx.add((2 * i + 0) as usize)) << 6) as FT_Pos;
                vec.y = ((*Dx.add((2 * i + 1) as usize)) << 6) as FT_Pos;
                if bNoTransform == 0 {
                    FT_Vector_Transform(&mut vec, &mut mat_transform);
                }
                X64 += vec.x as LONGLONG;
                Y64 -= vec.y as LONGLONG;
            } else {
                vec.x = ((*Dx.add(i as usize)) << 6) as FT_Pos;
                vec.y = 0;
                if bNoTransform == 0 {
                    FT_Vector_Transform(&mut vec, &mut mat_transform);
                }
                X64 += vec.x as LONGLONG;
                Y64 -= vec.y as LONGLONG;
            }

            previous = glyph_index as ULONG;

            i += 1;
        }

        if ((*pdcattr).flTextAlign & TA_UPDATECP) != 0 && !String.is_null() {
            (*pdcattr).ptlCurrent.x = DestRect.right - (*dc).ptlDCOrig.x;
        }

        if (*plf).lfUnderline != 0 || (*plf).lfStrikeOut != 0 {
            let underline_position: INT;
            let thickness: INT;
            let mut vecA64: FT_Vector = core::mem::zeroed();
            let mut vecB64: FT_Vector = core::mem::zeroed();

            DeltaX64 = X64 - RealXStart64;
            DeltaY64 = Y64 - RealYStart64;

            if (*face).units_per_EM == 0 {
                underline_position = 0;
                thickness = 1;
            } else {
                underline_position = ((*face).underline_position as INT * (*(*face).size).metrics.y_ppem as INT) / (*face).units_per_EM as INT;
                let mut t = ((*face).underline_thickness as INT * (*(*face).size).metrics.y_ppem as INT) / (*face).units_per_EM as INT;
                if t <= 0 {
                    t = 1;
                }
                thickness = t;
            }

            if (*plf).lfUnderline != 0 {
                vecA64.x = 0;
                vecA64.y = ((-underline_position - thickness / 2) << 6) as FT_Pos;
                vecB64.x = 0;
                vecB64.y = vecA64.y + ((thickness << 6) as FT_Pos);
                FT_Vector_Transform(&mut vecA64, &mut mat_transform);
                FT_Vector_Transform(&mut vecB64, &mut mat_transform);
                let X0: INT = ((RealXStart64 - vecA64.x as LONGLONG + 32) >> 6) as INT;
                let Y0: INT = ((RealYStart64 + vecA64.y as LONGLONG + 32) >> 6) as INT;
                let DX: INT = (DeltaX64 >> 6) as INT;
                if mat_transform.xy == 0 && mat_transform.yx == 0 {
                    let CY: INT = (((vecB64.y - vecA64.y) as LONGLONG + 32) >> 6) as INT;
                    IntEngFillBox(dc, X0, Y0, DX, CY, core::ptr::addr_of_mut!((*dc).eboText.BrushObject));
                } else {
                    let DY: INT = (DeltaY64 >> 6) as INT;
                    let X1: INT = X0 + (((vecA64.x - vecB64.x) as LONGLONG + 32) >> 6) as INT;
                    let Y1: INT = Y0 + (((vecB64.y - vecA64.y) as LONGLONG + 32) >> 6) as INT;
                    let mut Points: [POINT; 4] = [
                        POINT { x: X0, y: Y0 },
                        POINT { x: X0 + DX, y: Y0 + DY },
                        POINT { x: X1 + DX, y: Y1 + DY },
                        POINT { x: X1, y: Y1 },
                    ];
                    IntEngFillPolygon(dc, Points.as_mut_ptr() as _ , 4, core::ptr::addr_of_mut!((*dc).eboText.BrushObject));
                }
            }

            if (*plf).lfStrikeOut != 0 {
                vecA64.x = 0;
                vecA64.y = -(((*FontGDI).tmAscent << 6) as FT_Pos) / 3;
                vecB64.x = 0;
                vecB64.y = vecA64.y + ((thickness << 6) as FT_Pos);
                FT_Vector_Transform(&mut vecA64, &mut mat_transform);
                FT_Vector_Transform(&mut vecB64, &mut mat_transform);
                let X0: INT = ((RealXStart64 - vecA64.x as LONGLONG + 32) >> 6) as INT;
                let Y0: INT = ((RealYStart64 + vecA64.y as LONGLONG + 32) >> 6) as INT;
                let DX: INT = (DeltaX64 >> 6) as INT;
                if mat_transform.xy == 0 && mat_transform.yx == 0 {
                    let CY: INT = (((vecB64.y - vecA64.y) as LONGLONG + 32) >> 6) as INT;
                    IntEngFillBox(dc, X0, Y0, DX, CY, core::ptr::addr_of_mut!((*dc).eboText.BrushObject));
                } else {
                    let DY: INT = (DeltaY64 >> 6) as INT;
                    let X1: INT = X0 + (((vecA64.x - vecB64.x) as LONGLONG + 32) >> 6) as INT;
                    let Y1: INT = Y0 + (((vecB64.y - vecA64.y) as LONGLONG + 32) >> 6) as INT;
                    let mut Points: [POINT; 4] = [
                        POINT { x: X0, y: Y0 },
                        POINT { x: X0 + DX, y: Y0 + DY },
                        POINT { x: X1 + DX, y: Y1 + DY },
                        POINT { x: X1, y: Y1 },
                    ];
                    IntEngFillPolygon(dc, Points.as_mut_ptr() as _ , 4, core::ptr::addr_of_mut!((*dc).eboText.BrushObject));
                }
            }
        }

        FontLink_Chain_Finish(&mut Chain);

        IntUnLockFreeType();

        EXLATEOBJ_vCleanup(&mut exloRGB2Dst);
        EXLATEOBJ_vCleanup(&mut exloDst2RGB);
    }

    DC_vFinishBlit(dc, core::ptr::null_mut());

    if !TextObj.is_null() {
        w32r_TEXTOBJ_UnlockText(TextObj);
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn GreExtTextOutW(
    hDC: HDC,
    XStart: INT,
    YStart: INT,
    fuOptions: UINT,
    lprc: PRECTL,
    String: PCWCH,
    Count: INT,
    Dx: *const INT,
    dwCodePage: DWORD,
) -> BOOL {
    let bResult: BOOL;
    let dc: *mut DC;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    bResult = IntExtTextOutW(
        dc,
        XStart,
        YStart,
        fuOptions,
        lprc,
        String,
        Count,
        Dx,
        dwCodePage,
    );

    DC_UnlockDc(dc);

    bResult
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharWidthW(
    hDC: HDC,
    FirstChar: UINT,
    Count: UINT,
    UnSafepwc: PCWCH,
    fl: FLONG,
    Buffer: *mut core::ffi::c_void,
) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let SafeBuff: LPINT;
    let mut SafeBuffF: PFLOAT = core::ptr::null_mut();
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let face: FT_Face;
    let mut charmap: FT_CharMap;
    let mut found: FT_CharMap = core::ptr::null_mut();
    let mut i: UINT;
    let mut glyph_index: UINT;
    let BufferSize: UINT;
    let hFont: HFONT;
    let mut Safepwc: *mut u16 = core::ptr::null_mut();
    let plf: *mut LOGFONTW;

    if !UnSafepwc.is_null() {
        let pwcSize: UINT = Count.wrapping_mul(core::mem::size_of::<u16>() as UINT);
        Safepwc = ExAllocatePoolWithTag(PagedPool, pwcSize as usize, GDITAG_TEXT) as *mut u16;

        if Safepwc.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        Status = SafeProbeForRead((UnSafepwc as *const core::ffi::c_void as _), pwcSize as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(Safepwc as *mut core::ffi::c_void, UnSafepwc as *const core::ffi::c_void, pwcSize as usize)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    BufferSize = Count.wrapping_mul(core::mem::size_of::<INT>() as UINT);
    SafeBuff = ExAllocatePoolWithTag(PagedPool, BufferSize as usize, GDITAG_TEXT) as LPINT;
    if fl == 0 {
        SafeBuffF = SafeBuff as PFLOAT;
    }
    if SafeBuff.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    hFont = ((*pdcattr).hlfntNew as _);
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    FontGDI = ObjToGDI!((*TextObj).Font);

    face = (*(*FontGDI).SharedFace).Face;
    if (*face).charmap.is_null() {
        i = 0;
        while i < (*face).num_charmaps as UINT {
            charmap = *(*face).charmaps.add(i as usize);
            if (*charmap).encoding != 0 {
                found = charmap;
                break;
            }
            i += 1;
        }

        if found.is_null() {
            if !Safepwc.is_null() {
                ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
            }
            ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        IntLockFreeType();
        FT_Set_Charmap(face, found);
        IntUnLockFreeType();
    }

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);

    IntLockFreeType();
    IntRequestFontSize(dc, FontGDI, (*plf).lfWidth, (*plf).lfHeight);
    FT_Set_Transform(face, core::ptr::null_mut(), core::ptr::null_mut());

    i = FirstChar;
    while i < FirstChar.wrapping_add(Count) {
        if !Safepwc.is_null() {
            glyph_index = get_glyph_index_flagged(face, *Safepwc.add((i - FirstChar) as usize) as FT_ULong, (fl & GCW_INDICES) as BOOL);
        } else {
            glyph_index = get_glyph_index_flagged(face, i as FT_ULong, (fl & GCW_INDICES) as BOOL);
        }

        FT_Load_Glyph(face, glyph_index, FT_LOAD_DEFAULT as i32);
        if fl == 0 {
            *SafeBuffF.add((i - FirstChar) as usize) = (((*(*face).glyph).advance.x + 32) >> 6) as FLOAT;
        } else {
            *SafeBuff.add((i - FirstChar) as usize) = (((*(*face).glyph).advance.x + 32) >> 6) as INT;
        }
        i += 1;
    }
    IntUnLockFreeType();
    w32r_TEXTOBJ_UnlockText(TextObj);
    MmCopyToCaller(Buffer, SafeBuff as *const core::ffi::c_void, (BufferSize as usize as _));

    if !Safepwc.is_null() {
        ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
    }
    ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharWidthW(
    hDC: HDC,
    FirstChar: UINT,
    Count: UINT,
    UnSafepwc: PCWCH,
    fl: FLONG,
    Buffer: *mut core::ffi::c_void,
) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let SafeBuff: LPINT;
    let mut SafeBuffF: PFLOAT = core::ptr::null_mut();
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let face: FT_Face;
    let mut charmap: FT_CharMap;
    let mut found: FT_CharMap = core::ptr::null_mut();
    let mut i: UINT;
    let mut glyph_index: UINT;
    let BufferSize: UINT;
    let hFont: HFONT;
    let mut Safepwc: *mut u16 = core::ptr::null_mut();
    let plf: *mut LOGFONTW;

    if !UnSafepwc.is_null() {
        let pwcSize: UINT = Count.wrapping_mul(core::mem::size_of::<u16>() as UINT);
        Safepwc = ExAllocatePoolWithTag(PagedPool, pwcSize as usize, GDITAG_TEXT) as *mut u16;

        if Safepwc.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        Status = SafeProbeForRead(UnSafepwc as *const core::ffi::c_void as _ , pwcSize as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(Safepwc as *mut core::ffi::c_void, UnSafepwc as *const core::ffi::c_void, pwcSize as usize)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    BufferSize = Count.wrapping_mul(core::mem::size_of::<INT>() as UINT);
    SafeBuff = ExAllocatePoolWithTag(PagedPool, BufferSize as usize, GDITAG_TEXT) as LPINT;
    if fl == 0 {
        SafeBuffF = SafeBuff as PFLOAT;
    }
    if SafeBuff.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        if !Safepwc.is_null() {
            ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
        }
        ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    FontGDI = ObjToGDI!((*TextObj).Font);

    face = (*(*FontGDI).SharedFace).Face;
    if (*face).charmap.is_null() {
        i = 0;
        while i < (*face).num_charmaps as UINT {
            charmap = *(*face).charmaps.add(i as usize);
            if (*charmap).encoding != 0 {
                found = charmap;
                break;
            }
            i += 1;
        }

        if found.is_null() {
            if !Safepwc.is_null() {
                ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
            }
            ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        IntLockFreeType();
        FT_Set_Charmap(face, found);
        IntUnLockFreeType();
    }

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);

    IntLockFreeType();
    IntRequestFontSize(dc, FontGDI, (*plf).lfWidth, (*plf).lfHeight);
    FT_Set_Transform(face, core::ptr::null_mut(), core::ptr::null_mut());

    i = FirstChar;
    while i < FirstChar.wrapping_add(Count) {
        if !Safepwc.is_null() {
            glyph_index = get_glyph_index_flagged(face, *Safepwc.add((i - FirstChar) as usize) as FT_ULong, (fl & GCW_INDICES) as BOOL);
        } else {
            glyph_index = get_glyph_index_flagged(face, i as FT_ULong, (fl & GCW_INDICES) as BOOL);
        }

        FT_Load_Glyph(face, glyph_index, FT_LOAD_DEFAULT as i32);
        if fl == 0 {
            *SafeBuffF.add((i - FirstChar) as usize) = (((*(*face).glyph).advance.x + 32) >> 6) as FLOAT;
        } else {
            *SafeBuff.add((i - FirstChar) as usize) = (((*(*face).glyph).advance.x + 32) >> 6) as INT;
        }
        i += 1;
    }
    IntUnLockFreeType();
    w32r_TEXTOBJ_UnlockText(TextObj);
    MmCopyToCaller(Buffer, SafeBuff as *const core::ffi::c_void, BufferSize as usize as _ );

    if !Safepwc.is_null() {
        ExFreePoolWithTag(Safepwc as *mut core::ffi::c_void, GDITAG_TEXT);
    }
    ExFreePoolWithTag(SafeBuff as *mut core::ffi::c_void, GDITAG_TEXT);
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetGlyphIndicesW(
    hdc: HDC,
    pwc: PCWCH,
    cwc: INT,
    pgi: PWORD,
    iMode: DWORD,
) -> DWORD {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let potm: *mut OUTLINETEXTMETRICW;
    let mut i: INT;
    let mut DefChar: WCHAR = 0xffff;
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Size: ULONG;
    let pwcSize: ULONG;
    let mut Safepwc: PWSTR = core::ptr::null_mut();
    let UnSafepwc: LPCWSTR = pwc;
    let UnSafepgi: LPWORD = pgi;
    let Face: FT_Face;
    let pOS2: *mut TT_OS2;

    if cwc < 0 {
        return GDI_ERROR;
    }

    if UnSafepwc.is_null() && UnSafepgi.is_null() && cwc > 0 {
        return GDI_ERROR;
    }

    if UnSafepwc.is_null() != UnSafepgi.is_null() {
        return GDI_ERROR;
    }

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return GDI_ERROR;
    }
    pdcattr = (*dc).pdcattr;
    hFont = ((*pdcattr).hlfntNew as _);
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);
    if TextObj.is_null() {
        return GDI_ERROR;
    }
    FontGDI = ObjToGDI!((*TextObj).Font, FONT);
    w32r_TEXTOBJ_UnlockText(TextObj);

    if cwc == 0 {
        if UnSafepwc.is_null() && UnSafepgi.is_null() {
            let Face = (*(*FontGDI).SharedFace).Face;
            return (*Face).num_glyphs as DWORD;
        } else {
            Status = STATUS_UNSUCCESSFUL;
            // goto ErrorRet
            if Buffer.is_null() == false {
                ExFreePoolWithTag(Buffer as *mut _, GDITAG_TEXT);
            }
            if Safepwc.is_null() == false {
                ExFreePoolWithTag(Safepwc as *mut _, GDITAG_TEXT);
            }
            if NT_SUCCESS(Status) {
                return cwc as DWORD;
            }
            return GDI_ERROR;
        }
    }

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        ((cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _),
        GDITAG_TEXT,
    ) as PWSTR;
    if Buffer.is_null() {
        return GDI_ERROR;
    }

    'ErrorRet: {
        if iMode & GGI_MARK_NONEXISTING_GLYPHS != 0 {
            DefChar = 0xffff;
        } else {
            Face = (*(*FontGDI).SharedFace).Face;
            if FT_IS_SFNT(Face) != 0 {
                IntLockFreeType();
                pOS2 = FT_Get_Sfnt_Table(Face, ft_sfnt_os2) as *mut TT_OS2;
                DefChar = if (*pOS2).usDefaultChar != 0 {
                    get_glyph_index(Face, (*pOS2).usDefaultChar as UINT) as WCHAR
                } else {
                    0
                };
                IntUnLockFreeType();
            } else {
                ASSERT_FREETYPE_LOCK_NOT_HELD();
                Size = IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 0) as ULONG;
                if Size == 0 {
                    Status = STATUS_UNSUCCESSFUL;
                    break 'ErrorRet;
                }
                let potm_alloc = ExAllocatePoolWithTag(PagedPool, (Size as SIZE_T as _), GDITAG_TEXT)
                    as *mut OUTLINETEXTMETRICW;
                if potm_alloc.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'ErrorRet;
                }
                potm = potm_alloc;
                ASSERT_FREETYPE_LOCK_NOT_HELD();
                Size = IntGetOutlineTextMetrics(FontGDI, Size as UINT, potm, 0) as ULONG;
                if Size != 0 {
                    DefChar = (*potm).otmTextMetrics.tmDefaultChar as WCHAR;
                }
                ExFreePoolWithTag(potm as *mut _, GDITAG_TEXT);
            }
        }

        pwcSize = (cwc as usize * core::mem::size_of::<WCHAR>()) as ULONG;
        Safepwc = ExAllocatePoolWithTag(PagedPool, (pwcSize as SIZE_T as _), GDITAG_TEXT) as PWSTR;
        if Safepwc.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'ErrorRet;
        }

        Status = SafeProbeForRead(UnSafepwc as *mut c_void, pwcSize as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(Safepwc as *mut _, UnSafepwc as *const _, pwcSize as SIZE_T as _)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            break 'ErrorRet;
        }

        IntLockFreeType();
        i = 0;
        while i < cwc {
            *Buffer.offset(i as isize) =
                get_glyph_index((*(*FontGDI).SharedFace).Face, *Safepwc.offset(i as isize) as UINT)
                    as WCHAR;
            if *Buffer.offset(i as isize) == 0 {
                *Buffer.offset(i as isize) = DefChar;
            }
            i += 1;
        }
        IntUnLockFreeType();

        Status = SafeProbeForWrite(
            UnSafepgi as *mut _,
            ((cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _),
            1,
        );
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(UnSafepgi as *mut _, Buffer as *const _, ((cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _))
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as *mut _, GDITAG_TEXT);
    }
    if !Safepwc.is_null() {
        ExFreePoolWithTag(Safepwc as *mut _, GDITAG_TEXT);
    }

    if NT_SUCCESS(Status) {
        return cwc as DWORD;
    }

    GDI_ERROR
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetGlyphIndicesW(
    hdc: HDC,
    pwc: PCWCH,
    cwc: INT,
    pgi: PWORD,
    iMode: DWORD,
) -> DWORD {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let potm: *mut OUTLINETEXTMETRICW;
    let mut i: INT;
    let mut DefChar: WCHAR = 0xffff;
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut Size: ULONG;
    let pwcSize: ULONG;
    let mut Safepwc: PWSTR = core::ptr::null_mut();
    let UnSafepwc: LPCWSTR = pwc;
    let UnSafepgi: LPWORD = pgi;
    let Face: FT_Face;
    let pOS2: *mut TT_OS2;

    if cwc < 0 {
        return GDI_ERROR;
    }

    if UnSafepwc.is_null() && UnSafepgi.is_null() && cwc > 0 {
        return GDI_ERROR;
    }

    if UnSafepwc.is_null() != UnSafepgi.is_null() {
        return GDI_ERROR;
    }

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return GDI_ERROR;
    }
    pdcattr = (*dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);
    if TextObj.is_null() {
        return GDI_ERROR;
    }
    FontGDI = ObjToGDI!((*TextObj).Font, FONT);
    w32r_TEXTOBJ_UnlockText(TextObj);

    if cwc == 0 {
        if UnSafepwc.is_null() && UnSafepgi.is_null() {
            let Face = (*(*FontGDI).SharedFace).Face;
            return (*Face).num_glyphs as DWORD;
        } else {
            Status = STATUS_UNSUCCESSFUL;
            // goto ErrorRet
            if Buffer.is_null() == false {
                ExFreePoolWithTag(Buffer as *mut _, GDITAG_TEXT);
            }
            if Safepwc.is_null() == false {
                ExFreePoolWithTag(Safepwc as *mut _, GDITAG_TEXT);
            }
            if NT_SUCCESS(Status) {
                return cwc as DWORD;
            }
            return GDI_ERROR;
        }
    }

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _ ,
        GDITAG_TEXT,
    ) as PWSTR;
    if Buffer.is_null() {
        return GDI_ERROR;
    }

    'ErrorRet: {
        if iMode & GGI_MARK_NONEXISTING_GLYPHS != 0 {
            DefChar = 0xffff;
        } else {
            Face = (*(*FontGDI).SharedFace).Face;
            if FT_IS_SFNT(Face) != 0 {
                IntLockFreeType();
                pOS2 = FT_Get_Sfnt_Table(Face, ft_sfnt_os2) as *mut TT_OS2;
                DefChar = if (*pOS2).usDefaultChar != 0 {
                    get_glyph_index(Face, (*pOS2).usDefaultChar as UINT) as WCHAR
                } else {
                    0
                };
                IntUnLockFreeType();
            } else {
                ASSERT_FREETYPE_LOCK_NOT_HELD();
                Size = IntGetOutlineTextMetrics(FontGDI, 0, core::ptr::null_mut(), 0) as ULONG;
                if Size == 0 {
                    Status = STATUS_UNSUCCESSFUL;
                    break 'ErrorRet;
                }
                let potm_alloc = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T as _ , GDITAG_TEXT)
                    as *mut OUTLINETEXTMETRICW;
                if potm_alloc.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'ErrorRet;
                }
                potm = potm_alloc;
                ASSERT_FREETYPE_LOCK_NOT_HELD();
                Size = IntGetOutlineTextMetrics(FontGDI, Size as UINT, potm, 0) as ULONG;
                if Size != 0 {
                    DefChar = (*potm).otmTextMetrics.tmDefaultChar as WCHAR;
                }
                ExFreePoolWithTag(potm as *mut _, GDITAG_TEXT);
            }
        }

        pwcSize = (cwc as usize * core::mem::size_of::<WCHAR>()) as ULONG;
        Safepwc = ExAllocatePoolWithTag(PagedPool, pwcSize as SIZE_T as _ , GDITAG_TEXT) as PWSTR;
        if Safepwc.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'ErrorRet;
        }

        Status = SafeProbeForRead(UnSafepwc as *mut c_void, pwcSize as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(Safepwc as *mut _, UnSafepwc as *const _, pwcSize as SIZE_T as _)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            break 'ErrorRet;
        }

        IntLockFreeType();
        i = 0;
        while i < cwc {
            *Buffer.offset(i as isize) =
                get_glyph_index((*(*FontGDI).SharedFace).Face, *Safepwc.offset(i as isize) as UINT)
                    as WCHAR;
            if *Buffer.offset(i as isize) == 0 {
                *Buffer.offset(i as isize) = DefChar;
            }
            i += 1;
        }
        IntUnLockFreeType();

        Status = SafeProbeForWrite(
            UnSafepgi as *mut _,
            (cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _ ,
            1,
        );
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(UnSafepgi as *mut _, Buffer as *const _, (cwc as usize * core::mem::size_of::<WORD>()) as SIZE_T as _)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as *mut _, GDITAG_TEXT);
    }
    if !Safepwc.is_null() {
        ExFreePoolWithTag(Safepwc as *mut _, GDITAG_TEXT);
    }

    if NT_SUCCESS(Status) {
        return cwc as DWORD;
    }

    GDI_ERROR
}

// ---- Hand-translated NtGdi syscalls (SAL annotations caused scanner to skip them) ----

// NtGdiExtTextOutW: _NtGdiExtTextOutW@36 on i386 (9 args x 4 bytes).
// DUMMY_PSEH: skip _SEH2_TRY/_SEH2_EXCEPT; keep the normal (try-body) path.
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtTextOutW(
    hDC: HDC,
    XStart: INT,
    YStart: INT,
    fuOptions: UINT,
    UnsafeRect: LPCRECT,
    UnsafeString: PCWCH,
    Count: UINT,
    UnsafeDx: *const INT,
    dwCodePage: DWORD,
) -> BOOL {
    let mut Result: BOOL = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SafeRect: RECTL = core::mem::zeroed();
    let mut LocalBuffer: [BYTE; STACK_TEXT_BUFFER_SIZE] = [0; STACK_TEXT_BUFFER_SIZE];
    let mut Buffer: *mut c_void = LocalBuffer.as_mut_ptr() as *mut c_void;
    let mut SafeString: PCWCH = core::ptr::null();
    let mut SafeDx: *mut INT = core::ptr::null_mut();
    let mut BufSize: ULONG;
    let StringSize: ULONG;
    let mut DxSize: ULONG = 0;

    if Count > 0xFFFF || (Count > 0 && UnsafeString.is_null()) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Count > 0 {
        StringSize = Count * core::mem::size_of::<WCHAR>() as ULONG;
        BufSize = StringSize;
        if !UnsafeDx.is_null() {
            DxSize = Count * core::mem::size_of::<INT>() as ULONG
                * if fuOptions & ETO_PDY != 0 { 2 } else { 1 };
            BufSize += DxSize;
        }
        if BufSize > STACK_TEXT_BUFFER_SIZE as ULONG {
            Buffer = ExAllocatePoolWithTag(PagedPool, BufSize as usize, GDITAG_TEXT);
            if Buffer.is_null() {
                return 0;
            }
        }
        Status = SafeProbeForRead(UnsafeString as PVOID, StringSize as usize, 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(UnsafeDx as PVOID, DxSize as usize, 1);
        }
        // Validate-only probes; the copies are the access. Either one faulting lands in
        // Status, which is checked below and frees Buffer before returning.
        if NT_SUCCESS(Status) {
            SafeString = (Buffer as usize + DxSize as usize) as PCWCH;
            if !crate::copy::capture_from_user(
                SafeString as *mut c_void,
                UnsafeString as *const c_void,
                StringSize as usize,
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
            if NT_SUCCESS(Status) && !UnsafeDx.is_null() {
                SafeDx = Buffer as *mut INT;
                if !crate::copy::capture_from_user(
                    SafeDx as *mut c_void,
                    UnsafeDx as *const c_void,
                    DxSize as usize,
                ) {
                    Status = STATUS_ACCESS_VIOLATION;
                }
            }
        }
        if !NT_SUCCESS(Status) {
            if Buffer as *mut u8 != LocalBuffer.as_mut_ptr() {
                ExFreePoolWithTag(Buffer, GDITAG_TEXT);
            }
            return 0;
        }
    }

    if !UnsafeRect.is_null() {
        Status = SafeProbeForRead(UnsafeRect as PVOID, core::mem::size_of::<RECT>(), 1);
        if NT_SUCCESS(Status)
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(SafeRect) as *mut core::ffi::c_void,
                UnsafeRect as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if !NT_SUCCESS(Status) {
            if Buffer as *mut u8 != LocalBuffer.as_mut_ptr() {
                ExFreePoolWithTag(Buffer, GDITAG_TEXT);
            }
            return 0;
        }
    }

    Result = GreExtTextOutW(
        hDC, XStart, YStart, fuOptions,
        core::ptr::addr_of!(SafeRect) as PRECTL,
        SafeString, Count as INT, SafeDx, dwCodePage,
    );

    if Buffer as *mut u8 != LocalBuffer.as_mut_ptr() {
        ExFreePoolWithTag(Buffer, GDITAG_TEXT);
    }
    Result
}

// NtGdiGetCharABCWidthsW: _NtGdiGetCharABCWidthsW@24 on i386 (6 args x 4 bytes).
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetCharABCWidthsW(
    hDC: HDC,
    FirstChar: UINT,
    Count: ULONG,
    UnSafepwch: PCWCH,
    fl: FLONG,
    Buffer: PVOID,
) -> BOOL {
    ntgdi_get_char_abc_widths_w_impl(hDC, FirstChar, Count, UnSafepwch, fl, Buffer)
}

#[inline(always)]
unsafe fn ntgdi_get_char_abc_widths_w_impl(
    hDC: HDC,
    FirstChar: UINT,
    Count: ULONG,
    UnSafepwch: PCWCH,
    fl: FLONG,
    Buffer: PVOID,
) -> BOOL {
    let SafeBuff: LPABC;
    let mut SafeBuffF: LPABCFLOAT = core::ptr::null_mut();
    let dc: PDC;
    let pdcattr: *mut DC_ATTR;
    let TextObj: PTEXTOBJ;
    let FontGDI: PFONTGDI;
    let face: FT_Face;
    let mut charmap: FT_CharMap;
    let mut found: FT_CharMap = core::ptr::null_mut();
    let mut i: UINT;
    let mut glyph_index: UINT;
    let BufferSize: UINT;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Safepwch: *mut WCHAR = core::ptr::null_mut();
    let plf: *mut LOGFONTW;

    if Buffer.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !UnSafepwch.is_null() {
        let pwchSize: UINT = Count * core::mem::size_of::<WCHAR>() as UINT;
        Safepwch = ExAllocatePoolWithTag(PagedPool, pwchSize as usize, GDITAG_TEXT) as *mut WCHAR;
        if Safepwch.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        Status = SafeProbeForRead(UnSafepwch as PVOID, pwchSize as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault lands in Status,
        // which is checked below -- upstream's _SEH2_EXCEPT does exactly this.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(Safepwch as *mut c_void, UnSafepwch as *const c_void, pwchSize as usize)
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        if !Safepwch.is_null() {
            ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
        }
        SetLastNtError(Status);
        return 0;
    }

    BufferSize = Count * core::mem::size_of::<_ABC>() as UINT;
    SafeBuff = ExAllocatePoolWithTag(PagedPool, BufferSize as usize, GDITAG_TEXT) as LPABC;
    if fl == 0 { SafeBuffF = SafeBuff as LPABCFLOAT; }
    if SafeBuff.is_null() {
        if !Safepwch.is_null() {
            ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        ExFreePoolWithTag(SafeBuff as PVOID, GDITAG_TEXT);
        if !Safepwch.is_null() {
            ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*dc).pdcattr;
    hFont = (*pdcattr).hlfntNew as _ ;
    TextObj = RealizeFontInit(hFont);
    DC_UnlockDc(dc);

    if TextObj.is_null() {
        ExFreePoolWithTag(SafeBuff as PVOID, GDITAG_TEXT);
        if !Safepwch.is_null() {
            ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    // freetype.c: `FontGDI = ObjToGDI(TextObj->Font, FONT)`. This read `ObjToGDI(TextObj, ...)`,
    // the LFONT itself, as if it were the realized FONTGDI (the only such site in win32k), so
    // SharedFace was whatever lay at that offset in the TEXTOBJ: NULL, and the guard below
    // refused every call. That is what "an unrealized font reaches this syscall" was, and why
    // every SS_*ELLIPSIS static (whose fit is measured here) drew ONE glyph: file Properties,
    // dxdiag's values, Regional Options' samples.
    FontGDI = ObjToGDI!((*TextObj).Font, FONT);

    /* An unrealized font has no SharedFace, and dereferencing it here BUGCHECKs
     * the machine (0x1E, c0000005). Measured twice from 32-bit guests in the
     * applet phase of a full-drain ladder boot, at the two instructions this
     * pair compiles to: once loading `SharedFace->Face`, once later unlocking
     * through the garbage `Face` it had loaded. TextObj is null-checked just
     * above and survives, so FontGDI is fine — it is specifically SharedFace.
     *
     * Upstream does not check here either, but it is not consistent about it:
     * IntGetOutlineTextMetrics in this same file guards `(*FontGDI).SharedFace`
     * before use. We take the guarded reading. Killing the box is never the
     * better failure, and a syscall reachable from an unprivileged 32-bit guest
     * must not be able to do it — so this returns the same clean
     * ERROR_INVALID_HANDLE the TextObj-missing path above already returns.
     *
     * Loud on purpose: WHY an unrealized font reaches this syscall is a
     * separate open question (roadmap registry), and a silent guard would
     * retire the only evidence of it. */
    if (*FontGDI).SharedFace.is_null() || (*(*FontGDI).SharedFace).Face.is_null() {
        DbgPrint(
            b"NtGdiGetCharABCWidthsW: unrealized font (SharedFace=%p) - refusing rather than faulting\n\0"
                .as_ptr() as *const i8,
            (*FontGDI).SharedFace,
        );
        w32r_TEXTOBJ_UnlockText(TextObj);
        ExFreePoolWithTag(SafeBuff as PVOID, GDITAG_TEXT);
        if !Safepwch.is_null() {
            ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
        }
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    face = (*(*FontGDI).SharedFace).Face;

    if (*face).charmap.is_null() {
        let num_charmaps = (*face).num_charmaps as UINT;
        i = 0;
        while i < num_charmaps {
            charmap = *(*face).charmaps.add(i as usize);
            if (*charmap).encoding != 0 {
                found = charmap;
                break;
            }
            i += 1;
        }
        if found.is_null() {
            ExFreePoolWithTag(SafeBuff as PVOID, GDITAG_TEXT);
            if !Safepwch.is_null() {
                ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
            }
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }
        IntLockFreeType();
        FT_Set_Charmap(face, found);
        IntUnLockFreeType();
    }

    plf = core::ptr::addr_of_mut!((*TextObj).logfont.elfEnumLogfontEx.elfLogFont);

    IntLockFreeType();
    IntRequestFontSize(dc, FontGDI, (*plf).lfWidth, (*plf).lfHeight);
    FT_Set_Transform(face, core::ptr::null_mut(), core::ptr::null_mut());

    i = FirstChar;
    while i < FirstChar + Count {
        let adv: INT;
        let lsb: INT;
        let bbx: INT;
        let left: INT;
        let right: INT;

        if !Safepwch.is_null() {
            glyph_index = get_glyph_index_flagged(
                face,
                *Safepwch.add((i - FirstChar) as usize) as FT_ULong,
                (fl & GCABCW_INDICES as FLONG) as BOOL,
            );
        } else {
            glyph_index = get_glyph_index_flagged(
                face,
                i as FT_ULong,
                (fl & GCABCW_INDICES as FLONG) as BOOL,
            );
        }
        FT_Load_Glyph(face, glyph_index, FT_LOAD_DEFAULT as FT_Int32);

        left  = ((*(*face).glyph).metrics.horiBearingX as INT) & -64;
        right = (((*(*face).glyph).metrics.horiBearingX + (*(*face).glyph).metrics.width) as INT + 63) & -64;
        adv   = (((*(*face).glyph).advance.x as i64 + 32) >> 6) as INT;

        lsb = left >> 6;
        bbx = (right - left) >> 6;

        if fl == 0 {
            (*SafeBuffF.add((i - FirstChar) as usize)).abcfA = lsb as f32;
            (*SafeBuffF.add((i - FirstChar) as usize)).abcfB = bbx as f32;
            (*SafeBuffF.add((i - FirstChar) as usize)).abcfC = (adv - lsb - bbx) as f32;
        } else {
            (*SafeBuff.add((i - FirstChar) as usize)).abcA = lsb;
            (*SafeBuff.add((i - FirstChar) as usize)).abcB = bbx as UINT;
            (*SafeBuff.add((i - FirstChar) as usize)).abcC = adv - lsb - bbx;
        }
        i += 1;
    }
    IntUnLockFreeType();
    w32r_TEXTOBJ_UnlockText(TextObj);

    Status = MmCopyToCaller(Buffer, SafeBuff as *const c_void, BufferSize as ULONG);
    ExFreePoolWithTag(SafeBuff as PVOID, GDITAG_TEXT);
    if !Safepwch.is_null() {
        ExFreePoolWithTag(Safepwch as PVOID, GDITAG_TEXT);
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    1 // TRUE
}
