# Generated contract difftests

`rustconv difftest-contract` builds the genuine C source and an existing Rust module,
generates typed wrappers and inputs, and compares isolated host executions. It supports
stateless calls and stateful operation sequences. No model calls.
This complements `difftest-gen`, whose existing argument-kind inference is CRT-specific.

From the repository root, with the project's Python environment:

```sh
PYTHONPATH=tools/rustconv python -m rustconv difftest-contract \
  tools/rustconv/data/difftest/swab.json \
  --out "$HOME/vibeos-rust-staging/swab-grade-001"
```

The output directory must be new. `report.json` is authoritative; the command exits nonzero
for mismatches or invalid grading runs. Generated wrappers, complete inputs, compiler
commands, stdout/stderr, per-command timing and dependency hashes remain alongside it.
A mismatch also saves `counterexample.json` with the input and both observations. This is
the first encountered counterexample, **not a minimized one**.

## Writing a contract

Use `tolower.json` and `swab.json` for scalars/disjoint buffers, `memmove.json`
and `memchr.json` for shared arenas, pointer results and protected pages, `atoi64.json`
for a grammar-bounded NUL-terminated parser input, `strtoi64.json` for a nullable pointer slot
and normalized end pointer, `wtoi64.json` for UTF-16 inputs, `strlwr.json` for mutable ASCII
strings, `wcslwr.json` for mutable UTF-16 strings, and `strdup.json` for libc-owned string
results. `md4.json`/`md5.json`/`sha1.json` grade hash contexts from arbitrary sampled states
(records), `crc32.json` selects a lookup table with its function, and `encode.json` grades a
`UNICODE_STRING` whose `Buffer` is a `Length`-sized byte pointee, with a scripted host
`NtQuerySystemTime`. Paths are relative to the contract.
The required `validity` field records the reviewed domain and environmental assumptions.
The tool does not prove them. `scope` is `stateless-host` for independent calls or
`stateful-host` for ordered operation sequences.

Each function declares its C identifier, Rust path, exact integer/void result and argument
list. Supported integers are `u8/i8/u16/i16/u32/i32/u64/i64/usize/isize`. Every scalar has
explicit edge `values`; optional `random_range: [min, max]` supplies uniformly sampled
integers in that inclusive range. Without a range, random cases choose from `values`.
`rust_abi` defaults to `C` and also supports `system` and `Rust`; `rust_unsafe` defaults true.
The compiler checks a typed function-pointer assignment on each side; it does not cast an
incompatible function into the requested signature. A function may declare a supported scalar
`c_return_type` when a host fixture's typedef spelling differs from the Rust-side scalar type;
the C compiler still checks the original function against that exact return spelling. Pointer
arguments and results may use a restricted `rust_type` override for C `char` signedness, and
`rust_imports` names exact imports needed by selected bodies.

Pointers are `bytes`/`const_bytes` (`u8 *`), `void_bytes`/`const_void_bytes` (`void *`),
read-only `const_cstr` (`const char *`), writable `mut_cstr` (`char *`), read-only `const_wstr`
or writable `mut_wstr` (16-bit `uint16_t *`).
Byte-buffer `length` is either a fixed byte count or another
scalar argument's name. Buffers are initialized and non-null even at length zero; they are independently allocated
unless a `layouts` declaration places them in a shared arena. All bytes are compared, including unwritten destination tails. Bytes outside mutable argument views must remain unchanged. Const views may legitimately
change where they overlap a mutable view. Canary mode checks 32-byte guards on both ends.
An optional `c_type` spells a genuine `char *`/`const char *` prototype without pretending it
is an `unsigned char *`. This changes the checked C pointer spelling, not the byte payload.
A wider element type (`"c_type": "ULONG *"`, e.g. a hash result array) is accepted only with
an explicit `align` (1/2/4/8/16), which the protected-page placement then honours.
A byte buffer whose `length` names an unsigned scalar may list `examples` (hex payloads). Each
example becomes extra cases with that payload and the length argument set to its size, crossed
with the other scalars' edge values. Use them for structured input that seeded patterns never
form — `compress.json` feeds encoder-produced LZNT1 streams, without which its back-reference
mutant survives.

For structured input, a byte pointer may use a `record` instead of `length`. A record declares
the C layout type, its `sizeof` base size, and each scalar field's byte offset, fixed-width
integer type and edge values. `count: 1` fields cross their explicit values; fixed arrays repeat
their value domain according to the selected corpus pattern. A final `count_from` array follows an
earlier unsigned scalar field and may reserve a `min_count` for C flexible-array idioms whose
declared layout contains one element even when the logical count is zero. The generator packs
little-endian values, zeroes padding, samples seeded array contents, and records the field values
beside each case. Record pointers need an explicit C pointer spelling and cannot be combined
with custom shared layouts in this version.

A field may instead contain a nested `record` with its own C layout identifier, base size and
fields. Nested scalar domains cross with the root fields, and case details preserve the same
hierarchy. Inline nesting is capped at four levels and 256 expanded fields. A nested record field
may use `count` for a fixed C array of 1–64 inline records; each element's scalar domains cross
independently and paths include its index. Flexible tails and `count_from` arrays remain unsupported
inside nested records. For example, a parent field can use
`{"name":"Headers","offset":4,"count":2,"record":{"layout":"HEADER","base_size":4,"fields":[...]}}`.

The C wrapper emits `_Static_assert`s for each declared type's base `sizeof`, every field's
`offsetof`, each field element's width and declared array count. It also checks that each inline
member has the declared nested C type. This checks the contract against the host C layout before
any behavioral result can pass. Rust receives the pointer type declared by its
`rust_type` (commonly `*const u8`); no Rust struct layout is inferred or asserted. For example,
a flexible SID tail can be described as:

```json
{
  "type": "const_bytes",
  "c_type": "SID *",
  "rust_type": "*const u8",
  "record": {
    "layout": "SID",
    "base_size": 12,
    "fields": [
      {"name": "Revision", "offset": 0, "type": "u8", "count": 1, "values": [1]},
      {"name": "SubAuthorityCount", "offset": 1, "type": "u8", "count": 1, "values": [0, 1, 7, 15]},
      {"name": "Authority.Value", "offset": 2, "type": "u8", "count": 6, "values": [0, 1, 255]},
      {"name": "SubAuthority", "offset": 8, "type": "u32", "count_from": "SubAuthorityCount", "min_count": 1, "values": [0, 1, 4294967295]}
    ]
  }
}
```

A scalar pointer field may declare `points_to` with the name of another non-null byte or string
pointer argument, for example `{"name":"Buffer","offset":8,"points_to":"text"}`. A record
may also declare up to 16 separately allocated `pointees`, each with its own fixed record layout.
Use `"points_to":{"pointee":"next_node"}` to link a field to one of those records. Every
pointee must be reachable from a pointer in the root record; links may continue through other
pointees, so a small linked structure can be graded without adding fake function arguments.
Pointee records count toward the 256-field cap and cannot contain flexible tails or nested pointee
declarations. A pointee may instead be a byte buffer — the `UNICODE_STRING.Buffer` shape:
`{"name":"text","mutable":true,"bytes":{"c_type":"WCHAR","length":"Length","align":2}}`.
`length` is fixed or names a root `u8`/`u16`/`u32` field with `count: 1`, so the allocation ends
exactly where the record says the data ends and a read past it reaches the guard page.
`c_type` is the element type the C field points to, checked like any pointer field. `mutable` defaults to false, which declares const-qualified C links and rejects writes
to that allocation. Set it true when the C function may change the record; its bytes are compared
after the call.

The worker relocates fields to the declared target allocation separately for the C and Rust calls.
Reports normalize pointers to allocation plus offset and exclude process-specific address bytes
from comparisons. C layout assertions check the pointer field's offset and width; compiled
assignments check its target pointer type. Record arguments containing pointer fields or pointees
cannot be nullable or use custom shared layouts. `nullable` and `one_past` on a field explicitly
bound which results are valid if the function updates it. Initial links point to the target base;
for an empty external target, that base is also its endpoint and is accepted for that case.

A string pointer instead requires a `generator` with `max_bytes`, explicit `examples`,
and a finite grammar AST. `const_cstr` uses `encoding: "ascii"` (the default); `const_wstr` requires
`encoding: "utf16le"` and passes fixed-width `uint16_t` code units (`u16` in Rust), independent of
the host's `wchar_t` size. Its cap includes the two-byte NUL terminator. UTF-16 examples and grammar
nodes accept valid Unicode scalars, including characters encoded as surrogate pairs. Operators are
`literal`, `charset` (optional `min`/`max`),
`sequence`, `choice`, and bounded `repeat`. The size cap includes the final NUL byte; each generated
allocation ends exactly after that byte, so guard-page cases can catch a scan that reads past the
terminator. `mut_cstr` shares ASCII encoding; `mut_wstr` shares UTF-16LE encoding, with writable
views compared byte-for-byte afterward. Explicit examples cross with the scalar edge values;
seeded random cases sample the grammar. ASCII examples may contain an embedded NUL. The AST bounds how inputs
are generated; it does not prove that they are valid for the target parser. The author must record
the safe semantic domain in `validity` (for example, `atoi64.json` limits decimal runs to 18 digits
to keep the C reference free of signed overflow). Custom-layout string arguments remain
unsupported. Buffer and string input arguments
may set `nullable: true`; the generator crosses null and non-null cases, and null cases carry no
backing allocation. A `cstr_out` argument models a typed `char **` output slot. It declares
`within` input names for pointer normalization and whether the caller may pass a NULL slot; a
non-null slot starts at NULL, and the produced pointer must resolve inside one of the named views
(or one-past when explicitly allowed). C-string output slots are excluded from byte arenas, so
their process-specific pointer bytes are never compared directly.

Edge scalar combinations are crossed with zero, 255, ascending, descending and seeded random
buffer patterns. `seed` and `random_cases` make the full saved corpus reproducible. The tool
refuses oversized Cartesian products or more than 16 MiB of generated buffer payload.

## Stateful operation sequences and external-call fixtures

`stateful-host` v1 accepts scalar arguments and scalar or `void` results. A `sequences` entry
names an ordered list of functions by their C identifier. Each operation takes values from its
function's declared edge domains; the generator crosses the values across the whole sequence
and adds seeded random sequences. Every generated sequence runs in its own fresh worker process,
so static state cannot leak from one case into another. The cap is 512 generated sequences.

An optional `fixtures` entry declares a scalar external function intercepted on both sides.
The C compiler checks the generated prototype against the genuine source, while the Rust
candidate's module-level `extern` declaration is checked against the same signature. A sequence
provides an exact fixture script: each entry gives the expected call arguments and scripted
return value. Expected arguments can be literals or references to an operation argument, such
as `{"step": 1, "arg": "key"}`. The harness records fixture calls in order, checks every
argument, supplies the scripted response, and rejects extra or missing calls. For example:

```json
{
  "scope": "stateful-host",
  "fixtures": [{
    "name": "lookup",
    "c": "fixture_lookup",
    "rust_decl": "fixture_lookup",
    "return": "u32",
    "args": [{"name": "key", "type": "u32"}]
  }],
  "sequences": [{
    "name": "lookup_roundtrip",
    "steps": [
      {"function": "reset_state"},
      {"function": "remember_external"},
      {"function": "read_state"}
    ],
    "fixtures": {
      "lookup": [{"args": [{"step": 1, "arg": "key"}], "return": 137}]
    }
  }]
}
```

All functions in a stateful contract must appear in at least one sequence. Fixture return values
are exact per-call scripts; they are not inferred from the C implementation. Pointer arguments,
pointer returns, concurrent interleavings, and fixture scripts that depend on non-scalar state
remain unsupported in this scope. Stateful fixture contracts use the full Rust source module so
the checked `extern` declarations stay in scope.

`c_include_dirs`, `c_prelude` and `rust_support` name explicit, reviewed host support files;
the tool invents no stubs. A `rust_support` module may also be shipping code the selected shims
call (`compress.json` compiles `lznt1.rs` that way); a mutant with `"module": "<support key>"`
mutates that file instead of the candidate, so sensitivity can be shown there too. Supported C flags are defines/undefines, signed/unsigned-char
selection and `-fno-builtin`; Rust flags are `--cfg=...`. Dependencies discovered by both
compilers are hashed before linking and checked afterward and after execution. The C side
uses UBSan with nonrecovering errors; unresolved symbols and incompatible prototypes fail.
Windows C often reads a `WORD`/`ULONG` through a byte pointer at any offset, which UBSan kills
on every such input. `"c_allow_ub": ["alignment"]` — the only waivable check — compiles the
reference with `-fno-sanitize=alignment`; the `validity` text must justify it, and the report
records it under `c_allowed_ub` and in `limitations`. Where a narrower domain avoids the UB
instead (`sha1.json`'s `A_SHAFinal`), prefer that.
Builtin recognition is disabled, and both libraries bind their own definitions locally so
a libc symbol cannot silently replace the source being graded.

Normally the entire Rust source module is compiled. `rust_items` can select exact item
bytes via tree-sitter, including qualified paths such as `mmemmove::memmove`. Functions,
statics, consts, structs/unions/enums, type aliases and `macro_rules!` are selectable, so a
body can bring its lookup table or C-macro port with it. Same-named items are kept together
only when every copy is cfg-guarded (`#[cfg(test)]` beside `#[cfg(not(test))]`). Only an
enclosing `#![cfg(..)]` is carried onto selections; crate configuration such as
`#![cfg_attr(not(test), no_std)]` is dropped, and a `cfg_attr` that expands to a `cfg` is refused. `rust_helpers`
adds selected functions from other original files. Selections are placed together in the host
candidate module; function bodies are not rewritten. Conditional attributes from the function
and its ancestors are retained, so rustc must prove the selected definitions exist under the
recorded configuration. Relative `self::`/`super::` paths are refused because relocation could change their meaning.
Missing imports/dependencies fail compilation rather than being guessed.
Original sources, compiled support dependencies and assembled selected bytes are all hashed. This mode grades the selected bodies under the declared host context, not
whole-crate integration. Both wrappers disable Rust builtin recognition with `no_builtins`.
The `tolower` example grades the shipping pure core, not the separate C ABI shim. The
`swab` example grades the exact shipping `_swab` body against genuine standalone CRT C.

## What earns a pass

Both sides run twice in independent subprocesses. Missing output, crashes, timeouts,
nondeterminism and reference guard/const violations invalidate the grade — except one case:
a candidate that faults (or aborts, or hangs) inside the native call on an input the clean C
reference handled, and does so again in a second process, is a `mismatch`. The report carries
`candidate_fault`, `counterexample.json` holds the input, and the mutant phase is skipped. C undefined
behavior diagnosed by UBSan also invalidates it rather than blaming Rust.

Every contract includes `mutants`: named, unique exact-text replacements in the Rust source
(or selected item bytes). An exact match is preferred, so indentation may pick between
otherwise identical sites; only if the exact text is absent do its tokens match across any
whitespace, so a rustfmt reflow does not break the contract. Either way the match must be unique. All must compile, run, and change observed behavior relative to
the unmodified candidate. A compile failure or ordinary crash does **not** count as a detected mutant.
The explicit protected-page exception is described below.
Every graded function needs at least one detected sensitivity control. The supplied pilots
exercise wrong arithmetic, wrong boundaries, dropped writes and buffer overruns. Mutation
controls demonstrate specific detection abilities, not general test completeness.

## Shared arenas and returned pointers

A function's optional `layouts` array names each layout and declares `allocations` and
`bindings`. Every pointer argument must be bound exactly once; unused allocations are refused.
An extent is a nonnegative integer, scalar argument name, or `{ "arg": "n", "add": 1 }`.
Each generated pointer view is checked against its allocation before native code executes.
For example, a backward-overlapping move uses:

```json
{
  "name": "backward_overlap",
  "allocations": { "arena": { "arg": "count", "add": 1 } },
  "bindings": {
    "src": { "allocation": "arena", "offset": 0 },
    "dest": { "allocation": "arena", "offset": 1 }
  }
}
```

Each layout gets its own initialized arena on each side. The grader compares the entire arena,
including bytes outside all argument views, and checks writes against the union of mutable
views. It never creates disjoint Rust references for overlapping native arguments.

A borrowed pointer result declares `type`, `within` (allowed pointer argument names), `nullable`,
and `one_past`. Results are normalized to allocation identity plus offset, or null. Invalid
pointers are flagged without dereferencing them. Interior/end pointers must satisfy the declared
view bounds, then still match the C result: allowing one-past does not make a wrong offset pass.
An `owned_cstr` result instead declares `nullable` and a `max_bytes` bound. The worker copies
through the first NUL and frees the allocation with host libc `free` before comparing sides.
`owned_cstr` currently supports libc ownership only. An `owned_bytes` result declares
`allocator`, `nullable`, and an exact byte `length` (fixed or the name of a scalar input). The
worker copies exactly that many bytes; embedded NULs have no special meaning. The length must
stay within 65536 for every edge and random scalar value. `allocator: "libc"` releases the result
with host libc `free`. `allocator: "custom"` requires `c_free` and `rust_free` names plus an
optional `free_abi`; the generator emits typed, per-library wrappers and releases each result
through the deallocator paired with its allocator. `free_args` optionally lists up to eight
context arguments before the result pointer: `{"arg": "heap"}` forwards a scalar or pointer
input from the graded call, while `{"type": "u32", "value": 0}` supplies a typed integer
literal such as a flags value. C and Rust compilers check the complete deallocator signature,
including context and the exact returned pointer type. Deallocators must return `void`; context
that depends on generated state or values not present in the call remains unsupported.

## Protected-page read checks

Set `memory: "guard-pages"` to require Linux `mmap`/`mprotect`. Every generated memory case is
run twice: with each allocation's payload against its left inaccessible page, then its right
inaccessible page. The right placement is rounded down to the allocation's alignment (records:
their widest leaf; UTF-16 strings: 2; byte buffers and byte pointees: declared `align`), so a
record whose length is not a multiple of its alignment is not handed to the C misaligned. These are separate placements of the same input; reports count both executions.
This detects reads crossing those allocation boundaries, including zero-length reads. It does
not detect every overread of a subview inside a larger shared allocation. Accessible padding
still carries canaries. Failure to establish protection invalidates the run; there is no fallback.

The baseline C and Rust must both complete cleanly. A deliberately injected read-boundary
mutant may specify `expect: "memory_fault"`; the grader then requires SIGSEGV/SIGBUS **during
the native call in a protected-page case**, reproduced in a second independent process.
Faults during setup/observation, SIGABRT, timeouts, compile errors and ordinary output differences
do not satisfy that control. Every other mutant retains the original no-crash rule.
A candidate mismatch against a clean reference is reported as `mismatch` even if the mutant
phase then fails; that failure is kept as `mutant_error` rather than turning the finding into
`invalid`. Any unexpected exception still writes `report.json` with status `invalid`.
Reports record the faulting case, layout, placement, signal and number of completed cases;
unexecuted cases are not claimed as tested. `.fault.json`, stdout events and the saved corpus
locate baseline failures too. Timeouts retain partial output and elapsed-time metadata.

`memmove.json` uses six same-allocation layouts because the genuine C compares pointer order.
Its mutants test wrong copy direction and wrong returned pointer. `memchr.json` preserves CRT
signed-char search semantics; its mutants test a wrong returned offset and reads past either
allocation edge. Both compile the original shipping shim/helper bytes and genuine C sources.

## Limits and next work

A pass is sampled behavioral agreement within this contract. Host fixtures do not establish
Windows ABI correctness or cross-architecture behavior. Canaries alone do not catch reads;
protected pages catch allocation-edge crossings, and UBSan does not catch all C undefined
behavior. No branch coverage is reported. Report schema 16 records nullable inputs, typed C-string
output pointers, bounded owned string and byte results, generated records with nested values,
fixed arrays of inline records, relocated pointer fields and bounded hidden pointee graphs, and
the string corpus alongside scalar/buffer and layout formats. Older artifacts remain intact, but
the current worker does not replay their old format.

Still owed: coverage-guided corpus growth and counterexample minimization/replay. Unsupported types
fail explicitly. Guest
integration and smoke tests remain separate gates. Do not use this as a universal translation
acceptance switch.
