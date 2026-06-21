# kernel32_shared + kernel32_vista → Rust — translation plan

**Status:** ✅ DONE (2026-06-20). Phase 1 `kernel32-shared-rust` (`c28b8fd0f81`) +
Phase 2 `kernel32-vista-rust` (`e74b29f092c` WIP → `48d9c3b2e7b` complete), all
hand-translated. cargo green ×3; kernel32.dll + kernel32_vista.dll both link clean
on i386 (no undef/dup; ud2 baseline); decorations verified. Boot ×3 gate-owed.
Only genuinely-C remainder in kernel32.dll's surface now: `kernel32_vista/DllMain.c`
(the vista module entrypoint — can't live in a crate linked into both DLLs),
generated `kernel32_stubs.c`/`.def`, and the `fiber.S`/`thread.S` asm.

The kernel32 *client* API surface was already 100% Rust at link (`94d3c4511f7`);
this plan covered the genuine structural remainder — the two cmake static libs
shared between `kernel32.dll` and `kernel32_vista.dll`.

## The structural crux

Unlike the per-file client/file/wine/winnls cohab flips, these two are **static
libraries linked into TWO different DLLs**:

```
kernel32_shared      (shared_utils.c + fileutils.c, 197 LOC)
  ├─ linked into kernel32.dll        (CMakeLists.txt:155)
  └─ linked into kernel32_vista.dll  (kernel32_vista/CMakeLists: target_link_libraries)

kernel32_vista_static (15 .c, 2031 LOC)
  ├─ linked into kernel32.dll        (CMakeLists.txt:155)
  └─ linked into kernel32_vista.dll  (the MODULE: DllMain.c + the two static libs)
```

So a Rust translation must provide its symbols to **both** link lines. Commenting
a `.c` out of one consumer isn't enough — and folding the bodies into the big
`kernel32-rust` crate is wrong (linking the entire client-API crate into the lean
`kernel32_vista.dll` would bloat it and disturb the already-watched kernel32
SizeOfImage / base-address budget).

**Decision: two new dedicated Rust staticlib crates**, each built once and linked
into both DLLs:

| New crate | Replaces | Output |
|---|---|---|
| `sdk/lib/rust/kernel32-shared-rust` | `kernel32_shared` (shared_utils.c, fileutils.c) | `libkernel32_shared_rust.a` |
| `sdk/lib/rust/kernel32-vista-rust` | `kernel32_vista_static` (the 15 .c) | `libkernel32_vista_rust.a` |

Both are STANDARD user-mode targets (`i686`/`x86_64-pc-windows-gnu`,
`aarch64-pc-windows-gnullvm`); the workspace `.cargo/config.toml` already forces
i386 `-sse,-sse2` for every crate on that target, so the no-SSE user-mode rule is
inherited automatically (same as kernel32-rust).

### Cross-crate symbol routing (the rust-in-kernel.md "imports, not deps" rule)

- `kernel32-rust` already references the shared/vista symbols as **externs**
  (`BaseSetLastNTError` ×60 files, `FilenameA2W` ×3, `FilenameW2A_FitOrFail` ×1,
  `LocaleNameToLCID`, `GetUserDefaultLocaleName`). After the flip those resolve to
  the new rust libs at the cmake link. No cargo dep between the crates.
- `kernel32-vista-rust` calls shared symbols (`BaseSetLastNTError`, `FilenameA2W`,
  `FilenameU2A_FitOrFail`) and kernel32 exports (`GetVolumeNameForVolumeMountPointW`,
  `GetFullPathNameW`, …). It DECLARES them `extern` — resolved at the DLL link:
  shared via `libkernel32_shared_rust.a` in the same line; kernel32 exports via the
  existing `add_importlibs(kernel32_vista kernel32 …)` runtime import. **No cargo
  dependency on kernel32-rust** — context symbols live in the consumer, the shared
  crate only imports (rust-in-kernel.md cohab rule).

### The DllMain collision gotcha (vista DllMain.c stays C)

`kernel32_vista.dll` has its own 14-LOC `DllMain.c` (just `DisableThreadLibraryCalls`).
It can't go into `kernel32-vista-rust`, because that crate is **also linked into
kernel32.dll** — a `#[no_mangle] DllMain` there would collide with kernel32-rust's
own `DllMain`. Options: (a) leave `DllMain.c` as C in the vista MODULE only (it is
NOT in the static lib, so it never reaches kernel32.dll), or (b) a third micro-crate
linked only into vista.dll. **Recommend (a)** — it's irreducible entrypoint glue for
*this* module, exactly analogous to why a shared crate can't own an entrypoint.

## Build plumbing

Mirror the existing `rust_kernel32` custom_command in `dll/win32/kernel32/CMakeLists.txt`:
1. `add_custom_command` per crate → `cargo build --release --target <arch> -p <crate>`
   → copy the `.a`; `add_custom_target(rust_kernel32_shared …)` / `…_vista`.
   (Use `GLOB_RECURSE` on the crate's `src/**.rs` for the DEPENDS, per the lesson
   already baked into the kernel32 block.)
2. `target_link_libraries(kernel32 … libkernel32_shared_rust.a libkernel32_vista_rust.a)`
   — **drop** `kernel32_shared` + `kernel32_vista_static` from the line.
3. `target_link_libraries(kernel32_vista … libkernel32_shared_rust.a libkernel32_vista_rust.a)`
   — same swap.
4. Delete the now-empty `add_library(kernel32_shared …)` and
   `add_library(kernel32_vista_static …)` (an `add_library` with all sources
   commented fails to configure — drop the target, don't leave a placeholder). The
   `.c` files stay on disk (never-delete rule).
5. `add_dependencies(kernel32 … rust_kernel32_shared rust_kernel32_vista)` and the
   same for `kernel32_vista`.

## Phasing

### Phase 1 — `kernel32-shared-rust` (6 fns, ~197 LOC) — HIGH VALUE, LOW COST
`BaseSetLastNTError` (4-line: `RtlNtStatusToDosError` + `SetLastError`) and the
`Filename{A2W,U2A_FitOrFail,W2A_FitOrFail,A2W_N,W2A_N}` ANSI/Unicode converters.
- Small enough to **hand-translate faithfully — NO rustconv $ needed.**
- Validates the "shared rlib into two DLLs" build mechanism on the smallest surface
  → de-risks Phase 2.
- `BaseSetLastNTError` is on 60 error paths → boot-critical, heavily smoke-exercised.
- Verify: cargo ×3, kernel32.dll + kernel32_vista.dll link clean (no undef/dup),
  ud2 baseline, ×3 host-quiet gate.

### Phase 2 — `kernel32-vista-rust` (15 .c, ~2031 LOC) — COMPLETIONIST
**Hand-translated** (user directive 2026-06-20: skip rustconv for this ~2K LOC,
do it by hand — more faithful control over the tricky bits, $0). Sub-phase by
complexity (survey 2026-06-20):
- **2a LEAF (~331 LOC, mostly mechanical):** GetLocaleInfoEx, GetNamedPipeClientProcessId,
  GetTickCount64 (SharedUserData spin-read), GetUserDefaultLocaleName, InitOnce
  (3 RtlRunOnce* wrappers), IsValidLocaleName, LCIDToLocaleName, SetThreadDescription,
  + the ~9 `UNIMPLEMENTED` stubs in vista.c (faithful stub equivalents).
- **2b MEDIUM:** firmware.c (Nt env-var + ANSI/Unicode), GetFileInformationByHandleEx
  (16-case dispatch), EnumSystemLocalesEx (fn-ptr callback — straightforward in Rust),
  sync.c (13 SRW/condvar/critsec wrappers; `GetNtTimeout` ms→100ns helper; use
  ntdll_sys/bindgen `PRTL_SRWLOCK`/`PRTL_CONDITION_VARIABLE` layouts).
- **2c HEAVY (faithfulness-critical):**
  - `LocaleNameToLCID` — 37-entry hardcoded LCID remap table. **Python-extract the
    table → emit exact Rust** (user directive 2026-06-20: heavy tables go through a
    gen script, never hand-typed — version-rust / `gen_nls_data_tables.py` pattern;
    add `--check` byte-parity). Getting one entry wrong breaks locale detection. The
    surrounding bit-manipulation logic is still hand-translated.
  - `GetFinalPathNameByHandle` — NT-name→DOS-volume via MountMgr IOCTL, multi-format
    path assembly.
  - `vista.c CreateSymbolicLinkW/A` — REPARSE_DATA_BUFFER construction, path-type
    switch, IOCTL, goto-cleanup.
  - `vista.c QueryFullProcessImageNameW/A` — heap + NtQueryInformationProcess.
- Cohab flip whole-file-by-file as each `.rs` lands (comment the `.c`), like cdfs/hal;
  flip the last one only when vista-rust is symbol-complete for the static lib.
- Keep `DllMain.c` as C (collision gotcha above).

## Faithfulness watch-items (field-manual classes already known)
- LCID remap table verbatim (mis-typed constant → silent locale break).
- sync.c struct layouts must match Rtl binary layout (`#[repr(C)]` via bindgen types).
- i386 stdcall: every `.spec`-exported WINAPI fn `extern "system"` (→ `_Name@N`);
  CRT helpers (`lstrlenW`, etc.) stay `extern "C"`.
- DUMMY_PSEH: any `_SEH2_*` body that mutates state must be dead code.
- element-vs-byte on `RtlCopyMemory` count vs `ptr::add` element count.
- cast-before-offset (`(p as *T).offset(1)`) — keep offset-then-cast.

## Smoke-validatability caveat
`kernel32_vista_static` is in `kernel32.dll`, so its boot-path-called functions
(notably `LocaleNameToLCID`, called by kernel32-rust nls_subst/lcformat on the
locale-init path) ARE smoke-exercised. But many `kernel32_vista.dll`-only exports
(GetFinalPathNameByHandle, the locale-Ex/MUI/firmware APIs) are NOT hit by
boot-to-desktop — smoke proves link + load + the boot-critical subset, not every
function. Full validation of the rare exports would need targeted tests; note any
gap in the commit rather than implying full coverage.

## Effort
Both phases hand-translated ($0). Phase 1: ~1 focused hour (6 fns + plumbing +
gate). Phase 2: a focused session — hand-translate the 15 files (LEAF→MEDIUM→HEAVY)
+ cohab + ×3 gate. (The survey's "weeks" were human-from-scratch estimates; faithful
1:1 C→Rust transcription of mostly-thin-wrappers is far quicker.)

## Post-translation faithfulness audit (2026-06-20, while gate-queued)
Full line-by-line re-read of all 8 vista body files + the generated table vs the C
originals. **Result: faithful, no reachable functional bugs.** Detail:
- **locale.rs** — `LocaleNameToLCID` 3-way SubLangId logic (==NEUTRAL→default /
  ≥0x18→table-or-0 / else→passthrough) exact; `None=>return 0` correctly mirrors the
  C NDEBUG `ASSERT(FALSE);return 0`. `LCIDToLocaleName`/`GetUserDefaultLocaleName`
  (`Length/sizeof` ±1), `GetLocaleInfoEx` (`wcslen==2` neutral special-case) match.
- **localenametolcid_table.rs** — `gen_kernel32_vista_lcid_table.py --check` byte-parity
  (31 entries) green.
- **finalpath.rs** — DOS/GUID/NT/NONE length math, the `\Device\` split, `offset_from`
  element-vs-byte, `VolumeBuffer[53]`/`sizeof` all correct. ⚠️ Reproduces a *pre-existing
  C bug* (misplaced parens in `QueryDosVolumeNameForNtDeviceName`'s 3-append condition)
  as the clean short-circuit form — identical in every reachable case (106-byte volume
  buffer always fits; diverges only on an impossible buffer-too-small append). Documented
  in-code; left as-is (unreachable, zero-impact).
- **vista.rs** — REPARSE offsets verified (`offset_of __bindgen_anon_1` = FIELD_OFFSET
  GenericReparseBuffer = 8; PathBuffer = 20), `pBufTail` accumulation, `DeleteFileA`
  bindgen rename, goto→`'cleanup`-loop reaches `finish_symlink` with correct free-state
  on every path. ⚠️ Reproduces a 2nd *pre-existing C bug* faithfully: `bAllocatedTarget`
  set TRUE only on the RtlDosPathNameToNtPathName_U *failure* path → success-path
  TargetFileName.Buffer leak. Documented in-code.
- **fileinfo.rs** — 16-case dispatch faithful; the or-patterns compiling *proves* every
  arm is a constant pattern (a binding in a multi-alt or-pattern is a compile error) →
  no silent catch-all-binding bug (the class that bit the `FileNameInfo` const).
- **sync.rs** — `GetNtTimeout` = `(Timeout as u64).wrapping_mul((-10000i64) as u64)`
  reproduces `(ULONGLONG)Timeout*-10000`; SRW/condvar pass-throughs + `STATUS_TIMEOUT`
  handling match.
- **misc.rs** — `GetTickCount64` High1/High2 spin-read + `(low*mult>>24)+(high*mult<<8)`,
  `CTL_CODE→0x110030`, `sizeof("ClientProcessId")=16`, `HRESULT_FROM_NT=x|0x10000000`.
- **firmware.rs** — ANSI→W cleanup ordering exact (frees VariableName before bailing on
  the 2nd conversion fail; Namespace-then-VariableName on success).
- **enumlocales.rs** — `RT_STRING=6`, `MAKEINTRESOURCE((LOCALE_SNAME>>4)+1)=6`,
  neutral-data flags, and the `extern "system"`↔`ENUMRESLANGPROCW` transmute (ABI-identical
  per-arch) all correct.

**Gate-risk #1 (amd64/arm64 `ntdll_vista` link) de-risked across all 3 arches:** every
referenced ntdll symbol is in `dll/ntdll/def/ntdll.spec`; the 0x600-gated set
(`RtlLcidToLocaleName`/`RtlRunOnce*`/SRW/condvar) resolves via the *unconditional*
`add_delay_importlibs(kernel32 ntdll_vista)` (CMakeLists.txt:215) — same importlib the C
kernel32_vista.dll already used for these exact symbols; ungated ones via base `ntdll`;
the inline-only trio (`RtlInitEmptyUnicodeString`/`RtlCopyMemory`/`RtlGetProcessHeap`) are
local fns needing no symbol. Per-arch decoration handled by `extern "system"`+spec2def.
The only amd64/arm64 unknown is whether the coarse Rust cgu drags these symbols into
kernel32.dll at all — and resolution is covered whether it does or not. (Output-pointer
writes empirically safe: clean ud2 baseline ⇒ no vista output-write folded under
captures(none).)
