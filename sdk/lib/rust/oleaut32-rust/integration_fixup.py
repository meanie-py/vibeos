#!/usr/bin/env python3
"""One-shot integration fixups for oleaut32-rust rustconv modules.

Generates (from the real headers, never hand-typed):
  src/consts.rs      — VT_*/HRESULT/flag constants the translations use bare
  src/winemacros.rs  — V_* accessors (+_ref lvalue forms), DEC_*/DOS_*/FITS_*,
                       HIWORD-family, from oleauto.h + wine variant.h + C bodies
Transforms the rustconv module files in place:
  - strips extern decls for names we now provide at compile time
  - rewrites lvalue-macro calls  NAME(args) -> (*NAME_ref(args))
  - computes ((VT_A << 16) | VT_B)-style match-arm heads into literals
"""
import re, sys, pathlib

REPO = pathlib.Path('/home/curi/Projects/os-playground/vibeos-research')
CRATE = REPO / 'sdk/lib/rust/oleaut32-rust/src'
BINDGEN = REPO / 'sdk/lib/rust/com-sys/src/bindgen_amd64.rs'
MODULES = ['hash.rs', 'oleaut.rs', 'safearray.rs', 'variant.rs', 'usrmarshal.rs', 'vartype.rs']

# ---------- A. VT_* from com-sys ----------
bind = BINDGEN.read_text()
vt = dict(re.findall(r'pub const VARENUM_(VT_[A-Z0-9_]+): VARENUM = (\d+);', bind))
sf = dict(re.findall(r'pub const tagSF_TYPE_(SF_[A-Z0-9_]+): SF_TYPE = (\d+);', bind))

# tagVARIANT n3 union fields -> rust type
m = re.search(r'pub union tagVARIANT__bindgen_ty_1___tagVARIANT__bindgen_ty_1 \{(.*?)\n\}', bind, re.S)
n3_fields = dict(re.findall(r'pub (\w+): ([^,]+),', m.group(1))) if m else {}

# ---------- B. constants from headers ----------
def parse_defines(text):
    out = {}
    for name, val in re.findall(r'^#define\s+([A-Z_][A-Za-z0-9_]*)\s+(.+?)\s*(?:/\*.*)?$', text, re.M):
        out.setdefault(name, val.strip())
    return out

def c_int(expr, env):
    """Try to evaluate a C integer #define expression."""
    e = expr.strip()
    e = re.sub(r'_HRESULT_TYPEDEF_\((.+)\)', r'\1', e)
    e = re.sub(r'\(\s*(HRESULT|DWORD|ULONG|LONG|int|WORD|VARTYPE|SCODE|LONGLONG|ULONGLONG|VARIANT_BOOL)\s*\)', '', e)
    e = e.replace('UL', '').replace('L', '').replace('u', '')
    e = re.sub(r'\b([A-Z_][A-Za-z0-9_]*)\b', lambda m: str(env[m.group(1)]) if m.group(1) in env else m.group(1), e)
    try:
        return eval(e, {'__builtins__': {}}, {})
    except Exception:
        return None

hdrs = ''
BUILD = pathlib.Path('/home/curi/Projects/os-playground/vibeos/build-amd64')
for h in ['sdk/include/psdk/winnt.h', 'sdk/include/psdk/wtypes.h', 'sdk/include/psdk/oaidl.h']:
    p2 = BUILD / h
    if p2.exists():
        hdrs += p2.read_text(errors='replace') + '\n'
for h in ['sdk/include/psdk/winerror.h', 'sdk/include/psdk/oleauto.h',
          'sdk/include/psdk/winnt.h', 'sdk/include/psdk/winnls.h',
          'sdk/include/psdk/winbase.h', 'sdk/include/psdk/winuser.h',
          'sdk/include/psdk/wingdi.h', 'sdk/include/psdk/winreg.h',
          'sdk/include/psdk/minwindef.h', 'sdk/include/psdk/oleidl.h',
          'sdk/include/psdk/objbase.h', 'sdk/include/psdk/winsvc.h',
          'sdk/include/xdk/ntbasedef.h',
          'sdk/include/xdk/winnt_old.h', 'sdk/include/xdk/cmtypes.h',
          'sdk/include/psdk/rpcndr.h', 'sdk/include/psdk/olectl.h',
          'dll/win32/oleaut32/variant.h']:
    p = REPO / h
    if p.exists():
        hdrs += p.read_text(errors='replace') + '\n'
defines = parse_defines(hdrs)

# seed env with VT_* so VTBIT_x = (1 << VT_x) etc. evaluate
env = {k: int(v) for k, v in vt.items()}
env.update({k: int(v) for k, v in sf.items()})
for extra in ['DECIMAL_NEG', 'DECIMAL_POS']:  # variant.h: 0x80 / 0
    pass

missing = [l.strip() for l in open(sys.argv[1])] if len(sys.argv) > 1 else []
consts, unresolved = [], []
for name in missing:
    if name in vt:
        consts.append(f'pub const {name}: u16 = {vt[name]};  // VARTYPE')
    elif name in sf:
        consts.append(f'pub const {name}: SF_TYPE = {sf[name]};')
    elif name in defines:
        val = c_int(defines[name], env)
        if val is None:
            unresolved.append((name, defines[name]))
        else:
            if name.startswith(('FADF_', 'LANG_', 'SUBLANG_', 'SORT_')):
                ty = 'u16'
            elif name.startswith(('VARCMP_',)) or (0 <= val <= 0x7FFFFFFF and re.match(r'(S_|E_|DISP_E_|TYPE_E_|CLASS_E_|CO_E_|RPC_[ES]_|STG_E_|REGDB_E_|OLE_E_)', name)):
                ty = 'i32'
            else:
                ty = None
            if val < 0 or val > 0xFFFFFFFF:
                consts.append(f'pub const {name}: i64 = {val};')
            elif val > 0x7FFFFFFF:
                consts.append(f'pub const {name}: u32 = 0x{val:X};  // {defines[name]}')
            elif ty:
                consts.append(f'pub const {name}: {ty} = {val};')
            else:
                consts.append(f'pub const {name}: u32 = {val};')
            env[name] = val
    else:
        mm = re.search(r'pub const [A-Za-z0-9_]*?_(' + re.escape(name.lstrip('_')) + r'): [A-Za-z0-9_]+ = (\d+);', bind)
        if mm:
            consts.append(f'pub const {name}: u32 = {mm.group(2)};')
            env[name] = int(mm.group(2))
        else:
            unresolved.append((name, None))

# HRESULTs should be i32 (they're compared/returned as HRESULT = c_long)
fixed = []
for line in consts:
    mm = re.match(r'pub const ([A-Z0-9_]+): u32 = 0x([0-9A-F]+);', line)
    if mm and int(mm.group(2), 16) >= 0x80000000:
        # negative HRESULT-like value
        fixed.append(f'pub const {mm.group(1)}: i32 = 0x{mm.group(2)}u32 as i32;')
    else:
        fixed.append(line)
consts = fixed

hdr = ('//! GENERATED by oleaut_fixup.py from the SDK headers + com-sys — do not hand-edit\n'
       '//! wholesale; regenerate or append below the marker.\n'
       '#![allow(clippy::all)]\nuse crate::prelude::*;\n\n')
(CRATE / 'consts.rs').write_text(hdr + '\n'.join(sorted(set(consts))) + '\n')
print(f'consts.rs: {len(set(consts))} consts')
print('UNRESOLVED:', ' '.join(n for n, _ in unresolved))

# ---------- C. V_* accessors from oleauto.h ----------
olea = (REPO / 'sdk/include/psdk/oleauto.h').read_text(errors='replace')
vun = dict(re.findall(r'#define (V_[A-Z0-9_]+)\(A\)\s+V_UNION\(A,(\w+)\)', olea))
RUST_TY = lambda f: n3_fields.get(f, '???').strip()
acc = ['//! GENERATED V_* accessors (oleauto.h V_UNION map) + wine variant.h macro fns.',
       '#![allow(clippy::all)]', 'use crate::prelude::*;', '']
lv_names = []
for name, field in sorted(vun.items()):
    ty = RUST_TY(field)
    if ty == '???':
        continue
    acc.append(f'#[inline]\npub unsafe fn {name}_ref(v: *mut VARIANT) -> *mut {ty} {{')
    acc.append(f'    core::ptr::addr_of_mut!((*v).n1.n2.n3.{field})')
    acc.append('}')
    lv_names.append(name)
# V_VT + decVal (outside n3)
acc.append('#[inline]\npub unsafe fn V_VT_ref(v: *mut VARIANT) -> *mut VARTYPE {')
acc.append('    core::ptr::addr_of_mut!((*v).n1.n2.vt)\n}')
lv_names.append('V_VT')
acc.append('#[inline]\npub unsafe fn V_DECIMAL_ref(v: *mut VARIANT) -> *mut DECIMAL {')
acc.append('    core::ptr::addr_of_mut!((*v).n1.decVal)\n}')
lv_names.append('V_DECIMAL')
(CRATE / 'winemacros_gen.rs').write_text('\n'.join(acc) + '\n')
print(f'winemacros_gen.rs: {len(lv_names)} accessors')

# ---------- D. transform module files ----------
provided = set(lv_names) | {n for n, _ in
                            [(re.match(r'pub const ([A-Z0-9_]+)', c).group(1), None)
                             for c in consts if re.match(r'pub const ([A-Z0-9_]+)', c)]}
HAND = ['TRACE', 'WARN', 'ERR', 'FIXME', 'FAILED', 'SUCCEEDED', 'HIWORD', 'LOWORD', 'HIBYTE',
        'LOBYTE', 'MAKELANGID', 'MAKELCID', 'PRIMARYLANGID', 'SUBLANGID', 'LANGIDFROMLCID',
        'GetRValue', 'GetGValue', 'GetBValue', 'PALETTERGB', 'InterlockedIncrement',
        'InterlockedDecrement', 'ZeroMemory', 'IsLeapYear', 'DEC_HI32', 'DEC_MID32',
        'DEC_LO32', 'DEC_LO64', 'DEC_SIGN', 'DEC_SCALE', 'DEC_SIGNSCALE', 'SIGNSCALE',
        'DECIMAL_SETZERO', 'DOS_YEAR', 'DOS_MONTH', 'DOS_DAY', 'DOS_HOUR', 'DOS_MINUTE',
        'DOS_SECOND', 'FITS_AS_I1', 'FITS_AS_I2', 'FITS_AS_I4', 'V_ISBYREF', 'V_ISARRAY',
        'V_ISVECTOR', 'V_TYPE', 'V_EXTRA_TYPE', 'debugstr_a', 'debugstr_w', 'debugstr_an',
        'debugstr_wn', 'debugstr_guid', 'debugstr_vt', 'debugstr_vf', 'debugstr_variant',
        'wine_dbgstr_longlong', 'ARRAY_SIZE', 'FIELD_OFFSET']
provided |= set(HAND)
LVALUE = set(lv_names) | {'DEC_HI32', 'DEC_MID32', 'DEC_LO32', 'DEC_LO64', 'DEC_SIGNSCALE'}

def strip_extern_decls(src):
    """Remove extern-block decls (fn/static) whose name is in `provided`."""
    def drop(m):
        body = m.group(2)
        keep = []
        for item in re.split(r'(?<=;)\n', body):
            nm = re.search(r'\b(?:fn|static(?:\s+mut)?)\s+([A-Za-z0-9_]+)', item)
            if nm and nm.group(1) in provided:
                continue
            keep.append(item)
        inner = '\n'.join(k for k in keep if k.strip())
        if not inner.strip():
            return ''
        return m.group(1) + inner + '\n}'
    return re.sub(r'(extern "(?:C|system)" \{\n)(.*?)\n\}', drop, src, flags=re.S)

def lvalue_transform(src):
    """NAME(args) -> (*NAME_ref(args)) with paren matching, for LVALUE names."""
    out, i = [], 0
    pat = re.compile(r'\b(' + '|'.join(sorted(LVALUE, key=len, reverse=True)) + r')\(')
    while True:
        m = pat.search(src, i)
        if not m:
            out.append(src[i:])
            break
        out.append(src[i:m.start()])
        depth, j = 1, m.end()
        while depth and j < len(src):
            depth += src[j] == '('
            depth -= src[j] == ')'
            j += 1
        args = src[m.end():j - 1]
        out.append(f'(*{m.group(1)}_ref({args}))')
        i = j
    return ''.join(out)

def match_literal_transform(src):
    """((VT_A) << 16) | VT_B in pattern position -> computed literal."""
    def rep(m):
        a, b = m.group(1), m.group(2)
        if a in env and b in env:
            return f'{(env[a] << 16) | env[b]} /* (({a} << 16) | {b}) */'
        return m.group(0)
    return re.sub(r'\(\s*([A-Z0-9_]+)\s*<<\s*16\s*\)\s*\|\s*([A-Z0-9_]+)', rep, src)

for mod in MODULES:
    p = CRATE / mod
    if not p.exists():
        continue
    s = p.read_text()
    s = strip_extern_decls(s)
    s = lvalue_transform(s)
    s = match_literal_transform(s)
    p.write_text(s)
    print(f'transformed {mod}')
