"""Runtime kernel symbolicator for the Rust-cohabitation port.

Run inside the toolkit's gdb (via inspect.sh <run> --command raw "source .../runtime_symbolicate.py").
Solves "runtime addr -> module!symbol+off" without trusting find-kbase's guessed base:

  1. Bounded-scan kernel VA for ntoskrnl's PE (matched by TimeDateStamp+SizeOfImage from the
     build artifact, via the offline pe_inventory.tsv) -> CORRECT ntoskrnl runtime base.
  2. Validate KeBugCheckEx prologue at base+RVA  (proves sections load at file RVAs; no shift).
  3. Walk PsLoadedModuleList for the authoritative {base,size,name} of EVERY loaded module.
  4. For each target addr: find module, file_RVA = addr-base, nearest nm symbol -> name+off.

Env: SYMIDX_INV (inventory tsv), SYMIDX_TARGETS (comma hex), SYMIDX_BUILD (build-<arch> dir),
     SYMIDX_NM (llvm-nm path).
"""
import gdb, struct, os, subprocess, bisect, glob

INV=os.environ.get("SYMIDX_INV","/tmp/pe_inv_amd64.tsv")
BUILD=os.environ.get("SYMIDX_BUILD","build-amd64")
NM=os.environ.get("SYMIDX_NM",".build-tools/llvm-mingw/bin/llvm-nm")
TARGETS=[int(x,16) for x in os.environ.get("SYMIDX_TARGETS","").split(",") if x.strip()]
# Arch: i386 (PE32, 32-bit ptrs) vs amd64/arm64 (PE32+, 64-bit ptrs). Auto from
# SYMIDX_BUILD ("build-i386"/"build-amd64"/"build-arm64") unless SYMIDX_ARCH set.
ARCH=os.environ.get("SYMIDX_ARCH") or ("i386" if "i386" in BUILD else "arm64" if "arm64" in BUILD else "amd64")
if ARCH=="i386":
    PTRSZ=4
    LDR_DLLBASE=0x18; LDR_SOI=0x20; LDR_NAME_LEN=0x2c; LDR_NAME_BUF=0x30
    SCAN_LO=0x80000000; SCAN_HI=0x90000000  # i386 kernel VA; base shifts with image size
    KBC_SIG=bytes([0x55,0x89,0xe5,0x53,0x57,0x56,0x83,0xe4,0xf8])  # push ebp;mov ebp,esp;push ebx,edi,esi;and esp,-8
    KBC_SYM="_KeBugCheckEx@20"; PLML_SYM="_PsLoadedModuleList"
else:  # amd64 / arm64 (PE32+, 64-bit)
    PTRSZ=8
    LDR_DLLBASE=0x30; LDR_SOI=0x40; LDR_NAME_LEN=0x58; LDR_NAME_BUF=0x60
    SCAN_LO=0xfffff80000000000; SCAN_HI=0xfffff80010000000
    KBC_SIG=bytes([0x41,0x56,0x56,0x57,0x55,0x53,0x48,0x83,0xec,0x60])
    KBC_SYM="KeBugCheckEx"; PLML_SYM="PsLoadedModuleList"
# RVAs are derived from nm of the build ntoskrnl below (they shift per build).
NTOS_KBC_RVA=None; NTOS_PLML_RVA=None

inf=gdb.selected_inferior()
def rd(a,n):
    try: return inf.read_memory(a,n).tobytes()
    except gdb.MemoryError: return None

# --- load inventory: (ts,soi) -> (path, imagebase) ---
inv={}
ntos_ts=ntos_soi=ntos_path=ntos_ib=None
for ln in open(INV):
    if ln.startswith("timestamp"): continue
    ts,soi,ib,path=ln.rstrip("\n").split("\t")
    inv[(int(ts,16),int(soi,16))]=(path,int(ib,16))
    if path.endswith("ntoskrnl.exe"):
        ntos_ts,ntos_soi=int(ts,16),int(soi,16); ntos_path=path; ntos_ib=int(ib,16)
print(f"[idx] arch={ARCH} ptr={PTRSZ}B; inventory {len(inv)} modules; ntoskrnl ts={ntos_ts:#x} soi={ntos_soi:#x}")

# --- derive ntoskrnl bootstrap RVAs from nm (they shift per build; don't hardcode) ---
def _nm_rva(sym):
    try: out=subprocess.run([NM,ntos_path],capture_output=True,text=True).stdout
    except Exception: return None
    for l in out.splitlines():
        p=l.split()
        if len(p)>=3 and p[2]==sym:
            try: return int(p[0],16)-ntos_ib
            except: return None
    return None
NTOS_KBC_RVA=_nm_rva(KBC_SYM); NTOS_PLML_RVA=_nm_rva(PLML_SYM)
if NTOS_KBC_RVA is None or NTOS_PLML_RVA is None:
    print(f"[idx] FAILED nm RVA derivation (kbc={NTOS_KBC_RVA} plml={NTOS_PLML_RVA}) for {ntos_path}"); raise SystemExit
print(f"[idx] {KBC_SYM} rva={NTOS_KBC_RVA:#x}  {PLML_SYM} rva={NTOS_PLML_RVA:#x}")

def pe_at(a):
    h=rd(a,2)
    if h!=b'MZ': return None
    lf=rd(a+0x3c,4)
    if not lf: return None
    off=struct.unpack('<I',lf)[0]
    if off>0x400: return None
    if rd(a+off,4)!=b'PE\x00\x00': return None
    ts=struct.unpack('<I',rd(a+off+8,4))[0]
    soi=struct.unpack('<I',rd(a+off+24+56,4))[0]
    return ts,soi

# --- step 1: find ntoskrnl base by bounded chunked scan (SCAN_LO/HI set per-arch above) ---
# Read 1MB chunks for speed; on i386 kernel mappings have gaps so a 1MB read can
# throw (whole window skipped) — fall back to page-granular reads for that window
# (PE images are page-aligned, so checking each page start suffices).
ntos_base=None
a=SCAN_LO
while a<SCAN_HI:
    chunk=rd(a,0x100000)
    if chunk:
        i=0
        while True:
            j=chunk.find(b'MZ', i)
            if j<0: break
            if (a+j)&0xfff==0:
                info=pe_at(a+j)
                if info and info==(ntos_ts,ntos_soi):
                    ntos_base=a+j; break
            i=j+2
    else:
        # 1MB read failed (unmapped page in window) — probe each page start
        p=a
        while p<a+0x100000:
            if rd(p,2)==b'MZ':
                info=pe_at(p)
                if info and info==(ntos_ts,ntos_soi):
                    ntos_base=p; break
            p+=0x1000
    if ntos_base: break
    a+=0x100000
if not ntos_base:
    print("[idx] FAILED to find ntoskrnl base in scan range"); raise SystemExit
print(f"[idx] ntoskrnl runtime base = {ntos_base:#x}")

# --- step 2: validate no per-function shift ---
got=rd(ntos_base+NTOS_KBC_RVA,len(KBC_SIG))
print(f"[idx] {KBC_SYM}@base+RVA bytes={got.hex() if got else None} expect={KBC_SIG.hex()}  -> {'OK no-shift' if got==KBC_SIG else 'MISMATCH (shift exists!)'}")

# --- step 3: walk PsLoadedModuleList (offsets/ptr-width set per-arch above) ---
def u64(a): b=rd(a,8); return struct.unpack('<Q',b)[0] if b else None
def u32(a): b=rd(a,4); return struct.unpack('<I',b)[0] if b else None
def u16(a): b=rd(a,2); return struct.unpack('<H',b)[0] if b else None
def uptr(a): return u32(a) if PTRSZ==4 else u64(a)
head=ntos_base+NTOS_PLML_RVA
mods=[]  # (base, size, name)
node=uptr(head); cnt=0
while node and node!=head and cnt<128:
    base=uptr(node+LDR_DLLBASE); size=u32(node+LDR_SOI)
    blen=u16(node+LDR_NAME_LEN); bbuf=uptr(node+LDR_NAME_BUF)
    name="?"
    if bbuf and blen:
        nb=rd(bbuf,min(blen,200))
        if nb: name=nb.decode("utf-16-le","replace")
    if base and size: mods.append((base,size,name))
    node=uptr(node); cnt+=1
mods.sort()
print(f"[idx] walked {len(mods)} loaded modules")
if os.environ.get("SYMIDX_DUMP"):
    for b,s,n in sorted(mods):
        mark=""
        for pc in TARGETS:
            if b<=pc<b+s: mark=f"  <<< covers {pc:#x}"
        print(f"[mod] {b:#x}..{b+s:#x}  {n}{mark}")

# build name->file map (basename match against inventory paths)
inv_by_base={}
for (ts,soi),(path,ib) in inv.items():
    inv_by_base[os.path.basename(path).lower()]=path
def file_for(name):
    return inv_by_base.get(name.lower())

# --- nm cache ---
nmcache={}
def syms_for(path):
    if path in nmcache: return nmcache[path]
    try:
        out=subprocess.run([NM,"--numeric-sort",path],capture_output=True,text=True).stdout
    except Exception as e:
        nmcache[path]=([],[]); return nmcache[path]
    addrs=[]; names=[]
    for l in out.splitlines():
        p=l.split()
        if len(p)>=3 and p[1] in "TtWwRrDd":
            try: v=int(p[0],16)
            except: continue
            addrs.append(v); names.append(p[2])
    pairs=sorted(zip(addrs,names))
    A=[x for x,_ in pairs]; N=[y for _,y in pairs]
    nmcache[path]=(A,N); return nmcache[path]

inv_by_ts={}
for (ts,soi),(path,ib) in inv.items(): inv_by_ts.setdefault(ts,[]).append((soi,path,ib))

def scan_session_module(addr):
    """Session drivers (win32k etc.) aren't in PsLoadedModuleList. Scan pages at/below
    addr for a PE whose [base,base+soi) covers addr; identify via inventory ts."""
    a=addr & ~0xfff
    lo=a-0x600000
    while a>=lo:
        info=pe_at(a)
        if info:
            ts,soi=info
            if a<=addr<a+soi and ts in inv_by_ts:
                for s2,path,ib in inv_by_ts[ts]:
                    if s2==soi:
                        nm=os.path.basename(path); mods.append((a,soi,nm)); return (a,soi,nm)
        a-=0x1000
    return None

def module_of(addr):
    for base,size,name in mods:
        if base<=addr<base+size: return (base,size,name)
    return scan_session_module(addr)

# report win32k base for cross-boot determinism check
for b,s,n in mods:
    if n.lower().startswith("win32k"): print(f"[idx] win32k.sys in PsLoadedModuleList @ {b:#x}")

# --- step 4: symbolicate targets ---
for pc in TARGETS:
    m=module_of(pc)
    if not m:
        print(f"  {pc:#x}: NOT in any loaded module"); continue
    base,size,name=m
    path=file_for(name)
    rva=pc-base
    if not path:
        print(f"  {pc:#x}: {name}+{rva:#x}  (no build file for nm)"); continue
    # nm VAs are file-VA (RVA+imagebase). get imagebase from inventory
    ib=None
    for (ts,soi),(p2,b2) in inv.items():
        if os.path.basename(p2).lower()==name.lower(): ib=b2; break
    A,N=syms_for(path)
    fileva=rva+(ib or 0)
    k=bisect.bisect_right(A,fileva)-1
    if k>=0:
        print(f"  {pc:#x}: {name}!{N[k]}+{fileva-A[k]:#x}   (base {base:#x}, rva {rva:#x})")
    else:
        print(f"  {pc:#x}: {name}+{rva:#x} (no sym <= )")
print("[idx] done")
