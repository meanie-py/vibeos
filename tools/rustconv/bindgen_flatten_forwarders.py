#!/usr/bin/env python3
"""Generate flattening forwarder methods for nested bindgen anonymous struct/union members.

c2rust-anchored translation bodies access bitfield accessors *flat* on the outermost
type — e.g. `PortStatus.set_SetPortPower(1)` or `IntStatus.c2rust_unnamed.RootHubStatusChange()`
— because c2rust promotes anonymous struct/union members into the parent (C semantics).
bindgen instead nests them: the method lives several `__bindgen_anon_*` / `c2rust_unnamed*`
levels down. A single Deref (emitted by bindgen_unions_to_real.py) only bridges one level
and can't span a *multi*-anon union (a register with overlapping read/write bitfield views).

This pass walks the anon-member tree of every local struct/union and, for each type that has
anonymous members, emits an `impl T { ... }` of forwarder methods — one per bitfield accessor
reachable through its anon members — each delegating via the exact field path. After this,
flat access (`outer.set_SetPortPower(v)`) and one-level access (`outer.c2rust_unnamed.M()`)
both resolve, because every type on the access path carries forwarders for its subtree.

Run AFTER bindgen_unions_to_real.py. Idempotent (skips if a `// @flatten-forwarders` marker
is already present). Reusable across any gen_driver_types.sh crate.
"""
import re
import sys

TYPE_RE = re.compile(r'^(?P<indent>[ \t]*)pub (?P<kind>struct|union) (?P<name>\w+) \{')
ANON_FIELD_RE = re.compile(
    r'^\s*pub (?P<field>c2rust_unnamed(?:_\d+)?|__bindgen_anon_\d+):\s*(?P<ty>\w+),')
IMPL_RE = re.compile(r'^impl (?P<name>\w+) \{')
GETTER_RE = re.compile(r'^\s*pub fn (?P<n>\w+)\(&self\)\s*->\s*(?P<ret>[^{]+?)\s*\{')
SETTER_RE = re.compile(r'^\s*pub fn (?P<n>set_\w+)\(&mut self,\s*val:\s*(?P<vt>[^{]+?)\)\s*\{')
MARKER = '// @flatten-forwarders'


def parse(lines):
    """Return (anon_members, methods): type -> [(field,ty)], type -> [(name,kind,sig)]."""
    anon = {}    # type -> list of (field_name, field_type)  (deduped: prefer c2rust_unnamed)
    methods = {} # type -> list of (method_name, 'get'|'set', ret_or_valtype)
    n = len(lines)
    i = 0
    while i < n:
        line = lines[i]
        tm = TYPE_RE.match(line)
        if tm:
            close = f'{tm.group("indent")}}}'
            fields = []
            j = i + 1
            while j < n and lines[j] != close:
                fm = ANON_FIELD_RE.match(lines[j])
                if fm:
                    fields.append((fm.group('field'), fm.group('ty')))
                j += 1
            if fields:
                # dedup union alias pairs: a c2rust_unnamed and a __bindgen_anon_1 of the
                # same type are the same member — keep the c2rust_unnamed spelling only.
                seen_ty = {}
                dedup = []
                for f, ty in fields:
                    if ty in seen_ty:
                        # prefer c2rust_unnamed naming
                        if f.startswith('c2rust_unnamed') and not seen_ty[ty][0].startswith('c2rust_unnamed'):
                            dedup[seen_ty[ty][1]] = (f, ty)
                            seen_ty[ty] = (f, seen_ty[ty][1])
                        continue
                    seen_ty[ty] = (f, len(dedup))
                    dedup.append((f, ty))
                anon[tm.group('name')] = dedup
            i = j + 1
            continue
        im = IMPL_RE.match(line)
        if im:
            name = im.group('name')
            ms = methods.setdefault(name, [])
            j = i + 1
            depth = 1
            while j < n and depth > 0:
                gm = GETTER_RE.match(lines[j])
                sm = SETTER_RE.match(lines[j])
                if gm and not gm.group('n').endswith('_raw'):
                    ms.append((gm.group('n'), 'get', gm.group('ret').strip()))
                elif sm and not sm.group('n').endswith('_raw'):
                    ms.append((sm.group('n'), 'set', sm.group('vt').strip()))
                # track block end via the line that is exactly "}" at column 0
                if lines[j] == '}':
                    depth = 0
                j += 1
            i = j
            continue
        i += 1
    return anon, methods


def reachable(t, anon, methods, seen):
    """List of (path, name, kind, sig) for every method reachable from type `t`,
    path = field-access chain from t to the owning type ('' = directly on t)."""
    out = []
    for (name, kind, sig) in methods.get(t, []):
        out.append(('', name, kind, sig))
    if t in seen:
        return out
    seen = seen | {t}
    for (field, ty) in anon.get(t, []):
        for (sub, name, kind, sig) in reachable(ty, anon, methods, seen):
            out.append((f'.{field}{sub}', name, kind, sig))
    return out


def gen_forwarders(name, anon, methods):
    own = {m[0] for m in methods.get(name, [])}
    seen_names = set(own)
    lines = []
    for (field, ty) in anon.get(name, []):
        for (sub, mname, kind, sig) in reachable(ty, anon, methods, set()):
            if mname in seen_names:
                continue
            seen_names.add(mname)
            path = f'{field}{sub}'
            if kind == 'get':
                lines.append(
                    f'    #[inline] pub fn {mname}(&self) -> {sig} {{ unsafe {{ self.{path}.{mname}() }} }}')
            else:
                lines.append(
                    f'    #[inline] pub fn {mname}(&mut self, val: {sig}) {{ unsafe {{ self.{path}.{mname}(val) }} }}')
    return lines


def transform(text):
    if MARKER in text:
        return text
    lines = text.split('\n')
    anon, methods = parse(lines)
    blocks = []
    for name in anon:
        fwd = gen_forwarders(name, anon, methods)
        if fwd:
            blocks.append(f'impl {name} {{\n' + '\n'.join(fwd) + '\n}')
    if not blocks:
        return text
    return text.rstrip('\n') + '\n\n' + MARKER + '\n' + '\n'.join(blocks) + '\n'


def main(argv):
    for path in argv[1:]:
        src = open(path).read()
        dst = transform(src)
        open(path, 'w').write(dst)
        print(f'{path}: {"forwarders added" if dst != src else "no-op"}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv))
