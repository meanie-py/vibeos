# secur32.dll C→Rust Translation Audit

Status of the secur32.dll (Win32 SSPI client: security-provider dispatch, A/W
thunks, the secure-package wrapper, the LSA-LPC channel to lsasrv, and the built-in
wine SSPs — NTLM, Kerberos stub, Negotiate) C→Rust cohab translation. Lane:
`kernel32-port` (agent `kernel32`). Started + completed 2026-06-28.

## Architecture

- **`sdk/lib/rust/secur32-rust`** — the cohab staticlib (`libsecur32_rust.a`).
  Each translated `.c` is commented out of `dll/win32/secur32/CMakeLists.txt`
  (`# replaced by Rust (secur32-rust)`); the crate's `#[no_mangle]` bodies take
  over. Linked FIRST with `--allow-multiple-definition` (lld first-match).
- **`sdk/lib/rust/secur32-sys`** — bindgen substrate of SSPI / security-provider /
  LSA-LPC / NTLM-MSV / wincred types absent from `ntdll-sys` (NT-internal only).
  Regenerate: `bash sdk/lib/rust/secur32-sys/generate_bindings.sh`. ~5287 lines/arch.
  CURATED (explicit `pub use`) into the prelude so base types (HANDLE/ULONG/LUID…)
  don't collide with `ntdll_sys::nt`.

secur32 talks to lsass over **NDK LPC** (`lsass/lsass.h` `LSA_API_MSG` port
messages) — there are NO RPC client stubs (unlike advapi32). The LSA-LPC opcodes
(`_LSA_API_NUMBER`) and `LSA_API_MSG` layout were verified against the C header.

## Substrate conventions (see `secur32-rust/src/{prelude,externs,types}.rs`)

- Every translated file: `use crate::prelude::*;` ONLY.
- Prelude re-exports `ntdll_sys::nt::*` + curated `secur32_sys` SSPI types + the
  SEC_E_*/SEC_I_* status codes (bindgen can't const-eval their HRESULT macros) +
  SECPKG_* flags + TRUE/FALSE(BOOL) + helpers (NT_SUCCESS, Rtl{Zero,Copy,Fill}Memory)
  + `crate::externs::*` + `crate::types::*`.
- **`types.rs`** — hand-translated secur32-private structs from the module's own
  headers: `wine/ntlm.h` (HelperMode/arc4_info/NegoHelper/NtlmCredentials/NTLMSSP_*),
  `wine/hmac_md5.h` (MD5_CTX/HMAC_MD5_CTX), `md4.h` (MD4_CTX), `secur32_priv.h`
  (SecureProvider/SecurePackage), plus the wine `list_init`/`list_add_tail`
  intrusive-list ops (static-inline in C → provided as symbols here).
- **`externs.rs`** — kernel32/advapi32/netapi32/wincred Win32 imports + CRT + cryptlib
  MD4/MD5. **ABI rule (i386-critical):** WINAPI/__stdcall/NTAPI/SEC_ENTRY →
  `extern "system"` (`_Name@N`); plain-C/CRT → `extern "C"` (`_Name`).

## Translation mechanism

`python -m rustconv translate <file> -m opus` (per-function, faithful, auto-
assembles to the crate) → swap imports to prelude → wire `pub mod` in `lib.rs` →
`cargo build` ×3 → fix substrate/casts (integration grind) → **cohab-flip + `ninja
secur32` link-gate ×3** (catches the i386 ABI/decoration bugs crate-build can't see).

## Files (13 .c, ~7.5K LOC) — all ✅ flipped

| Group | Files | LOC |
|------|-------|-----|
| leaf | dllmain, stubs, wine/{hmac_md5, base64_codec, util} | ~0.6K |
| mid | lsalpc, wine/{kerberos, dispatcher, negotiate} | ~1.7K |
| heavy | thunks, wrapper, sspi | ~3.2K |
| ntlm | wine/ntlm (the NTLM SSP) | ~2.0K |

**🏆 secur32.dll body is 100% Rust — all 13 .c flipped.** Crate builds clean ×3;
secur32.dll links 0-undefined on i386 (569K) / amd64 (603K) / arm64 (686K).
Tip `85782805907` on kernel32-port. Translation spend ~$2 (opus).

## Notable bugs fixed during integration (reusable)

- **SEC_ENTRY ABI (systemic):** the SSP/thunk functions (`kerberos_*`/`nego_*`/
  `ntlm_*`/`thunk_*`) were translated with the wrong ABI; they're SEC_ENTRY (WINAPI)
  → `extern "system"`. They're stored into bindgen `SecurityFunctionTable` fn-ptr
  fields spelled `extern "stdcall"`(i386)/`extern "C"`(amd64/arm64); a `sec_entry!`
  transmute macro reconciles the i386 ABI *token* (convention is identical).
- **Translator-dropped function:** `fork_helper` (dispatcher.c CreateProcessA path)
  was silently dropped — recovered faithfully (a translation gap to watch for; grep
  the C for functions absent from the .rs after a batch run).
- **Translator-dropped statics:** `securityFunctionTableA/W` (sspi), `negoTableA/W`
  (negotiate) — recovered from the C; stored `static mut` (bindgen table type embeds
  non-Sync raw-ptr fields).
- **const-as-binding (latent, would mis-execute):** consts missing from scope
  (`EXTENDED_NAME_FORMAT`, `SECPKG_CRED_*`) made `match` arms silently become
  catch-all bindings, collapsing the match to its first arm. Added the consts.
- **LSA_API_MSG anon-union (lsalpc):** the translation used flat C field names; the
  bindgen struct nests `__bindgen_anon_1.__bindgen_anon_1...` — rewrote the access
  paths (kept the size-verified bindgen layout).
- **i386 link-gate-only decoration bugs:** stubs.rs exports + sspi.rs Reg*/lstrlenW
  were `extern "C"` but are WINAPI → `extern "system"`; wine `list_*` needed impls.

## Final gate

Full cohab flip → ×3 cold link gate (done, green) → i386 direct-qemu self-boot →
flag control-plane (`meaniepy`) for cold ×3 gate + landing. secur32 is a separate
DLL (advapi32 delay-imports it; lsass/winlogon use it for SSPI), boot-touched.
