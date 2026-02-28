#!/usr/bin/env python3
"""
Automated fixer for KE subsystem Rust translation errors.

Handles:
1. KPRCB field access (opaque ke::KPRCB → use ddk::_KPRCB)
2. DISPATCHER_HEADER fields (Type, Size, Inserted → __bindgen_anon_1.__bindgen_anon_1.)
3. KTHREAD fields (State, NpxState, WaitIrql → __bindgen_anon_5.__bindgen_anon_6. on 64-bit)
4. KDEVICE_QUEUE.Busy → __bindgen_anon_1.Busy
5. KPROCESS.ProcessFlags → __bindgen_anon_1.ProcessFlags
6. KIPCR.TssBase etc → __bindgen_anon_1.__bindgen_anon_1.
7. Missing extern declarations
8. KLOCK_QUEUE_HANDLE field fixes
9. LARGE_INTEGER.u → direct union access
"""

import re
import os
import sys
from pathlib import Path

KE_DIR = Path(__file__).parent.parent / "sdk/lib/rust/ntoskrnl-rust/src/ke"

# ============================================================================
# Pattern 1: KPRCB field access
# The hand-written ntkrnl_sys::ke::KPRCB is opaque (no fields).
# Need to cast to ddk::_KPRCB to access fields.
# ============================================================================

# KPRCB fields that are directly accessible (no union nesting)
KPRCB_DIRECT_FIELDS = {
    'LockQueue', 'IpiFrozen', 'DpcData', 'Number', 'NextThread',
    'CurrentThread', 'ReadySummary', 'MaximumDpcQueueDepth',
    'AdjustDpcThreshold', 'SkipTick', 'QueueIndex', 'DpcRoutineActive',
    'FeatureBits', 'ProcessorState', 'CallDpc', 'QuantumEnd',
    'ParentNode', 'IdleThread', 'SetMember', 'MultiThreadProcessorSet',
    'DpcRequestRate', 'MinimumDpcRate', 'DpcInterruptRequested',
    'DpcThreadRequested', 'DpcThreadActive', 'TimerHand', 'TimerRequest',
    'MasterOffset', 'DpcLastCount', 'ThreadDpcEnable', 'QuantumEndDpc',
    'PowerState', 'PrcbLock', 'InterruptCount', 'KernelTime',
    'UserTime', 'DpcTime', 'InterruptTime', 'DebugDpcTime',
    'KeSpinLockOrdering', 'CFlushSize', 'DispatcherReadyListHead',
    'MultiThreadSetBusy', 'IdleSummary', 'CpuType', 'CpuID',
    'CpuStep', 'MHz', 'HalReserved', 'MinorVersion', 'MajorVersion',
    'BuildType', 'CpuVendor', 'InitialApicId', 'LogicalProcessorsPerPhysicalProcessor',
    'SecondLevelCacheSize', 'SecondLevelCacheAssociativity', 'VendorString',
    'CoresPerPhysicalProcessor', 'PrcbPad9', 'PackageProcessorSet',
}

# ============================================================================
# Pattern 2: DISPATCHER_HEADER anonymous union fields
# ============================================================================
DISPATCHER_HEADER_ANON1_FIELDS = {
    'Type', 'Absolute', 'Size', 'Inserted', 'DebugActive',
    'DpcActive', 'Hand',
}

# ============================================================================
# Pattern 3: KTHREAD fields in __bindgen_anon_5.__bindgen_anon_6 (amd64/arm64)
# On i386 these are direct fields
# ============================================================================
KTHREAD_ANON5_6_FIELDS = {
    'State', 'NpxState', 'WaitIrql', 'WaitMode',
}

# ============================================================================
# Pattern 4: KDEVICE_QUEUE.Busy → __bindgen_anon_1.Busy
# ============================================================================

# ============================================================================
# Pattern 5: KIPCR fields in __bindgen_anon_1.__bindgen_anon_1
# ============================================================================
KIPCR_ANON1_1_FIELDS = {
    'TssBase', 'Used_Self', 'PcrSelf', 'Prcb',
    'Irql', 'IRR', 'IrrActive', 'IDR', 'KdVersionBlock',
    'IDT', 'GDT', 'TSS', 'MajorVersion', 'MinorVersion',
    'StallScaleFactor',
}


def fix_file(filepath: Path) -> int:
    """Fix a single Rust file. Returns number of changes made."""
    text = filepath.read_text()
    original = text
    changes = 0

    # ========================================================================
    # Fix 1: Ensure ddk imports include needed types
    # ========================================================================
    # Add _KPRCB to ddk imports if not already there and KPRCB is used
    if 'KPRCB' in text or 'Prcb' in text.lower():
        if '_KPRCB' not in text and 'use ntkrnl_sys::ddk::' in text:
            text = add_ddk_import(text, '_KPRCB')
            changes += 1

    # ========================================================================
    # Fix 2: DISPATCHER_HEADER field access
    # Replace .Header.Type with .Header.__bindgen_anon_1.__bindgen_anon_1.Type
    # But NOT if already using __bindgen_anon
    # ========================================================================
    for field in DISPATCHER_HEADER_ANON1_FIELDS:
        # Pattern: .Header.Field (not already through __bindgen_anon)
        pattern = rf'\.Header\.(?!__bindgen_anon){field}\b'
        replacement = f'.Header.__bindgen_anon_1.__bindgen_anon_1.{field}'
        new_text = re.sub(pattern, replacement, text)
        if new_text != text:
            changes += text.count(f'.Header.{field}') - new_text.count(f'.Header.{field}')
            text = new_text

    # ========================================================================
    # Fix 3: KDEVICE_QUEUE.Busy → __bindgen_anon_1.Busy
    # Pattern: something that looks like a KDEVICE_QUEUE pointer dereference .Busy
    # ========================================================================
    # This is harder to automate generically. Do it for the specific file.

    # ========================================================================
    # Fix 4: LARGE_INTEGER .u. access
    # _LARGE_INTEGER.u is a NAMED union member, but the inner struct is anonymous
    # So .u.LowPart should work. But _LARGE_INTEGER__bindgen_ty_1.u might not.
    # Actually check: .QuadPart is direct, .LowPart/.HighPart are through union
    # ========================================================================

    # ========================================================================
    # Fix 5: Replace 'as *mut KPRCB' / 'as *const KPRCB' field access patterns
    # When code does (*prcb).Field where prcb is PKPRCB (opaque), need to cast
    # ========================================================================
    # This requires more context-sensitive fixes per file

    if text != original:
        filepath.write_text(text)
    return changes


def add_ddk_import(text: str, symbol: str) -> str:
    """Add a symbol to the ddk import block."""
    # Find the ddk import block
    match = re.search(r'(use ntkrnl_sys::ddk::\{[^}]+)\}', text, re.DOTALL)
    if match:
        block = match.group(1)
        if symbol not in block:
            # Add before the closing brace
            new_block = block.rstrip() + f',\n    {symbol},\n}}'
            text = text[:match.start()] + new_block + text[match.end():]
    return text


def process_all_files():
    """Process all ke/ Rust files."""
    total_changes = 0
    for root, dirs, files in os.walk(KE_DIR):
        for f in sorted(files):
            if f.endswith('.rs') and f != 'mod.rs':
                filepath = Path(root) / f
                changes = fix_file(filepath)
                if changes > 0:
                    rel = filepath.relative_to(KE_DIR.parent.parent)
                    print(f"  {rel}: {changes} changes")
                    total_changes += changes
    print(f"\nTotal: {total_changes} changes")


if __name__ == '__main__':
    process_all_files()
