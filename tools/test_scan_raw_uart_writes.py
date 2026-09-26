"""Tests for scan_raw_uart_writes — a debug byte stored straight into the PL011 data register.

The fire-cases are the shapes the 87 real stores took in the tree when this lint landed
(`*u = b as ULONG`, `write_volatile(uart, ..)`, a deref of the base itself). The must-not
cases pin the thing that makes a name-based scan trustworthy: a store is only a finding
if the NEAREST preceding `let` of that name took its value from the UART base, so an
unrelated `u` in another function of the same file is left alone.
"""

from __future__ import annotations

import sys
import textwrap
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_raw_uart_writes import scan_text


def hits(src: str) -> list[int]:
    return [line for line, _ in scan_text(textwrap.dedent(src))]


# ── fires ────────────────────────────────────────────────────────────────────────────


def test_deref_store_through_a_bound_pointer():
    assert hits(
        """
        unsafe fn t(msg: &[u8]) {
            let u = Arm64UartKernelBase as *mut ULONG;
            for &b in msg {
                *u = b as ULONG;
            }
        }
        """
    ) == [5]


def test_write_volatile_through_a_bound_pointer():
    assert hits(
        """
        let uart = core::ptr::read_volatile(
            core::ptr::addr_of!(Arm64UartKernelBase)) as *mut u32;
        core::ptr::write_volatile(uart, c as u32);
        """
    ) == [4]


def test_store_through_the_base_itself():
    assert hits("*(Arm64UartKernelBase as *mut u32) = 0x41;\n") == [1]
    assert hits("write_volatile(Arm64UartKernelBase as *mut u32, 0x41);\n") == [1]


# ── must not fire ────────────────────────────────────────────────────────────────────


def test_the_sanctioned_primitive():
    assert (
        hits(
            """
        let u = Arm64UartKernelBase as *mut ULONG;
        if !u.is_null() { Arm64UartPutChar(b as u8); }
        """
        )
        == []
    )


def test_an_unrelated_binding_of_the_same_name():
    # `u` is rebound to something else before the store: the nearest binding wins.
    assert (
        hits(
            """
        let u = Arm64UartKernelBase as *mut ULONG;
        Arm64UartPutChar(b'x');
        let u = buffer.as_mut_ptr();
        *u = 7;
        """
        )
        == []
    )


def test_comments_and_strings_are_not_code():
    assert (
        hits(
            """
        let u = Arm64UartKernelBase as *mut ULONG;
        // *u = c as ULONG;   (the C this replaced)
        let s = "*u = 1;";
        """
        )
        == []
    )


def test_a_comparison_is_not_a_store():
    assert (
        hits(
            """
        let u = Arm64UartKernelBase as *mut ULONG;
        if *u == 0 { return; }
        """
        )
        == []
    )
