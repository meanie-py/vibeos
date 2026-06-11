#!/usr/bin/env python3
"""Offline PE inventory: for each build PE, extract (TimeDateStamp, SizeOfImage,
ImageBase, path). Used to match runtime-loaded modules to their build artifacts
for the runtime symbol index. Usage: pe_inventory.py build-amd64 > inv.tsv"""
import sys, os, struct, glob
def pe_info(path):
    with open(path,"rb") as f: d=f.read(0x400)
    if d[:2]!=b'MZ': return None
    lf=struct.unpack_from("<I", d, 0x3c)[0]
    if d[lf:lf+4]!=b'PE\x00\x00': return None
    ts=struct.unpack_from("<I", d, lf+8)[0]           # COFF TimeDateStamp
    opt=lf+24
    magic=struct.unpack_from("<H", d, opt)[0]
    if magic==0x20b:  # PE32+
        imgbase=struct.unpack_from("<Q", d, opt+24)[0]
    else:
        imgbase=struct.unpack_from("<I", d, opt+28)[0]
    soi=struct.unpack_from("<I", d, opt+56)[0]         # SizeOfImage
    return ts, soi, imgbase
root=sys.argv[1]
seen=set()
print("timestamp\tsizeofimage\timagebase\tpath")
for ext in ("*.exe","*.sys","*.dll"):
    for p in glob.glob(os.path.join(root,"**",ext), recursive=True):
        try: info=pe_info(p)
        except Exception: info=None
        if not info: continue
        ts,soi,ib=info
        key=(ts,soi)
        if key in seen: continue
        seen.add(key)
        print(f"{ts:#010x}\t{soi:#x}\t{ib:#x}\t{p}")
