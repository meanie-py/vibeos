use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{INT_PTR, PULONGLONG};
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;

use core::ffi::c_void;

type RPN_BINARY_OPERATOR = Option<unsafe extern "C" fn(ULONGLONG, ULONGLONG) -> ULONGLONG>;

#[repr(C)]
#[derive(Copy, Clone)]
enum RPN_OP_TYPE {
    RpnOpNop,
    RpnOpBinaryOperator,
    RpnOpUnaryOperator,
    RpnOpImmediate,
    RpnOpRegister,
    RpnOpDereference,
}

#[repr(C)]
#[derive(Copy, Clone)]
union RPN_OP_DATA {
    BinaryOperator: RPN_BINARY_OPERATOR,
    Immediate: ULONGLONG,
    Register: UCHAR,
    DerefMemorySize: UCHAR,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct RPN_OP {
    Type: RPN_OP_TYPE,
    CharacterOffset: ULONG,
    Data: RPN_OP_DATA,
}

type PRPN_OP = *mut RPN_OP;

#[repr(C)]
struct RPN_STACK {
    Size: ULONG,
    Sp: ULONG,
    Ops: [RPN_OP; 1],
}

type PRPN_STACK = *mut RPN_STACK;

const RPN_OP_STACK_SIZE: usize = 256;
const RPN_VALUE_STACK_SIZE: usize = 256;

static mut RpnStack: RPN_STACK_STORAGE = RPN_STACK_STORAGE {
    Size: RPN_OP_STACK_SIZE as ULONG,
    Sp: 0,
    Ops: [unsafe { core::mem::zeroed() }; RPN_OP_STACK_SIZE],
};

#[repr(C)]
struct RPN_STACK_STORAGE {
    Size: ULONG,
    Sp: ULONG,
    Ops: [RPN_OP; RPN_OP_STACK_SIZE],
}

struct RegisterMapping {
    Name: &'static [u8],
    Offset: UCHAR,
    Size: UCHAR,
}

#[cfg(target_arch = "x86")]
static REGISTER_TO_TRAP_FRAME: &[RegisterMapping] = &[
    RegisterMapping { Name: b"eip\0", Offset: offset_of_eip(), Size: size_of_eip() },
    RegisterMapping { Name: b"eflags\0", Offset: offset_of_eflags(), Size: size_of_eflags() },
    RegisterMapping { Name: b"eax\0", Offset: offset_of_eax(), Size: size_of_eax() },
    RegisterMapping { Name: b"ebx\0", Offset: offset_of_ebx(), Size: size_of_ebx() },
    RegisterMapping { Name: b"ecx\0", Offset: offset_of_ecx(), Size: size_of_ecx() },
    RegisterMapping { Name: b"edx\0", Offset: offset_of_edx(), Size: size_of_edx() },
    RegisterMapping { Name: b"esi\0", Offset: offset_of_esi(), Size: size_of_esi() },
    RegisterMapping { Name: b"edi\0", Offset: offset_of_edi(), Size: size_of_edi() },
    RegisterMapping { Name: b"esp\0", Offset: offset_of_esp(), Size: size_of_esp() },
    RegisterMapping { Name: b"ebp\0", Offset: offset_of_ebp(), Size: size_of_ebp() },
    RegisterMapping { Name: b"cs\0", Offset: offset_of_cs(), Size: 2 },
    RegisterMapping { Name: b"ds\0", Offset: offset_of_ds(), Size: size_of_ds() },
    RegisterMapping { Name: b"es\0", Offset: offset_of_es(), Size: size_of_es() },
    RegisterMapping { Name: b"fs\0", Offset: offset_of_fs(), Size: size_of_fs() },
    RegisterMapping { Name: b"gs\0", Offset: offset_of_gs(), Size: size_of_gs() },
    RegisterMapping { Name: b"ss\0", Offset: offset_of_ss(), Size: size_of_ss() },
    RegisterMapping { Name: b"dr0\0", Offset: offset_of_dr0(), Size: size_of_dr0() },
    RegisterMapping { Name: b"dr1\0", Offset: offset_of_dr1(), Size: size_of_dr1() },
    RegisterMapping { Name: b"dr2\0", Offset: offset_of_dr2(), Size: size_of_dr2() },
    RegisterMapping { Name: b"dr3\0", Offset: offset_of_dr3(), Size: size_of_dr3() },
    RegisterMapping { Name: b"dr6\0", Offset: offset_of_dr6(), Size: size_of_dr6() },
    RegisterMapping { Name: b"dr7\0", Offset: offset_of_dr7(), Size: size_of_dr7() },
];

#[cfg(not(target_arch = "x86"))]
static REGISTER_TO_TRAP_FRAME: &[RegisterMapping] = &[
    RegisterMapping { Name: b"rip\0", Offset: offset_of_rip(), Size: size_of_rip() },
    RegisterMapping { Name: b"eflags\0", Offset: offset_of_eflags(), Size: size_of_eflags() },
    RegisterMapping { Name: b"rax\0", Offset: offset_of_rax(), Size: size_of_rax() },
    RegisterMapping { Name: b"rbx\0", Offset: offset_of_rbx(), Size: size_of_rbx() },
    RegisterMapping { Name: b"rcx\0", Offset: offset_of_rcx(), Size: size_of_rcx() },
    RegisterMapping { Name: b"rdx\0", Offset: offset_of_rdx(), Size: size_of_rdx() },
    RegisterMapping { Name: b"rsi\0", Offset: offset_of_rsi(), Size: size_of_rsi() },
    RegisterMapping { Name: b"rdi\0", Offset: offset_of_rdi(), Size: size_of_rdi() },
    RegisterMapping { Name: b"rsp\0", Offset: offset_of_rsp(), Size: size_of_rsp() },
    RegisterMapping { Name: b"rbp\0", Offset: offset_of_rbp(), Size: size_of_rbp() },
    RegisterMapping { Name: b"cs\0", Offset: offset_of_cs(), Size: 2 },
    RegisterMapping { Name: b"ds\0", Offset: offset_of_ds(), Size: size_of_ds() },
    RegisterMapping { Name: b"es\0", Offset: offset_of_es(), Size: size_of_es() },
    RegisterMapping { Name: b"fs\0", Offset: offset_of_fs(), Size: size_of_fs() },
    RegisterMapping { Name: b"gs\0", Offset: offset_of_gs(), Size: size_of_gs() },
    RegisterMapping { Name: b"ss\0", Offset: offset_of_ss(), Size: size_of_ss() },
    RegisterMapping { Name: b"dr0\0", Offset: offset_of_dr0(), Size: size_of_dr0() },
    RegisterMapping { Name: b"dr1\0", Offset: offset_of_dr1(), Size: size_of_dr1() },
    RegisterMapping { Name: b"dr2\0", Offset: offset_of_dr2(), Size: size_of_dr2() },
    RegisterMapping { Name: b"dr3\0", Offset: offset_of_dr3(), Size: size_of_dr3() },
    RegisterMapping { Name: b"dr6\0", Offset: offset_of_dr6(), Size: size_of_dr6() },
    RegisterMapping { Name: b"dr7\0", Offset: offset_of_dr7(), Size: size_of_dr7() },
];

#[cfg(target_arch = "x86")]
const fn offset_of_eip() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_eip() -> u8 { 4 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rip() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rip() -> u8 { 8 }

const fn offset_of_eflags() -> u8 { 0 }
const fn size_of_eflags() -> u8 { 4 }

#[cfg(target_arch = "x86")]
const fn offset_of_eax() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_eax() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_ebx() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_ebx() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_ecx() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_ecx() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_edx() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_edx() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_esi() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_esi() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_edi() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_edi() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_esp() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_esp() -> u8 { 4 }
#[cfg(target_arch = "x86")]
const fn offset_of_ebp() -> u8 { 0 }
#[cfg(target_arch = "x86")]
const fn size_of_ebp() -> u8 { 4 }

#[cfg(not(target_arch = "x86"))]
const fn offset_of_rax() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rax() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rbx() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rbx() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rcx() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rcx() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rdx() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rdx() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rsi() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rsi() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rdi() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rdi() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rsp() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rsp() -> u8 { 8 }
#[cfg(not(target_arch = "x86"))]
const fn offset_of_rbp() -> u8 { 0 }
#[cfg(not(target_arch = "x86"))]
const fn size_of_rbp() -> u8 { 8 }

const fn offset_of_cs() -> u8 { 0 }
const fn offset_of_ds() -> u8 { 0 }
const fn size_of_ds() -> u8 { 4 }
const fn offset_of_es() -> u8 { 0 }
const fn size_of_es() -> u8 { 4 }
const fn offset_of_fs() -> u8 { 0 }
const fn size_of_fs() -> u8 { 4 }
const fn offset_of_gs() -> u8 { 0 }
const fn size_of_gs() -> u8 { 4 }
const fn offset_of_ss() -> u8 { 0 }
const fn size_of_ss() -> u8 { 4 }
const fn offset_of_dr0() -> u8 { 0 }
const fn size_of_dr0() -> u8 { 4 }
const fn offset_of_dr1() -> u8 { 0 }
const fn size_of_dr1() -> u8 { 4 }
const fn offset_of_dr2() -> u8 { 0 }
const fn size_of_dr2() -> u8 { 4 }
const fn offset_of_dr3() -> u8 { 0 }
const fn size_of_dr3() -> u8 { 4 }
const fn offset_of_dr6() -> u8 { 0 }
const fn size_of_dr6() -> u8 { 4 }
const fn offset_of_dr7() -> u8 { 0 }
const fn size_of_dr7() -> u8 { 4 }

unsafe extern "C" fn RpnBinaryOperatorAdd(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    a.wrapping_add(b)
}

unsafe extern "C" fn RpnBinaryOperatorSub(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    a.wrapping_sub(b)
}

unsafe extern "C" fn RpnBinaryOperatorMul(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    a.wrapping_mul(b)
}

unsafe extern "C" fn RpnBinaryOperatorDiv(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    a / b
}

unsafe extern "C" fn RpnBinaryOperatorMod(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    a % b
}

unsafe extern "C" fn RpnBinaryOperatorEquals(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a == b) as ULONGLONG
}

unsafe extern "C" fn RpnBinaryOperatorNotEquals(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a != b) as ULONGLONG
}

unsafe extern "C" fn RpnBinaryOperatorLessThan(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a < b) as ULONGLONG
}

unsafe extern "C" fn RpnBinaryOperatorLessThanOrEquals(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a <= b) as ULONGLONG
}

unsafe extern "C" fn RpnBinaryOperatorGreaterThan(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a > b) as ULONGLONG
}

unsafe extern "C" fn RpnBinaryOperatorGreaterThanOrEquals(a: ULONGLONG, b: ULONGLONG) -> ULONGLONG {
    (a >= b) as ULONGLONG
}

unsafe fn RpnpClearStack(Stack: PRPN_STACK) {
    (*Stack).Sp = 0;
}

unsafe fn RpnpPushStack(Stack: PRPN_STACK, Op: PRPN_OP) -> BOOLEAN {
    if (*Stack).Sp >= (*Stack).Size {
        return 0u8;
    }

    let dest = ((*Stack).Ops.as_mut_ptr() as *mut RPN_OP).add((*Stack).Sp as usize);
    core::ptr::copy_nonoverlapping(Op, dest, 1);
    (*Stack).Sp += 1;

    1u8
}

unsafe fn RpnpPopStack(Stack: PRPN_STACK, Op: PRPN_OP) -> BOOLEAN {
    if (*Stack).Sp == 0 {
        return 0u8;
    }

    (*Stack).Sp -= 1;
    if !Op.is_null() {
        let src = ((*Stack).Ops.as_ptr() as *const RPN_OP).add((*Stack).Sp as usize);
        core::ptr::copy_nonoverlapping(src, Op, 1);
    }

    1u8
}

unsafe fn RpnpTopStack(Stack: PRPN_STACK, Op: PRPN_OP) -> BOOLEAN {
    if (*Stack).Sp == 0 {
        return 0u8;
    }

    let src = ((*Stack).Ops.as_ptr() as *const RPN_OP).add(((*Stack).Sp - 1) as usize);
    core::ptr::copy_nonoverlapping(src, Op, 1);

    1u8
}

unsafe fn stricmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut p1 = s1;
    let mut p2 = s2;
    loop {
        let c1 = (*p1).to_ascii_lowercase();
        let c2 = (*p2).to_ascii_lowercase();
        if c1 != c2 || c1 == 0 {
            return (c1 as i32) - (c2 as i32);
        }
        p1 = p1.add(1);
        p2 = p2.add(1);
    }
}

unsafe fn strcspn(s: *const u8, reject: *const u8) -> usize {
    let mut count = 0;
    let mut p = s;
    while *p != 0 {
        let mut r = reject;
        while *r != 0 {
            if *p == *r {
                return count;
            }
            r = r.add(1);
        }
        count += 1;
        p = p.add(1);
    }
    count
}

unsafe fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dest.add(i) = 0;
        i += 1;
    }
    dest
}

unsafe fn strlen(s: *const u8) -> usize {
    let mut count = 0;
    while *s.add(count) != 0 {
        count += 1;
    }
    count
}

unsafe fn strtoull(s: *const u8, endptr: *mut *mut u8, base: i32) -> u64 {
    let mut p = s;
    let mut result: u64 = 0;
    let mut base = base as u32;

    while (*p as char).is_ascii_whitespace() {
        p = p.add(1);
    }

    if base == 0 {
        if *p == b'0' {
            p = p.add(1);
            if *p == b'x' || *p == b'X' {
                base = 16;
                p = p.add(1);
            } else {
                base = 8;
            }
        } else {
            base = 10;
        }
    } else if base == 16 && *p == b'0' {
        let next = *p.add(1);
        if next == b'x' || next == b'X' {
            p = p.add(2);
        }
    }

    let start = p;
    while *p != 0 {
        let digit = match *p {
            b'0'..=b'9' => (*p - b'0') as u32,
            b'a'..=b'z' => (*p - b'a' + 10) as u32,
            b'A'..=b'Z' => (*p - b'A' + 10) as u32,
            _ => break,
        };
        if digit >= base {
            break;
        }
        result = result * (base as u64) + (digit as u64);
        p = p.add(1);
    }

    if !endptr.is_null() {
        if p == start {
            *endptr = s as *mut u8;
        } else {
            *endptr = p as *mut u8;
        }
    }

    result
}

unsafe fn isspace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

extern "C" {
    fn KdbpSafeReadMemory(dest: *mut c_void, src: *const c_void, size: usize) -> NTSTATUS;
}

type KDB_KTRAP_FRAME = c_void;
type PKDB_KTRAP_FRAME = *mut KDB_KTRAP_FRAME;

const TAG_KDBG: u32 = u32::from_le_bytes(*b"KDBG");

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn RpnpParseExpression(
    Stack: PRPN_STACK,
    Expression: *mut u8,
    End: *mut *mut u8,
    CharacterOffset: ULONG,
    ErrOffset: *mut LONG,
    ErrMsg: *mut u8,
) -> BOOLEAN {
    let mut p = Expression;
    let mut pend: *mut u8 = core::ptr::null_mut();
    let mut Operator: *mut u8 = core::ptr::null_mut();
    let mut OperatorOffset: LONG = -1;
    let mut RpnOp: RPN_OP = core::mem::zeroed();
    let mut PoppedOperator: RPN_OP = core::mem::zeroed();
    let mut HavePoppedOperator = 0u8;
    let mut ComparativeOp: RPN_OP = core::mem::zeroed();
    let mut ComparativeOpFilled = 0u8;
    let mut IsComparativeOp: BOOLEAN;
    let mut i: INT_PTR;
    let mut i2: INT_PTR;
    let mut ull: u64;
    let mut MemorySize: UCHAR;
    let mut Buffer: [u8; 16] = [0; 16];
    let mut First: BOOLEAN;
    let mut CharacterOffset = CharacterOffset;

    First = 1u8;
    loop {
        while isspace(*p) {
            p = p.add(1);
            CharacterOffset += 1;
        }

        if *p == 0 || *p == b')' || *p == b']' {
            break;
        }

        if First == 0 {
            Operator = p;
            p = p.add(1);
            OperatorOffset = CharacterOffset as LONG;
            CharacterOffset += 1;

            HavePoppedOperator = 0u8;
            if *Operator == b'*' || *Operator == b'/' || *Operator == b'%' {
                if RpnpTopStack(Stack, &raw mut PoppedOperator) != 0
                    && PoppedOperator.Type as u32 == RPN_OP_TYPE::RpnOpBinaryOperator as u32
                    && (PoppedOperator.Data.BinaryOperator == Some(RpnBinaryOperatorAdd)
                        || PoppedOperator.Data.BinaryOperator == Some(RpnBinaryOperatorSub))
                {
                    RpnpPopStack(Stack, core::ptr::null_mut());
                    HavePoppedOperator = 1u8;
                } else if PoppedOperator.Type as u32 == RPN_OP_TYPE::RpnOpNop as u32 {
                    RpnpPopStack(Stack, core::ptr::null_mut());
                }
            } else if (*Operator == b'=' && *Operator.add(1) == b'=')
                || (*Operator == b'!' && *Operator.add(1) == b'=')
                || *Operator == b'<'
                || *Operator == b'>'
            {
                if *Operator == b'=' || *Operator == b'!' || (*Operator == b'<' && *Operator.add(1) == b'=') || (*Operator == b'>' && *Operator.add(1) == b'=') {
                    p = p.add(1);
                    CharacterOffset += 1;
                }
            } else if *Operator != b'+' && *Operator != b'-' {
                if !ErrMsg.is_null() {
                    let msg = b"Operator expected\0";
                    core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                }
                if !ErrOffset.is_null() {
                    *ErrOffset = OperatorOffset;
                }
                return 0u8;
            }

            while isspace(*p) {
                p = p.add(1);
                CharacterOffset += 1;
            }
        }

        MemorySize = core::mem::size_of::<ULONG_PTR>() as UCHAR;

        'get_operand: loop {
            i = strcspn(p, b"+-*/%()[]<>!=\0".as_ptr()) as INT_PTR;
            if i > 0 {
                i2 = i;
                while i2 > 0 && isspace(*p.add((i2 - 1) as usize)) {
                    i2 -= 1;
                }
                i2 = core::cmp::min(i2, (Buffer.len() - 1) as INT_PTR);
                strncpy(Buffer.as_mut_ptr(), p, i2 as usize);
                Buffer[i2 as usize] = 0;

                if *p.add(i as usize) == b'[' {
                    if stricmp(Buffer.as_ptr(), b"byte\0".as_ptr()) == 0 {
                        MemorySize = 1;
                    } else if stricmp(Buffer.as_ptr(), b"word\0".as_ptr()) == 0 {
                        MemorySize = 2;
                    } else if stricmp(Buffer.as_ptr(), b"dword\0".as_ptr()) == 0 {
                        MemorySize = 4;
                    } else if stricmp(Buffer.as_ptr(), b"qword\0".as_ptr()) == 0 {
                        MemorySize = 8;
                    } else {
                        if !ErrMsg.is_null() {
                            let msg = b"Invalid memory size prefix\0";
                            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                        }
                        if !ErrOffset.is_null() {
                            *ErrOffset = CharacterOffset as LONG;
                        }
                        return 0u8;
                    }

                    p = p.add(i as usize);
                    CharacterOffset += i as ULONG;
                    continue 'get_operand;
                }

                let mut found = REGISTER_TO_TRAP_FRAME.len();
                for (idx, reg) in REGISTER_TO_TRAP_FRAME.iter().enumerate() {
                    if stricmp(reg.Name.as_ptr(), Buffer.as_ptr()) == 0 {
                        found = idx;
                        break;
                    }
                }

                if found < REGISTER_TO_TRAP_FRAME.len() {
                    RpnOp.Type = RPN_OP_TYPE::RpnOpRegister;
                    RpnOp.CharacterOffset = CharacterOffset;
                    RpnOp.Data.Register = found as UCHAR;
                    i = strlen(REGISTER_TO_TRAP_FRAME[found].Name.as_ptr()) as INT_PTR;
                    CharacterOffset += i as ULONG;
                    p = p.add(i as usize);
                } else {
                    ull = strtoull(p, &raw mut pend, 0);
                    if p != pend {
                        RpnOp.Type = RPN_OP_TYPE::RpnOpImmediate;
                        RpnOp.CharacterOffset = CharacterOffset;
                        RpnOp.Data.Immediate = ull;
                        CharacterOffset += pend.offset_from(p) as ULONG;
                        p = pend;
                    } else {
                        if !ErrMsg.is_null() {
                            let msg = b"Operand expected\0";
                            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                        }
                        if !ErrOffset.is_null() {
                            *ErrOffset = CharacterOffset as LONG;
                        }
                        return 0u8;
                    }
                }

                if RpnpPushStack(Stack, &raw mut RpnOp) == 0 {
                    if !ErrMsg.is_null() {
                        let msg = b"RPN op stack overflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }
            } else if i == 0 {
                if *p == b'(' || *p == b'[' {
                    if RpnpParseExpression(
                        Stack,
                        p.add(1),
                        &raw mut pend,
                        CharacterOffset + 1,
                        ErrOffset,
                        ErrMsg,
                    ) == 0
                    {
                        return 0u8;
                    } else if pend == p.add(1) {
                        if !ErrMsg.is_null() {
                            let msg = b"Expression expected\0";
                            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                        }
                        if !ErrOffset.is_null() {
                            *ErrOffset = (CharacterOffset + 1) as LONG;
                        }
                        return 0u8;
                    }

                    if *p == b'[' {
                        if *pend != b']' {
                            if !ErrMsg.is_null() {
                                let msg = b"']' expected\0";
                                core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                            }
                            if !ErrOffset.is_null() {
                                *ErrOffset = (CharacterOffset + pend.offset_from(p) as ULONG) as LONG;
                            }
                            return 0u8;
                        }

                        RpnOp.Type = RPN_OP_TYPE::RpnOpDereference;
                        RpnOp.CharacterOffset = CharacterOffset;
                        RpnOp.Data.DerefMemorySize = MemorySize;

                        if RpnpPushStack(Stack, &raw mut RpnOp) == 0 {
                            if !ErrMsg.is_null() {
                                let msg = b"RPN op stack overflow\0";
                                core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                            }
                            if !ErrOffset.is_null() {
                                *ErrOffset = -1;
                            }
                            return 0u8;
                        }
                    } else {
                        if *pend != b')' {
                            if !ErrMsg.is_null() {
                                let msg = b"')' expected\0";
                                core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                            }
                            if !ErrOffset.is_null() {
                                *ErrOffset = (CharacterOffset + pend.offset_from(p) as ULONG) as LONG;
                            }
                            return 0u8;
                        }
                    }

                    RpnOp.Type = RPN_OP_TYPE::RpnOpNop;
                    if RpnpPushStack(Stack, &raw mut RpnOp) == 0 {
                        if !ErrMsg.is_null() {
                            let msg = b"RPN op stack overflow\0";
                            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                        }
                        if !ErrOffset.is_null() {
                            *ErrOffset = -1;
                        }
                        return 0u8;
                    }

                    pend = pend.add(1);

                    CharacterOffset += pend.offset_from(p) as ULONG;
                    p = pend;
                } else if First != 0 && *p == b'-' {
                    RpnOp.Type = RPN_OP_TYPE::RpnOpImmediate;
                    RpnOp.CharacterOffset = CharacterOffset;
                    RpnOp.Data.Immediate = 0;

                    if RpnpPushStack(Stack, &raw mut RpnOp) == 0 {
                        if !ErrMsg.is_null() {
                            let msg = b"RPN op stack overflow\0";
                            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                        }
                        if !ErrOffset.is_null() {
                            *ErrOffset = -1;
                        }
                        return 0u8;
                    }
                } else {
                    if !ErrMsg.is_null() {
                        let msg = b"Operand expected\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = CharacterOffset as LONG;
                    }
                    return 0u8;
                }
            } else {
                if !ErrMsg.is_null() {
                    let msg = b"strcspn() failed\0";
                    core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                }
                if !ErrOffset.is_null() {
                    *ErrOffset = -1;
                }
                return 0u8;
            }
            break;
        }

        if First == 0 {
            RpnOp.CharacterOffset = OperatorOffset as ULONG;
            RpnOp.Type = RPN_OP_TYPE::RpnOpBinaryOperator;
            IsComparativeOp = 0u8;

            match *Operator {
                b'+' => {
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorAdd);
                }
                b'-' => {
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorSub);
                }
                b'*' => {
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorMul);
                }
                b'/' => {
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorDiv);
                }
                b'%' => {
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorMod);
                }
                b'=' => {
                    IsComparativeOp = 1u8;
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorEquals);
                }
                b'!' => {
                    IsComparativeOp = 1u8;
                    RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorNotEquals);
                }
                b'<' => {
                    IsComparativeOp = 1u8;
                    if *Operator.add(1) == b'=' {
                        RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorLessThanOrEquals);
                    } else {
                        RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorLessThan);
                    }
                }
                b'>' => {
                    IsComparativeOp = 1u8;
                    if *Operator.add(1) == b'=' {
                        RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorGreaterThanOrEquals);
                    } else {
                        RpnOp.Data.BinaryOperator = Some(RpnBinaryOperatorGreaterThan);
                    }
                }
                _ => {}
            }

            if IsComparativeOp != 0 {
                if ComparativeOpFilled != 0 && RpnpPushStack(Stack, &raw mut ComparativeOp) == 0 {
                    if !ErrMsg.is_null() {
                        let msg = b"RPN op stack overflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }

                core::ptr::copy_nonoverlapping(&RpnOp, &raw mut ComparativeOp, 1);
                ComparativeOpFilled = 1u8;
            } else if RpnpPushStack(Stack, &raw mut RpnOp) == 0 {
                if !ErrMsg.is_null() {
                    let msg = b"RPN op stack overflow\0";
                    core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                }
                if !ErrOffset.is_null() {
                    *ErrOffset = -1;
                }
                return 0u8;
            }

            if HavePoppedOperator != 0 {
                if RpnpPushStack(Stack, &raw mut PoppedOperator) == 0 {
                    if !ErrMsg.is_null() {
                        let msg = b"RPN op stack overflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }
            }
        }

        First = 0u8;
    }

    if ComparativeOpFilled != 0 && RpnpPushStack(Stack, &raw mut ComparativeOp) == 0 {
        if !ErrMsg.is_null() {
            let msg = b"RPN op stack overflow\0";
            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
        }
        if !ErrOffset.is_null() {
            *ErrOffset = -1;
        }
        return 0u8;
    }

    while isspace(*p) {
        p = p.add(1);
        CharacterOffset += 1;
    }

    if !End.is_null() {
        *End = p;
    }

    1u8
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn RpnpEvaluateStack(
    Stack: PRPN_STACK,
    TrapFrame: PKDB_KTRAP_FRAME,
    Result: PULONGLONG,
    ErrOffset: *mut LONG,
    ErrMsg: *mut u8,
) -> BOOLEAN {
    let mut ValueStack: [ULONGLONG; RPN_VALUE_STACK_SIZE] = [0; RPN_VALUE_STACK_SIZE];
    let mut ValueStackPointer: ULONG = 0;
    let mut index: ULONG;
    let mut ull: ULONGLONG = 0;
    let mut ul: ULONG = 0;
    let mut us: USHORT = 0;
    let mut uc: UCHAR = 0;
    let mut p: PVOID;
    let mut IsOk: BOOLEAN;

    index = 0;
    while index < (*Stack).Sp {
        let Op = ((*Stack).Ops.as_ptr() as *const RPN_OP).add(index as usize);

        match (*Op).Type as u32 {
            x if x == RPN_OP_TYPE::RpnOpNop as u32 => {}
            x if x == RPN_OP_TYPE::RpnOpImmediate as u32 => {
                if ValueStackPointer == RPN_VALUE_STACK_SIZE as ULONG {
                    if !ErrMsg.is_null() {
                        let msg = b"Value stack overflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }
                ValueStack[ValueStackPointer as usize] = (*Op).Data.Immediate;
                ValueStackPointer += 1;
            }
            x if x == RPN_OP_TYPE::RpnOpRegister as u32 => {
                if ValueStackPointer == RPN_VALUE_STACK_SIZE as ULONG {
                    if !ErrMsg.is_null() {
                        let msg = b"Value stack overflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }

                ul = (*Op).Data.Register as ULONG;
                p = ((TrapFrame as ULONG_PTR) + (REGISTER_TO_TRAP_FRAME[ul as usize].Offset as ULONG_PTR)) as PVOID;

                match REGISTER_TO_TRAP_FRAME[ul as usize].Size {
                    1 => ull = *(p as *const UCHAR) as ULONGLONG,
                    2 => ull = *(p as *const USHORT) as ULONGLONG,
                    4 => ull = *(p as *const ULONG) as ULONGLONG,
                    8 => ull = *(p as *const ULONGLONG),
                    _ => return 0u8,
                }

                ValueStack[ValueStackPointer as usize] = ull;
                ValueStackPointer += 1;
            }
            x if x == RPN_OP_TYPE::RpnOpDereference as u32 => {
                if ValueStackPointer < 1 {
                    if !ErrMsg.is_null() {
                        let msg = b"Value stack underflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }

                p = ValueStack[(ValueStackPointer - 1) as usize] as ULONG_PTR as PVOID;
                IsOk = 0u8;

                match (*Op).Data.DerefMemorySize {
                    1 => {
                        if NT_SUCCESS(KdbpSafeReadMemory(&raw mut uc as *mut c_void, p, core::mem::size_of::<UCHAR>())) {
                            IsOk = 1u8;
                            ull = uc as ULONGLONG;
                        }
                    }
                    2 => {
                        if NT_SUCCESS(KdbpSafeReadMemory(&raw mut us as *mut c_void, p, core::mem::size_of::<USHORT>())) {
                            IsOk = 1u8;
                            ull = us as ULONGLONG;
                        }
                    }
                    4 => {
                        if NT_SUCCESS(KdbpSafeReadMemory(&raw mut ul as *mut c_void, p, core::mem::size_of::<ULONG>())) {
                            IsOk = 1u8;
                            ull = ul as ULONGLONG;
                        }
                    }
                    8 => {
                        if NT_SUCCESS(KdbpSafeReadMemory(&raw mut ull as *mut c_void, p, core::mem::size_of::<ULONGLONG>())) {
                            IsOk = 1u8;
                        }
                    }
                    _ => return 0u8,
                }

                if IsOk == 0 {
                    if !ErrMsg.is_null() {
                        let msg = b"Couldn't access memory\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, core::cmp::min(msg.len(), 128));
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = (*Op).CharacterOffset as LONG;
                    }
                    return 0u8;
                }

                ValueStack[(ValueStackPointer - 1) as usize] = ull;
            }
            x if x == RPN_OP_TYPE::RpnOpBinaryOperator as u32 => {
                if ValueStackPointer < 2 {
                    if !ErrMsg.is_null() {
                        let msg = b"Value stack underflow\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = -1;
                    }
                    return 0u8;
                }

                ValueStackPointer -= 1;
                ull = ValueStack[ValueStackPointer as usize];

                if ull == 0 && (*Op).Data.BinaryOperator == Some(RpnBinaryOperatorDiv) {
                    if !ErrMsg.is_null() {
                        let msg = b"Division by zero\0";
                        core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
                    }
                    if !ErrOffset.is_null() {
                        *ErrOffset = (*Op).CharacterOffset as LONG;
                    }
                    return 0u8;
                }

                ull = ((*Op).Data.BinaryOperator.unwrap())(ValueStack[(ValueStackPointer - 1) as usize], ull);
                ValueStack[(ValueStackPointer - 1) as usize] = ull;
            }
            _ => return 0u8,
        }

        index += 1;
    }

    if ValueStackPointer != 1 {
        if !ErrMsg.is_null() {
            let msg = b"Stack not empty after evaluation\0";
            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
        }
        if !ErrOffset.is_null() {
            *ErrOffset = -1;
        }
        return 0u8;
    }

    *Result = ValueStack[0];
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpRpnEvaluateExpression(
    Expression: *mut u8,
    TrapFrame: PKDB_KTRAP_FRAME,
    Result: PULONGLONG,
    ErrOffset: *mut LONG,
    ErrMsg: *mut u8,
) -> BOOLEAN {
    let Stack = &raw mut RpnStack as PRPN_STACK;

    RpnpClearStack(Stack);
    if RpnpParseExpression(Stack, Expression, core::ptr::null_mut(), 0, ErrOffset, ErrMsg) == 0 {
        return 0u8;
    }

    if RpnpEvaluateStack(Stack, TrapFrame, Result, ErrOffset, ErrMsg) == 0 {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpRpnParseExpression(
    Expression: *mut u8,
    ErrOffset: *mut LONG,
    ErrMsg: *mut u8,
) -> PVOID {
    let Stack = &raw mut RpnStack as PRPN_STACK;

    RpnpClearStack(Stack);
    if RpnpParseExpression(Stack, Expression, core::ptr::null_mut(), 0, ErrOffset, ErrMsg) == 0 {
        return core::ptr::null_mut();
    }

    let Size = core::mem::size_of::<RPN_STACK>() + (core::mem::size_of::<RPN_OP>() * (((*Stack).Sp - 1) as usize));
    let NewStack = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_KDBG) as PRPN_STACK;

    if NewStack.is_null() {
        if !ErrMsg.is_null() {
            let msg = b"Out of memory\0";
            core::ptr::copy_nonoverlapping(msg.as_ptr(), ErrMsg, msg.len());
        }
        if !ErrOffset.is_null() {
            *ErrOffset = -1;
        }
        return core::ptr::null_mut();
    }

    core::ptr::copy_nonoverlapping(Stack as *const u8, NewStack as *mut u8, Size);
    (*NewStack).Size = (*NewStack).Sp;

    NewStack as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn KdbpRpnEvaluateParsedExpression(
    Expression: PVOID,
    TrapFrame: PKDB_KTRAP_FRAME,
    Result: PULONGLONG,
    ErrOffset: *mut LONG,
    ErrMsg: *mut u8,
) -> BOOLEAN {
    let Stack = Expression as PRPN_STACK;
    RpnpEvaluateStack(Stack, TrapFrame, Result, ErrOffset, ErrMsg)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2005 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/kdbg/kdb_expr.c
//  * PURPOSE:         Kernel debugger expression evaluation
//  * PROGRAMMER:      Gregor Anich (blight@blight.eu.org)
//  * UPDATE HISTORY:
//  *                  Created 15/01/2005
//  */
//
// /* Note:
//  *
//  * The given expression is parsed and stored in reverse polish notation,
//  * then it is evaluated and the result is returned.
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kdb.h"
//
// #define NDEBUG
// #include "debug.h"
//
// /* TYPES *********************************************************************/
// typedef enum _RPN_OP_TYPE
// {
//     RpnOpNop,
//     RpnOpBinaryOperator,
//     RpnOpUnaryOperator,
//     RpnOpImmediate,
//     RpnOpRegister,
//     RpnOpDereference
// } RPN_OP_TYPE;
//
// typedef ULONGLONG (*RPN_BINARY_OPERATOR)(ULONGLONG a, ULONGLONG b);
//
// typedef struct _RPN_OP
// {
//     RPN_OP_TYPE  Type;
//     ULONG        CharacterOffset;
//     union
//     {
//         /* RpnOpBinaryOperator */
//         RPN_BINARY_OPERATOR  BinaryOperator;
//         /* RpnOpImmediate */
//         ULONGLONG            Immediate;
//         /* RpnOpRegister */
//         UCHAR                Register;
//         /* RpnOpDereference */
//         UCHAR                DerefMemorySize;
//     }
//     Data;
// }
// RPN_OP, *PRPN_OP;
//
// typedef struct _RPN_STACK
// {
//     ULONG   Size;     /* Number of RPN_OPs on Ops */
//     ULONG   Sp;       /* Stack pointer */
//     RPN_OP  Ops[1];   /* Array of RPN_OPs */
// }
// RPN_STACK, *PRPN_STACK;
//
// /* DEFINES *******************************************************************/
// #define stricmp _stricmp
//
// #ifndef RTL_FIELD_SIZE
// # define RTL_FIELD_SIZE(type, field) (sizeof(((type *)0)->field))
// #endif
//
// #define CONST_STRCPY(dst, src) \
//     do { if ((dst)) { memcpy(dst, src, sizeof(src)); } } while (0);
//
// #define RPN_OP_STACK_SIZE     256
// #define RPN_VALUE_STACK_SIZE  256
//
// /* GLOBALS *******************************************************************/
// static struct
// {
//     ULONG Size;
//     ULONG Sp;
//     RPN_OP Ops[RPN_OP_STACK_SIZE];
// }
// RpnStack =
// {
//     RPN_OP_STACK_SIZE,
//     0
// };
//
// static const struct
// {
//     PCHAR Name;
//     UCHAR Offset;
//     UCHAR Size;
// }
// RegisterToTrapFrame[] =
// {
//     /* FIXME: X86 only */
// #ifdef _M_IX86
//     {"eip",     FIELD_OFFSET(KDB_KTRAP_FRAME, Eip),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Eip)},
// #else
//     {"rip",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rip),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rip)},
// #endif
//     {"eflags",  FIELD_OFFSET(KDB_KTRAP_FRAME, EFlags),  RTL_FIELD_SIZE(KDB_KTRAP_FRAME, EFlags)},
// #ifdef _M_IX86
//     {"eax",     FIELD_OFFSET(KDB_KTRAP_FRAME, Eax),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Eax)},
//     {"ebx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Ebx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Ebx)},
//     {"ecx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Ecx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Ecx)},
//     {"edx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Edx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Edx)},
//     {"esi",     FIELD_OFFSET(KDB_KTRAP_FRAME, Esi),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Esi)},
//     {"edi",     FIELD_OFFSET(KDB_KTRAP_FRAME, Edi),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Edi)},
//     {"esp",     FIELD_OFFSET(KDB_KTRAP_FRAME, Esp),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Esp)},
//     {"ebp",     FIELD_OFFSET(KDB_KTRAP_FRAME, Ebp),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Ebp)},
// #else
//     {"rax",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rax),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rax)},
//     {"rbx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rbx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rbx)},
//     {"rcx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rcx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rcx)},
//     {"rdx",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rdx),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rdx)},
//     {"rsi",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rsi),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rsi)},
//     {"rdi",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rdi),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rdi)},
//     {"rsp",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rsp),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rsp)},
//     {"rbp",     FIELD_OFFSET(KDB_KTRAP_FRAME, Rbp),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Rbp)},
// #endif
//     {"cs",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegCs),      2 }, /* Use only the lower 2 bytes */
//     {"ds",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegDs),      RTL_FIELD_SIZE(KDB_KTRAP_FRAME, SegDs)},
//     {"es",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegEs),      RTL_FIELD_SIZE(KDB_KTRAP_FRAME, SegEs)},
//     {"fs",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegFs),      RTL_FIELD_SIZE(KDB_KTRAP_FRAME, SegFs)},
//     {"gs",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegGs),      RTL_FIELD_SIZE(KDB_KTRAP_FRAME, SegGs)},
//     {"ss",      FIELD_OFFSET(KDB_KTRAP_FRAME, SegSs),      RTL_FIELD_SIZE(KDB_KTRAP_FRAME, SegSs)},
//     {"dr0",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr0),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr0)},
//     {"dr1",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr1),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr1)},
//     {"dr2",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr2),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr2)},
//     {"dr3",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr3),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr3)},
//     {"dr6",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr6),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr6)},
//     {"dr7",     FIELD_OFFSET(KDB_KTRAP_FRAME, Dr7),     RTL_FIELD_SIZE(KDB_KTRAP_FRAME, Dr7)}
// };
// static const INT RegisterToTrapFrameCount = sizeof (RegisterToTrapFrame) / sizeof (RegisterToTrapFrame[0]);
//
// /* FUNCTIONS *****************************************************************/
//
// ULONGLONG
// RpnBinaryOperatorAdd(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return a + b;
// }
//
// ULONGLONG
// RpnBinaryOperatorSub(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return a - b;
// }
//
// ULONGLONG
// RpnBinaryOperatorMul(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return a * b;
// }
//
// ULONGLONG
// RpnBinaryOperatorDiv(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return a / b;
// }
//
// ULONGLONG
// RpnBinaryOperatorMod(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return a % b;
// }
//
// ULONGLONG
// RpnBinaryOperatorEquals(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a == b);
// }
//
// ULONGLONG
// RpnBinaryOperatorNotEquals(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a != b);
// }
//
// ULONGLONG
// RpnBinaryOperatorLessThan(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a < b);
// }
//
// ULONGLONG
// RpnBinaryOperatorLessThanOrEquals(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a <= b);
// }
//
// ULONGLONG
// RpnBinaryOperatorGreaterThan(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a > b);
// }
//
// ULONGLONG
// RpnBinaryOperatorGreaterThanOrEquals(
//     ULONGLONG a,
//     ULONGLONG b)
// {
//     return (a >= b);
// }
//
// #ifdef DEBUG_RPN
// /*!\brief Dumps the given RPN stack content
//  *
//  * \param Stack  Pointer to a RPN_STACK structure.
//  */
// VOID
// RpnpDumpStack(
//     IN PRPN_STACK Stack)
// {
//     ULONG ul;
//
//     ASSERT(Stack);
//     KdbPrintf("\nStack size: %ld\n", Stack->Sp);
//
//     for (ul = 0; ul < Stack->Sp; ul++)
//     {
//         PRPN_OP Op = Stack->Ops + ul;
//         switch (Op->Type)
//         {
//             case RpnOpNop:
//                 KdbPuts("NOP,");
//                 break;
//
//             case RpnOpImmediate:
//                 KdbPrintf("0x%I64x,", Op->Data.Immediate);
//                 break;
//
//             case RpnOpBinaryOperator:
//                 if (Op->Data.BinaryOperator == RpnBinaryOperatorAdd)
//                     KdbPuts("+,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorSub)
//                     KdbPuts("-,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorMul)
//                     KdbPuts("*,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorDiv)
//                     KdbPuts("/,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorMod)
//                     KdbPuts("%%,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorEquals)
//                     KdbPuts("==,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorNotEquals)
//                     KdbPuts("!=,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorLessThan)
//                     KdbPuts("<,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorLessThanOrEquals)
//                     KdbPuts("<=,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorGreaterThan)
//                     KdbPuts(">,");
//                 else if (Op->Data.BinaryOperator == RpnBinaryOperatorGreaterThanOrEquals)
//                     KdbPuts(">=,");
//                 else
//                     KdbPuts("UNKNOWN OP,");
//
//                 break;
//
//             case RpnOpRegister:
//                 KdbPrintf("%s,", RegisterToTrapFrame[Op->Data.Register].Name);
//                 break;
//
//             case RpnOpDereference:
//                 KdbPrintf("[%s],",
//                     (Op->Data.DerefMemorySize == 1) ? ("byte") :
//                     ((Op->Data.DerefMemorySize == 2) ? ("word") :
//                     ((Op->Data.DerefMemorySize == 4) ? ("dword") : ("qword"))));
//                 break;
//
//             default:
//                 KdbPrintf("\nUnsupported Type: %d\n", Op->Type);
//                 ul = Stack->Sp;
//                 break;
//         }
//     }
//
//     KdbPuts("\n");
// }
// #endif // DEBUG_RPN
//
// /*!\brief Clears the given RPN stack.
//  *
//  * \param Stack  Pointer to a RPN_STACK structure.
//  */
// static VOID
// RpnpClearStack(
//     OUT PRPN_STACK Stack)
// {
//     ASSERT(Stack);
//     Stack->Sp = 0;
// }
//
// /*!\brief Pushes an RPN_OP onto the stack.
//  *
//  * \param Stack  Pointer to a RPN_STACK structure.
//  * \param Op     RPN_OP to be copied onto the stack.
//  */
// static BOOLEAN
// RpnpPushStack(
//     IN OUT PRPN_STACK Stack,
//     IN     PRPN_OP Op)
// {
//     ASSERT(Stack);
//     ASSERT(Op);
//
//     if (Stack->Sp >= Stack->Size)
//         return FALSE;
//
//     memcpy(Stack->Ops + Stack->Sp, Op, sizeof (RPN_OP));
//     Stack->Sp++;
//
//     return TRUE;
// }
//
// /*!\brief Pops the top op from the stack.
//  *
//  * \param Stack  Pointer to a RPN_STACK structure.
//  * \param Op     Pointer to an RPN_OP to store the popped op into (can be NULL).
//  *
//  * \retval TRUE   Success.
//  * \retval FALSE  Failure (stack empty)
//  */
// static BOOLEAN
// RpnpPopStack(
//     IN OUT PRPN_STACK Stack,
//     OUT    PRPN_OP Op  OPTIONAL)
// {
//     ASSERT(Stack);
//
//     if (Stack->Sp == 0)
//         return FALSE;
//
//     Stack->Sp--;
//     if (Op)
//         memcpy(Op, Stack->Ops + Stack->Sp, sizeof (RPN_OP));
//
//     return TRUE;
// }
//
// /*!\brief Gets the top op from the stack (not popping it)
//  *
//  * \param Stack  Pointer to a RPN_STACK structure.
//  * \param Op     Pointer to an RPN_OP to copy the top op into.
//  *
//  * \retval TRUE   Success.
//  * \retval FALSE  Failure (stack empty)
//  */
// static BOOLEAN
// RpnpTopStack(
//     IN  PRPN_STACK Stack,
//     OUT PRPN_OP Op)
// {
//     ASSERT(Stack);
//     ASSERT(Op);
//
//     if (Stack->Sp == 0)
//         return FALSE;
//
//     memcpy(Op, Stack->Ops + Stack->Sp - 1, sizeof (RPN_OP));
//
//     return TRUE;
// }
//
// /*!\brief Parses an expression.
//  *
//  * This functions parses the given expression until the end of string or a closing
//  * brace is found. As the function parses the string it pushes RPN_OPs onto the
//  * stack.
//  *
//  * Examples: 1+2*3 ; eax+10 ; (eax+16) * (ebx+4) ; dword[eax]
//  *
//  * \param Stack            Pointer to a RPN_STACK structure.
//  * \param Expression       String to parse.
//  * \param CharacterOffset  Character offset of the subexpression from the beginning of the expression.
//  * \param End              On success End is set to the character at which parsing stopped.
//  * \param ErrOffset        On failure this is set to the character offset at which the error occoured.
//  * \param ErrMsg           On failure a message describing the problem is copied into this buffer (128 bytes)
//  *
//  * \retval TRUE   Success.
//  * \retval FALSE  Failure.
//  */
// static BOOLEAN
// RpnpParseExpression(
//     IN  PRPN_STACK Stack,
//     IN  PCHAR Expression,
//     OUT PCHAR *End  OPTIONAL,
//     IN  ULONG CharacterOffset,
//     OUT PLONG ErrOffset  OPTIONAL,
//     OUT PCHAR ErrMsg  OPTIONAL)
// {
//     PCHAR p = Expression;
//     PCHAR pend;
//     PCHAR Operator = NULL;
//     LONG OperatorOffset = -1;
//     RPN_OP RpnOp;
//     RPN_OP PoppedOperator;
//     BOOLEAN HavePoppedOperator = FALSE;
//     RPN_OP ComparativeOp;
//     BOOLEAN ComparativeOpFilled = FALSE;
//     BOOLEAN IsComparativeOp;
//     INT_PTR i, i2;
//     ULONG64 ull;
//     UCHAR MemorySize;
//     CHAR Buffer[16];
//     BOOLEAN First;
//
//     ASSERT(Stack);
//     ASSERT(Expression);
//
//     First = TRUE;
//     for (;;)
//     {
//         /* Skip whitespace */
//         while (isspace(*p))
//         {
//             p++;
//             CharacterOffset++;
//         }
//
//         /* Check for end of expression */
//         if (p[0] == '\0' || p[0] == ')' || p[0] == ']')
//             break;
//
//         if (!First)
//         {
//             /* Remember operator */
//             Operator = p++;
//             OperatorOffset = CharacterOffset++;
//
//             /* Pop operator (to get the right operator precedence) */
//             HavePoppedOperator = FALSE;
//             if (*Operator == '*' || *Operator == '/' || *Operator == '%')
//             {
//                 if (RpnpTopStack(Stack, &PoppedOperator) &&
//                     PoppedOperator.Type == RpnOpBinaryOperator &&
//                     (PoppedOperator.Data.BinaryOperator == RpnBinaryOperatorAdd ||
//                     PoppedOperator.Data.BinaryOperator == RpnBinaryOperatorSub))
//                 {
//                     RpnpPopStack(Stack, NULL);
//                     HavePoppedOperator = TRUE;
//                 }
//                 else if (PoppedOperator.Type == RpnOpNop)
//                 {
//                     RpnpPopStack(Stack, NULL);
//                     /* Discard the NOP - it was only pushed to indicate there was a
//                      * closing brace, so the previous operator shouldn't be popped.
//                      */
//                 }
//             }
//             else if ((Operator[0] == '=' && Operator[1] == '=') ||
//                      (Operator[0] == '!' && Operator[1] == '=') ||
//                      Operator[0] == '<' || Operator[0] == '>')
//             {
//                 if (Operator[0] == '=' || Operator[0] == '!' ||
//                     (Operator[0] == '<' && Operator[1] == '=') ||
//                     (Operator[0] == '>' && Operator[1] == '='))
//                 {
//                     p++;
//                     CharacterOffset++;
//                 }
// #if 0
//                 /* Parse rest of expression */
//                 if (!RpnpParseExpression(Stack, p + 1, &pend, CharacterOffset + 1,
//                                          ErrOffset, ErrMsg))
//                 {
//                     return FALSE;
//                 }
//                 else if (pend == p + 1)
//                 {
//                     CONST_STRCPY(ErrMsg, "Expression expected");
//
//                     if (ErrOffset)
//                         *ErrOffset = CharacterOffset + 1;
//
//                     return FALSE;
//                 }
//
//                 goto end_of_expression; /* return */
// #endif
//             }
//             else if (Operator[0] != '+' && Operator[0] != '-')
//             {
//                 CONST_STRCPY(ErrMsg, "Operator expected");
//
//                 if (ErrOffset)
//                     *ErrOffset = OperatorOffset;
//
//                 return FALSE;
//             }
//
//             /* Skip whitespace */
//             while (isspace(*p))
//             {
//                 p++;
//                 CharacterOffset++;
//             }
//         }
//
//         /* Get operand */
//         MemorySize = sizeof(ULONG_PTR); /* default to pointer size */
//
// get_operand:
//         i = strcspn(p, "+-*/%()[]<>!=");
//         if (i > 0)
//         {
//             i2 = i;
//
//             /* Copy register name/memory size */
//             while (isspace(p[--i2]));
//
//             i2 = min(i2 + 1, (INT)sizeof (Buffer) - 1);
//             strncpy(Buffer, p, i2);
//             Buffer[i2] = '\0';
//
//             /* Memory size prefix */
//             if (p[i] == '[')
//             {
//                 if (stricmp(Buffer, "byte") == 0)
//                     MemorySize = 1;
//                 else if (stricmp(Buffer, "word") == 0)
//                     MemorySize = 2;
//                 else if (stricmp(Buffer, "dword") == 0)
//                     MemorySize = 4;
//                 else if (stricmp(Buffer, "qword") == 0)
//                     MemorySize = 8;
//                 else
//                 {
//                     CONST_STRCPY(ErrMsg, "Invalid memory size prefix");
//
//                     if (ErrOffset)
//                         *ErrOffset = CharacterOffset;
//
//                     return FALSE;
//                 }
//
//                 p += i;
//                 CharacterOffset += i;
//                 goto get_operand;
//             }
//
//             /* Try to find register */
//             for (i = 0; i < RegisterToTrapFrameCount; i++)
//             {
//                 if (stricmp(RegisterToTrapFrame[i].Name, Buffer) == 0)
//                     break;
//             }
//
//             if (i < RegisterToTrapFrameCount)
//             {
//                 RpnOp.Type = RpnOpRegister;
//                 RpnOp.CharacterOffset = CharacterOffset;
//                 RpnOp.Data.Register = i;
//                 i = strlen(RegisterToTrapFrame[i].Name);
//                 CharacterOffset += i;
//                 p += i;
//             }
//             else
//             {
//                 /* Immediate value */
//                 ull = strtoull(p, &pend, 0);
//                 if (p != pend)
//                 {
//                     RpnOp.Type = RpnOpImmediate;
//                     RpnOp.CharacterOffset = CharacterOffset;
//                     RpnOp.Data.Immediate = ull;
//                     CharacterOffset += pend - p;
//                     p = pend;
//                 }
//                 else
//                 {
//                     CONST_STRCPY(ErrMsg, "Operand expected");
//
//                     if (ErrOffset)
//                         *ErrOffset = CharacterOffset;
//
//                     return FALSE;
//                 }
//             }
//
//             /* Push operand */
//             if (!RpnpPushStack(Stack, &RpnOp))
//             {
//                 CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                 if (ErrOffset)
//                     *ErrOffset = -1;
//
//                 return FALSE;
//             }
//         }
//         else if (i == 0)
//         {
//             if (p[0] == '(' || p[0] == '[') /* subexpression */
//             {
//                 if (!RpnpParseExpression(Stack, p + 1, &pend, CharacterOffset + 1,
//                                          ErrOffset, ErrMsg))
//                 {
//                     return FALSE;
//                 }
//                 else if (pend == p + 1)
//                 {
//                     CONST_STRCPY(ErrMsg, "Expression expected");
//
//                     if (ErrOffset)
//                         *ErrOffset = CharacterOffset + 1;
//
//                     return FALSE;
//                 }
//
//                 if (p[0] == '[') /* dereference */
//                 {
//                     ASSERT(MemorySize == 1 || MemorySize == 2 ||
//                            MemorySize == 4 || MemorySize == 8);
//
//                     if (pend[0] != ']')
//                     {
//                         CONST_STRCPY(ErrMsg, "']' expected");
//
//                         if (ErrOffset)
//                             *ErrOffset = CharacterOffset + (pend - p);
//
//                         return FALSE;
//                     }
//
//                     RpnOp.Type = RpnOpDereference;
//                     RpnOp.CharacterOffset = CharacterOffset;
//                     RpnOp.Data.DerefMemorySize = MemorySize;
//
//                     if (!RpnpPushStack(Stack, &RpnOp))
//                     {
//                         CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                         if (ErrOffset)
//                             *ErrOffset = -1;
//
//                         return FALSE;
//                     }
//                 }
//                 else /* p[0] == '(' */
//                 {
//                     if (pend[0] != ')')
//                     {
//                         CONST_STRCPY(ErrMsg, "')' expected");
//
//                         if (ErrOffset)
//                             *ErrOffset = CharacterOffset + (pend - p);
//
//                         return FALSE;
//                     }
//                 }
//
//                 /* Push a "nop" to prevent popping of the + operator (which would
//                  * result in (10+10)/2 beeing evaluated as 15)
//                  */
//                 RpnOp.Type = RpnOpNop;
//                 if (!RpnpPushStack(Stack, &RpnOp))
//                 {
//                     CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 /* Skip closing brace/bracket */
//                 pend++;
//
//                 CharacterOffset += pend - p;
//                 p = pend;
//             }
//             else if (First && p[0] == '-') /* Allow expressions like "- eax" */
//             {
//                 RpnOp.Type = RpnOpImmediate;
//                 RpnOp.CharacterOffset = CharacterOffset;
//                 RpnOp.Data.Immediate = 0;
//
//                 if (!RpnpPushStack(Stack, &RpnOp))
//                 {
//                     CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//             }
//             else
//             {
//                 CONST_STRCPY(ErrMsg, "Operand expected");
//
//                 if (ErrOffset)
//                     *ErrOffset = CharacterOffset;
//
//                 return FALSE;
//             }
//         }
//         else
//         {
//             CONST_STRCPY(ErrMsg, "strcspn() failed");
//
//             if (ErrOffset)
//                 *ErrOffset = -1;
//
//             return FALSE;
//         }
//
//         if (!First)
//         {
//             /* Push operator */
//             RpnOp.CharacterOffset = OperatorOffset;
//             RpnOp.Type = RpnOpBinaryOperator;
//             IsComparativeOp = FALSE;
//
//             switch (*Operator)
//             {
//                 case '+':
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorAdd;
//                     break;
//
//                 case '-':
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorSub;
//                     break;
//
//                 case '*':
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorMul;
//                     break;
//
//                 case '/':
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorDiv;
//                     break;
//
//                 case '%':
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorMod;
//                     break;
//
//                 case '=':
//                     ASSERT(Operator[1] == '=');
//                     IsComparativeOp = TRUE;
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorEquals;
//                     break;
//
//                 case '!':
//                     ASSERT(Operator[1] == '=');
//                     IsComparativeOp = TRUE;
//                     RpnOp.Data.BinaryOperator = RpnBinaryOperatorNotEquals;
//                     break;
//
//                 case '<':
//                     IsComparativeOp = TRUE;
//
//                     if (Operator[1] == '=')
//                         RpnOp.Data.BinaryOperator = RpnBinaryOperatorLessThanOrEquals;
//                     else
//                         RpnOp.Data.BinaryOperator = RpnBinaryOperatorLessThan;
//
//                     break;
//
//                 case '>':
//                     IsComparativeOp = TRUE;
//
//                     if (Operator[1] == '=')
//                         RpnOp.Data.BinaryOperator = RpnBinaryOperatorGreaterThanOrEquals;
//                     else
//                         RpnOp.Data.BinaryOperator = RpnBinaryOperatorGreaterThan;
//
//                     break;
//
//                 default:
//                     ASSERT(0);
//                     break;
//             }
//
//             if (IsComparativeOp)
//             {
//                 if (ComparativeOpFilled && !RpnpPushStack(Stack, &ComparativeOp))
//                 {
//                     CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 memcpy(&ComparativeOp, &RpnOp, sizeof(RPN_OP));
//                 ComparativeOpFilled = TRUE;
//             }
//             else if (!RpnpPushStack(Stack, &RpnOp))
//             {
//                 CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                 if (ErrOffset)
//                     *ErrOffset = -1;
//
//                 return FALSE;
//             }
//
//             /* Push popped operator */
//             if (HavePoppedOperator)
//             {
//                 if (!RpnpPushStack(Stack, &PoppedOperator))
//                 {
//                     CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//             }
//         }
//
//         First = FALSE;
//     }
//
// //end_of_expression:
//
//     if (ComparativeOpFilled && !RpnpPushStack(Stack, &ComparativeOp))
//     {
//         CONST_STRCPY(ErrMsg, "RPN op stack overflow");
//
//         if (ErrOffset)
//             *ErrOffset = -1;
//
//         return FALSE;
//     }
//
//     /* Skip whitespace */
//     while (isspace(*p))
//     {
//         p++;
//         CharacterOffset++;
//     }
//
//     if (End)
//         *End = p;
//
//     return TRUE;
// }
//
// /*!\brief Evaluates the RPN op stack and returns the result.
//  *
//  * \param Stack      Pointer to a RPN_STACK structure.
//  * \param TrapFrame  Register values.
//  * \param Result     Pointer to an ULONG to store the result into.
//  * \param ErrOffset  On failure this is set to the character offset at which the error occoured.
//  * \param ErrMsg     Buffer which receives an error message on failure (128 bytes)
//  *
//  * \retval TRUE   Success.
//  * \retval FALSE  Failure.
//  */
// static BOOLEAN
// RpnpEvaluateStack(
//     IN  PRPN_STACK Stack,
//     IN  PKDB_KTRAP_FRAME TrapFrame,
//     OUT PULONGLONG Result,
//     OUT PLONG ErrOffset  OPTIONAL,
//     OUT PCHAR ErrMsg  OPTIONAL)
// {
//     ULONGLONG ValueStack[RPN_VALUE_STACK_SIZE];
//     ULONG ValueStackPointer = 0;
//     ULONG index;
//     ULONGLONG ull;
//     ULONG ul;
//     USHORT us;
//     UCHAR uc;
//     PVOID p;
//     BOOLEAN Ok;
// #ifdef DEBUG_RPN
//     ULONG ValueStackPointerMax = 0;
// #endif
//
//     ASSERT(Stack);
//     ASSERT(TrapFrame);
//     ASSERT(Result);
//
//     for (index = 0; index < Stack->Sp; index++)
//     {
//         PRPN_OP Op = Stack->Ops + index;
//
// #ifdef DEBUG_RPN
//         ValueStackPointerMax = max(ValueStackPointerMax, ValueStackPointer);
// #endif
//
//         switch (Op->Type)
//         {
//             case RpnOpNop:
//                 /* No operation */
//                 break;
//
//             case RpnOpImmediate:
//                 if (ValueStackPointer == RPN_VALUE_STACK_SIZE)
//                 {
//                     CONST_STRCPY(ErrMsg, "Value stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 ValueStack[ValueStackPointer++] = Op->Data.Immediate;
//                 break;
//
//             case RpnOpRegister:
//                 if (ValueStackPointer == RPN_VALUE_STACK_SIZE)
//                 {
//                     CONST_STRCPY(ErrMsg, "Value stack overflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 ul = Op->Data.Register;
//                 p = (PVOID)((ULONG_PTR)TrapFrame + RegisterToTrapFrame[ul].Offset);
//
//                 switch (RegisterToTrapFrame[ul].Size)
//                 {
//                     case 1: ull = (ULONGLONG)(*(PUCHAR)p); break;
//                     case 2: ull = (ULONGLONG)(*(PUSHORT)p); break;
//                     case 4: ull = (ULONGLONG)(*(PULONG)p); break;
//                     case 8: ull = (ULONGLONG)(*(PULONGLONG)p); break;
//                     default: ASSERT(0); return FALSE; break;
//                 }
//
//                 ValueStack[ValueStackPointer++] = ull;
//                 break;
//
//             case RpnOpDereference:
//                 if (ValueStackPointer < 1)
//                 {
//                     CONST_STRCPY(ErrMsg, "Value stack underflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 /* FIXME: Print a warning when address is out of range */
//                 p = (PVOID)(ULONG_PTR)ValueStack[ValueStackPointer - 1];
//                 Ok = FALSE;
//
//                 switch (Op->Data.DerefMemorySize)
//                 {
//                     case 1:
//                         if (NT_SUCCESS(KdbpSafeReadMemory(&uc, p, sizeof (uc))))
//                         {
//                             Ok = TRUE;
//                             ull = (ULONGLONG)uc;
//                         }
//                         break;
//
//                     case 2:
//                         if (NT_SUCCESS(KdbpSafeReadMemory(&us, p, sizeof (us))))
//                         {
//                             Ok = TRUE;
//                             ull = (ULONGLONG)us;
//                         }
//                         break;
//
//                     case 4:
//                         if (NT_SUCCESS(KdbpSafeReadMemory(&ul, p, sizeof (ul))))
//                         {
//                             Ok = TRUE;
//                             ull = (ULONGLONG)ul;
//                         }
//                         break;
//
//                     case 8:
//                         if (NT_SUCCESS(KdbpSafeReadMemory(&ull, p, sizeof (ull))))
//                         {
//                             Ok = TRUE;
//                         }
//                         break;
//
//                     default:
//                         ASSERT(0);
//                         return FALSE;
//                         break;
//                 }
//
//                 if (!Ok)
//                 {
//                     _snprintf(ErrMsg, 128, "Couldn't access memory at 0x%p", p);
//
//                     if (ErrOffset)
//                         *ErrOffset = Op->CharacterOffset;
//
//                     return FALSE;
//                 }
//
//                 ValueStack[ValueStackPointer - 1] = ull;
//                 break;
//
//             case RpnOpBinaryOperator:
//                 if (ValueStackPointer < 2)
//                 {
//                     CONST_STRCPY(ErrMsg, "Value stack underflow");
//
//                     if (ErrOffset)
//                         *ErrOffset = -1;
//
//                     return FALSE;
//                 }
//
//                 ValueStackPointer--;
//                 ull = ValueStack[ValueStackPointer];
//
//                 if (ull == 0 && Op->Data.BinaryOperator == RpnBinaryOperatorDiv)
//                 {
//                     CONST_STRCPY(ErrMsg, "Division by zero");
//
//                     if (ErrOffset)
//                         *ErrOffset = Op->CharacterOffset;
//
//                     return FALSE;
//                 }
//
//                 ull = Op->Data.BinaryOperator(ValueStack[ValueStackPointer - 1], ull);
//                 ValueStack[ValueStackPointer - 1] = ull;
//                 break;
//
//             default:
//                 ASSERT(0);
//                 return FALSE;
//         }
//     }
//
// #ifdef DEBUG_RPN
//     DPRINT1("Max value stack pointer: %d\n", ValueStackPointerMax);
// #endif
//
//     if (ValueStackPointer != 1)
//     {
//         CONST_STRCPY(ErrMsg, "Stack not empty after evaluation");
//
//         if (ErrOffset)
//             *ErrOffset = -1;
//
//         return FALSE;
//     }
//
//     *Result = ValueStack[0];
//     return TRUE;
// }
//
// /*!\brief Evaluates the given expression
//  *
//  * \param Expression  Expression to evaluate.
//  * \param TrapFrame   Register values.
//  * \param Result      Variable which receives the result on success.
//  * \param ErrOffset   Variable which receives character offset on parse error (-1 on other errors)
//  * \param ErrMsg      Buffer which receives an error message on failure (128 bytes)
//  *
//  * \retval TRUE   Success.
//  * \retval FALSE  Failure.
//  */
// BOOLEAN
// KdbpRpnEvaluateExpression(
//     IN  PCHAR Expression,
//     IN  PKDB_KTRAP_FRAME TrapFrame,
//     OUT PULONGLONG Result,
//     OUT PLONG ErrOffset  OPTIONAL,
//     OUT PCHAR ErrMsg  OPTIONAL)
// {
//     PRPN_STACK Stack = (PRPN_STACK)&RpnStack;
//
//     ASSERT(Expression);
//     ASSERT(TrapFrame);
//     ASSERT(Result);
//
//     /* Clear the stack and parse the expression */
//     RpnpClearStack(Stack);
//     if (!RpnpParseExpression(Stack, Expression, NULL, 0, ErrOffset, ErrMsg))
//         return FALSE;
//
// #ifdef DEBUG_RPN
//     RpnpDumpStack(Stack);
// #endif
//
//     /* Evaluate the stack */
//     if (!RpnpEvaluateStack(Stack, TrapFrame, Result, ErrOffset, ErrMsg))
//         return FALSE;
//
//     return TRUE;
// }
//
// /*!\brief Parses the given expression and returns a "handle" to it.
//  *
//  * \param Expression  Expression to evaluate.
//  * \param ErrOffset   Variable which receives character offset on parse error (-1 on other errors)
//  * \param ErrMsg      Buffer which receives an error message on failure (128 bytes)
//  *
//  * \returns "Handle" for the expression, NULL on failure.
//  *
//  * \sa KdbpRpnEvaluateExpression
//  */
// PVOID
// KdbpRpnParseExpression(
//     IN  PCHAR Expression,
//     OUT PLONG ErrOffset  OPTIONAL,
//     OUT PCHAR ErrMsg  OPTIONAL)
// {
//     LONG Size;
//     PRPN_STACK Stack = (PRPN_STACK)&RpnStack;
//     PRPN_STACK NewStack;
//
//     ASSERT(Expression);
//
//     /* Clear the stack and parse the expression */
//     RpnpClearStack(Stack);
//     if (!RpnpParseExpression(Stack, Expression, NULL, 0, ErrOffset, ErrMsg))
//         return FALSE;
//
// #ifdef DEBUG_RPN
//     RpnpDumpStack(Stack);
// #endif
//
//     /* Duplicate the stack and return a pointer/handle to it */
//     ASSERT(Stack->Sp >= 1);
//     Size = sizeof (RPN_STACK) + (RTL_FIELD_SIZE(RPN_STACK, Ops[0]) * (Stack->Sp - 1));
//     NewStack = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_KDBG);
//
//     if (!NewStack)
//     {
//         CONST_STRCPY(ErrMsg, "Out of memory");
//
//         if (ErrOffset)
//             *ErrOffset = -1;
//
//         return NULL;
//     }
//
//     memcpy(NewStack, Stack, Size);
//     NewStack->Size = NewStack->Sp;
//
//     return NewStack;
// }
//
// /*!\brief Evaluates the given expression and returns the result.
//  *
//  * \param Expression  Expression "handle" returned by KdbpRpnParseExpression.
//  * \param TrapFrame   Register values.
//  * \param Result      Variable which receives the result on success.
//  * \param ErrOffset   Variable which receives character offset on parse error (-1 on other errors)
//  * \param ErrMsg      Buffer which receives an error message on failure (128 bytes)
//  *
//  * \returns "Handle" for the expression, NULL on failure.
//  *
//  * \sa KdbpRpnParseExpression
//  */
// BOOLEAN
// KdbpRpnEvaluateParsedExpression(
//     IN  PVOID Expression,
//     IN  PKDB_KTRAP_FRAME TrapFrame,
//     OUT PULONGLONG Result,
//     OUT PLONG ErrOffset  OPTIONAL,
//     OUT PCHAR ErrMsg  OPTIONAL)
// {
//     PRPN_STACK Stack = (PRPN_STACK)Expression;
//
//     ASSERT(Expression);
//     ASSERT(TrapFrame);
//     ASSERT(Result);
//
//     /* Evaluate the stack */
//     return RpnpEvaluateStack(Stack, TrapFrame, Result, ErrOffset, ErrMsg);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
