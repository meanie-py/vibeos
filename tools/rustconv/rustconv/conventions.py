"""The calling-convention alias table — the SINGLE source of truth.

Three consumers need to know that `WSAAPI` means __stdcall:

  * abi_enforce  — derives a translated fn's convention from the RAW C text
  * scanner      — records `calling_convention` on each parsed CFunction
  * verify_conv_macros — re-derives this table from the headers and fails on drift

They used to keep three separate lists (abi_enforce's `_CONV_KEYWORDS`, scanner's
`CALLING_CONVENTIONS`, and RE_UNPARSEABLE_CC), which is the "two copies of the truth"
ghost this repo has a whole program about — and they had DIFFERENT contents, so the same
macro could be a known convention to one and invisible to the other. This module is the
one copy. It is a leaf (imports nothing from the package) so both sides can depend on it
without a cycle: abi_enforce already imports scanner.

See tools/verify_conv_macros.py — it resolves the transitive closure of `#define`s in the
real SDK headers and diffs it against this table, wired into `vibeos-cli check` with an
EMPTY baseline. Do not hand-add an entry without a file:line citation.
"""

from __future__ import annotations

# Convention macros as they appear in the C text. Order = lookup priority.
#
# The table below is the TRANSITIVE CLOSURE of every convention-aliasing macro in the
# SDK headers, not a hand-picked list — because a single-level reading misses the
# second-order aliases. `#define WSPAPI WSAAPI` (ws2spi.h:39) -> `#define WSAAPI FAR
# PASCAL` (winsock2.h:578) -> `#define PASCAL WINAPI` -> `__stdcall`. We read the RAW C
# text (not preprocessed output), so an unknown alias does NOT fall back to "unknown" —
# `_derive_convention` returns **cdecl**, silently. On i386 that is a stack imbalance on
# every call the OS makes through the pointer.
#
# ws2_32 is the case that forced this: EVERY one of its 187 exports is declared
#   SOCKET
#   WSAAPI
#   socket(int af, int type, int protocol)
# so before this table the whole DLL would have been derived cdecl. `SEC_ENTRY` (sspi.h)
# does the same to secur32, `NET_API_FUNCTION` to netapi32, `WINGDIPAPI` to gdiplus,
# `RPC_ENTRY`/`__RPC_USER` to rpcrt4 + every generated proxy, `DESK_EXT_CALLBACK` to the
# desk control panel, `ACMAPI` to the audio codecs.
#
# ole32/oleaut32 escaped this only by luck: Wine's style writes `HRESULT WINAPI Foo(...)`
# at definition sites and reserves STDAPI/WINOLEAPI for the *headers*. ROS-native code
# (ws2_32, mswsock) puts the macro on the definition, so it does not get that luck.
#
# Regenerate with tools/verify_conv_macros.py --emit; `vibeos-cli check` runs it in
# --check mode and FAILS if a header adds an alias this table doesn't know. Do not hand-
# edit an entry without re-running it. Citations are file:line into the real headers.
#
# `WINAPIV` is __cdecl (minwindef.h:114), NOT a WINAPI variant — as are the other V-suffix
# variadic forms (STDAPIV, VFWAPIV, STDMETHODVCALLTYPE, RPC_VAR_ENTRY). Listed explicitly
# so nobody "fixes" them into the system bucket. (`\b` already keeps the WINAPI pattern
# from matching WINAPIV.)
#
# `CALLBACK`/`PASCAL` are __stdcall (minwindef.h:109/:117). Missing them was not academic:
# they mark the fns C hands to the OS as *function pointers* (an `EnumFontFamiliesEx` enum
# proc, a `DialogBoxParam` DlgProc), so user32/gdi32 calls them through the pointer with
# stdcall, and a Rust-ABI body there corrupts the i386 stack on a call we never make
# ourselves. Their absence silently derived oleaut32's `font_enum_proc` (olefont.c) and
# `property_sheet_proc` (olepropframe.c) as cdecl — caught by the census, 2026-07.
KW_TO_CONV: dict[str, str] = {
    # --- the raw compiler keywords: the ROOTS every alias below resolves to. They are in
    # the table because the scanner normalizes the unparseable aliases to `__stdcall`
    # before tree-sitter sees them (see scanner._normalize_callbacks), so a CFunction's
    # `calling_convention` field routinely IS `__stdcall` — and abi_enforce's fallback
    # looks the field up right here. Omitting them means that lookup misses and silently
    # returns cdecl.
    "__stdcall": "system",
    "_stdcall": "system",
    "__cdecl": "cdecl",
    "_cdecl": "cdecl",
    "__fastcall": "fastcall",
    "_fastcall": "fastcall",
    # `this` in ECX on i386, and the DEFAULT C convention everywhere else — the compiler
    # accepts and ignores the keyword off x86. Absent from this table, msvcrt's cpp.c lost
    # 25 of its 62 bodies from the translation plan, SILENTLY: the drop is the documented
    # `TYPE * <conv> name(...)` shape that tree-sitter-c cannot parse in any spelling, and
    # scanner.RE_CC_AFTER_PTR (which repairs it) is built from the keys of THIS dict. The
    # split was invisible because it fell along the return type — `void __thiscall
    # bad_typeid_dtor(...)` parsed while `bad_typeid * __thiscall bad_typeid_ctor(...)`
    # next to it did not, so the file looked partially translatable rather than broken.
    # Same class as the `FILE * CDECL fopen(...)` drop that cost file.c 15 functions.
    "__thiscall": "thiscall",
    "_thiscall": "thiscall",
    # --- __stdcall ("system") ---
    "NTAPI": "system",
    "WINAPI": "system",  # psdk/minwindef.h
    "APIENTRY": "system",
    "STDCALL": "system",
    "CALLBACK": "system",  # psdk/minwindef.h:109
    "PASCAL": "system",  # psdk/minwindef.h:117
    "MI_CALL": "system",  # psdk/mi.h:310 (OMI MI.h: __stdcall on _WIN32, as mi.dll)
    "pascal": "system",  # psdk/minwindef.h:100
    "STDMETHODCALLTYPE": "system",  # psdk/basetyps.h:37
    "STDAPICALLTYPE": "system",  # psdk/basetyps.h:39
    "STDAPI": "system",  # psdk/basetyps.h:41  (EXTERN_C HRESULT STDAPICALLTYPE)
    "STDMETHODIMP": "system",  # psdk/basetyps.h:43
    "BOOLAPI": "system",  # psdk/winhttp.h:29  (BOOL WINAPI — type-led, so easy to miss)
    "WINOLEAPI": "system",  # psdk/objbase.h:1054
    "WSAAPI": "system",  # psdk/winsock2.h:578  (FAR PASCAL)
    "WSPAPI": "system",  # psdk/ws2spi.h:39     (-> WSAAPI)
    "SEC_ENTRY": "system",  # psdk/sspi.h:27
    "NET_API_FUNCTION": "system",  # psdk/lmcons.h:7
    "WINGDIPAPI": "system",  # psdk/gdiplusmem.h:22
    "ACMAPI": "system",  # psdk/msacm.h
    "VFWAPI": "system",  # psdk/vfw.h:26
    "WMIAPI": "system",  # psdk/evntrace.h:8
    "EVNTAPI": "system",  # psdk/evntprov.h:28
    "IMAGEAPI": "system",  # psdk/imagehlp.h:84
    "DBHLPAPI": "system",  # psdk/imagehlp.h:95
    "INSTAPI": "system",  # psdk/odbcinst.h:11
    "SQL_API": "system",  # psdk/sqltypes.h:8
    "DWMAPI": "system",  # psdk/dwmapi.h:28
    "PSSTDAPI": "system",  # psdk/propvarutil.h:27
    "STRSAFEAPI": "system",  # psdk/strsafe.h:80
    "STRSAFE_INLINE_API": "system",  # psdk/strsafe.h:81
    "WINSNMPAPI": "system",  # psdk/winsnmp.h:34
    "NETIOAPI_API": "system",  # psdk/netioapi.h:16
    "NETIOAPI_API_": "system",  # psdk/netioapi.h:10
    "PFAPIENTRY": "system",  # psdk/fltdefs.h:38
    "NCB_POST": "system",  # psdk/nb30.h:91
    "EXPENTRY": "system",  # psdk/ddeml.h:45
    "DDAPI": "system",  # psdk/ddrawi.h:36
    "APIPRIVATE": "system",  # psdk/wingdi.h
    "CPSUICALLBACK": "system",  # psdk/compstui.h:812
    "DESK_EXT_CALLBACK": "system",  # reactos/dll/desk/deskcplx.h:4
    "MSGDUMP_API": "system",  # reactos/msgdump.h:30
    "RPC_ENTRY": "system",  # psdk/rpc.h:63
    "__RPC_API": "system",  # psdk/rpc.h:60
    "__RPC_USER": "system",  # psdk/rpc.h:61
    "__RPC_STUB": "system",  # psdk/rpc.h:62
    "__RPC_CALLEE": "system",  # psdk/rpcndr.h:86
    # --- __cdecl ---
    "CDECL": "cdecl",
    "MI_MAIN_CALL": "cdecl",  # psdk/mi.h:288 (MI_Application_InitializeV1, cdecl on i386 Win11)
    "WINAPIV": "cdecl",  # psdk/minwindef.h:114
    "STDAPIV": "cdecl",  # psdk/basetyps.h:45
    "STDAPIVCALLTYPE": "cdecl",  # psdk/basetyps.h:40
    "STDMETHODIMPV": "cdecl",  # psdk/basetyps.h:47
    "STDMETHODVCALLTYPE": "cdecl",  # psdk/basetyps.h:38
    "VFWAPIV": "cdecl",  # psdk/vfw.h:27
    "RPC_VAR_ENTRY": "cdecl",  # psdk/rpcndr.h:87
    "EFIAPI": "cdecl",  # vendor/edk2/ProcessorBind.h:260
    # --- __fastcall ---
    "FASTCALL": "fastcall",
}
# Lookup priority. Longest-first so a V-suffix variadic form is never shadowed by its
# stdcall stem (`\b` already prevents WINAPI from matching WINAPIV, but the ordering is
# the belt to that suspenders — a future non-\b caller of this tuple stays correct).
CONV_KEYWORDS: tuple[str, ...] = tuple(sorted(KW_TO_CONV, key=len, reverse=True))

# The aliases above that are NOT only a convention: each expands to a RETURN TYPE, and
# some to a linkage specifier as well. `STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE`, so
# the whole return type of the function lives inside the macro.
#
# That distinction is not cosmetic. The scanner normalizes an unparseable alias to the raw
# `__stdcall` keyword, which for these DELETES the return type: `STDAPI DllRegisterServer
# (void)` becomes `__stdcall DllRegisterServer(void)`, a definition with no type at all,
# which tree-sitter does not recognize as a function_definition — so the function silently
# leaves the translation plan. netcfgx.c lost `DllGetClassObject`, `DllRegisterServer` and
# `DllUnregisterServer` that way, all three .spec-exported, and the whole TU refused to
# translate for a year. It is the same class as the `TYPE * CDECL fn()` drop (0b2d3521e0a)
# that cost msvcrt 15 exports: a convention keyword taken for the whole decoration.
#
# The value is the header's own expansion, VERBATIM, resolved by fixpoint — `WINOLEAPI` is
# `STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE` — so the existing `EXTERN_C` and alias
# normalizations finish the job and no judgement is encoded here. Every entry is proven by
# a test that extracts a function defined with it (tests/test_composite_conventions.py):
# an expansion nobody parsed is a guess, and a guess here drops functions silently.
KW_EXPANSION: dict[str, str] = {
    "STDAPI": "EXTERN_C HRESULT STDAPICALLTYPE",  # psdk/basetyps.h:41
    "STDAPIV": "EXTERN_C HRESULT STDAPIVCALLTYPE",  # psdk/basetyps.h:45
    "STDMETHODIMP": "HRESULT STDMETHODCALLTYPE",  # psdk/basetyps.h:43
    "STDMETHODIMPV": "HRESULT STDMETHODVCALLTYPE",  # psdk/basetyps.h:47
    "WINOLEAPI": "STDAPI",  # psdk/objbase.h:1054
    "PSSTDAPI": "STDAPI",  # psdk/propvarutil.h:27 (DECLSPEC_IMPORT arm is declaration-only)
    "BOOLAPI": "BOOL WINAPI",  # psdk/winhttp.h:29, psdk/wininet.h:27
    "NETIOAPI_API": "NETIO_STATUS NETIOAPI_API_",  # psdk/netioapi.h:16
    "NCB_POST": "void CALLBACK",  # psdk/nb30.h:91
    "STRSAFEAPI": "HRESULT WINAPI",  # psdk/strsafe.h:80 (`static __inline` dropped: see test)
    "STRSAFE_INLINE_API": "HRESULT WINAPI",  # psdk/strsafe.h:81 (ditto `__CRT_INLINE`)
}

# The FUNCTION-LIKE half of the same family: these take the return type as an argument, so
# the keyword table above cannot reach them and `STDMETHODIMP_(ULONG) AddRef()` survived
# into the translated signature as a literal return type `STDMETHODIMP_(ULONG)`. Values are
# the header's own expansions, verbatim, with `\1` for the argument — the existing
# EXTERN_C/alias normalizations then finish the job exactly as they do for the keyword
# forms. Every hand-rolled COM class declares AddRef and Release this way.
KW_EXPANSION_FN: dict[str, str] = {
    "STDAPI_": r"EXTERN_C \1 STDAPICALLTYPE",  # psdk/basetyps.h:42
    "STDMETHODIMP_": r"\1 STDMETHODCALLTYPE",  # psdk/basetyps.h:44
    "STDAPIV_": r"EXTERN_C \1 STDAPIVCALLTYPE",  # psdk/basetyps.h:46
    "STDMETHODIMPV_": r"\1 STDMETHODVCALLTYPE",  # psdk/basetyps.h:48
}
