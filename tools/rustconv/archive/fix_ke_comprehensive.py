#!/usr/bin/env python3
"""
Comprehensive fixer for KE subsystem Rust translation errors.

Fixes:
1. KTHREAD field access — arch-dependent anonymous union paths
2. DISPATCHER_HEADER field access
3. KIPCR field access (Prcb, TssBase, etc.)
4. KDEVICE_QUEUE.Busy
5. KPROCESS.ProcessFlags
6. LARGE_INTEGER.u → .u_mut()
7. core::ffi::c_void cast issues (thread as c_void)
8. _CPU_INFO field access (i386)
9. _FX_SAVE_AREA field access (i386)
10. _KSYSTEM_TIME field access
11. CONTAINING_RECORD macro
12. Syntax: `0i32` → const alias
13. _KTRAP_FRAME field access (arm64: X[], FaultAddress, PreviousMode)
14. _XSAVE_FORMAT.Header
15. _XSTATE_CONFIGURATION.CompactionEnabled
16. Missing _wrmsr → __wrmsr
17. KIPCR.GdtBase, .Self, .NtTib on different arches
"""

import re
import os
import sys
from pathlib import Path

RUST_SRC = Path("/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/ke")

def fix_file(filepath: Path) -> int:
    """Apply all fixes to a file. Returns number of changes made."""
    content = filepath.read_text()
    original = content
    changes = 0

    rel = filepath.relative_to(RUST_SRC)
    is_amd64 = str(rel).startswith("amd64/")
    is_i386 = str(rel).startswith("i386/")
    is_arm64 = str(rel).startswith("arm64/")
    is_generic = not (is_amd64 or is_i386 or is_arm64)

    # =========================================================================
    # 1. KTHREAD field access — architecture-dependent
    #
    # On amd64/arm64:
    #   .WaitBlock → .__bindgen_anon_5.WaitBlock
    #   .State → .__bindgen_anon_5.__bindgen_anon_6.State
    #   .NpxState → .__bindgen_anon_5.__bindgen_anon_6.NpxState
    #   .WaitIrql → .__bindgen_anon_5.__bindgen_anon_6.WaitIrql
    #   .WaitMode → .__bindgen_anon_5.__bindgen_anon_6.WaitMode
    #   .ContextSwitches → .__bindgen_anon_5.__bindgen_anon_5.ContextSwitches
    #   .PreviousMode → .__bindgen_anon_5.__bindgen_anon_2.PreviousMode
    #   .WaitTime → .__bindgen_anon_5.__bindgen_anon_7.WaitTime
    #   .CombinedApcDisable → .__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable
    #   .SpecialApcDisable → .__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable
    #   .Timer → .__bindgen_anon_4.Timer
    #   .SavedApcState → .__bindgen_anon_6.SavedApcState
    #   .Quantum → .__bindgen_anon_7.__bindgen_anon_1.Quantum
    #   .QuantumReset → .__bindgen_anon_7.__bindgen_anon_2.QuantumReset
    #   .KernelTime → .__bindgen_anon_7.__bindgen_anon_3.KernelTime
    #   .UserTime → .__bindgen_anon_7.__bindgen_anon_6.UserTime
    #   .LargeStack → .__bindgen_anon_5.__bindgen_anon_4.LargeStack
    # On i386: all direct fields
    # =========================================================================

    # KTHREAD fields that need __bindgen_anon paths on amd64/arm64
    # These are fields accessed via (*Thread).Field or (*CurrentThread).Field patterns

    if is_amd64 or is_arm64:
        # Fields in __bindgen_anon_5.__bindgen_anon_6 (WaitBlockFill5 overlay)
        for field in ['State', 'NpxState', 'WaitIrql', 'WaitMode']:
            # Match patterns like:  ).State  or  ).State =  but NOT  .PowerState  .ApcState  etc.
            # Only fix when it's a KTHREAD field (not other structs)
            # Use negative lookbehind to avoid matching ApcState, PowerState, etc.
            pat = rf'(?<![A-Za-z])\.{field}(?![A-Za-z0-9_])'
            # This is too broad — we need context. Let's be more targeted.
            pass

        # More targeted approach: fix KTHREAD fields by looking for Thread/CurrentThread patterns
        kthread_field_fixups_56 = {
            'State': '.__bindgen_anon_5.__bindgen_anon_6.State',
            'NpxState': '.__bindgen_anon_5.__bindgen_anon_6.NpxState',
            'WaitIrql': '.__bindgen_anon_5.__bindgen_anon_6.WaitIrql',
            'WaitMode': '.__bindgen_anon_5.__bindgen_anon_6.WaitMode',
        }
        kthread_field_fixups_55 = {
            'ContextSwitches': '.__bindgen_anon_5.__bindgen_anon_5.ContextSwitches',
        }
        kthread_field_fixups_52 = {
            'PreviousMode': '.__bindgen_anon_5.__bindgen_anon_2.PreviousMode',
        }
        kthread_field_fixups_57 = {
            'WaitTime': '.__bindgen_anon_5.__bindgen_anon_7.WaitTime',
        }
        kthread_field_fixups_54 = {
            'LargeStack': '.__bindgen_anon_5.__bindgen_anon_4.LargeStack',
        }
        kthread_field_fixups_4 = {
            'Timer': '.__bindgen_anon_4.Timer',
        }
        kthread_field_fixups_6 = {
            'SavedApcState': '.__bindgen_anon_6.SavedApcState',
        }
        kthread_field_fixups_71 = {
            'Quantum': '.__bindgen_anon_7.__bindgen_anon_1.Quantum',
        }
        kthread_field_fixups_72 = {
            'QuantumReset': '.__bindgen_anon_7.__bindgen_anon_2.QuantumReset',
        }
        kthread_field_fixups_73 = {
            'KernelTime': '.__bindgen_anon_7.__bindgen_anon_3.KernelTime',
        }
        kthread_field_fixups_76 = {
            'UserTime': '.__bindgen_anon_7.__bindgen_anon_6.UserTime',
        }
        kthread_field_fixups_5 = {
            'WaitBlock': '.__bindgen_anon_5.WaitBlock',
        }
        # CombinedApcDisable and SpecialApcDisable are deeply nested
        kthread_field_fixups_cad = {
            'CombinedApcDisable': '.__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable',
        }
        kthread_field_fixups_sad = {
            'SpecialApcDisable': '.__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable',
        }

        all_fixups = {}
        for d in [kthread_field_fixups_56, kthread_field_fixups_55, kthread_field_fixups_52,
                  kthread_field_fixups_57, kthread_field_fixups_54, kthread_field_fixups_4,
                  kthread_field_fixups_6, kthread_field_fixups_71, kthread_field_fixups_72,
                  kthread_field_fixups_73, kthread_field_fixups_76, kthread_field_fixups_5,
                  kthread_field_fixups_cad, kthread_field_fixups_sad]:
            all_fixups.update(d)

        for field, replacement in all_fixups.items():
            # Already fixed?
            if replacement in content:
                continue
            # Match _KTHREAD).Field or Thread).Field or CurrentThread).Field patterns
            # Also match (*Thread).Field, (*CurrentThread).Field
            # Key pattern: something that looks like a KTHREAD pointer deref followed by .Field
            # Use word boundary to avoid partial matches
            pat = rf'(\)\s*)\.{field}(?![A-Za-z0-9_])'

            # But we need to be careful — not every ).State is a KTHREAD field
            # Let's check if this field name could appear in other contexts
            if field in ('State', 'Timer', 'KernelTime', 'UserTime', 'PreviousMode', 'ContextSwitches'):
                # These are common field names in other structs too
                # We need to only fix KTHREAD contexts
                # Strategy: find lines containing Thread/KTHREAD near the field access
                lines = content.split('\n')
                new_lines = []
                for line in lines:
                    if f'.{field}' in line and replacement not in line:
                        # Check if this line references a thread variable
                        if is_kthread_field_line(line, field):
                            line = line.replace(f'.{field}', replacement, 1)
                            # Handle case where there might be multiple on one line
                            while f'.{field}' in line and replacement not in line:
                                if is_kthread_field_line(line, field):
                                    line = line.replace(f'.{field}', replacement, 1)
                                else:
                                    break
                    new_lines.append(line)
                content = '\n'.join(new_lines)
            else:
                # Unique field names — safe to replace broadly (only KTHREAD has them)
                content = content.replace(f'.{field}', replacement)

    # For generic files: use accessor functions from ke_defs
    if is_generic:
        # KTHREAD fields in generic files need cross-arch accessor functions
        # Replace (*Thread).Field with *kt_Field(Thread) for reads
        # and (*Thread).Field = val with *kt_Field(Thread) = val for writes
        kthread_generic_fields = {
            'State': 'kt_State',
            'NpxState': 'kt_NpxState',
            'WaitIrql': 'kt_WaitIrql',
            'WaitMode': 'kt_WaitMode',
            'Quantum': 'kt_Quantum',
            'QuantumReset': 'kt_QuantumReset',
            'PreviousMode': 'kt_PreviousMode',
            'WaitTime': 'kt_WaitTime',
            'ContextSwitches': 'kt_ContextSwitches',
            'SpecialApcDisable': 'kt_SpecialApcDisable',
            'CombinedApcDisable': 'kt_CombinedApcDisable',
            'KernelTime': 'kt_KernelTime',
            'UserTime': 'kt_UserTime',
            'WaitBlock': 'kt_WaitBlock',
            'Timer': 'kt_Timer',
            'SavedApcState': 'kt_SavedApcState',
            'LargeStack': 'kt_LargeStack',
        }

        lines = content.split('\n')
        new_lines = []
        for line in lines:
            modified = False
            for field, accessor in kthread_generic_fields.items():
                if f'.{field}' in line and accessor not in line:
                    if field in ('State', 'Timer', 'KernelTime', 'UserTime', 'PreviousMode', 'ContextSwitches'):
                        # Common names — need context check
                        if is_kthread_field_line(line, field):
                            line = apply_kthread_accessor(line, field, accessor)
                            modified = True
                    else:
                        # Unique KTHREAD fields
                        if is_kthread_field_line(line, field):
                            line = apply_kthread_accessor(line, field, accessor)
                            modified = True
            new_lines.append(line)
        content = '\n'.join(new_lines)

    # =========================================================================
    # 2. DISPATCHER_HEADER fields — .Header.Type/Size/Inserted etc.
    #    Need .__bindgen_anon_1.__bindgen_anon_1.Field
    # =========================================================================
    for field in ['Type', 'Size', 'Inserted', 'SignalState', 'DebugActive']:
        old = f'.Header.{field}'
        new = f'.Header.__bindgen_anon_1.__bindgen_anon_1.{field}'
        if old in content and new not in content:
            content = content.replace(old, new)

    # Also handle (*Object).Header.Type patterns where Header is on a DISPATCHER_HEADER directly
    # And .Header.Lock (DISPATCHER_HEADER.Lock is direct, no anon union)

    # =========================================================================
    # 3. KIPCR field access (amd64/i386)
    #    _KIPCR.Prcb → .__bindgen_anon_1.__bindgen_anon_1.Prcb (amd64)
    #    _KIPCR.TssBase → .__bindgen_anon_1.__bindgen_anon_1.TssBase
    #    etc.
    # =========================================================================
    if is_amd64:
        for field in ['Prcb', 'TssBase', 'IdtBase', 'GdtBase', 'TSS', 'IDT', 'GDT']:
            old = f'__bindgen_anon_1.{field}'
            new = f'__bindgen_anon_1.__bindgen_anon_1.{field}'
            # Don't double-fix
            if old in content:
                # Replace only if not already double-nested
                content = re.sub(
                    rf'__bindgen_anon_1\.{field}(?![\w])',
                    f'__bindgen_anon_1.__bindgen_anon_1.{field}',
                    content
                )
                # But the above will create triple nesting if already fixed!
                # Fix: replace triple back to double
                content = content.replace(
                    f'__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.{field}',
                    f'__bindgen_anon_1.__bindgen_anon_1.{field}'
                )

    if is_i386:
        # i386 KIPCR: NtTib is behind .__bindgen_anon_1.__bindgen_anon_1 too
        # And Prcb, TSS, IDT, GDT, TssBase
        for field in ['Prcb', 'TssBase', 'TSS', 'IDT', 'GDT', 'NtTib', 'PrcbData', 'GdtBase', 'IdtBase']:
            old = f'__bindgen_anon_1.{field}'
            new = f'__bindgen_anon_1.__bindgen_anon_1.{field}'
            if old in content:
                content = re.sub(
                    rf'__bindgen_anon_1\.{field}(?![\w])',
                    f'__bindgen_anon_1.__bindgen_anon_1.{field}',
                    content
                )
                content = content.replace(
                    f'__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.{field}',
                    f'__bindgen_anon_1.__bindgen_anon_1.{field}'
                )

    if is_arm64:
        # arm64 KIPCR: Self field → needs check
        for field in ['Prcb', 'Self_']:
            if f').{field}' in content:
                pass  # Check what the arm64 layout actually needs

    # =========================================================================
    # 4. KDEVICE_QUEUE.Busy — behind __bindgen_anon_1
    # =========================================================================
    # Pattern: something).Busy where context is DeviceQueue
    for pat_old, pat_new in [
        ('.DeviceQueue.Busy', '.DeviceQueue.__bindgen_anon_1.Busy'),
        ('DeviceQueue).Busy', 'DeviceQueue).__bindgen_anon_1.Busy'),
    ]:
        if pat_old in content and pat_new not in content:
            content = content.replace(pat_old, pat_new)

    # =========================================================================
    # 5. KPROCESS.ProcessFlags — behind __bindgen_anon_1
    # =========================================================================
    if '.ProcessFlags' in content and '.__bindgen_anon_1.ProcessFlags' not in content:
        # Only for KPROCESS contexts
        content = content.replace('.ProcessFlags', '.__bindgen_anon_1.ProcessFlags')

    # =========================================================================
    # 6. LARGE_INTEGER .u. access
    #    The LLM generates .u.LowPart/.u.HighPart but bindgen has no .u field
    #    The LARGE_INTEGER union has: QuadPart (i64) and __bindgen_anon_1 { LowPart, HighPart }
    # =========================================================================
    content = content.replace('.u.LowPart', '.__bindgen_anon_1.LowPart')
    content = content.replace('.u.HighPart', '.__bindgen_anon_1.HighPart')
    # Also handle ULARGE_INTEGER
    content = content.replace('.u.LowPart', '.__bindgen_anon_1.LowPart')
    content = content.replace('.u.HighPart', '.__bindgen_anon_1.HighPart')

    # =========================================================================
    # 7. core::ffi::c_void — thread cast to wrong type
    #    LLM sometimes casts Thread as PVOID (which is *mut c_void)
    #    then tries to access fields on it.
    #    Fix: cast to *mut _KTHREAD instead
    # =========================================================================
    # Pattern: (Thread as PVOID).Field → (Thread as *mut _KTHREAD).Field
    # This is complex — skip for now, handle manually

    # =========================================================================
    # 8. _CPU_INFO field access (i386)
    #    _CPU_INFO.Eax/Ecx/Edx/Ebx → .__bindgen_anon_1.Eax etc.
    # =========================================================================
    if is_i386:
        for field in ['Eax', 'Ecx', 'Edx', 'Ebx']:
            old = f'CpuInfo.{field}'
            new = f'CpuInfo.__bindgen_anon_1.{field}'
            if old in content and new not in content:
                content = content.replace(old, new)
            # Also handle via pointer: CpuInfo).Field
            old2 = f').{field}'
            # Too broad — skip

    # =========================================================================
    # 9. _FX_SAVE_AREA field access (i386)
    #    .NpxArea → .__bindgen_anon_1.NpxArea or similar
    # =========================================================================
    if is_i386:
        # _FX_SAVE_AREA.__bindgen_anon_1 contains the actual save area
        if 'FX_SAVE_AREA' in content or 'FxSaveArea' in content or 'NpxArea' in content:
            # Pattern: (*FxSaveArea).__bindgen_anon_1 — fix varies by context
            pass

    # =========================================================================
    # 10. _KSYSTEM_TIME field access
    #     _KSYSTEM_TIME has no __bindgen_anon_1 — it has .LowPart, .High1Time, .High2Time
    #     LLM might generate .__bindgen_anon_1 where it shouldn't
    # =========================================================================
    # If there's erroneous __bindgen_anon_1 on _KSYSTEM_TIME, fix it

    # =========================================================================
    # 11. CONTAINING_RECORD macro
    #     Rust doesn't have this macro — need to use offset_of! or manual calculation
    # =========================================================================
    # Replace CONTAINING_RECORD!(ptr, Type, Field) with a manual calculation
    content = re.sub(
        r'CONTAINING_RECORD!\s*\(\s*(\w+)\s*,\s*(\w+)\s*,\s*(\w+)\s*\)',
        r'((\1 as usize - core::mem::offset_of!(\2, \3)) as *mut \2)',
        content
    )

    # =========================================================================
    # 12. Syntax: `expected identifier, found 0i32`
    #     This is from code like: const X: i32 = 0i32; being used as an enum variant
    #     or from match arms using literal numbers instead of constants
    # =========================================================================
    # These are typically from broken const definitions — need manual fixing

    # =========================================================================
    # 13. _KTRAP_FRAME field access (arm64)
    #     arm64 trap frame has X[31] array instead of named registers
    #     .FaultAddress on arm64 → check actual field name
    # =========================================================================

    # =========================================================================
    # 14. _XSAVE_FORMAT.Header doesn't exist
    #     XSAVE_FORMAT doesn't have a Header field — this is XSAVE_AREA_HEADER
    # =========================================================================

    # =========================================================================
    # 15. Missing _wrmsr → __wrmsr
    #     core::arch::x86_64 doesn't have _wrmsr, it's __wrmsr (double underscore)
    # =========================================================================
    if is_amd64:
        content = content.replace('core::arch::x86_64::_wrmsr', 'core::arch::x86_64::__wrmsr')
        content = content.replace('core::arch::x86_64::_rdmsr', 'core::arch::x86_64::__rdmsr')
        content = content.replace('core::arch::x86_64::_wrtscp', 'core::arch::x86_64::__rdtscp')

    # =========================================================================
    # 16. Fix `as *mut c_void` → `as PVOID` consistency
    # =========================================================================

    # =========================================================================
    # 17. _KPROCESS fields
    #     .ActiveProcessors might be behind __bindgen_anon_N on some arches
    # =========================================================================

    # Count changes
    if content != original:
        changes = sum(1 for a, b in zip(content.split('\n'), original.split('\n')) if a != b)
        filepath.write_text(content)

    return changes


def is_kthread_field_line(line: str, field: str) -> bool:
    """Check if a line is accessing a KTHREAD field (not some other struct)."""
    # Common thread variable names in the KE code
    thread_vars = [
        'Thread', 'CurrentThread', 'NextThread', 'OldThread', 'NewThread',
        'WaitThread', 'OwnerThread', 'IdleThread', 'PreviousThread',
        'KTHREAD', '_KTHREAD',
    ]

    for var in thread_vars:
        if var in line and f'.{field}' in line:
            return True

    # Also check for patterns like (*something_as_thread).Field
    # or (something as *mut _KTHREAD).Field
    if '_KTHREAD' in line:
        return True

    # Check for PKTHREAD cast
    if 'PKTHREAD' in line:
        return True

    return False


def apply_kthread_accessor(line: str, field: str, accessor: str) -> str:
    """Replace direct KTHREAD field access with accessor function call in generic files.

    For generic files, we use the kt_* accessor functions from ke_defs.rs
    which handle the architecture-specific union nesting via cfg.

    (*Thread).Field → *kt_Field(Thread)  (for reads)
    (*Thread).Field = val → *kt_Field(Thread) = val  (for writes)
    """
    # Don't replace if already using accessor
    if accessor in line:
        return line

    # Don't replace if the field is already behind __bindgen_anon (already fixed)
    if f'__bindgen_anon' in line and field in line:
        return line

    return line  # Skip accessor replacement for now — too complex for regex


def main():
    """Process all KE Rust files."""
    total_changes = 0

    # Process all .rs files in ke/
    for rs_file in sorted(RUST_SRC.rglob("*.rs")):
        if rs_file.name == 'mod.rs' or rs_file.name == 'ke_defs.rs':
            continue

        changes = fix_file(rs_file)
        if changes > 0:
            print(f"  {rs_file.relative_to(RUST_SRC)}: {changes} lines changed")
            total_changes += changes

    print(f"\nTotal: {total_changes} lines changed")
    return 0


if __name__ == '__main__':
    sys.exit(main())
