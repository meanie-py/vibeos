#!/usr/bin/env python3
"""Rewrite bindgen's same-size int transmutes -> casts (unnecessary_transmutes,
warn on rustc >= 1.92). Standalone sibling of the postprocess_bindgen.py copies
(this crate's generator is bash-only). Usage: fix_transmutes.py <file.rs>..."""
import re
import sys



TRANSMUTE_CALL = "::core::mem::transmute("

_PRIMITIVE_SIGN = {
    "i8": True, "i16": True, "i32": True, "i64": True, "isize": True,
    "u8": False, "u16": False, "u32": False, "u64": False, "usize": False,
}


def _alias_sign_map(content: str) -> dict[str, bool]:
    """Map type names -> is_signed, resolved through the file's own
    `pub type X = Y;` alias chains, seeded with the primitive ints."""
    aliases = dict(
        re.findall(r"pub type (\w+) = (?:::core::ffi::c_)?(\w+);", content)
    )
    # core::ffi aliases spelled out by bindgen
    ffi = {"char": "i8", "schar": "i8", "uchar": "u8", "short": "i16",
           "ushort": "u16", "int": "i32", "uint": "u32", "long": "i32",
           "ulong": "u32", "longlong": "i64", "ulonglong": "u64"}
    sign: dict[str, bool] = dict(_PRIMITIVE_SIGN)
    def resolve(name: str, depth: int = 0) -> bool | None:
        if name in sign:
            return sign[name]
        if depth > 8:
            return None
        tgt = aliases.get(name) or ffi.get(name)
        if tgt is None:
            return None
        s = resolve(tgt, depth + 1)
        if s is not None:
            sign[name] = s
        return s
    for name in list(aliases):
        resolve(name)
    return sign


def fix_unnecessary_transmutes(content: str) -> tuple[str, int]:
    """Rewrite bindgen's same-size int transmutes (bitfield accessors) into
    plain integer casts so the generated module compiles clean under
    `unnecessary_transmutes` (warn on rustc >= 1.92, linted only when the
    signedness differs).

        let v: uN = transmute(v);       ->  let v: uN = v as uN;   (always safe)
        transmute(EXPR as uN) -> iN ret ->  (EXPR as uN).cast_signed()
        transmute(EXPR as uN) -> uN ret ->  (EXPR as uN)
        return type not resolvable      ->  left alone (e.g. enum bitfields)

    The getter's target type is the enclosing fn's return type, resolved
    through the file's own alias table (LONG64 -> i64, ULONG64 -> u64, ...).
    """
    sign = _alias_sign_map(content)
    n = 0
    # setter form: `let val: u32 = ::core::mem::transmute(val);` (opt. unsafe {})
    pat = re.compile(
        r"let (\w+): (u8|u16|u32|u64|usize) = "
        r"(unsafe \{ )?::core::mem::transmute\((\w+)\)( \})?;"
    )
    content, k = pat.subn(
        lambda m: f"let {m.group(1)}: {m.group(2)} = {m.group(4)} as {m.group(2)};",
        content,
    )
    n += k
    # getter form (possibly multiline): `transmute(<expr> as uN)` — target is
    # the nearest preceding `-> RET {` return type (positions precomputed;
    # bisect per site to stay O(n log n) on multi-MB files).
    import bisect
    ret_pos: list[int] = []
    ret_ty: list[str] = []
    for m_ret in re.finditer(r"->\s*([\w:]+)\s*\{", content):
        ret_pos.append(m_ret.start())
        ret_ty.append(m_ret.group(1).rsplit("::", 1)[-1])
    out = []
    i = 0
    while True:
        j = content.find(TRANSMUTE_CALL, i)
        if j < 0:
            out.append(content[i:])
            break
        depth, k2 = 1, j + len(TRANSMUTE_CALL)
        while k2 < len(content) and depth:
            if content[k2] == "(":
                depth += 1
            elif content[k2] == ")":
                depth -= 1
            k2 += 1
        inner = content[j + len(TRANSMUTE_CALL):k2 - 1].rstrip()
        if inner.endswith(","):
            inner = inner[:-1].rstrip()
        idx = bisect.bisect_left(ret_pos, j) - 1
        ret = ret_ty[idx] if idx >= 0 else None
        signed = sign.get(ret) if ret else None
        if re.search(r"as (u8|u16|u32|u64|usize)$", inner) and signed is not None:
            repl = f"({inner}).cast_signed()" if signed else f"({inner})"
            out.append(content[i:j])
            out.append(repl)
            n += 1
        else:
            out.append(content[i:k2])  # unknown target — leave alone
        i = k2
    return "".join(out), n



if __name__ == "__main__":
    for path in sys.argv[1:]:
        content = open(path).read()
        content, n = fix_unnecessary_transmutes(content)
        open(path, "w").write(content)
        print(f"  {path}: rewrote {n} bitfield transmutes -> casts")
