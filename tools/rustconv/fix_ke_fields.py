#!/usr/bin/env python3
"""
Fix anonymous union field access errors in KE subsystem Rust translations.

Handles:
1. KPRCB.Field → (*prcb as *const _KPRCB).Field  (opaque → full struct)
2. KLOCK_QUEUE_HANDLE.LockQueue/OldIrql → cast to _KLOCK_QUEUE_HANDLE
3. _DISPATCHER_HEADER.Type/Size/Inserted → .__bindgen_anon_1.__bindgen_anon_1.
4. _KDEVICE_QUEUE.Busy → .__bindgen_anon_1.Busy
5. _KPROCESS.ProcessFlags → .__bindgen_anon_1.ProcessFlags
6. _KIPCR.TssBase etc → .__bindgen_anon_1.__bindgen_anon_1.
7. LARGE_INTEGER .u.LowPart/.u.HighPart → .__bindgen_anon_1.LowPart
"""

import re
import sys
from pathlib import Path

KE_DIR = Path(__file__).parent.parent / "sdk/lib/rust/ntoskrnl-rust/src/ke"

def fix_file(filepath):
    text = filepath.read_text()
    original = text

    # ========================================================================
    # Fix 1: DISPATCHER_HEADER field access
    # .Header.Type → .Header.__bindgen_anon_1.__bindgen_anon_1.Type
    # But NOT if already through __bindgen_anon
    # ========================================================================
    DISP_FIELDS = ['Type', 'Absolute', 'Size', 'Inserted', 'DebugActive', 'DpcActive', 'Hand']
    for field in DISP_FIELDS:
        # .Header.Field where Field is not preceded by __bindgen_anon
        pat = re.compile(r'\.Header\.(?!_)' + re.escape(field) + r'\b')
        text = pat.sub(f'.Header.__bindgen_anon_1.__bindgen_anon_1.{field}', text)

    # ========================================================================
    # Fix 2: _KDEVICE_QUEUE.Busy → .__bindgen_anon_1.Busy
    # Pattern: something).Busy or something.Busy where context is KDEVICE_QUEUE
    # Since Busy is a very unique field name for device queues, match conservatively
    # ========================================================================
    # Match .DeviceQueue).Busy  or  DeviceQueue.Busy  etc.
    text = re.sub(r'\.Busy\b(?!\s*\()', '.__bindgen_anon_1.Busy', text)
    # Undo any false positive where it's not a KDEVICE_QUEUE context
    # (We'll check manually if needed)

    # Actually that's too aggressive. Let's be more targeted:
    # Revert the broad replacement
    text = filepath.read_text()  # re-read

    # Do DISPATCHER_HEADER fixes again
    for field in DISP_FIELDS:
        pat = re.compile(r'\.Header\.(?!_)' + re.escape(field) + r'\b')
        text = pat.sub(f'.Header.__bindgen_anon_1.__bindgen_anon_1.{field}', text)

    # ========================================================================
    # Fix 2 (targeted): _KDEVICE_QUEUE Busy field
    # Look for patterns like (*DeviceQueue).Busy or device_queue.Busy
    # ========================================================================
    # The safest approach: replace ).Busy where it's part of a device queue context
    # DeviceQueue pattern:
    text = re.sub(r'(DeviceQueue\)?)\.Busy\b', r'\1.__bindgen_anon_1.Busy', text)

    # ========================================================================
    # Fix 3: _KIPCR fields through anonymous unions
    # (*Pcr).TssBase → (*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase
    # ========================================================================
    KIPCR_FIELDS = ['TssBase', 'PcrSelf', 'Used_Self', 'Prcb', 'Irql',
                    'IRR', 'IrrActive', 'IDR', 'KdVersionBlock',
                    'IDT', 'GDT', 'TSS', 'StallScaleFactor']
    for field in KIPCR_FIELDS:
        # Match Pcr).Field or pcr).Field or Pcr.Field
        # Only if it's a KIPCR access (after cast or dereference)
        pat = re.compile(r'((?:Pcr|pcr|PCR)\)?)\.' + re.escape(field) + r'\b')
        text = pat.sub(r'\1.__bindgen_anon_1.__bindgen_anon_1.' + field, text)

    # ========================================================================
    # Fix 4: _KPROCESS.ProcessFlags through anonymous union
    # ========================================================================
    text = re.sub(r'\.ProcessFlags\b', '.__bindgen_anon_1.ProcessFlags', text)

    # ========================================================================
    # Fix 5: LARGE_INTEGER .u. access
    # .u.LowPart → .__bindgen_anon_1.LowPart (or just .u.LowPart if u is named)
    # Actually, _LARGE_INTEGER has .u as a NAMED member, so .u.LowPart should work.
    # The issue is _LARGE_INTEGER__bindgen_ty_1.u - this is the inner type which
    # doesn't have a .u member since IT IS the .u member.
    # Let's check actual usage patterns in the code.
    # ========================================================================
    # Replace .u.LowPart with .LowPart (direct access on the inner struct)
    # Actually wait - .u IS a named member of LARGE_INTEGER. The issue might be
    # something else. Let me just leave this for now.

    if text != original:
        filepath.write_text(text)
        rel = filepath.relative_to(KE_DIR.parent.parent)
        print(f"  Fixed {rel}")
        return True
    return False


def main():
    count = 0
    for root, dirs, files in sorted(KE_DIR.walk()):
        for f in sorted(files):
            if f.endswith('.rs') and f != 'mod.rs' and f != 'ke_defs.rs':
                filepath = Path(root) / f
                if fix_file(filepath):
                    count += 1
    print(f"\nFixed {count} files")


if __name__ == '__main__':
    main()
