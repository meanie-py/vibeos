#!/usr/bin/env python3
"""Auto-apply faithful `as` casts for primitive-int E0308 mismatches (c2rust cast tail).

c2rust-anchored translations carry a long tail of integer-type frictions: a value of one
integer width/signedness used where another is expected (size_of::<T>() as SIZE_T passed to
a ULONG param, ULONG bitfield result compared to an i32 #define, usize index vs u32, ...).
Each is a per-site `as` cast that c2rust normally inserts but the LLM refinement dropped.

This tool runs the crate build with `--message-format=json`, finds every E0308 "mismatched
types" whose *expected* type is a primitive integer, and wraps the offending expression span
in `(<expr> as <expected>)` — the truncating C-equivalent conversion (NOT `.try_into().unwrap()`,
which would add panic paths / ud2 bloat to kernel code). Edits are applied back-to-front per
file so byte offsets stay valid, then the build is re-run; it loops until no new casts apply.

Skips non-int expected types (fn-ptr ABI, bool, structs) — those need real fixes, not a cast.
Reusable across any cohab crate; pair with bindgen_unions_to_real.py + flatten_forwarders.py.
"""
import json
import re
import subprocess
import sys

INT_TYPES = {
    'u8', 'u16', 'u32', 'u64', 'u128', 'usize',
    'i8', 'i16', 'i32', 'i64', 'i128', 'isize',
}
EXPECTED_RE = re.compile(r'expected `([^`]+)`, found `([^`]+)`')


def run_build(pkg, target, manifest, rustflags):
    cmd = [
        'cargo', 'build', '--release', '--target', target,
        '-p', pkg, '--manifest-path', manifest, '--message-format=json',
    ]
    env = {'RUSTC_BOOTSTRAP': '1', 'RUSTFLAGS': rustflags}
    import os
    e = dict(os.environ)
    e.update(env)
    p = subprocess.run(cmd, capture_output=True, text=True, env=e)
    return p.stdout


def collect_casts(json_out):
    """-> dict file -> list of (byte_start, byte_end, expected_type)."""
    edits = {}
    for line in json_out.splitlines():
        line = line.strip()
        if not line.startswith('{'):
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get('reason') != 'compiler-message':
            continue
        d = msg.get('message', {})
        code = (d.get('code') or {}).get('code')
        if code == 'E0308':
            for span in d.get('spans', []):
                if not span.get('is_primary'):
                    continue
                label = span.get('label') or ''
                m = EXPECTED_RE.search(label) or EXPECTED_RE.search(d.get('message', ''))
                if not m:
                    continue
                expected, found = m.group(1), m.group(2)
                cast = None
                if expected in INT_TYPES:
                    cast = expected
                elif expected.startswith(('*mut ', '*const ')) and \
                        found.startswith(('*mut ', '*const ')):
                    # pointer-to-pointer mismatch (e.g. KSPIN_LOCK *mut u32 vs *mut usize,
                    # void* vs typed*). Faithful `as <expected>` — but never on a &str found
                    # (string literal; that needs a b"..." rewrite, handled elsewhere).
                    cast = expected
                if cast is None:
                    # `*const i8` (DbgPrint/DPRINT fmt) fed a Rust `&str` literal — rewrite
                    # the string span to a NUL-terminated byte-string pointer.
                    if found == '&str' and ('i8' in expected or 'u8' in expected
                                            or 'c_char' in expected):
                        edits.setdefault(span['file_name'], []).append(
                            (span['byte_start'], span['byte_end'], '__BSTR__'))
                    continue
                fn = span['file_name']
                edits.setdefault(fn, []).append(
                    (span['byte_start'], span['byte_end'], cast))
        elif code == 'E0617':
            # `can't pass `T` to variadic function` — C promotes small ints to int;
            # wrap the arg span in `(... as c_int)` (the faithful default-arg-promotion).
            for span in d.get('spans', []):
                if not span.get('is_primary'):
                    continue
                fn = span['file_name']
                edits.setdefault(fn, []).append(
                    (span['byte_start'], span['byte_end'], '::core::ffi::c_int'))
    return edits


def apply(edits):
    n = 0
    for fn, spans in edits.items():
        data = open(fn, 'rb').read()
        # de-dup + sort descending so earlier offsets stay valid
        seen = set()
        uniq = []
        for s in sorted(spans, key=lambda x: x[0], reverse=True):
            if (s[0], s[1]) in seen:
                continue
            seen.add((s[0], s[1]))
            uniq.append(s)
        for bs, be, ty in uniq:
            expr = data[bs:be]
            if ty == '__BSTR__':
                # expr is a Rust string literal `"..."`; emit a NUL-terminated *const i8.
                if expr.startswith(b'"') and expr.endswith(b'"'):
                    inner = expr[1:-1]
                    data = (data[:bs] + b'b"' + inner + b'\\0".as_ptr() as *const i8'
                            + data[be:])
                    n += 1
                continue
            # skip if it already looks wrapped with this cast
            if expr.startswith(b'(') and expr.endswith((f' as {ty})').encode()):
                continue
            data = data[:bs] + b'(' + expr + f' as {ty})'.encode() + data[be:]
            n += 1
        open(fn, 'wb').write(data)
    return n


def main(argv):
    if len(argv) < 4:
        print('usage: autocast.py <pkg> <target> <manifest> [rustflags] [max_iters]',
              file=sys.stderr)
        return 2
    pkg, target, manifest = argv[1], argv[2], argv[3]
    rustflags = argv[4] if len(argv) > 4 else ''
    max_iters = int(argv[5]) if len(argv) > 5 else 8
    total = 0
    for it in range(max_iters):
        out = run_build(pkg, target, manifest, rustflags)
        edits = collect_casts(out)
        if not edits:
            print(f'iter {it}: no int-cast E0308 left')
            break
        applied = apply(edits)
        total += applied
        print(f'iter {it}: applied {applied} casts')
        if applied == 0:
            break
    print(f'total casts applied: {total}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv))
