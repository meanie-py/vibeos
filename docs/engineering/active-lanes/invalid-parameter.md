<!-- ucrtbase de-Microsoft-ification quest, 2026-09-20. Produced by a read-only
     survey agent and reviewed before landing. FINDINGS, not a plan of record.
     Two of this round's reports CORRECTED an earlier round's claims, so treat
     every statement here as re-derivable rather than settled: claims are cited
     to file:line, and each report names what it could NOT determine.
     Burn-down: `python3 tools/ucrt_burndown.py --arch i386`.
     Declarations for implemented families: tools/ucrt-takeover.yaml. -->

# ucrtbase invalid-parameter family — findings

Repo: `/home/curi/Projects/os-playground/vibeos-msvcrt-homebrew` (all paths below are
relative to it unless stated otherwise). Target C source:
`sdk/lib/ucrt/misc/invalid_parameter.cpp`. Proposed Rust: `../invalid-parameter.rs` (staged
outside the repo, NOT wired in — see its header).

## 1. Contract of each of the 7 exports

Signatures from `sdk/include/ucrt/stdlib.h` and `sdk/include/ucrt/corecrt.h`; export names
and calling convention confirmed against `dll/win32/ucrtbase/ucrtbase.spec` (both the
canonical name and its `_o_` ordinal-forwarder alias, which just forwards — no separate
behaviour to model).

| Export | Header decl | spec line(s) | Default (no handler installed) |
|---|---|---|---|
| `_get_invalid_parameter_handler` | `stdlib.h:182` | 427, 1050 (`_o_` alias) | returns `NULL` |
| `_get_thread_local_invalid_parameter_handler` | `stdlib.h:188` | 435, 1056 | returns `NULL` |
| `_invalid_parameter_noinfo` | `corecrt.h:382` (`_ACRTIMP_ALT`) | 486, 1097 | falls through to `_invoke_watson` |
| `_invalid_parameter_noinfo_noreturn` | `corecrt.h:383` (`noreturn`) | 487, 1098 | calls `_invalid_parameter` then unconditionally `_invoke_watson` too |
| `_invoke_watson` | `corecrt.h:386-391` (`noreturn`) | 488 | fast-fails / terminates the process; never returns |
| `_set_invalid_parameter_handler` | `stdlib.h:178-181` | 1955, 1397 (`_o_`) | installs handler, returns previous (`NULL` first time) |
| `_set_thread_local_invalid_parameter_handler` | `stdlib.h:184-187` | 1961, 1400 (`_o_`) | installs per-thread handler, returns previous (`NULL` first time) |

`_invalid_parameter` itself (the 8th name in the same C file) is declared `_ACRTIMP` only
under `#ifdef _DEBUG` (`corecrt.h:373-380`) — it is NOT one of the 7 requested and is
deliberately out of scope for the staged file; see its header comment for why.

All handler-typed parameters/returns use:
```c
typedef void (__cdecl* _invalid_parameter_handler)(
    wchar_t const*, wchar_t const*, wchar_t const*, unsigned int, uintptr_t);
```
— `stdlib.h:162-168`.

## 2. Global vs thread-local precedence — the subtle part

From `sdk/lib/ucrt/misc/invalid_parameter.cpp`, `_invalid_parameter_internal` (lines ~94-113
of that file):

```
raw_ptd = ptd.get_raw_ptd_noexit()
if raw_ptd && raw_ptd->_thread_local_iph:
    call raw_ptd->_thread_local_iph(...); return
global_handler = decode(__acrt_invalid_parameter_handler.value(ptd))
if global_handler:
    call global_handler(...); return
_invoke_watson(...)
```

So: **thread-local wins if set, else global, else `_invoke_watson`.** This is NOT
re-implemented in the staged Rust (it lives in the not-yet-written `_invalid_parameter`
body), but it is the contract the 4 accessor/handler-slot functions in scope must support
without drifting from.

- **Thread-local default**: unset (`_thread_local_iph` is a field of `__acrt_ptd`,
  zero-initialized per-thread block — no explicit init call sets it to anything but 0/NULL).
- **`_set_*` return value**: both setters return the handler that was installed
  immediately before the call (or `NULL` the first time), never the one just being
  installed. Confirmed both by the C (`old_handler` captured before overwrite in both
  `_set_invalid_parameter_handler` and `_set_thread_local_invalid_parameter_handler`) and by
  the winetest (`modules/rostests/winetests/ucrtbase/misc.c:384-425`,
  `test_invalid_parameter_handler`) — see the transcript below.
- **The two slots are fully independent** — setting one never affects the other, and both
  start `NULL`. Confirmed by the winetest sequence: it sets thread-local first, reads global
  and finds it still `NULL`, sets global, reads thread-local and finds its own earlier value
  still intact, then triggers an actual invalid-parameter call (`_ltoa_s(0, NULL, 0, 0)`)
  twice — once while the thread-local handler is set (only IT fires) and once after clearing
  it with `_set_thread_local_invalid_parameter_handler(NULL)` (only the GLOBAL one fires).
  That second transition is the direct proof of the precedence order above, not just a
  documentation claim.

```c
// modules/rostests/winetests/ucrtbase/misc.c:384-425
ret = _get_invalid_parameter_handler();             ok(!ret, ...);   // NULL by default
ret = _get_thread_local_invalid_parameter_handler(); ok(!ret, ...);   // NULL by default
ret = _set_thread_local_invalid_parameter_handler(thread_invalid_parameter_handler);
ok(!ret, ...);                                                        // prev was NULL
...
SET_EXPECT(thread_invalid_parameter_handler);
_ltoa_s(0, NULL, 0, 0);
CHECK_CALLED(thread_invalid_parameter_handler);       // thread-local wins over global
ret = _set_thread_local_invalid_parameter_handler(NULL);
ok(ret == thread_invalid_parameter_handler, ...);      // returns the PREVIOUS value
SET_EXPECT(global_invalid_parameter_handler);
_ltoa_s(0, NULL, 0, 0);
CHECK_CALLED(global_invalid_parameter_handler);        // falls through to global once
                                                        // thread-local is cleared
```

## 3. What `_invalid_parameter_noinfo` / `_noinfo_noreturn` pass, and how they differ

Both call `_invalid_parameter(nullptr, nullptr, nullptr, 0, 0)` — all-null
expression/function/file, line 0, reserved 0 (`invalid_parameter.cpp`, the
`_invalid_parameter_noinfo` body). This is the entry point release-mode `_s`-function
contract checks route through when built without `_DEBUG`: `corecrt.h:392-410` shows the
macro `_CRT_SECURE_INVALID_PARAMETER(expr)` expands to
`::_invalid_parameter(_CRT_WIDE(#expr), __FUNCTIONW__, __FILEW__, __LINE__, 0)` under
`_DEBUG` (real diagnostic info) but to `::_invalid_parameter_noinfo_noreturn()` otherwise —
i.e. release builds intentionally throw away the expression/function/file/line detail.

The `_noreturn` variant differs by ALSO calling `_invoke_watson(nullptr, nullptr, nullptr,
0, 0)` immediately afterward, unconditionally — even if an installed handler's
`_invalid_parameter` call returned normally to the caller. `corecrt.h:397-406`'s own
comment states this explicitly: *"Even if `_set_invalid_parameter_handler()` is used to set
a new invalid parameter handler which does return control to the application,
`_invalid_parameter_noinfo_noreturn()` will terminate the application and invoke Watson."*
This is a deliberate double-call, not redundancy to simplify away.

## 4. `_invoke_watson` — realism check

Two branches in the vendored C, gated on `(defined _M_IX86 || defined _M_X64) &&
!defined _UCRT_ENCLAVE_BUILD`:

- **x86/x64 branch** (what i386 and amd64 use here — `_UCRT_ENCLAVE_BUILD` is not defined
  anywhere in this tree):
  1. `IsProcessorFeaturePresent(PF_FASTFAIL_AVAILABLE)` → `__fastfail(FAST_FAIL_INVALID_ARG)`
     if true. `__fastfail` is a single non-maskable trap instruction (`int 0x29` on x86/x64)
     that never returns and does not unwind — closest thing Windows has to an abort()
     primitive purpose-built for CRT/library contract violations.
  2. Otherwise, `__acrt_call_reportfault`: ~80 lines of hand-written, per-arch inline
     assembly that captures raw register state into a `CONTEXT` record (reading
     `eax/ecx/edx/ebx/esi/edi`, segment registers, EFLAGS by hand on i386; RtlVirtualUnwind
     on x64), fakes an `EXCEPTION_RECORD`, clears any installed unhandled-exception filter,
     and calls `UnhandledExceptionFilter` — **this is the actual mechanism that shows the
     Watson/WER "this program has stopped working" dialog on real Windows** and lets a
     debugger intercept the fault first. Falls back to `TerminateProcess(GetCurrentProcess(),
     STATUS_INVALID_CRUNTIME_PARAMETER)` if that doesn't end the process.
- **Non-x86/x64 branch** (`_M_ARM64` etc. — theoretically covers our arm64 target, though in
  practice it's dead in this fork since `_UCRT_ENCLAVE_BUILD` is never set and the guard is
  actually `#if (_M_IX86 || _M_X64) && !enclave` / `#else`, so arm64 DOES take this simpler
  `#else` arm): unconditionally just `__fastfail(FAST_FAIL_INVALID_ARG)`, no reportfault path
  at all.

**What's realistic for ReactOS, and what I'm unsure of:**
- Watson/WER has no ReactOS analogue — there is nothing meaningful to report to, so
  reproducing `__acrt_call_reportfault`'s ~80 lines of raw per-arch asm to feed a dialog
  that doesn't exist is not worth doing. I did not attempt it.
- `TerminateProcess(GetCurrentProcess(), STATUS_INVALID_CRUNTIME_PARAMETER)` is not an
  invented shortcut — it is literally what the real C does too, as its OWN fallback after
  `__acrt_call_reportfault` returns (which it always eventually does, since
  `UnhandledExceptionFilter` returning `EXCEPTION_CONTINUE_SEARCH` with no debugger present
  just breaks into the debugger hook and then falls through). So using it directly, skipping
  the reportfault detour, reproduces the REAL observable end state (process dies with that
  exact status code) for the common case of no attached debugger/filter.
- **What I could NOT determine**: whether this tree has (or should get) a `__fastfail`
  intrinsic already. I grepped `sdk/lib/rust/` for `fastfail`/`FAST_FAIL_INVALID_ARG` and
  found no existing wrapper. `FAST_FAIL_INVALID_ARG` itself is defined at
  `sdk/include/xdk/rtlfuncs.h:11` as `5`, so the constant exists, but nothing implements the
  actual trap instruction in Rust anywhere I could find. Building that (a 2-3 instruction
  per-arch asm stub) is a small, separate, well-scoped task — I left it out rather than
  bundling an unrelated primitive into this file, and flagged it with `// NEEDS WIRING` at
  the `_invoke_watson` body instead of inventing it.
- I could not determine whether a debugger being attached at the moment of a genuine
  invalid-parameter fault should behave specially in ReactOS (real UCRT's `_CRT_DEBUGGER_HOOK`
  path). Left unaddressed; the staged file's `_invoke_watson` never attempts a debugger
  break-in.

## 5. The encoding question — confirmed

Real UCRT encodes the GLOBAL handler pointer, XOR against `__security_cookie`, via
`__crt_fast_encode_pointer`/`__crt_fast_decode_pointer`
(`sdk/lib/vcruntime/inc/internal_shared.h:186-218`):

```cpp
template<typename T>
T __crt_fast_encode_pointer(T ptr) {
    return reinterpret_cast<T>(reinterpret_cast<uintptr_t>(ptr) ^ __security_cookie);
}
// __crt_fast_decode_pointer is identical (XOR is its own inverse)
```

`invalid_parameter.cpp`'s `_set_invalid_parameter_handler`/`_get_invalid_parameter_handler`
call these explicitly around the `__acrt_invalid_parameter_handler` slot. The
**thread-local** slot (`_thread_local_iph`) is NOT encoded anywhere in the C — it is read
and written as a bare pointer in `__acrt_ptd`.

`__security_cookie` itself: declared `extern uintptr_t __security_cookie;` in
`sdk/include/vcruntime/vcruntime.h:220` (and again in `sdk/include/crt/process.h:372`),
defined in `sdk/lib/vcruntime/__security_init_cookie.c:21-22` alongside its complement, and
consumed by the i386/amd64 `__security_check_cookie.s` stack-cookie stubs.

**Does our tree already have a Rust equivalent?** Yes: `sdk/lib/rust/crt-startup-rust/src/gs.rs`
defines
```rust
pub static mut __security_cookie: usize = DEFAULT_SECURITY_COOKIE;
pub static mut __security_cookie_complement: usize = !DEFAULT_SECURITY_COOKIE;
```
with a real init routine that randomizes both (lines ~18-67). So the primitive exists, just
not in `ucrt-rust` and not reachable from it today — I found no dependency edge from
`ucrt-rust` to `crt-startup-rust`, and `msvcrt-crt-rust`/`dll/win32/msvcrt/` have their OWN
`_encode_pointer`/`_decode_pointer` (`dll/win32/msvcrt/misc.c:439-460`) that just forward to
`kernel32`'s `EncodePointer`/`DecodePointer` — a completely different mechanism (OS-level
opaque encoding, not the CRT's own cookie-XOR), used for the PUBLIC `_encode_pointer` API,
not for this internal handler-slot encoding. **This is a real, not-yet-closed dependency**:
the staged `.rs` file stores the global handler UNENCODED (identity encode/decode against a
stub cookie fixed at 0) and marks the gap with `// NEEDS WIRING` rather than silently
pretending it's solved. It does not affect the functional (get/set round-trip) contract,
only the security property (resistance to a memory-disclosure attack reading the raw
function pointer out of process memory).

## 6. What our tree already has

**`dll/win32/msvcrt/errno.c`** (Wine-derived C, msvcrt.dll, `_MSVCR_VER=0`) implements ALL
7 requested names PLUS `_invalid_parameter` itself, but under `#if _MSVCR_VER >= 80` /
`#if _MSVCR_VER >= 140` guards that are **not compiled** at this DLL's `_MSVCR_VER=0` — so
only bare `_invalid_parameter` (errno.c:459-479) actually ships in msvcrt.dll. It:
```c
void __cdecl _invalid_parameter(...) {
#if _MSVCR_VER >= 140
    /* thread-local lookup — NOT COMPILED at ver 0 */
#endif
    if (invalid_parameter_handler) invalid_parameter_handler(...);
    else {
        ERR(...);
#if _MSVCR_VER >= 80
        RaiseException(STATUS_INVALID_CRUNTIME_PARAMETER, ...); /* NOT COMPILED at ver 0 */
#endif
    }
}
```
i.e. at `_MSVCR_VER=0` there is **no thread-local handler, no fastfail/RaiseException, no
Watson** — just "call the installed global handler if any, else log and RETURN NORMALLY to
the caller." This already has a Rust body:
`sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/errno.rs:507-524`, gated behind the
`invalid-parameter` cargo feature, reproducing exactly that reduced contract (confirmed
faithful by its own comment block, and by `docs/engineering/rust-port-status.md:154-160`
noting the harness's call-counter and the real slot are "identical in effect and nothing is
hidden" at ver 0).

**This is a fundamentally different contract from UCRT's**, and the two must not be
conflated:
- msvcrt @ ver 0: single global slot only, no encoding, unhandled case LOGS and RETURNS
  (does not terminate).
- ucrtbase (genuine UCRT, `_MSVCR_VER=140` semantics): global slot (XOR-encoded) AND
  thread-local slot with precedence, unhandled case calls `_invoke_watson` which
  **terminates the process**.

The msvcrt Rust body is a useful STYLE reference (same crate ecosystem, same
`_invalid_parameter_handler` typedef spelling, same "island doesn't have this typedef, spell
it from the header" pattern) but its BEHAVIOUR must not be reused or assumed equivalent —
the staged `.rs` file implements UCRT's real contract, not msvcrt's reduced one, and cites
this distinction inline.

`sdk/lib/rust/ucrt-rust/` (the crate the staged file targets) currently implements only
`bits` (rotate/byteswap, `src/bits.rs`) and always-on `locale` layout pins (`src/locale.rs`)
— nothing touching invalid-parameter, handler slots, or per-thread state yet.
`tools/ucrt-takeover.yaml` has no `invalid_parameter`/`invoke_watson` family entry.

**No Rust `__acrt_ptd`/`__acrt_getptd` exists anywhere** under `sdk/lib/rust/` (grepped both
names, zero hits outside the vendored C headers themselves). msvcrt's own per-thread block
(`thread_data_t`, `TlsGetValue(msvcrt_tls_index)`, `msvcrt-crt-rust/src/msvcrt/thread.rs`)
is msvcrt.dll's OWN unrelated per-thread structure for a different DLL and must not be
reused as a stand-in for `__acrt_ptd` — this is exactly why the task instructions say to
leave the thread-local half a documented stub rather than invent a mechanism, and that's
what the staged file does.

## 7. Test coverage

`modules/rostests/winetests/ucrtbase/misc.c`:
- `test_invalid_parameter_handler` (lines 384-425) — see §2's transcript. Exercises both
  slots' defaults, both setters' return values, precedence (thread-local over global), and
  the transition when the thread-local handler is cleared. This is the primary oracle for
  the 4 accessor/setter functions.
- Further indirect exercises of the global handler firing via real `_s`-function contract
  violations at lines ~478, ~514, ~1063, ~1557-1587 (e.g. after `_set_invalid_parameter_handler`
  is (re)installed at line 426, later tests rely on it staying installed and firing on
  `_ltoa_s`/similar invalid calls).

`modules/rostests/winetests/msvcrt/misc.c` (the msvcrt-side, NOT ucrtbase-side test) only
exercises bare `_invalid_parameter` itself (`test__invalid_parameter`, line 410), consistent
with `_MSVCR_VER=0` not compiling the other 6 names into msvcrt.dll at all — that test
resolves `_invalid_parameter` via `GetProcAddress` and skips if unavailable, so it says
nothing about the 7 exports this task covers.

No dedicated test found for `_invoke_watson`, `_invalid_parameter_noinfo`, or
`_invalid_parameter_noinfo_noreturn` directly (all three either terminate the process or
require a debugger/exception-filter harness winetest doesn't set up) — consistent with them
being inherently hard to unit-test black-box. I did not find a difftest/host-harness for
this family either (unlike, e.g., `errno.c`'s host-reachable harness noted in
rust-port-status.md); confirm before assuming one exists.

## What I could not determine (explicit list)

1. Whether `ucrt-rust` is permitted/intended to take a dependency edge on
   `crt-startup-rust` for `__security_cookie`, or whether the cookie should instead be
   re-declared as an `extern` and resolved purely at link time. Both are viable; I did not
   find a rule in `Cargo.toml`/`tools/ucrt-takeover.yaml` deciding it either way. Flagged as
   `NEEDS WIRING` rather than guessed.
2. Whether this tree already has, or should get, a `__fastfail` intrinsic. Not found under
   `sdk/lib/rust/`. Left as a separate `NEEDS WIRING` note at `_invoke_watson` rather than
   bundled into this family's implementation.
3. What TLS/per-thread-data primitive `ucrt-rust` should eventually use for
   `__acrt_ptd`-equivalent storage (a fresh `TlsAlloc`-based slot, a port of the real
   `__acrt_ptd` struct, or something else). Left as the documented stub per the task's
   explicit instruction not to invent one.
4. Whether a debugger-attached fast path (`_CRT_DEBUGGER_HOOK`) matters for ReactOS's
   `_invoke_watson`. Not addressed either way in the staged file.
5. Init ordering for the global handler's "encoded NULL" value once the real
   `__security_cookie` is wired in (see the `global_iph_encoded_null`/one-time-init note in
   the staged `.rs` file) — I identified the hazard precisely but did not solve it, since
   solving it requires knowing ucrtbase's real startup call sequence, which is outside what
   I verified here.
