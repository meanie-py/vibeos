#!/usr/bin/env python3
"""Generate a delay-import library (.a) from an implib .def, for llvm-mingw COFF.

Replaces GNU dlltool's `--output-delaylib` (which llvm-dlltool lacks). Emits a per-DLL
assembly file (descriptor + INT/IAT + name strings + per-function thunk + import stub),
compiles it with clang, and archives it with llvm-ar.

The thunk passes (pidd, &IATslot) to __delayLoadHelper2 (sdk/lib/delayimp), which
LoadLibrary+GetProcAddress+patches the IAT slot. All image-relative fields use `.rva`.
On amd64/arm64 the thunk must save/restore the volatile arg registers across the helper
call (the call clobbers them); on i386 args are on the stack so the thunk is trivial.

Validated against the i386 spike in docs/engineering/delay-import-llvm-mingw-design.md §5b.
"""
import argparse, re, subprocess, sys, os

ARCH = {
    "i386":  dict(triple="i686-w64-windows-gnu",   ptr=4, helper="___delayLoadHelper2@8",
                  pub=lambda s: "_"+s,       imp=lambda s: "__imp__"+s),
    "amd64": dict(triple="x86_64-w64-windows-gnu",  ptr=8, helper="__delayLoadHelper2",
                  pub=lambda s: s,           imp=lambda s: "__imp_"+s),
    "arm64": dict(triple="aarch64-w64-windows-gnu", ptr=8, helper="__delayLoadHelper2",
                  pub=lambda s: s,           imp=lambda s: "__imp_"+s),
}

# The delay-import table's bracket (see emit). The magic is exactly 16 bytes, so the
# first descriptor follows it 4-aligned; pefixup.c's DELAY_TABLE_MAGIC must match.
DELAY_TABLE_MAGIC = "VIBEOS-DLYIMP-01"
DELAY_TABLE_HEAD = "__vibeos_delayimp_head"
DELAY_TABLE_TAIL = "__vibeos_delayimp_tail"
assert len(DELAY_TABLE_MAGIC) == 16


def parse_def(path):
    dll, exports, in_exports, seen = None, [], False, set()
    for line in open(path):
        line = line.split(";")[0].rstrip()
        if not line.strip():
            continue
        m = re.match(r"\s*(NAME|LIBRARY)\s+(\S+)", line, re.I)
        if m:
            dll = m.group(2); continue
        if re.match(r"\s*EXPORTS\b", line, re.I):
            in_exports = True; continue
        if in_exports:
            parts = line.split()
            tok = parts[0]
            if tok.startswith("@"):           # ordinal-only export (@123 [NONAME]) — not delay-bound by name
                continue
            name = tok.split("=")[0]          # strip '=forwarder.dll.Target' (GetProcAddress follows the forward)
            if name in seen:                  # dedup (a real export + a forwarder alias can share a name)
                continue
            seen.add(name)
            is_data = "DATA" in parts[1:]
            # `Name @127 NONAME`: the DLL exports NO name, only the ordinal, so a by-name import
            # can never resolve. Binding it by name made every call raise 0xc06d007f —
            # browseui's SHOnCWMCommandLine, i.e. every folder window opened from another process.
            ordinal = None
            if "NONAME" in parts[1:]:
                ords = [t for t in parts[1:] if re.fullmatch(r"@\d+", t)]
                if not ords:
                    sys.exit(f"{path}: NONAME export {name} has no @ordinal")
                ordinal = int(ords[0][1:])
            exports.append((name, is_data, ordinal))
    if not dll:
        sys.exit(f"{path}: no NAME/LIBRARY line")
    return dll, exports

def q(sym):  # quote symbols containing '@' for the assembler
    return f'"{sym}"' if "@" in sym else sym

def emit(dll, exports, arch):
    cfg = ARCH[arch]
    ptr = cfg["ptr"]; helper = cfg["helper"]
    fns = [n for n, d, _o in exports if not d]
    ords = {n: o for n, d, o in exports if not d}
    datas = [n for n, d, _o in exports if d]
    desc = f"___DELAY_DESC_{re.sub(r'[^A-Za-z0-9_]','_',dll)}"
    L, ptrdir = [], (".long" if ptr == 4 else ".quad")
    # The IAT and INT hold pointer-sized slots, so align them to the pointer. On
    # arm64 this is not cosmetic: the import stub loads a slot with
    # `ldr x16, [x16, :lo12:__imp_X]`, whose scaled offset must be a multiple of
    # 8, and a 4-aligned IAT fails the link ("misaligned ldr/str offset").
    ptralign = "\t.p2align 2" if ptr == 4 else "\t.p2align 3"

    # --- .rdata: dll name, hint/name strings, INT ---
    # The trailing .rva pair is what keeps the table's head and tail (below) through
    # --gc-sections: nothing else references them, and this section is live because
    # the descriptor names __dll_name.
    L += ['\t.section .rdata,"dr"', '\t.p2align 2',
          f'\t.rva {DELAY_TABLE_HEAD}', f'\t.rva {DELAY_TABLE_TAIL}',
          f'__dll_name: .asciz "{dll}"']
    for i, fn in enumerate(fns):
        if ords[fn] is not None:
            continue
        # IMAGE_IMPORT_BY_NAME { WORD Hint; char Name[]; }. The Name is the *undecorated*
        # export name (DLLs export `ScriptBreak`, not `ScriptBreak@16`) — GetProcAddress
        # looks it up by that. The @N stays only on the import symbols (_Name@N/__imp__Name@N).
        und = fn.split("@")[0]
        L += ['\t.p2align 1', f'__hn_{i}: .short 0', f'\t.asciz "{und}"']
    L += [ptralign, '__delay_int:']
    for i, fn in enumerate(fns):
        if ords[fn] is not None:
            # IMAGE_ORDINAL_FLAG (the top bit of the pointer-sized slot) | ordinal
            L += ([f'\t.long {0x80000000 | ords[fn]:#x}'] if ptr == 4
                  else [f'\t.long {ords[fn]:#x}', '\t.long 0x80000000'])
            continue
        # INT entry holds an RVA in low 32 bits; pad to ptr width
        L += [f'\t.rva __hn_{i}'] + (['\t.long 0'] if ptr == 8 else [])
    L += ['\t.long 0'] + (['\t.long 0'] if ptr == 8 else [])    # null terminator (ptr-sized)

    # --- .data: IAT (patched at runtime) + hmod cache ---
    L += ['\t.section .data,"dw"', ptralign, '__delay_iat:']
    for i, fn in enumerate(fns):
        L += [f'\t.globl {q(cfg["imp"](fn))}', f'{q(cfg["imp"](fn))}:', f'\t{ptrdir} __thunk_{i}']
    L += [f'\t{ptrdir} 0', '__delay_hmod:', f'\t{ptrdir} 0']

    # --- descriptor (ImgDelayDescr, all RVAs) ---
    # lld leaves data directory 13 empty for descriptors it did not make itself, and
    # the Windows loader never needs it (the thunk hands the helper its descriptor).
    # But anything that walks a module's delay imports -- the sdk delayimp apitest,
    # dbghelp, a debugger -- reads the directory, and the table it points at must be
    # one contiguous, null-terminated array. So every library's descriptor goes in
    # the grouped section `.CRT$VBDLYIMPB`, which the linker sorts between a COMDAT
    # head (`A`, a magic marker) and tail (`C`, the null descriptor) that every delay
    # library carries and the link keeps one of. `pefixup --delayimport` then finds
    # the marker in the image and sets the directory.
    # Why `.CRT$` and not `.rdata$`: lld in MinGW mode keeps GNU ld's semantics and
    # does NOT sort `$`-grouped sections, except `.CRT$` (and `.tls$`), which it sorts
    # by name and merges into .rdata. `.rdata$` came out head, tail, unrelated data,
    # then the descriptor. `VB` sorts before the CRT's own `.CRT$XC*`/`XL*` ranges, so
    # the table cannot land inside the initializer or TLS-callback arrays.
    L += [f'\t.section .CRT$VBDLYIMPA,"dr",discard,{DELAY_TABLE_HEAD}', '\t.p2align 2',
          f'\t.globl {DELAY_TABLE_HEAD}', f'{DELAY_TABLE_HEAD}:',
          f'\t.ascii "{DELAY_TABLE_MAGIC}"']
    L += [f'\t.section .CRT$VBDLYIMPC,"dr",discard,{DELAY_TABLE_TAIL}', '\t.p2align 2',
          f'\t.globl {DELAY_TABLE_TAIL}', f'{DELAY_TABLE_TAIL}:', '\t.zero 32']
    L += ['\t.section .CRT$VBDLYIMPB,"dr"', '\t.p2align 2', f'\t.globl {desc}', f'{desc}:',
          '\t.long 1', '\t.rva __dll_name', '\t.rva __delay_hmod',
          '\t.rva __delay_iat', '\t.rva __delay_int', '\t.long 0', '\t.long 0', '\t.long 0']

    # --- .text: import stubs + per-function thunks ---
    L += ['\t.section .text,"xr"']
    for i, fn in enumerate(fns):
        pub, imp = cfg["pub"](fn), cfg["imp"](fn)
        L += [f'\t.globl {q(pub)}', f'{q(pub)}:']
        if arch == "i386":
            L += [f'\tjmp  *{q(imp)}']
        elif arch == "amd64":
            L += [f'\tjmp  *{q(imp)}(%rip)']
        else:  # arm64
            L += [f'\tadrp x16, {q(imp)}', f'\tldr  x16, [x16, :lo12:{q(imp)}]', '\tbr   x16']
        # thunk
        L += [f'__thunk_{i}:']
        if arch == "i386":
            L += [f'\tpushl ${q(imp)}', f'\tpushl ${desc}', f'\tcalll {helper}', '\tjmpl  *%eax']
        elif arch == "amd64":
            L += [
                '\tsubq $0x88, %rsp',
                '\tmovq %rcx,0x20(%rsp)', '\tmovq %rdx,0x28(%rsp)',
                '\tmovq %r8,0x30(%rsp)',  '\tmovq %r9,0x38(%rsp)',
                '\tmovaps %xmm0,0x40(%rsp)', '\tmovaps %xmm1,0x50(%rsp)',
                '\tmovaps %xmm2,0x60(%rsp)', '\tmovaps %xmm3,0x70(%rsp)',
                f'\tleaq {q(imp)}(%rip), %rdx', f'\tleaq {desc}(%rip), %rcx',
                f'\tcallq {helper}', '\tmovq %rax, %r11',
                '\tmovaps 0x70(%rsp),%xmm3', '\tmovaps 0x60(%rsp),%xmm2',
                '\tmovaps 0x50(%rsp),%xmm1', '\tmovaps 0x40(%rsp),%xmm0',
                '\tmovq 0x38(%rsp),%r9', '\tmovq 0x30(%rsp),%r8',
                '\tmovq 0x28(%rsp),%rdx', '\tmovq 0x20(%rsp),%rcx',
                '\taddq $0x88, %rsp', '\tjmpq *%r11']
        else:  # arm64: save x0-x8, q0-q7, x30 across the helper bl
            L += [
                '\tsub  sp, sp, #0xe0',
                '\tstp  x0, x1, [sp, #0x00]', '\tstp  x2, x3, [sp, #0x10]',
                '\tstp  x4, x5, [sp, #0x20]', '\tstp  x6, x7, [sp, #0x30]',
                '\tstr  x8, [sp, #0x40]',
                '\tstp  q0, q1, [sp, #0x50]', '\tstp  q2, q3, [sp, #0x70]',
                '\tstp  q4, q5, [sp, #0x90]', '\tstp  q6, q7, [sp, #0xb0]',
                '\tstr  x30, [sp, #0xd0]',
                f'\tadrp x0, {desc}', f'\tadd  x0, x0, :lo12:{desc}',
                f'\tadrp x1, {q(imp)}', f'\tadd  x1, x1, :lo12:{q(imp)}',
                f'\tbl   {helper}', '\tmov  x16, x0',
                '\tldr  x30, [sp, #0xd0]',
                '\tldp  x0, x1, [sp, #0x00]', '\tldp  x2, x3, [sp, #0x10]',
                '\tldp  x4, x5, [sp, #0x20]', '\tldp  x6, x7, [sp, #0x30]',
                '\tldr  x8, [sp, #0x40]',
                '\tldp  q0, q1, [sp, #0x50]', '\tldp  q2, q3, [sp, #0x70]',
                '\tldp  q4, q5, [sp, #0x90]', '\tldp  q6, q7, [sp, #0xb0]',
                '\tadd  sp, sp, #0xe0', '\tbr   x16']
    if datas:
        L += [f'\t/* NOTE: {len(datas)} DATA export(s) not delay-bound: {", ".join(datas)} */']
    return "\n".join(L) + "\n"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--def", dest="deff", required=True)
    ap.add_argument("--arch", required=True, choices=ARCH)
    ap.add_argument("--out", required=True, help="output .a")
    ap.add_argument("--clang", default="clang")
    ap.add_argument("--ar", default="llvm-ar")
    ap.add_argument("--keep-asm", action="store_true")
    a = ap.parse_args()
    dll, exports = parse_def(a.deff)
    asm = emit(dll, exports, a.arch)
    base = os.path.splitext(a.out)[0]
    sfile, ofile = base + ".delay.s", base + ".delay.o"
    open(sfile, "w").write(asm)
    subprocess.run([a.clang, f"--target={ARCH[a.arch]['triple']}", "-c", "-o", ofile, sfile], check=True)
    if os.path.exists(a.out):
        os.remove(a.out)
    subprocess.run([a.ar, "crs", a.out, ofile], check=True)
    if not a.keep_asm:
        os.remove(sfile)
    print(f"delaylib: {a.out}  ({dll}, {len([e for e in exports if not e[1]])} fns, {a.arch})")

if __name__ == "__main__":
    main()
