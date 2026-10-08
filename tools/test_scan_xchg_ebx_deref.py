"""scan_xchg_ebx_deref: i386 code that swaps ebx out, then dereferences ebx."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_xchg_ebx_deref as scanner

# The shipped bug, verbatim from ntdll.dll's _RtlInterlockedPushEntrySList@8.
BROKEN = """
7e0803d0 <_RtlInterlockedPushEntrySList@8>:
7e0803ff:      	movl	%esi, %eax
7e080401:      	xchgl	%ebx, %edi
7e080403:      	lock
7e080404:      	cmpxchg8b	(%ebx)
7e080407:      	xchgl	%ebx, %edi
7e080409:      	sete	%al
"""

# The fix: addr pinned to edi, ebx swap is around a deref of edi.
FIXED = BROKEN.replace("cmpxchg8b	(%ebx)", "cmpxchg8b	(%edi)").replace(
    "xchgl	%ebx, %edi", "xchgl	%ebx, %ecx"
)

# CPUID's ebx save: swap, cpuid, swap back -- no memory operand at all.
CPUID = """
80001000 <HalpCpuid>:
80001000:      	xchgl	%esi, %ebx
80001002:      	cpuid
80001004:      	xchgl	%esi, %ebx
80001006:      	movl	%ebx, (%edi)
"""


def test_the_shipped_slist_pattern_is_found():
    assert scanner.findings(BROKEN) == [("_RtlInterlockedPushEntrySList@8", "7e080404")]


def test_the_fixed_pattern_and_cpuid_saves_are_clean():
    assert scanner.findings(FIXED) == []
    # The deref of ebx AFTER the swap-back is ordinary code, not a finding.
    assert scanner.findings(CPUID.replace("(%edi)", "(%ebx)")) == []


def test_jump_table_data_decoded_as_xchg_eax_is_not_a_finding():
    # user32's EditWndProc jump table: 0x93 0x01 0x00 ... decodes as this.
    data = """
77a52098 <_EditWndProc_common@20>:
77a520a4:      	xchgl	%ebx, %eax
77a520a5:      	addl	%eax, (%eax)
77a520a7:      	addb	%dl, -0x6cffffff(%ebx)
"""
    assert scanner.findings(data) == []


def test_non_i386_refuses_instead_of_clean(tmp_path, monkeypatch, capsys):
    d = tmp_path / "build-amd64"
    d.mkdir()
    monkeypatch.setattr(sys, "argv", ["scan_xchg_ebx_deref", str(d), "--check"])
    assert scanner.main() == 2
    assert "not an i386 build" in capsys.readouterr().err
