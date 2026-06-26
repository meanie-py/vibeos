# advapi32.dll C→Rust Translation Audit

Status of the advapi32.dll (Win32 Advanced API: registry, services/SCM, security/
SID/ACL, crypto, eventlog, logon) C→Rust cohab translation. Lane: `kernel32-port`
(agent `kernel32`). Started 2026-06-26.

## Architecture

- **`sdk/lib/rust/advapi32-rust`** — the cohab staticlib (`libadvapi32_rust.a`).
  Each translated `.c` is commented out of `dll/win32/advapi32/CMakeLists.txt`
  (`# replaced by Rust (advapi32-rust)`); the crate's `#[no_mangle]` bodies take
  over. Linked FIRST with `--allow-multiple-definition` (lld first-match). Until a
  `.c` is commented out, the C object wins (Rust inert) — so an un-flipped file is
  a safe no-op.
- **`sdk/lib/rust/advapi32-sys`** — bindgen substrate of advapi32-specific Win32-
  SDK / RPC-client / winsafer / aclapi / EFS / services / cryptlib types absent
  from `ntdll-sys` (NT-internal only). Regenerate: `bash sdk/lib/rust/advapi32-sys/
  generate_bindings.sh`. Broad allowlist; CURATED (explicit `pub use`) into the
  prelude so base types (HANDLE/ULONG/SID/HKEY…) don't collide with `ntdll_sys::nt`.

## Substrate conventions (see `advapi32-rust/src/prelude.rs` + `externs.rs`)

- Every translated file: `use crate::prelude::*;` ONLY (never `ntdll_sys::nt::*`
  directly — that re-introduces bindgen's u32 `TRUE`/`FALSE` and causes ambiguity).
- Prelude shadows `TRUE`/`FALSE` as `BOOL`(i32) and `RPC_S_OK` as `RPC_STATUS`(i32)
  over bindgen's u32 versions; provides NT_SUCCESS, Rtl*/Hkcr* FORCEINLINEs, etc.
- **`externs.rs`** declares functions NOT defined in the crate (sibling-DLL imports
  + advapi32-own exports in not-yet-flipped `.c`). **ABI rule (i386-critical):**
  `WINAPI`/`__stdcall`/`NTAPI`/`__RPC_USER`/RPC-stubs → `extern "system"`; plain-C
  + CRT → `extern "C"`. Wrong ABI links clean on amd64/arm64 but yields a wrong
  `_Name@N` decoration → undefined symbol (or silent divergence) on i386. Verify
  each against the C header/`.spec`/generated `*_c.h`. Glob-shadowing means a later
  in-crate definition silently overrides the decl (minimal churn on flip).

## Translation mechanism

`python -m rustconv translate <file> -m opus` (per-function, faithful, auto-
assembles to the crate) → `rustconv pp` → swap imports to prelude → wire `pub mod`
in `lib.rs` → `cargo build` ×3 → fix substrate/casts → **i386 cohab-flip + `ninja
advapi32` link-gate** (catches ABI/decoration bugs crate-build can't see).

## Wave status

| Wave | Files | LOC | State |
|------|-------|-----|-------|
| 1 | token, misc/{unicode,msi,dllmain,evntrace,hwprofiles,efs,shutdown}, service/winreg, sec/{trustee,safer}, wine/{crypt_des,crypt_lmhash} (13) | ~3.6K | ✅ translated, integrated, flipped, i386-linked, ×3 compile |
| 2 | sec/sec, sec/audit, misc/sysfunc, reg/hkcr, service/sctrl (5) | ~4.5K | ✅ translated, integrated, flipped, i386-linked, ×3 compile |
| 3 | misc/logon, service/eventlog, sec/misc, wine/cred, sec/lsa, wine/crypt (6) | ~11.7K | 🔄 in progress |
| 4 | service/scm, wine/security, reg/reg (3) | ~12.7K | ⏳ pending |

### Deferred

- **sec/ac.c → Wave 3** (with sec/misc.c): its `Acc*` access-control calls route
  through the `NtMartaStatic` dynamic-ntmarta function-pointer table +
  `CheckNtMartaPresent`, both defined in `sec/misc.c`. The translated `ac.rs` (on
  disk, `pub mod sec_ac` commented in lib.rs) flattened those into direct imports
  with no symbol; the faithful `NtMartaStatic.X.unwrap()(…)` call-through is done
  when sec/misc.c flips. `NTMARTA` is in the advapi32-sys bindgen. ac.c stays
  active C until then (no regression hidden — ac.rs was never link-live).

## RPC interfaces

advapi32 uses RPC **client** stubs (svcctl/winreg/lsa/seclogon/eventlogrpc) which
stay generated-C (`*_c.c`, PCH_SKIP_SOURCE); the Rust bodies call them by decorated
name (`extern "system"`). No server stub crate needed (unlike lsasrv/samsrv).

## Final gate (Wave 4 done)

Full cohab flip → ×3 cold link gate → i386 direct-qemu self-boot → flag control-
plane (`meaniepy`) for cold ×3 gate + landing.
