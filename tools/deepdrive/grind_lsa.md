# Grind: fix lsalib-rust anonymous-union field paths

You are fixing Rust compile errors in `sdk/lib/rust/lsalib-rust/src/lsa.rs` ONLY.
Do NOT touch any other file. Do NOT change extern "C"/"system" ABI on any
declaration. Do NOT change logic, constants, or function signatures. Your ONLY
job is to correct anonymous-union/struct FIELD ACCESS PATHS to match the bindgen
layout of `LSA_API_MSG` (from the lsass-sys crate), then make it compile.

## The bindgen layout (this is ground truth — from lsass-sys/src/bindgen_*.rs)

```
struct _LSA_API_MSG {
    h: PORT_MESSAGE,
    __bindgen_anon_1: {          // union
        ConnectInfo: LSA_CONNECTION_INFO,
        __bindgen_anon_1: {      // struct
            ApiNumber: LSA_API_NUMBER,
            Status: NTSTATUS,
            __bindgen_anon_1: {  // union of the per-request message structs
                LogonUser: LSA_LOGON_USER_MSG,
                CallAuthenticationPackage: LSA_CALL_AUTHENTICATION_PACKAGE_MSG,
                DeregisterLogonProcess: LSA_DEREGISTER_LOGON_PROCESS_MSG,
                LookupAuthenticationPackage: LSA_LOOKUP_AUTHENTICATION_PACKAGE_MSG,
                ...
            }
        }
    }
}
```
Each per-request message struct (LSA_LOGON_USER_MSG etc.) is:
```
struct _LSA_LOGON_USER_MSG { __bindgen_anon_1: { Request: {...}, Reply: {...} } }
```
And `LSA_CONNECTION_INFO` has DIRECT fields (Status, OperationalMode, Length,
LogonProcessNameBuffer, CreateContext, TrustedCaller) — NO anon wrapper.

## The correct access paths (let `m` be the LSA_API_MSG var, e.g. ApiMessage)

- `m.ApiNumber`                  -> `m.__bindgen_anon_1.__bindgen_anon_1.ApiNumber`
- `m`'s API Status               -> `m.__bindgen_anon_1.__bindgen_anon_1.Status`
- `m.LogonUser`                  -> `m.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUser`
- `m.CallAuthenticationPackage`  -> `m.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage`
- `m.LookupAuthenticationPackage`-> `m.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LookupAuthenticationPackage`
- `m.DeregisterLogonProcess`     -> `m.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.DeregisterLogonProcess`
- a message's `.Request`/`.Reply`-> insert `.__bindgen_anon_1` before it, e.g.
  `<...>.LogonUser.Request.OriginName`
   -> `<...>.LogonUser.__bindgen_anon_1.Request.OriginName`
- For a `LSA_CONNECTION_INFO` value (e.g. local var `ConnectInfo`):
  `ConnectInfo.__bindgen_anon_1.Status` -> `ConnectInfo.Status` (DROP the anon — it has direct fields)

The grinder used THREE inconsistent prefixes that ALL mean "the inner struct":
`.u.`, `.c2rust_unnamed.c2rust_unnamed.`, direct (no prefix), and the partial
`.__bindgen_anon_1.` (one level short). Normalize ALL of them to the correct
paths above.

## Verify
Run: `cd sdk/lib/rust && CARGO_TARGET_DIR=/tmp/lsa-grind cargo check --release --target i686-pc-windows-gnu -p lsalib-rust`
Iterate until it reports `Finished` with zero errors. Warnings are fine. Also there
may be a couple of `[u16; N] cannot be dereferenced` errors on wide-string
transmute literals — for those, the expression is like `&transmute::<[u8;N],[u16;M]>(*b"...")`;
if a `*` deref is wrong, remove it so the value is a `[u16;M]` array referenced by `&`.
Do not introduce `unsafe` blocks that weren't there; the functions are already `unsafe`.
