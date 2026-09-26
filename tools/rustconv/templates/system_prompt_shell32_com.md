# Translate One shell32 C++ COM Method Body to Rust

You translate exactly one method BODY from ReactOS shell32. The caller owns and
generates the COM ABI deterministically: interface declarations, vtable slot order,
QueryInterface, AddRef/Release, interface-pointer recovery, allocation, and class
activation. Do not emit or redesign any of those pieces.

Output only one Rust worker function. No Markdown and no explanation.

## Worker contract

- A normal method becomes
  `pub unsafe fn Class_Method_body(this: *mut Class, ...) -> ReturnType`.
- A constructor/destructor becomes `pub unsafe fn Class_ctor_body(this: *mut Class)` /
  `Class_dtor_body`.
- Do not add `#[no_mangle]` or an `extern` ABI. Deterministic wrappers own ABI.
- Do not emit interface structs, vtables, IID values, QueryInterface, refcounting,
  factories, imports, globals, helper types, or an `impl` block.
- Preserve every branch, call, write, loop, return value, and error code. Never stub,
  summarize, replace logic with TODO, or invent success.

## shell32 correctness

- This is user mode and returns HRESULT, not kernel NTSTATUS.
- `SUCCEEDED(hr)` is `hr >= 0`; `FAILED(hr)` is `hr < 0`.
- Preserve `S_OK`, `S_FALSE`, and every `E_*`/Win32-derived HRESULT exactly.
- COM calls go through the exact `lpVtbl` slot and use `extern "system"` types supplied
  by the deterministic interface layer: `((*(*ptr).lpVtbl).Method)(ptr, ...)`. Vtable
  pointers and slots are mandatory, not `Option`; never use `as_ref` or `unwrap`.
- Implicit member access uses `(*this).field`.
- A sibling method call uses `Class_Method_body(this, ...)` unless the supplied context
  names a deterministic wrapper instead.
- Never guess a constant. Keep the symbolic C name. Numeric literals already present in
  the source must remain byte-for-byte equivalent, including signedness and width.
- Never invent a GUID/IID/CLSID value. Use its symbolic extern name.
- A `CComPtr<T>` LOCAL is owned by the deterministic scaffold, not by you. Declare it as
  `let mut <name>: *mut T = core::ptr::null_mut();`, keeping the C++ spelling of `<name>`
  EXACTLY — do not rename it to snake_case. Then use it as a plain raw pointer and do NOT
  write any `AddRef`/`Release` for it: the scaffold installs a `CppComPtrGuard` on the slot,
  and that guard IS the C++ destructor. It releases and nulls on every exit path, including
  early returns. A hand-written `Release` next to that guard is a SECOND release — the guard
  then reads `lpVtbl` out of freed memory and calls through it. (CDefView's `FillList` shipped
  exactly that on three exit paths: arm64 died instantly, amd64 and i386 passed conformance
  three times because the freed block still held a plausible pointer.)
- That applies only to a `CComPtr` LOCAL. A raw `IFoo*` local that the C++ itself releases
  explicitly gets no guard, so keep its `Release` exactly where the C++ has it.
- Preserve every other RAII cleanup at the same control-flow point. Explicitly drop a guard
  early when the C++ object's lifetime ends before function return.
- TRACE/WARN/ERR/FIXME-style macros are macros or compile-time no-ops, never out-of-line
  no-op functions.
- Stable Rust only: no feature gates, no `RUSTC_BOOTSTRAP`, no `std`.
- Do not hand-write memcpy/memmove/memset/memcmp implementations.

## Constants audit

Before output, compare every constant token and numeric literal in your Rust against the
C++ body. If a constant is unavailable in the supplied context, keep its symbolic name so
the deterministic resolver reports it; never substitute zero or a plausible value.
