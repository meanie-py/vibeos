You extract the exact behavioral contract of a single ReactOS libcntpr
C-runtime function FROM THE PROVIDED SOURCE ONLY, into one row of the
`cnt-rust` manifest. You are reading comprehension on the given code —
NOT recalling libc from memory. If the source contradicts textbook
libc, the source wins; that deviation is the point.

## Context

`cnt-rust` reimplements libcntpr in two layers, single source of truth:
- `core_rs/`  — idiomatic, slice-based, no_std, no-unsafe. THE impl.
- `cabi/`     — generated `#[no_mangle] extern "C"` shim: unsafe
                slice-reconstruct from the C ptr ABI, call the core,
                collapse the core result back into the EXACT legacy C
                contract (including any misfeature).

Faithful to ReactOS, NOT textbook libc. Real examples already pinned:
`strcmp` returns the *signed-char* difference (`_TCHAR` is signed
`char`), `strlen(NULL)==0`, `atoi` wraps i64 then truncates to int
with NO overflow error, `memcpy` is overlap-safe (it's a memmove).
Your job is to surface this function's equivalents, not "fix" them.

## Output — STRICT

Your reply MUST begin with the fenced ```toml block as the very first
characters (no preamble, no prose before it). Then `## rationale`
(prose only). Do NOT write ANY Rust code, ANY ```rust block, or any
implementation — only the manifest row + prose rationale. If a new
core is needed, describe its signature in one prose sentence in the
rationale; never code it.

## Output — emit EXACTLY one fenced ```toml block, then `## rationale`

```toml
[symbols.NAME]
c_signature  = "..."          # exact prototype from the source
family       = "string|wstring|mem|radix|ctype|printf|sort"
slice_pattern = "..."         # how the shim rebuilds core slices:
  # nul_in      : `*const char` NUL-terminated arg(s) → content slice
  # ptr_len     : `*X, size_t` pair → slice of that len
  # dst_fixed_n : `*mut char dst, src(nul_in), size_t n`
  # mem_raw     : `*mut void,*const void,size_t` raw, maybe-overlap
  # set_raw     : `*mut void,int,size_t`
core_fn      = "..."          # existing core to call, or "NEW:<name>"
return_collapse = "..."       # core return → exact C return:
  # usize | i32_passthrough | ptr_or_null_from_opt | ret_dst
  # | i64_trunc_i32 | i64_passthrough | NEW:<describe>
inout_effects = "..."         # which params written + EXACTLY what
preserved_misfeature = "..."  # the load-bearing deviation from
  # textbook libc, quoting the source. "none" only if truly textbook.
fuzz_domain  = ["...", "..."] # input equivalence-classes a
  # differential test MUST cover to pin every branch incl. the
  # misfeature and any overflow/empty/not-found boundary
```

Existing cores you may target (else propose `NEW:<name>` with a
one-line signature+behavior in the rationale — DO NOT implement it):
`string::len_of`, `string::cmp` (signed-char), `string::find_sub`,
`string::ncpy`, `mem::set`, `mem::cmp` (unsigned), `mem::chr`,
`radix::parse_i64_wrapping` (→ `(i64, overflowed)`), plus shim
intrinsics `raw_move` (overlap-safe), `slice_len`.

## The 5 questions your fields must answer (cite source lines)

1. Exact return value as a function of inputs — incl. overflow, empty,
   not-found, n==0?
2. Which parameters are written, and precisely what (terminate? pad?
   `*endptr`? nothing)?
3. NULL / zero-length / not-found behavior — deref-crash, sentinel,
   or no-op? (Quote the guard if present.)
4. Any deliberate deviation from textbook libc — quote the lines.
5. Minimal input equivalence-classes a differential test must include
   to exercise every branch (esp. #4 and any boundary).

Be precise. Signed vs unsigned char comparison, whether a search
includes the NUL terminator, exact overflow behavior, and n==0 edge
cases are the things that matter most and are most often gotten wrong.
