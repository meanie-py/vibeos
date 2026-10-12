#!/usr/bin/env python3
"""Rewrite bindgen union-emulation structs into real Rust unions.

gen_driver_types.sh emits unions as the `__BindgenUnionField<T>` emulation shape
(a struct with PhantomData-style wrappers + a `bindgen_union_field` storage array)
whenever a union member is not `Copy` (bindgen's fallback). c2rust-anchored
translation bodies, by contrast, access unions via *direct field access* and name
the anonymous member `c2rust_unnamed` (e.g. `PortSC.c2rust_unnamed.PortPower()`,
`PortSC.AsULONG = ...`). The two representations don't line up.

This pass reconciles the TYPES to the c2rust body model:
  1. Add `#[derive(Copy, Clone)]` to every local struct/union that lacks a derive
     (bindgen output is pure-POD, so the Copy closure is total) — this satisfies the
     real-union member-is-Copy requirement.
  2. Convert each emulation struct (one that contains a `bindgen_union_field` line)
     into a real `#[repr(C)] pub union`:
       - `pub struct _N {`                              -> `pub union _N {`
       - drop the `pub bindgen_union_field: <storage>,` line
       - unwrap `pub F: __BindgenUnionField<T>,`        -> `pub F: T,`
       - rename anon members `__bindgen_anon_<k>`       -> `c2rust_unnamed[_{k-1}]`

Idempotent: re-running on already-converted output is a no-op. Reusable across any
gen_driver_types.sh crate (usbehci/ohci/uhci today; usbport next).
"""
import re
import sys

DERIVE_RE = re.compile(r'^\s*#\[derive\(')
REPR_RE = re.compile(r'^\s*#\[repr\(')
ATTR_RE = re.compile(r'^\s*#\[')
# The brace may wrap to the next line: a name past rustfmt's width (ntoskrnl_vista's
# `_DISPATCHER_HEADER__bindgen_ty_1__bindgen_ty_1__bindgen_ty_3__bindgen_ty_2__bindgen_ty_1`)
# is emitted as `pub struct NAME` / `{`, and missing it left that bitfield struct without
# Copy inside a real union (E0740).
STRUCT_RE = re.compile(r'^(?P<indent>\s*)pub struct (?P<name>\w+)\s*(?:\{|$)')
UNIONFIELD_RE = re.compile(
    r'^(?P<indent>\s*)pub (?P<field>\w+):\s*__BindgenUnionField<(?P<ty>.+)>,\s*$')
ANON_RE = re.compile(r'__bindgen_anon_(\d+)')


def anon_to_c2rust(name: str) -> str:
    m = ANON_RE.fullmatch(name)
    if not m:
        return name
    k = int(m.group(1))
    return 'c2rust_unnamed' if k == 1 else f'c2rust_unnamed_{k - 1}'


FIELD_START_RE = re.compile(r'^(?P<indent>\s*)pub \w+:')


def _field_complete(buf: str) -> bool:
    """A struct/union field declaration is complete when angle brackets balance and
    the statement is comma-terminated.

    The `->` of a function-pointer return type is NOT a closing angle bracket. Counting it
    as one makes the tally permanently lopsided, so coalescing never terminates and swallows
    the rest of the file into a single line. That silently TRUNCATED types_amd64.rs /
    types_arm64.rs from 3301 lines to 1602 on a field bindgen had wrapped as

        pub pfnAllocate:
            ::core::option::Option<unsafe extern "C" fn(arg1: SIZE_T) -> *mut c_void>,

    — the name alone on one line (so the buffer starts un-terminated) and a `->` on the
    next. i386 wrapped the same field differently and was unaffected, which is exactly how
    this hid: the per-arch islands disagreed rather than all failing.
    """
    angles = buf.replace('->', '')
    return angles.count('<') == angles.count('>') and buf.rstrip().endswith(',')


def coalesce_unionfields(lines):
    """Join multi-line `pub F: __BindgenUnionField<\n  Long\n>,` (and the variant where
    the field name and type wrap separately) onto one physical line so the single-line
    unwrap regex can match. bindgen wraps long union member types across lines. Only
    `pub <name>:` field declarations are coalesced — impl/fn bodies are untouched."""
    out = []
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        m = FIELD_START_RE.match(line)
        if m and not _field_complete(line):
            buf = line.rstrip()
            j = i + 1
            while j < n and not _field_complete(buf):
                buf += ' ' + lines[j].strip()
                j += 1
            out.append(buf)
            i = j
            continue
        out.append(line)
        i += 1
    return out


_INCOMPLETE_ARRAY_RE = re.compile(
    r'^#\[derive\(Default\)\]\n(pub struct __IncompleteArrayField<T>\()', re.M)


def transform(text: str) -> str:
    # Step 1 makes every local struct `Copy`, and a struct that ends in a C flexible array
    # holds bindgen's `__IncompleteArrayField<T>`, which bindgen derives only `Default` for:
    # legide's island then had 19 E0204 + 13 `Clone` errors of its own. It is a zero-sized
    # marker (`PhantomData<T>, [T; 0]`), so Copy/Clone are sound for any `T: Copy`.
    text = _INCOMPLETE_ARRAY_RE.sub(r'#[derive(Default, Copy, Clone)]\n\1', text)
    return _transform(text)


def _transform(text: str) -> str:
    lines = coalesce_unionfields(text.splitlines(keepends=False))
    out = []
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        m = STRUCT_RE.match(line)
        if m and not line.rstrip().endswith('{') and (i + 1 >= n or lines[i + 1].strip() != '{'):
            m = None  # `pub struct X;` or a tuple struct, not a braced body
        if not m:
            out.append(line)
            i += 1
            continue

        # Found a `pub struct NAME {`. Gather its body (to matching close brace at
        # the struct's indentation) and the attribute lines already emitted above it.
        indent = m.group('indent')
        body = []
        j = i + 1
        close = f'{indent}}}'
        while j < n and lines[j] != close:
            body.append(lines[j])
            j += 1
        # lines[j] is the closing brace (or EOF)
        is_union = any('pub bindgen_union_field' in b for b in body)

        # Does this struct already carry a #[derive(...)] in the attr block above?
        has_derive = False
        k = len(out) - 1
        while k >= 0 and (ATTR_RE.match(out[k]) or out[k].strip() == ''):
            if DERIVE_RE.match(out[k]):
                has_derive = True
            k -= 1

        if not has_derive:
            # Insert derive just above the first attribute line of this item (so it
            # sits with #[repr(...)]), or directly above the struct line.
            ins = len(out)
            while ins - 1 >= 0 and ATTR_RE.match(out[ins - 1]):
                ins -= 1
            out.insert(ins, f'{indent}#[derive(Copy, Clone)]')

        if not is_union:
            out.append(line)
            out.extend(body)
            if j < n:
                out.append(lines[j])
            i = j + 1
            continue

        # Convert to a real union. Anonymous members keep BOTH their bindgen name
        # (`__bindgen_anon_N`) AND a c2rust alias (`c2rust_unnamed[_k]`) so the bodies'
        # mixed naming both resolve. When there's exactly one anonymous member we also
        # emit Deref/DerefMut to its (bitfield) struct so direct method calls on the
        # union (`tok.set_Status(x)`, `tok.Status()`) resolve via deref coercion.
        uname = m.group('name')
        out.append(f'{indent}pub union {uname} {{')
        anon = []  # (c2rust_field_name, ty)
        finner = f'{indent}    '
        for b in body:
            if 'pub bindgen_union_field' in b:
                continue  # drop the storage filler
            um = UNIONFIELD_RE.match(b)
            if um:
                ty = um.group('ty').rstrip().rstrip(',').rstrip()
                raw = um.group('field')
                if ANON_RE.fullmatch(raw):
                    alias = anon_to_c2rust(raw)
                    out.append(f'{finner}pub {raw}: {ty},')
                    out.append(f'{finner}pub {alias}: {ty},')
                    anon.append((alias, ty))
                else:
                    out.append(f'{finner}pub {raw}: {ty},')
            else:
                out.append(ANON_RE.sub(lambda mm: anon_to_c2rust(mm.group(0)), b))
        if j < n:
            out.append(lines[j])  # closing brace
        if len(anon) == 1:
            alias, ty = anon[0]
            out.append(f'{indent}impl core::ops::Deref for {uname} {{')
            out.append(f'{indent}    type Target = {ty};')
            out.append(f'{indent}    #[inline] fn deref(&self) -> &{ty} {{ unsafe {{ &self.{alias} }} }}')
            out.append(f'{indent}}}')
            out.append(f'{indent}impl core::ops::DerefMut for {uname} {{')
            out.append(f'{indent}    #[inline] fn deref_mut(&mut self) -> &mut {ty} {{ unsafe {{ &mut self.{alias} }} }}')
            out.append(f'{indent}}}')
        i = j + 1

    return _unwrap_union_accessors('\n'.join(out) + ('\n' if text.endswith('\n') else ''))


_RAW_FIELD_RE = re.compile(
    r'\(\*::core::ptr::(addr_of(?:_mut)?)!\(\(\*this\)\.(\w+)\)\)\.as_(?:ref|mut)\(\) as \*(const|mut) _'
)
_SELF_FIELD_RE = re.compile(r'\bself\.(\w+)\.as_(?:ref|mut)\(\)')


def _unwrap_union_accessors(text: str) -> str:
    """A converted union's bitfield accessors still call `__BindgenUnionField::as_ref()`.

    bindgen emits accessors on the union itself (`self._bitfield_1.as_ref().get(..)`, and the
    `*_raw` forms through `addr_of!((*this)._bitfield_1)).as_ref()`) because it wrapped the
    field in `__BindgenUnionField`. After the conversion the field is the bare
    `__BindgenBitfieldUnit`, so those calls resolve to nothing (winegdi's
    `_RTL_BALANCED_NODE__bindgen_ty_2`, 8 errors per arch) -- and orphan rules keep a prelude
    from patching around it. Rewritten only inside the `impl` of a union this pass produced.
    """
    unions = set(re.findall(r'^\s*pub union (\w+) \{', text, re.M))
    if not unions:
        return text
    lines = text.split('\n')
    out = []
    inside = None
    for ln in lines:
        m = re.match(r'^(\s*)impl (\w+) \{$', ln)
        if inside is None and m and m.group(2) in unions:
            inside = m.group(1) + '}'
        elif inside is not None and ln == inside:
            inside = None
        elif inside is not None:
            ln = _RAW_FIELD_RE.sub(lambda r: f'::core::ptr::{r.group(1)}!((*this).{r.group(2)})', ln)
            ln = _SELF_FIELD_RE.sub(r'self.\1', ln)
        out.append(ln)
    return '\n'.join(out)


def main(argv):
    if len(argv) < 2:
        print('usage: bindgen_unions_to_real.py <types_*.rs> [...]', file=sys.stderr)
        return 2
    for path in argv[1:]:
        with open(path) as f:
            src = f.read()
        dst = transform(src)
        _assert_not_mangled(path, src, dst)
        with open(path, 'w') as f:
            f.write(dst)
        nu = dst.count('pub union ')
        print(f'{path}: {nu} unions after transform')
    return 0


def _assert_not_mangled(path: str, src: str, dst: str) -> None:
    """Refuse to write output that lost content or unbalanced the file.

    This pass rewrites a GENERATED file in place, so a silent mangle is invisible until
    something far away fails to compile — and it already shipped one: an unterminated
    coalesce ate 1761 lines of types_amd64.rs, leaving a file that still looked plausible.
    Both invariants are cheap and total: the transform only ever rewrites field lines and
    inserts derives, so it must never drop a declaration, and its output must still be
    brace-balanced. Fail loud (AGENTS.md) rather than hand back a truncated island.
    """
    for token in ('pub struct ', 'pub union ', 'pub type ', 'pub const '):
        before, after = src.count(token), dst.count(token)
        if token == 'pub struct ':
            # structs legitimately become unions; only the SUM is conserved.
            before += src.count('pub union ')
            after += dst.count('pub union ')
            token = 'pub struct+union '
        if after < before:
            raise SystemExit(
                f'{path}: REFUSING to write — `{token}` count fell {before} -> {after}. '
                f'The transform dropped declarations; the island would be silently '
                f'incomplete. This is a bug in this script, not in the input.'
            )
    if dst.count('{') != dst.count('}'):
        raise SystemExit(
            f'{path}: REFUSING to write — output braces unbalanced '
            f'({dst.count("{")} open vs {dst.count("}")} close).'
        )


if __name__ == '__main__':
    raise SystemExit(main(sys.argv))
