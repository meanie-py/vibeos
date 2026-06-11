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
# ntoskrnl bootstrap RVAs (file VA - ImageBase 0x400000)
NTOS_KBC_RVA=0xc9910            # KeBugCheckEx
NTOS_PLML_RVA=0x21af78         # PsLoadedModuleList
KBC_SIG=bytes([0x41,0x56,0x56,0x57,0x55,0x53,0x48,0x83,0xec,0x60])

inf=gdb.selected_inferior()
def rd(a,n):
    try: return inf.read_memory(a,n).tobytes()
    except gdb.MemoryError: return None

# --- load inventory: (ts,soi) -> (path, imagebase) ---
inv={}
ntos_ts=ntos_soi=None
for ln in open(INV):
    if ln.startswith("timestamp"): continue
    ts,soi,ib,path=ln.rstrip("\n").split("\t")
    inv[(int(ts,16),int(soi,16))]=(path,int(ib,16))
    if path.endswith("ntoskrnl.exe"): ntos_ts,ntos_soi=int(ts,16),int(soi,16)
print(f"[idx] inventory {len(inv)} modules; ntoskrnl ts={ntos_ts:#x} soi={ntos_soi:#x}")

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

# --- step 1: find ntoskrnl base by bounded chunked scan ---
ntos_base=None
SCAN_LO=0xfffff80000000000; SCAN_HI=0xfffff80010000000
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
    if ntos_base: break
    a+=0x100000
if not ntos_base:
    print("[idx] FAILED to find ntoskrnl base in scan range"); raise SystemExit
print(f"[idx] ntoskrnl runtime base = {ntos_base:#x}")

# --- step 2: validate no per-function shift ---
got=rd(ntos_base+NTOS_KBC_RVA,10)
print(f"[idx] KeBugCheckEx@base+RVA bytes={got.hex() if got else None} expect={KBC_SIG.hex()}  -> {'OK no-shift' if got==KBC_SIG else 'MISMATCH (shift exists!)'}")

# --- step 3: walk PsLoadedModuleList ---
def u64(a): b=rd(a,8); return struct.unpack('<Q',b)[0] if b else None
def u32(a): b=rd(a,4); return struct.unpack('<I',b)[0] if b else None
def u16(a): b=rd(a,2); return struct.unpack('<H',b)[0] if b else None
head=ntos_base+NTOS_PLML_RVA
mods=[]  # (base, size, name)
node=u64(head); cnt=0
while node and node!=head and cnt<128:
    base=u64(node+0x30); size=u32(node+0x40)
    blen=u16(node+0x58); bbuf=u64(node+0x58+8)
    name="?"
    if bbuf and blen:
        nb=rd(bbuf,min(blen,200))
        if nb: name=nb.decode("utf-16-le","replace")
    if base and size: mods.append((base,size,name))
    node=u64(node); cnt+=1
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
