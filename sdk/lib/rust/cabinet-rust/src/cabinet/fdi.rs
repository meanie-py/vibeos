use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};
use crate::types::*;

// ---- file-scope type preamble (translated) ----
type cab_UBYTE = u8;
type cab_UWORD = u16;
type cab_ULONG = u32;
type cab_LONG = i32;
type cab_off_t = u32;

#[repr(C)]
pub struct fdi_file {
    pub next: *mut fdi_file,
    pub filename: LPSTR,
    pub fh: i32,
    pub length: cab_ULONG,
    pub offset: cab_ULONG,
    pub index: cab_UWORD,
    pub time: cab_UWORD,
    pub date: cab_UWORD,
    pub attribs: cab_UWORD,
    pub oppressed: BOOL,
}

#[repr(C)]
pub struct fdi_folder {
    pub next: *mut fdi_folder,
    pub offset: cab_off_t,
    pub comp_type: cab_UWORD,
    pub comp_size: cab_ULONG,
    pub num_splits: cab_UBYTE,
    pub num_blocks: cab_UWORD,
}

#[repr(C)]
pub struct MORE_ISCAB_INFO {
    pub prevname: *mut i8,
    pub previnfo: *mut i8,
    pub nextname: *mut i8,
    pub nextinfo: *mut i8,
    pub hasnext: BOOL,
    pub folder_resv: i32,
    pub header_resv: i32,
    pub block_resv: cab_UBYTE,
}
pub type PMORE_ISCAB_INFO = *mut MORE_ISCAB_INFO;

#[repr(C)]
pub struct FDI_Int {
    pub magic: u32,
    pub alloc: PFNALLOC,
    pub free: PFNFREE,
    pub open: PFNOPEN,
    pub read: PFNREAD,
    pub write: PFNWRITE,
    pub close: PFNCLOSE,
    pub seek: PFNSEEK,
    pub perf: PERF,
}

pub const FDI_INT_MAGIC: u32 = 0xfdfdfd05;

#[repr(C)]
pub union Methods {
    pub zip: ZIPstate,
    pub qtm: QTMstate,
    pub lzx: LZXstate,
}

#[repr(C)]
pub struct fdi_cds_fwd {
    pub fdi: *mut FDI_Int,
    pub filehf: INT_PTR,
    pub cabhf: INT_PTR,
    pub current: *mut fdi_folder,
    pub offset: cab_ULONG,
    pub outpos: *mut cab_UBYTE,
    pub outlen: cab_UWORD,
    pub decompress: Option<unsafe extern "C" fn(i32, i32, *mut fdi_cds_fwd) -> i32>,
    pub inbuf: [cab_UBYTE; CAB_INPUTMAX as usize + 2],
    pub outbuf: [cab_UBYTE; CAB_BLOCKMAX as usize],
    pub methods: Methods,
    pub q_length_base: [cab_UBYTE; 27],
    pub q_length_extra: [cab_UBYTE; 27],
    pub q_extra_bits: [cab_UBYTE; 42],
    pub q_position_base: [cab_ULONG; 42],
    pub lzx_position_base: [cab_ULONG; 51],
    pub extra_bits: [cab_UBYTE; 51],
    pub setID: USHORT,
    pub iCabinet: USHORT,
    pub decomp_cab: *mut fdi_cds_fwd,
    pub mii: MORE_ISCAB_INFO,
    pub firstfol: *mut fdi_folder,
    pub firstfile: *mut fdi_file,
    pub next: *mut fdi_cds_fwd,
}

pub static mut fdi_decomp_state: fdi_cds_fwd = unsafe { core::mem::zeroed() };
pub type fdi_decomp_state = fdi_cds_fwd;

pub const DECR_OK: i32 = 0;
pub const DECR_DATAFORMAT: i32 = 1;
pub const DECR_ILLEGALDATA: i32 = 2;
pub const DECR_NOMEMORY: i32 = 3;
pub const DECR_CHECKSUM: i32 = 4;
pub const DECR_INPUT: i32 = 5;
pub const DECR_OUTPUT: i32 = 6;
pub const DECR_USERABORT: i32 = 7;

// ---- consts / statics / type aliases (deduped) ----
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;
const ERROR_INVALID_HANDLE: u32 = 6;
const ERROR_BAD_ARGUMENTS: u32 = 160;
const TRUE: i32 = 1;
const CAB_BLOCKMAX: i32 = 32768; // typical value

extern "C" {
    fn strlen(s: *const i8) -> usize;
}

// ============================================================================
// Bitstream / Huffman engine — faithful port of the cabinet.h bit-reader macros
// (Q_*/READ_BITS/READ_HUFFSYM/BUILD_TABLE/GET_SYMBOL/READ_LENGTHS).
//
// The C macros mutate the enclosing function's `bitbuf` / `bitsleft` / `inpos`
// locals in place, and the table macros `return DECR_ILLEGALDATA` from the
// enclosing function on error. Rust `macro_rules!` are hygienic, so the
// bit-state is threaded explicitly as place-expression arguments; the early
// `return` is preserved because the macro body expands into the caller.
//
// Runtime shift counts use wrapping_shl / wrapping_shr, which mask the count to
// 5 bits exactly as x86 SHL/SHR do — and exactly as the original C lowered to.
// ============================================================================

const CAB_ULONG_BITS: i32 = 32;

// LZX bitstream (Intel little-endian byte order): (inpos[1]<<8)|inpos[0]
macro_rules! ENSURE_BITS {
    ($bb:expr, $bl:expr, $ip:expr, $n:expr) => {
        while $bl < ($n as i32) {
            $bb |= (((*$ip.add(1) as u32) << 8) | (*$ip.add(0) as u32))
                       .wrapping_shl((CAB_ULONG_BITS - 16 - $bl) as u32);
            $bl += 16;
            $ip = $ip.add(2);
        }
    };
}

// READ_BITS(var, n): take n bits (n may be 0)
macro_rules! READ_BITS {
    ($v:expr, $n:expr, $bb:expr, $bl:expr, $ip:expr) => {{
        let __n: i32 = $n as i32;
        if __n != 0 {
            ENSURE_BITS!($bb, $bl, $ip, __n);
            $v = ($bb.wrapping_shr((CAB_ULONG_BITS - __n) as u32)) as _; // PEEK_BITS
            $bb = $bb.wrapping_shl(__n as u32);                          // REMOVE_BITS
            $bl -= __n;
        } else {
            $v = 0 as _;
        }
    }};
}

// Quantum bitstream (normal byte order): (inpos[0]<<8)|inpos[1]
macro_rules! Q_FILL_BUFFER {
    ($bb:expr, $bl:expr, $ip:expr) => {
        if $bl <= (CAB_ULONG_BITS - 16) {
            $bb |= (((*$ip.add(0) as u32) << 8) | (*$ip.add(1) as u32))
                       .wrapping_shl((CAB_ULONG_BITS - 16 - $bl) as u32);
            $bl += 16;
            $ip = $ip.add(2);
        }
    };
}

// Q_READ_BITS(var, n): accumulate in u32 then store (avoids debug shift panics)
macro_rules! Q_READ_BITS {
    ($v:expr, $n:expr, $bb:expr, $bl:expr, $ip:expr) => {{
        let mut __acc: u32 = 0;
        let mut __need: i32 = $n as i32;
        while __need != 0 {
            Q_FILL_BUFFER!($bb, $bl, $ip);
            let __run: i32 = if __need > $bl { $bl } else { __need };
            __acc = __acc.wrapping_shl(__run as u32)
                    | $bb.wrapping_shr((CAB_ULONG_BITS - __run) as u32); // Q_PEEK_BITS
            $bb = $bb.wrapping_shl(__run as u32);                        // Q_REMOVE_BITS
            $bl -= __run;
            __need -= __run;
        }
        $v = __acc as _;
    }};
}

// GET_SYMBOL(model, var): Quantum arithmetic-coder symbol fetch. C/H/L persist.
macro_rules! GET_SYMBOL {
    ($model:expr, $var:expr, $bb:expr, $bl:expr, $ip:expr, $cc:expr, $hh:expr, $ll:expr) => {{
        let __m: *mut QTMmodel = $model;
        let __freq0: u32 = (*(*__m).syms.add(0)).cumfreq as u32;
        let mut __range: u32 = ((($hh as i32 - $ll as i32) & 0xFFFF) + 1) as u32;
        let __cl1: u32 = ($cc as u32).wrapping_sub($ll as u32).wrapping_add(1);
        let __symf: u32 = ((__cl1.wrapping_mul(__freq0)).wrapping_sub(1) / __range) & 0xFFFF;
        let mut __i: i32 = 1;
        while __i < (*__m).entries {
            if (*(*__m).syms.add(__i as usize)).cumfreq as u32 <= __symf { break; }
            __i += 1;
        }
        $var = (*(*__m).syms.add((__i - 1) as usize)).sym as _;
        __range = ($hh as i32 - $ll as i32 + 1) as u32;
        let __freq_im1: u32 = (*(*__m).syms.add((__i - 1) as usize)).cumfreq as u32;
        let __freq_i: u32 = (*(*__m).syms.add(__i as usize)).cumfreq as u32;
        let __oldl: u32 = $ll as u32;
        $hh = __oldl.wrapping_add(__freq_im1.wrapping_mul(__range) / __freq0).wrapping_sub(1) as u16;
        $ll = __oldl.wrapping_add(__freq_i.wrapping_mul(__range) / __freq0) as u16;
        loop {
            if ($ll & 0x8000) != ($hh & 0x8000) {
                if ($ll & 0x4000) != 0 && ($hh & 0x4000) == 0 {
                    // underflow case
                    $cc ^= 0x4000;
                    $ll &= 0x3FFF;
                    $hh |= 0x4000;
                } else {
                    break;
                }
            }
            $ll = ($ll as u32).wrapping_shl(1) as u16;
            $hh = (($hh as u32).wrapping_shl(1) | 1) as u16;
            Q_FILL_BUFFER!($bb, $bl, $ip);
            $cc = (($cc as u32).wrapping_shl(1) | $bb.wrapping_shr(31)) as u16; // Q_PEEK_BITS(1)
            $bb = $bb.wrapping_shl(1);
            $bl -= 1;
        }
        QTMupdatemodel(__m, __i);
    }};
}

// BUILD_TABLE: make_decode_table(maxsymbols, tablebits, lentable, symtable)
macro_rules! BUILD_TABLE {
    ($maxsym:expr, $tablebits:expr, $lentable:expr, $symtable:expr) => {
        if make_decode_table($maxsym as u32, $tablebits as u32,
                              $lentable as *const u8, $symtable as *mut u16) != 0 {
            return DECR_ILLEGALDATA;
        }
    };
}

// READ_HUFFSYM(maxsymbols, tablebits, symtable, lentable, var): decode one symbol
macro_rules! READ_HUFFSYM {
    ($maxsym:expr, $tablebits:expr, $symtable:expr, $lentable:expr,
     $var:expr, $bb:expr, $bl:expr, $ip:expr) => {{
        ENSURE_BITS!($bb, $bl, $ip, 16);
        let __huff: *const u16 = $symtable as *const u16;
        let __tb: i32 = $tablebits as i32;
        let __maxs: u32 = $maxsym as u32;
        let mut __i: u32 = *__huff.add($bb.wrapping_shr((CAB_ULONG_BITS - __tb) as u32) as usize) as u32;
        if __i >= __maxs {
            let mut __j: u32 = 1u32.wrapping_shl((CAB_ULONG_BITS - __tb) as u32);
            loop {
                __j >>= 1;
                __i <<= 1;
                __i |= if ($bb & __j) != 0 { 1 } else { 0 };
                if __j == 0 { return DECR_ILLEGALDATA; }
                __i = *__huff.add(__i as usize) as u32;
                if __i < __maxs { break; }
            }
        }
        let __len: u32 = *($lentable as *const u8).add(__i as usize) as u32;
        $var = __i as _;
        $bb = $bb.wrapping_shl(__len);  // REMOVE_BITS(len)
        $bl -= __len as i32;
    }};
}

// READ_LENGTHS(lentable, first, last): save bit-state, call read_lens, restore
macro_rules! READ_LENGTHS {
    ($lentable:expr, $first:expr, $last:expr, $lb:expr, $bb:expr, $bl:expr, $ip:expr, $ds:expr) => {{
        $lb.bb = $bb;
        $lb.bl = $bl;
        $lb.ip = $ip;
        if fdi_lzx_read_lens($lentable as *mut u8, $first as u32, $last as u32, &mut $lb, $ds) != 0 {
            return DECR_ILLEGALDATA;
        }
        $bb = $lb.bb;
        $bl = $lb.bl;
        $ip = $lb.ip;
    }};
}

// ---- LZX constants (cabinet.h) ----
const LZX_MIN_MATCH: i32 = 2;
const LZX_NUM_CHARS: u32 = 256;
const LZX_BLOCKTYPE_VERBATIM: u16 = 1;
const LZX_BLOCKTYPE_ALIGNED: u16 = 2;
const LZX_BLOCKTYPE_UNCOMPRESSED: u16 = 3;
const LZX_NUM_PRIMARY_LENGTHS: i32 = 7;
const LZX_NUM_SECONDARY_LENGTHS: u32 = 249;
const LZX_PRETREE_MAXSYMBOLS: u32 = 20;
const LZX_PRETREE_TABLEBITS: u32 = 6;
const LZX_MAINTREE_MAXSYMBOLS: u32 = LZX_NUM_CHARS + 50 * 8;
const LZX_MAINTREE_TABLEBITS: u32 = 12;
const LZX_LENGTH_MAXSYMBOLS: u32 = LZX_NUM_SECONDARY_LENGTHS + 1;
const LZX_LENGTH_TABLEBITS: u32 = 12;
const LZX_ALIGNED_MAXSYMBOLS: u32 = 8;
const LZX_ALIGNED_TABLEBITS: u32 = 7;

// PFNFDINOTIFY — the FDICopy progress/extract callback ABI (non-null in practice)
pub type PFNFDINOTIFY =
    Option<unsafe extern "C" fn(fdint: FDINOTIFICATIONTYPE, pfdin: *mut FDINOTIFICATION) -> isize>;

extern "system" {
    fn SetLastError(dwErrCode: u32);
}
extern "C" {
    // debugstr_a is a cdecl shim provided by rust_cohab_stub.c (_debugstr_a);
    // must NOT be extern "system" or it resolves to the missing _debugstr_a@4.
    fn debugstr_a(s: *const i8) -> *const i8;
}

unsafe fn set_error(fdi: *mut FDI_Int, oper: i32, err: i32) {
    (*(*fdi).perf).erfOper = oper;
    (*(*fdi).perf).erfType = err;
    (*(*fdi).perf).fError = 1;
    if err != 0 {
        SetLastError(err as u32);
    }
}

unsafe fn get_fdi_ptr(hfdi: HFDI) -> *mut FDI_Int {
    let fdi: *mut FDI_Int = hfdi as *mut FDI_Int;
    if fdi.is_null() || (*fdi).magic != FDI_INT_MAGIC {
        SetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }
    fdi
}

unsafe fn QTMupdatemodel(model: *mut QTMmodel, sym: i32) {
    let mut temp: QTMmodelsym;
    let mut i: i32;
    let mut j: i32;

    i = 0;
    while i < sym {
        (*(*model).syms.add(i as usize)).cumfreq += 8;
        i += 1;
    }

    if (*(*model).syms.add(0usize)).cumfreq > 3800 {
        (*model).shiftsleft = (*model).shiftsleft - 1;
        if (*model).shiftsleft != 0 {
            i = (*model).entries - 1;
            while i >= 0 {
                (*(*model).syms.add(i as usize)).cumfreq >>= 1;
                if (*(*model).syms.add(i as usize)).cumfreq <= (*(*model).syms.add((i + 1) as usize)).cumfreq {
                    (*(*model).syms.add(i as usize)).cumfreq = (*(*model).syms.add((i + 1) as usize)).cumfreq + 1;
                }
                i -= 1;
            }
        } else {
            (*model).shiftsleft = 50;
            i = 0;
            while i < (*model).entries {
                (*(*model).syms.add(i as usize)).cumfreq -= (*(*model).syms.add((i + 1) as usize)).cumfreq;
                (*(*model).syms.add(i as usize)).cumfreq += 1;
                (*(*model).syms.add(i as usize)).cumfreq >>= 1;
                i += 1;
            }

            i = 0;
            while i < (*model).entries - 1 {
                j = i + 1;
                while j < (*model).entries {
                    if (*(*model).syms.add(i as usize)).cumfreq < (*(*model).syms.add(j as usize)).cumfreq {
                        temp = *(*model).syms.add(i as usize);
                        *(*model).syms.add(i as usize) = *(*model).syms.add(j as usize);
                        *(*model).syms.add(j as usize) = temp;
                    }
                    j += 1;
                }
                i += 1;
            }

            i = (*model).entries - 1;
            while i >= 0 {
                (*(*model).syms.add(i as usize)).cumfreq += (*(*model).syms.add((i + 1) as usize)).cumfreq;
                i -= 1;
            }

            i = 0;
            while i < (*model).entries {
                let sym_val = (*(*model).syms.add(i as usize)).sym;
                *((*model).tabloc.as_mut_ptr().add(sym_val as usize)) = i as u16;
                i += 1;
            }
        }
    }
}

unsafe fn make_decode_table(nsyms: u32, nbits: u32, length: *const u8, table: *mut u16) -> i32 {
    let mut sym: u16;
    let mut leaf: u32;
    let mut bit_num: u8 = 1;
    let mut fill: u32;
    let mut pos: u32 = 0;
    let mut table_mask: u32 = 1 << nbits;
    let mut bit_mask: u32 = table_mask >> 1;
    let mut next_symbol: u32 = bit_mask;

    while (bit_num as u32) <= nbits {
        sym = 0;
        while (sym as u32) < nsyms {
            if *length.add(sym as usize) == bit_num {
                leaf = pos;
                pos += bit_mask;
                if pos > table_mask {
                    return 1;
                }
                fill = bit_mask;
                while {
                    let prev_fill = fill;
                    fill -= 1;
                    prev_fill > 0
                } {
                    *table.add(leaf as usize) = sym;
                    leaf += 1;
                }
            }
            sym += 1;
        }
        bit_mask >>= 1;
        bit_num += 1;
    }

    if pos != table_mask {
        sym = pos as u16;
        while (sym as u32) < table_mask {
            *table.add(sym as usize) = 0;
            sym += 1;
        }

        pos <<= 16;
        table_mask <<= 16;
        bit_mask = 1 << 15;

        while bit_num <= 16 {
            sym = 0;
            while (sym as u32) < nsyms {
                if *length.add(sym as usize) == bit_num {
                    leaf = pos >> 16;
                    fill = 0;
                    while fill < (bit_num as u32) - nbits {
                        if *table.add(leaf as usize) == 0 {
                            *table.add((next_symbol << 1) as usize) = 0;
                            *table.add(((next_symbol << 1) + 1) as usize) = 0;
                            *table.add(leaf as usize) = next_symbol as u16;
                            next_symbol += 1;
                        }
                        leaf = (*table.add(leaf as usize) as u32) << 1;
                        if ((pos >> (15 - fill)) & 1) != 0 {
                            leaf += 1;
                        }
                        fill += 1;
                    }
                    *table.add(leaf as usize) = sym;

                    pos += bit_mask;
                    if pos > table_mask {
                        return 1;
                    }
                }
                sym += 1;
            }
            bit_mask >>= 1;
            bit_num += 1;
        }
    }

    if pos == table_mask {
        return 0;
    }

    sym = 0;
    while (sym as u32) < nsyms {
        if *length.add(sym as usize) != 0 {
            return 1;
        }
        sym += 1;
    }
    0
}

unsafe fn checksum(mut data: *const u8, bytes: u16, mut csum: u32) -> u32 {
    let mut len: i32;
    let mut ul: u32 = 0;

    len = (bytes >> 2) as i32;
    while {
        let current = len;
        len -= 1;
        current != 0
    } {
        csum ^= ((*data as u32) | ((*data.add(1) as u32) << 8) | ((*data.add(2) as u32) << 16) | ((*data.add(3) as u32) << 24));
        data = data.add(4);
    }

    match bytes & 3 {
        3 => {
            ul |= (*data as u32) << 16;
            data = data.add(1);
            ul |= (*data as u32) << 8;
            data = data.add(1);
            ul |= *data as u32;
        }
        2 => {
            ul |= (*data as u32) << 8;
            data = data.add(1);
            ul |= *data as u32;
        }
        1 => {
            ul |= *data as u32;
        }
        _ => {}
    }
    csum ^= ul;

    csum
}

#[no_mangle]
pub unsafe extern "C" fn FDICreate(
    pfnalloc: PFNALLOC,
    pfnfree: PFNFREE,
    pfnopen: PFNOPEN,
    pfnread: PFNREAD,
    pfnwrite: PFNWRITE,
    pfnclose: PFNCLOSE,
    pfnseek: PFNSEEK,
    cpuType: i32,
    perf: PERF,
) -> *mut FDI_Int {
    winedebug_rust::TRACE!("(pfnalloc == ^%p, pfnfree == ^%p, pfnopen == ^%p, pfnread == ^%p, pfnwrite == ^%p, pfnclose == ^%p, pfnseek == ^%p, cpuType == %d, perf == ^%p)\n", pfnalloc, pfnfree, pfnopen, pfnread, pfnwrite, pfnclose, pfnseek, cpuType, perf);

    if pfnalloc.is_none() || pfnfree.is_none() {
        (*perf).erfOper = FDIERROR_NONE;
        (*perf).erfType = ERROR_BAD_ARGUMENTS as i32;
        (*perf).fError = TRUE;

        SetLastError(ERROR_BAD_ARGUMENTS);
        return core::ptr::null_mut();
    }

    let fdi = (pfnalloc.unwrap_unchecked())(core::mem::size_of::<FDI_Int>() as u32) as *mut FDI_Int;
    if fdi.is_null() {
        (*perf).erfOper = FDIERROR_ALLOC_FAIL;
        (*perf).erfType = 0;
        (*perf).fError = TRUE;
        return core::ptr::null_mut();
    }

    (*fdi).magic = 0xfdfdfd05u32; // FDI_INT_MAGIC
    (*fdi).alloc = pfnalloc;
    (*fdi).free = pfnfree;
    (*fdi).open = pfnopen;
    (*fdi).read = pfnread;
    (*fdi).write = pfnwrite;
    (*fdi).close = pfnclose;
    (*fdi).seek = pfnseek;
    (*fdi).perf = perf;

    fdi
}

unsafe fn FDI_getoffset(fdi: *mut FDI_Int, hf: INT_PTR) -> i32 {
    (((*fdi).seek).unwrap_unchecked())(hf as isize, 0, SEEK_CUR)
}

unsafe fn QTMfdi_initmodel(m: *mut QTMmodel, sym: *mut QTMmodelsym, n: i32, s: i32) {
    (*m).shiftsleft = 4;
    (*m).entries = n;
    (*m).syms = sym;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*m).tabloc) as *mut u8,
        0xFF,
        core::mem::size_of_val(&(*m).tabloc),
    );
    for i in 0..n {
        (*m).tabloc[(i + s) as usize] = i as u16;
        let sym_ptr = (*m).syms.offset(i as isize);
        (*sym_ptr).sym = (i + s) as u16;
        (*sym_ptr).cumfreq = (n - i) as u16;
    }
    let sym_n = (*m).syms.offset(n as isize);
    (*sym_n).cumfreq = 0;
}

unsafe extern "C" fn LZXfdi_init(window: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    unsafe {
        const BITS: [u8; 51] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
            7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14,
            15, 15, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
            17, 17, 17,
        ];
        const BASE: [u32; 51] = [
            0, 1, 2, 3, 4, 6, 8, 12,
            16, 24, 32, 48, 64, 96, 128, 192,
            256, 384, 512, 768, 1024, 1536, 2048, 3072,
            4096, 6144, 8192, 12288, 16384, 24576, 32768, 49152,
            65536, 98304, 131072, 196608, 262144, 393216, 524288, 655360,
            786432, 917504, 1048576, 1179648, 1310720, 1441792, 1572864, 1703936,
            1835008, 1966080, 2097152,
        ];
        const DECR_DATAFORMAT: i32 = 1;
        const DECR_NOMEMORY: i32 = 3;
        const DECR_OK: i32 = 0;
        // cabinet.h:186 `#define LZX_NUM_CHARS (256)`. It is very much critical: this
        // function computes main_elements = LZX_NUM_CHARS + (posn_slots << 3), and the
        // decoder splits literal from match on `main_element < LZX_NUM_CHARS` using the
        // correct file-scope 256. A 0 here sized the main tree 256 symbols short.
        const LZX_NUM_CHARS: u32 = 256;
        const LZX_BLOCKTYPE_INVALID: i32 = 0;

        let wndsize: u32 = 1u32 << (window as u32);
        let mut posn_slots: i32;

        if window < 15 || window > 21 {
            return DECR_DATAFORMAT;
        }

        if (*decomp_state).methods.lzx.actual_size < wndsize {
            if !(*decomp_state).methods.lzx.window.is_null() {
                (((*(*decomp_state).fdi).free).unwrap_unchecked())((*decomp_state).methods.lzx.window as *mut core::ffi::c_void);
            }
            (*decomp_state).methods.lzx.window = core::ptr::null_mut();
        }

        if (*decomp_state).methods.lzx.window.is_null() {
            let mem = (((*(*decomp_state).fdi).alloc).unwrap_unchecked())(wndsize as u32) as *mut u8;
            if mem.is_null() {
                return DECR_NOMEMORY;
            }
            (*decomp_state).methods.lzx.window = mem;
            (*decomp_state).methods.lzx.actual_size = wndsize;
        }

        (*decomp_state).methods.lzx.window_size = wndsize;

        core::ptr::copy_nonoverlapping(
            BITS.as_ptr(),
            core::ptr::addr_of_mut!((*decomp_state).extra_bits).cast::<u8>(),
            51,
        );
        core::ptr::copy_nonoverlapping(
            BASE.as_ptr(),
            core::ptr::addr_of_mut!((*decomp_state).lzx_position_base).cast::<u32>(),
            51,
        );

        if window == 20 {
            posn_slots = 42;
        } else if window == 21 {
            posn_slots = 50;
        } else {
            posn_slots = window << 1;
        }

        /* posn_slots=i=0; while (i < wndsize) i += 1 << CAB(extra_bits)[posn_slots++]; */

        let one: i32 = 1;
        (*decomp_state).methods.lzx.R2 = 1;
        (*decomp_state).methods.lzx.R1 = 1;
        (*decomp_state).methods.lzx.R0 = 1;

        (*decomp_state).methods.lzx.main_elements = (LZX_NUM_CHARS + ((posn_slots as u32) << 3)) as u16;
        (*decomp_state).methods.lzx.header_read = 0;
        (*decomp_state).methods.lzx.frames_read = 0;
        (*decomp_state).methods.lzx.block_remaining = 0;
        (*decomp_state).methods.lzx.block_type = LZX_BLOCKTYPE_INVALID as u16;
        (*decomp_state).methods.lzx.intel_curpos = 0;
        (*decomp_state).methods.lzx.intel_started = 0;
        (*decomp_state).methods.lzx.window_posn = 0;

        let size_MAINTREE = core::mem::size_of_val(&(*decomp_state).methods.lzx.MAINTREE_len);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*decomp_state).methods.lzx.MAINTREE_len).cast::<u8>(),
            0,
            size_MAINTREE,
        );

        let size_LENGTH = core::mem::size_of_val(&(*decomp_state).methods.lzx.LENGTH_len);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*decomp_state).methods.lzx.LENGTH_len).cast::<u8>(),
            0,
            size_LENGTH,
        );

        DECR_OK
    }
}

unsafe extern "C" fn NONEfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    if inlen != outlen {
        return DECR_ILLEGALDATA;
    }
    if outlen > CAB_BLOCKMAX {
        return DECR_DATAFORMAT;
    }
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*decomp_state).inbuf) as *const u8,
        core::ptr::addr_of_mut!((*decomp_state).outbuf) as *mut u8,
        inlen as usize,
    );
    DECR_OK
}

unsafe fn fdi_Ziphuft_free(fdi: *mut FDI_Int, t: *mut Ziphuft) {
    let mut p: *mut Ziphuft = t;
    let mut q: *mut Ziphuft;
    // Go through linked list, freeing from the allocated (t[-1]) address.
    while p != core::ptr::null_mut() {
        p = p.sub(1);
        q = (*p).v.t;
        (((*fdi).free).unwrap_unchecked())(p as *mut core::ffi::c_void);
        p = q;
    }
}

unsafe fn fdi_Zipinflate_codes(tl: *const Ziphuft, td: *const Ziphuft, bl: i32, bd: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    // wine min()/max() are macros and Zipmask is a cabinet.h static table — the
    // mechanical translation left them as undefined externs. Provide locally.
    const Zipmask: [u32; 17] = [
        0x0000, 0x0001, 0x0003, 0x0007, 0x000f, 0x001f, 0x003f, 0x007f,
        0x00ff, 0x01ff, 0x03ff, 0x07ff, 0x0fff, 0x1fff, 0x3fff, 0x7fff, 0xffff,
    ];
    fn max(a: u32, b: u32) -> u32 { if a > b { a } else { b } }
    fn min(a: u32, b: u32) -> u32 { if a < b { a } else { b } }
    const ZIPWSIZE: u32 = 0x8000;

    let mut e: u32;
    let mut n: u32;
    let mut d: u32;
    let mut w: u32;
    let mut t: *const Ziphuft;
    let mut ml: u32;
    let mut md: u32;
    let mut b: u32;
    let mut k: u32;

    b = (*decomp_state).methods.zip.bb;
    k = (*decomp_state).methods.zip.bk;
    w = (*decomp_state).methods.zip.window_posn;

    ml = Zipmask[bl as usize];
    md = Zipmask[bd as usize];

    'main: loop {
        while k < (bl as u32) {
            let c = core::ptr::read((*decomp_state).methods.zip.inpos);
            (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
            b |= (c as u32) << k;
            k += 8;
        }
        t = tl.add((b & ml) as usize);
        e = (*t).e as u32;
        if e > 16 {
            loop {
                if e == 99 {
                    return 1;
                }
                b >>= (*t).b as u32;
                k -= (*t).b as u32;
                e -= 16;
                while k < e {
                    let c = core::ptr::read((*decomp_state).methods.zip.inpos);
                    (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                    b |= (c as u32) << k;
                    k += 8;
                }
                t = (*t).v.t;
                t = t.add((b & Zipmask[e as usize]) as usize);
                e = (*t).e as u32;
                if e <= 16 {
                    break;
                }
            }
        }
        b >>= (*t).b as u32;
        k -= (*t).b as u32;

        if e == 16 {
            (*decomp_state).outbuf[w as usize] = (*t).v.n as u8;
            w += 1;
        } else {
            /* it's an EOB or a length */
            if e == 15 {
                break 'main;
            }

            /* get length of block to copy */
            while k < e {
                let c = core::ptr::read((*decomp_state).methods.zip.inpos);
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            n = (*t).v.n as u32 + (b & Zipmask[e as usize]);
            b >>= e;
            k -= e;

            /* decode distance of block to copy */
            while k < (bd as u32) {
                let c = core::ptr::read((*decomp_state).methods.zip.inpos);
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            t = td.add((b & md) as usize);
            e = (*t).e as u32;
            if e > 16 {
                loop {
                    if e == 99 {
                        return 1;
                    }
                    b >>= (*t).b as u32;
                    k -= (*t).b as u32;
                    e -= 16;
                    while k < e {
                        let c = core::ptr::read((*decomp_state).methods.zip.inpos);
                        (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                        b |= (c as u32) << k;
                        k += 8;
                    }
                    t = (*t).v.t;
                    t = t.add((b & Zipmask[e as usize]) as usize);
                    e = (*t).e as u32;
                    if e <= 16 {
                        break;
                    }
                }
            }
            b >>= (*t).b as u32;
            k -= (*t).b as u32;
            while k < e {
                let c = core::ptr::read((*decomp_state).methods.zip.inpos);
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            d = w - (*t).v.n as u32 - (b & Zipmask[e as usize]);
            b >>= e;
            k -= e;

            loop {
                d &= ZIPWSIZE - 1;
                e = ZIPWSIZE - max(d, w);
                e = min(e, n);
                n -= e;
                loop {
                    (*decomp_state).outbuf[w as usize] = (*decomp_state).outbuf[d as usize];
                    w += 1;
                    d += 1;
                    e -= 1;
                    if e == 0 {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
        }
    }

    /* restore the globals from the locals */
    (*decomp_state).methods.zip.window_posn = w;
    (*decomp_state).methods.zip.bb = b;
    (*decomp_state).methods.zip.bk = k;

    /* done */
    return 0;
}

unsafe fn fdi_Zipinflate_stored(decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut b: u32; // bit buffer
    let mut k: u32; // number of bits in bit buffer
    let mut w: u32; // current window position

    // make local copies of globals
    b = (*decomp_state).methods.zip.bb;
    k = (*decomp_state).methods.zip.bk;
    w = (*decomp_state).methods.zip.window_posn;

    // go to byte boundary
    let n = k & 7;
    b >>= n;
    k -= n;

    // get the length and its complement
    // ZIPNEEDBITS(16)
    while k < 16 {
        let p = (*decomp_state).methods.zip.inpos;
        let c: u8 = *p;
        (*decomp_state).methods.zip.inpos = p.add(1);
        b |= (c as u32) << k;
        k += 8;
    }
    let mut n = b & 0xffff;
    b >>= 16;
    k -= 16;
    // ZIPNEEDBITS(16)
    while k < 16 {
        let p = (*decomp_state).methods.zip.inpos;
        let c: u8 = *p;
        (*decomp_state).methods.zip.inpos = p.add(1);
        b |= (c as u32) << k;
        k += 8;
    }
    if n != ((!b) & 0xffff) {
        return 1; // error in compressed data
    }
    b >>= 16;
    k -= 16;

    // read and output the compressed data
    while n > 0 {
        n -= 1;
        // ZIPNEEDBITS(8)
        while k < 8 {
            let p = (*decomp_state).methods.zip.inpos;
            let c: u8 = *p;
            (*decomp_state).methods.zip.inpos = p.add(1);
            b |= (c as u32) << k;
            k += 8;
        }
        (*decomp_state).outbuf[w as usize] = b as u8;
        w += 1;
        b >>= 8;
        k -= 8;
    }

    // restore the globals from the locals
    (*decomp_state).methods.zip.window_posn = w;
    (*decomp_state).methods.zip.bb = b;
    (*decomp_state).methods.zip.bk = k;

    return 0;
}

unsafe extern "C" fn QTMfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut inpos = (*decomp_state).inbuf.as_mut_ptr();
    let window = (*decomp_state).methods.qtm.window;
    let mut window_posn = (*decomp_state).methods.qtm.window_posn;
    let window_size = (*decomp_state).methods.qtm.window_size;

    // used by bitstream macros
    let mut bitsleft: i32 = 0; // Q_INIT_BITSTREAM
    let mut bitbuf: u32 = 0;

    let mut togo: i32 = outlen;
    let mut match_length: i32 = 0;
    let mut copy_length: i32;
    let mut selector: u8 = 0;
    let mut sym: u8 = 0;
    let mut match_offset: u32 = 0;
    let mut extra: i32;

    // arithmetic-coder state (C/H/L persist across GET_SYMBOL)
    let mut h: u16 = 0xFFFF;
    let mut l: u16 = 0;
    let mut c: u16 = 0;

    winedebug_rust::TRACE!("(inlen == %d, outlen == %d)\n", inlen, outlen);

    // read initial value of C
    Q_READ_BITS!(c, 16, bitbuf, bitsleft, inpos);

    // apply 2^x-1 mask
    window_posn &= window_size - 1;
    // runs can't straddle the window wraparound
    if (window_posn + togo as u32) > window_size {
        winedebug_rust::TRACE!("straddled run\n");
        return DECR_DATAFORMAT;
    }

    let qtm = core::ptr::addr_of_mut!((*decomp_state).methods.qtm);

    while togo > 0 {
        GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model7), selector, bitbuf, bitsleft, inpos, c, h, l);
        match selector {
            0 => {
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model00), sym, bitbuf, bitsleft, inpos, c, h, l);
                *window.add(window_posn as usize) = sym; window_posn += 1; togo -= 1;
            }
            1 => {
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model40), sym, bitbuf, bitsleft, inpos, c, h, l);
                *window.add(window_posn as usize) = sym; window_posn += 1; togo -= 1;
            }
            2 => {
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model80), sym, bitbuf, bitsleft, inpos, c, h, l);
                *window.add(window_posn as usize) = sym; window_posn += 1; togo -= 1;
            }
            3 => {
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).modelC0), sym, bitbuf, bitsleft, inpos, c, h, l);
                *window.add(window_posn as usize) = sym; window_posn += 1; togo -= 1;
            }
            4 => {
                // selector 4 = fixed length of 3
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model4), sym, bitbuf, bitsleft, inpos, c, h, l);
                Q_READ_BITS!(extra, (*decomp_state).q_extra_bits[sym as usize] as u32, bitbuf, bitsleft, inpos);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
                match_length = 3;
            }
            5 => {
                // selector 5 = fixed length of 4
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model5), sym, bitbuf, bitsleft, inpos, c, h, l);
                Q_READ_BITS!(extra, (*decomp_state).q_extra_bits[sym as usize] as u32, bitbuf, bitsleft, inpos);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
                match_length = 4;
            }
            6 => {
                // selector 6 = variable length
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model6len), sym, bitbuf, bitsleft, inpos, c, h, l);
                Q_READ_BITS!(extra, (*decomp_state).q_length_extra[sym as usize] as u32, bitbuf, bitsleft, inpos);
                match_length = (*decomp_state).q_length_base[sym as usize] as i32 + extra + 5;
                GET_SYMBOL!(core::ptr::addr_of_mut!((*qtm).model6pos), sym, bitbuf, bitsleft, inpos, c, h, l);
                Q_READ_BITS!(extra, (*decomp_state).q_extra_bits[sym as usize] as u32, bitbuf, bitsleft, inpos);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
            }
            _ => {
                winedebug_rust::TRACE!("Selector is bogus\n");
                return DECR_ILLEGALDATA;
            }
        }

        // if this is a match
        if selector >= 4 {
            let mut rundest = window.add(window_posn as usize);
            togo -= match_length;
            let mut runsrc;
            // copy any wrapped around source data
            if window_posn >= match_offset {
                runsrc = rundest.offset(-(match_offset as isize));
            } else {
                runsrc = rundest.add((window_size - match_offset) as usize);
                copy_length = (match_offset - window_posn) as i32;
                if copy_length < match_length {
                    match_length -= copy_length;
                    window_posn += copy_length as u32;
                    while copy_length > 0 {
                        *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1);
                        copy_length -= 1;
                    }
                    runsrc = window;
                }
            }
            window_posn += match_length as u32;
            // copy match data - no worries about destination wraps
            while match_length > 0 {
                *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1);
                match_length -= 1;
            }
        }
    } // while (togo > 0)

    if togo != 0 {
        winedebug_rust::TRACE!("Frame overflow, this_run = %d\n", togo);
        return DECR_ILLEGALDATA;
    }

    let src = (if window_posn == 0 { window_size } else { window_posn }) - outlen as u32;
    core::ptr::copy_nonoverlapping(window.add(src as usize), (*decomp_state).outbuf.as_mut_ptr(), outlen as usize);

    (*decomp_state).methods.qtm.window_posn = window_posn;
    DECR_OK
}

unsafe fn fdi_lzx_read_lens(lens: *mut u8, first: u32, last: u32, lb: *mut lzx_bits, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut bitbuf: u32 = (*lb).bb;
    let mut bitsleft: i32 = (*lb).bl;
    let mut inpos: *mut u8 = (*lb).ip;

    let lzx = core::ptr::addr_of_mut!((*decomp_state).methods.lzx);
    let pretree_len = (*lzx).PRETREE_len.as_mut_ptr();
    let pretree_tab = (*lzx).PRETREE_table.as_mut_ptr();

    let mut y: u32;
    let mut z: i32;

    for x in 0..20usize {
        READ_BITS!(y, 4, bitbuf, bitsleft, inpos);
        *pretree_len.add(x) = y as u8;
    }
    BUILD_TABLE!(LZX_PRETREE_MAXSYMBOLS, LZX_PRETREE_TABLEBITS, pretree_len, pretree_tab);

    let mut x: u32 = first;
    while x < last {
        READ_HUFFSYM!(LZX_PRETREE_MAXSYMBOLS, LZX_PRETREE_TABLEBITS, pretree_tab, pretree_len,
                      z, bitbuf, bitsleft, inpos);
        if z == 17 {
            READ_BITS!(y, 4, bitbuf, bitsleft, inpos); y += 4;
            while y > 0 { *lens.add(x as usize) = 0; x += 1; y -= 1; }
        } else if z == 18 {
            READ_BITS!(y, 5, bitbuf, bitsleft, inpos); y += 20;
            while y > 0 { *lens.add(x as usize) = 0; x += 1; y -= 1; }
        } else if z == 19 {
            READ_BITS!(y, 1, bitbuf, bitsleft, inpos); y += 4;
            READ_HUFFSYM!(LZX_PRETREE_MAXSYMBOLS, LZX_PRETREE_TABLEBITS, pretree_tab, pretree_len,
                          z, bitbuf, bitsleft, inpos);
            z = *lens.add(x as usize) as i32 - z; if z < 0 { z += 17; }
            while y > 0 { *lens.add(x as usize) = z as u8; x += 1; y -= 1; }
        } else {
            z = *lens.add(x as usize) as i32 - z; if z < 0 { z += 17; }
            *lens.add(x as usize) = z as u8; x += 1;
        }
    }

    (*lb).bb = bitbuf;
    (*lb).bl = bitsleft;
    (*lb).ip = inpos;
    0
}

unsafe extern "C" fn LZXfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut inpos: *mut u8 = (*decomp_state).inbuf.as_mut_ptr();
    let endinp: *const u8 = inpos.add(inlen as usize) as *const u8;
    let window: *mut u8 = (*decomp_state).methods.lzx.window;
    let mut runsrc: *mut u8;
    let mut rundest: *mut u8;

    let mut window_posn: u32 = (*decomp_state).methods.lzx.window_posn;
    let window_size: u32 = (*decomp_state).methods.lzx.window_size;
    let mut r0: u32 = (*decomp_state).methods.lzx.R0;
    let mut r1: u32 = (*decomp_state).methods.lzx.R1;
    let mut r2: u32 = (*decomp_state).methods.lzx.R2;

    let mut bitbuf: u32 = 0; // INIT_BITSTREAM
    let mut bitsleft: i32 = 0;
    let mut match_offset: u32;
    let mut i: u32;
    let mut j: u32;
    let mut k: u32;
    let mut lb: lzx_bits = core::mem::zeroed();

    let mut togo: i32 = outlen;
    let mut this_run: i32;
    let mut main_element: i32;
    let mut aligned_bits: i32;
    let mut match_length: i32;
    let mut copy_length: i32;
    let mut length_footer: i32;
    let mut extra: i32;
    let mut verbatim_bits: i32;

    winedebug_rust::TRACE!("(inlen == %d, outlen == %d)\n", inlen, outlen);

    let lzx = core::ptr::addr_of_mut!((*decomp_state).methods.lzx);

    // read header if necessary
    if (*lzx).header_read == 0 {
        i = 0; j = 0;
        READ_BITS!(k, 1, bitbuf, bitsleft, inpos);
        if k != 0 {
            READ_BITS!(i, 16, bitbuf, bitsleft, inpos);
            READ_BITS!(j, 16, bitbuf, bitsleft, inpos);
        }
        (*lzx).intel_filesize = ((i << 16) | j) as i32; // or 0 if not encoded
        (*lzx).header_read = 1;
    }

    // main decoding loop
    while togo > 0 {
        // last block finished, new block expected
        if (*lzx).block_remaining == 0 {
            if (*lzx).block_type == LZX_BLOCKTYPE_UNCOMPRESSED {
                if ((*lzx).block_length & 1) != 0 { inpos = inpos.add(1); } // realign to word
                bitsleft = 0; bitbuf = 0; // INIT_BITSTREAM
            }

            READ_BITS!((*lzx).block_type, 3, bitbuf, bitsleft, inpos);
            READ_BITS!(i, 16, bitbuf, bitsleft, inpos);
            READ_BITS!(j, 8, bitbuf, bitsleft, inpos);
            (*lzx).block_length = (i << 8) | j;
            (*lzx).block_remaining = (*lzx).block_length;

            let bt = (*lzx).block_type;
            if bt == LZX_BLOCKTYPE_ALIGNED {
                let aligned_len = (*lzx).ALIGNED_len.as_mut_ptr();
                let aligned_tab = (*lzx).ALIGNED_table.as_mut_ptr();
                i = 0;
                while i < 8 {
                    READ_BITS!(j, 3, bitbuf, bitsleft, inpos);
                    *aligned_len.add(i as usize) = j as u8;
                    i += 1;
                }
                BUILD_TABLE!(LZX_ALIGNED_MAXSYMBOLS, LZX_ALIGNED_TABLEBITS, aligned_len, aligned_tab);
                // rest of aligned header is same as verbatim
            }
            if bt == LZX_BLOCKTYPE_ALIGNED || bt == LZX_BLOCKTYPE_VERBATIM {
                let maintree_len = (*lzx).MAINTREE_len.as_mut_ptr();
                let maintree_tab = (*lzx).MAINTREE_table.as_mut_ptr();
                let length_len = (*lzx).LENGTH_len.as_mut_ptr();
                let length_tab = (*lzx).LENGTH_table.as_mut_ptr();
                READ_LENGTHS!(maintree_len, 0, 256, lb, bitbuf, bitsleft, inpos, decomp_state);
                READ_LENGTHS!(maintree_len, 256, (*lzx).main_elements as u32, lb, bitbuf, bitsleft, inpos, decomp_state);
                BUILD_TABLE!(LZX_MAINTREE_MAXSYMBOLS, LZX_MAINTREE_TABLEBITS, maintree_len, maintree_tab);
                if *maintree_len.add(0xE8) != 0 { (*lzx).intel_started = 1; }
                READ_LENGTHS!(length_len, 0, LZX_NUM_SECONDARY_LENGTHS, lb, bitbuf, bitsleft, inpos, decomp_state);
                BUILD_TABLE!(LZX_LENGTH_MAXSYMBOLS, LZX_LENGTH_TABLEBITS, length_len, length_tab);
            } else if bt == LZX_BLOCKTYPE_UNCOMPRESSED {
                (*lzx).intel_started = 1; // because we can't assume otherwise
                ENSURE_BITS!(bitbuf, bitsleft, inpos, 16); // get up to 16 pad bits
                if bitsleft > 16 { inpos = inpos.offset(-2); } // align the bitstream
                r0 = (*inpos.add(0) as u32) | ((*inpos.add(1) as u32) << 8) | ((*inpos.add(2) as u32) << 16) | ((*inpos.add(3) as u32) << 24); inpos = inpos.add(4);
                r1 = (*inpos.add(0) as u32) | ((*inpos.add(1) as u32) << 8) | ((*inpos.add(2) as u32) << 16) | ((*inpos.add(3) as u32) << 24); inpos = inpos.add(4);
                r2 = (*inpos.add(0) as u32) | ((*inpos.add(1) as u32) << 8) | ((*inpos.add(2) as u32) << 16) | ((*inpos.add(3) as u32) << 24); inpos = inpos.add(4);
            } else {
                return DECR_ILLEGALDATA;
            }
        }

        // buffer exhaustion check
        if (inpos as *const u8) > endinp {
            if (inpos as *const u8) > endinp.add(2) || bitsleft < 16 { return DECR_ILLEGALDATA; }
        }

        while {
            this_run = (*lzx).block_remaining as i32;
            this_run > 0 && togo > 0
        } {
            if this_run > togo { this_run = togo; }
            togo -= this_run;
            (*lzx).block_remaining -= this_run as u32;

            // apply 2^x-1 mask
            window_posn &= window_size - 1;
            // runs can't straddle the window wraparound
            if (window_posn + this_run as u32) > window_size { return DECR_DATAFORMAT; }

            let bt = (*lzx).block_type;
            if bt == LZX_BLOCKTYPE_VERBATIM {
                let maintree_len = (*lzx).MAINTREE_len.as_mut_ptr();
                let maintree_tab = (*lzx).MAINTREE_table.as_mut_ptr();
                let length_len = (*lzx).LENGTH_len.as_mut_ptr();
                let length_tab = (*lzx).LENGTH_table.as_mut_ptr();
                while this_run > 0 {
                    READ_HUFFSYM!(LZX_MAINTREE_MAXSYMBOLS, LZX_MAINTREE_TABLEBITS, maintree_tab, maintree_len, main_element, bitbuf, bitsleft, inpos);
                    if (main_element as u32) < LZX_NUM_CHARS {
                        // literal: 0 to LZX_NUM_CHARS-1
                        *window.add(window_posn as usize) = main_element as u8;
                        window_posn += 1;
                        this_run -= 1;
                    } else {
                        // match: LZX_NUM_CHARS + ((slot<<3) | length_header (3 bits))
                        main_element -= LZX_NUM_CHARS as i32;
                        match_length = main_element & LZX_NUM_PRIMARY_LENGTHS;
                        if match_length == LZX_NUM_PRIMARY_LENGTHS {
                            READ_HUFFSYM!(LZX_LENGTH_MAXSYMBOLS, LZX_LENGTH_TABLEBITS, length_tab, length_len, length_footer, bitbuf, bitsleft, inpos);
                            match_length += length_footer;
                        }
                        match_length += LZX_MIN_MATCH;
                        match_offset = (main_element >> 3) as u32;
                        if match_offset > 2 {
                            // not repeated offset
                            if match_offset != 3 {
                                extra = (*decomp_state).extra_bits[match_offset as usize] as i32;
                                READ_BITS!(verbatim_bits, extra, bitbuf, bitsleft, inpos);
                                match_offset = (*decomp_state).lzx_position_base[match_offset as usize].wrapping_sub(2).wrapping_add(verbatim_bits as u32);
                            } else {
                                match_offset = 1;
                            }
                            // update repeated offset LRU queue
                            r2 = r1; r1 = r0; r0 = match_offset;
                        } else if match_offset == 0 {
                            match_offset = r0;
                        } else if match_offset == 1 {
                            match_offset = r1; r1 = r0; r0 = match_offset;
                        } else {
                            match_offset = r2; r2 = r0; r0 = match_offset;
                        }
                        rundest = window.add(window_posn as usize);
                        this_run -= match_length;
                        if window_posn >= match_offset {
                            runsrc = rundest.offset(-(match_offset as isize));
                        } else {
                            runsrc = rundest.add((window_size - match_offset) as usize);
                            copy_length = (match_offset - window_posn) as i32;
                            if copy_length < match_length {
                                match_length -= copy_length;
                                window_posn += copy_length as u32;
                                while copy_length > 0 { *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1); copy_length -= 1; }
                                runsrc = window;
                            }
                        }
                        window_posn += match_length as u32;
                        while match_length > 0 { *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1); match_length -= 1; }
                    }
                }
            } else if bt == LZX_BLOCKTYPE_ALIGNED {
                let maintree_len = (*lzx).MAINTREE_len.as_mut_ptr();
                let maintree_tab = (*lzx).MAINTREE_table.as_mut_ptr();
                let length_len = (*lzx).LENGTH_len.as_mut_ptr();
                let length_tab = (*lzx).LENGTH_table.as_mut_ptr();
                let aligned_len = (*lzx).ALIGNED_len.as_mut_ptr();
                let aligned_tab = (*lzx).ALIGNED_table.as_mut_ptr();
                while this_run > 0 {
                    READ_HUFFSYM!(LZX_MAINTREE_MAXSYMBOLS, LZX_MAINTREE_TABLEBITS, maintree_tab, maintree_len, main_element, bitbuf, bitsleft, inpos);
                    if (main_element as u32) < LZX_NUM_CHARS {
                        // literal: 0 to LZX_NUM_CHARS-1
                        *window.add(window_posn as usize) = main_element as u8;
                        window_posn += 1;
                        this_run -= 1;
                    } else {
                        // match: LZX_NUM_CHARS + ((slot<<3) | length_header (3 bits))
                        main_element -= LZX_NUM_CHARS as i32;
                        match_length = main_element & LZX_NUM_PRIMARY_LENGTHS;
                        if match_length == LZX_NUM_PRIMARY_LENGTHS {
                            READ_HUFFSYM!(LZX_LENGTH_MAXSYMBOLS, LZX_LENGTH_TABLEBITS, length_tab, length_len, length_footer, bitbuf, bitsleft, inpos);
                            match_length += length_footer;
                        }
                        match_length += LZX_MIN_MATCH;
                        match_offset = (main_element >> 3) as u32;
                        if match_offset > 2 {
                            // not repeated offset
                            extra = (*decomp_state).extra_bits[match_offset as usize] as i32;
                            match_offset = (*decomp_state).lzx_position_base[match_offset as usize].wrapping_sub(2);
                            if extra > 3 {
                                // verbatim and aligned bits
                                extra -= 3;
                                READ_BITS!(verbatim_bits, extra, bitbuf, bitsleft, inpos);
                                match_offset = match_offset.wrapping_add((verbatim_bits as u32) << 3);
                                READ_HUFFSYM!(LZX_ALIGNED_MAXSYMBOLS, LZX_ALIGNED_TABLEBITS, aligned_tab, aligned_len, aligned_bits, bitbuf, bitsleft, inpos);
                                match_offset = match_offset.wrapping_add(aligned_bits as u32);
                            } else if extra == 3 {
                                // aligned bits only
                                READ_HUFFSYM!(LZX_ALIGNED_MAXSYMBOLS, LZX_ALIGNED_TABLEBITS, aligned_tab, aligned_len, aligned_bits, bitbuf, bitsleft, inpos);
                                match_offset = match_offset.wrapping_add(aligned_bits as u32);
                            } else if extra > 0 {
                                // verbatim bits only
                                READ_BITS!(verbatim_bits, extra, bitbuf, bitsleft, inpos);
                                match_offset = match_offset.wrapping_add(verbatim_bits as u32);
                            } else {
                                // extra == 0
                                match_offset = 1;
                            }
                            // update repeated offset LRU queue
                            r2 = r1; r1 = r0; r0 = match_offset;
                        } else if match_offset == 0 {
                            match_offset = r0;
                        } else if match_offset == 1 {
                            match_offset = r1; r1 = r0; r0 = match_offset;
                        } else {
                            match_offset = r2; r2 = r0; r0 = match_offset;
                        }
                        rundest = window.add(window_posn as usize);
                        this_run -= match_length;
                        if window_posn >= match_offset {
                            runsrc = rundest.offset(-(match_offset as isize));
                        } else {
                            runsrc = rundest.add((window_size - match_offset) as usize);
                            copy_length = (match_offset - window_posn) as i32;
                            if copy_length < match_length {
                                match_length -= copy_length;
                                window_posn += copy_length as u32;
                                while copy_length > 0 { *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1); copy_length -= 1; }
                                runsrc = window;
                            }
                        }
                        window_posn += match_length as u32;
                        while match_length > 0 { *rundest = *runsrc; rundest = rundest.add(1); runsrc = runsrc.add(1); match_length -= 1; }
                    }
                }
            } else if bt == LZX_BLOCKTYPE_UNCOMPRESSED {
                if (inpos.add(this_run as usize) as *const u8) > endinp { return DECR_ILLEGALDATA; }
                core::ptr::copy_nonoverlapping(inpos, window.add(window_posn as usize), this_run as usize);
                inpos = inpos.add(this_run as usize);
                window_posn += this_run as u32;
            } else {
                return DECR_ILLEGALDATA; // might as well
            }
        }
    }

    if togo != 0 { return DECR_ILLEGALDATA; }
    let src = (if window_posn == 0 { window_size } else { window_posn }) - outlen as u32;
    core::ptr::copy_nonoverlapping(window.add(src as usize), (*decomp_state).outbuf.as_mut_ptr(), outlen as usize);

    (*lzx).window_posn = window_posn;
    (*lzx).R0 = r0;
    (*lzx).R1 = r1;
    (*lzx).R2 = r2;

    // intel E8 decoding (frames_read always increments; body gated on old value)
    let old_frames = (*lzx).frames_read;
    (*lzx).frames_read = old_frames.wrapping_add(1);
    if (old_frames < 32768) && (*lzx).intel_filesize != 0 {
        if outlen <= 6 || (*lzx).intel_started == 0 {
            (*lzx).intel_curpos += outlen;
        } else {
            let data0 = (*decomp_state).outbuf.as_mut_ptr();
            let mut data = data0;
            let dataend = data0.wrapping_offset((outlen - 10) as isize);
            let mut curpos: i32 = (*lzx).intel_curpos;
            let filesize: i32 = (*lzx).intel_filesize;
            (*lzx).intel_curpos = curpos + outlen;
            while (data as *const u8) < (dataend as *const u8) {
                if *data != 0xE8 { data = data.add(1); curpos += 1; continue; }
                data = data.add(1);
                let abs_off: i32 = (*data.add(0) as i32) | ((*data.add(1) as i32) << 8) | ((*data.add(2) as i32) << 16) | ((*data.add(3) as i32) << 24);
                if (abs_off >= -curpos) && (abs_off < filesize) {
                    let rel_off: i32 = if abs_off >= 0 { abs_off - curpos } else { abs_off + filesize };
                    *data.add(0) = rel_off as u8;
                    *data.add(1) = (rel_off >> 8) as u8;
                    *data.add(2) = (rel_off >> 16) as u8;
                    *data.add(3) = (rel_off >> 24) as u8;
                }
                data = data.add(4);
                curpos += 5;
            }
        }
    }
    DECR_OK
}

unsafe extern "C" fn free_decompression_temps(fdi: *mut FDI_Int, fol: *const fdi_folder, decomp_state: *mut fdi_decomp_state) {
    match (*fol).comp_type & 0x000F {
        // cffoldCOMPTYPE_LZX
        0x0003 => {
            if !(*decomp_state).methods.lzx.window.is_null() {
                (((*fdi).free).unwrap_unchecked())((*decomp_state).methods.lzx.window as *mut core::ffi::c_void);
                (*decomp_state).methods.lzx.window = core::ptr::null_mut();
            }
        }
        // cffoldCOMPTYPE_QUANTUM
        0x0004 => {
            if !(*decomp_state).methods.qtm.window.is_null() {
                (((*fdi).free).unwrap_unchecked())((*decomp_state).methods.qtm.window as *mut core::ffi::c_void);
                (*decomp_state).methods.qtm.window = core::ptr::null_mut();
            }
        }
        _ => {}
    }
}

unsafe fn free_decompression_mem(fdi: *mut FDI_Int, decomp_state: *mut fdi_decomp_state) {
    let mut decomp_state = decomp_state;
    let mut fol: *mut fdi_folder;
    while !decomp_state.is_null() {
        let prev_fds: *mut fdi_decomp_state;
        (((*fdi).close).unwrap_unchecked())((*decomp_state).cabhf as isize);
        if !(*decomp_state).mii.nextname.is_null() {
            (((*fdi).free).unwrap_unchecked())((*decomp_state).mii.nextname as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.nextinfo.is_null() {
            (((*fdi).free).unwrap_unchecked())((*decomp_state).mii.nextinfo as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.prevname.is_null() {
            (((*fdi).free).unwrap_unchecked())((*decomp_state).mii.prevname as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.previnfo.is_null() {
            (((*fdi).free).unwrap_unchecked())((*decomp_state).mii.previnfo as *mut core::ffi::c_void);
        }
        while !(*decomp_state).firstfol.is_null() {
            fol = (*decomp_state).firstfol;
            (*decomp_state).firstfol = (*fol).next;
            (((*fdi).free).unwrap_unchecked())(fol as *mut core::ffi::c_void);
        }
        while !(*decomp_state).firstfile.is_null() {
            let file = (*decomp_state).firstfile;
            if !(*file).filename.is_null() {
                (((*fdi).free).unwrap_unchecked())((*file).filename as *mut core::ffi::c_void);
            }
            (*decomp_state).firstfile = (*file).next;
            (((*fdi).free).unwrap_unchecked())(file as *mut core::ffi::c_void);
        }
        prev_fds = decomp_state;
        decomp_state = (*decomp_state).next;
        (((*fdi).free).unwrap_unchecked())(prev_fds as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FDIDestroy(hfdi: isize) -> i32 {
    let fdi: *mut FDI_Int = get_fdi_ptr(hfdi as HFDI);
    winedebug_rust::TRACE!("(hfdi == ^%p)\n", hfdi);
    if fdi.is_null() {
        return 0;
    }
    (*fdi).magic = 0;
    (((*fdi).free).unwrap_unchecked())(fdi as *mut core::ffi::c_void);
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn FDITruncateCabinet(
    hfdi: HFDI,
    pszCabinetName: *mut i8,
    iFolderToDelete: u16,
) -> i32 {
    let fdi = get_fdi_ptr(hfdi as *mut core::ffi::c_void);
    winedebug_rust::FIXME!("(hfdi == ^%p, pszCabinetName == %s, iFolderToDelete == %hu): stub\n", hfdi as *const core::ffi::c_void, debugstr_a(pszCabinetName), iFolderToDelete as u32);
    if fdi.is_null() {
        return 0;
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

unsafe fn FDI_read_string(fdi: *mut FDI_Int, hf: INT_PTR, cabsize: i32) -> *mut i8 {
    let mut len: usize = 256;
    let base: i32 = FDI_getoffset(fdi, hf as INT_PTR);
    let maxlen: i32 = cabsize - base;
    let mut ok: i32 = 0;
    let mut buf: *mut u8 = core::ptr::null_mut();

    winedebug_rust::TRACE!("(fdi == %p, hf == %ld, cabsize == %ld)\n", fdi as *const core::ffi::c_void, hf, cabsize);

    loop {
        if len > (maxlen as usize) {
            len = maxlen as usize;
        }
        buf = (((*fdi).alloc).unwrap_unchecked())(len as u32) as *mut u8;
        if buf.is_null() {
            break;
        }
        if (((*fdi).read).unwrap_unchecked())(hf as isize, buf as *mut core::ffi::c_void, len as u32) == 0 {
            break;
        }

        for i in 0..len as usize {
            if *buf.add(i) == 0 {
                ok = 1;
                break;
            }
        }

        if ok == 0 {
            if len == (maxlen as usize) {
                winedebug_rust::ERR!("cabinet is truncated\n");
                break;
            }
            (((*fdi).seek).unwrap_unchecked())(hf as isize, base, SEEK_SET);
            (((*fdi).free).unwrap_unchecked())(buf as *mut core::ffi::c_void);
            buf = core::ptr::null_mut();
            len *= 2;
        }

        if ok != 0 {
            break;
        }
    }

    if ok == 0 {
        if !buf.is_null() {
            (((*fdi).free).unwrap_unchecked())(buf as *mut core::ffi::c_void);
        } else {
            winedebug_rust::ERR!("out of memory!\n");
        }
        return core::ptr::null_mut();
    }

    let seek_pos = (base as i32).wrapping_add((strlen(buf as *const i8) + 1) as i32);
    (((*fdi).seek).unwrap_unchecked())(hf as isize, seek_pos, SEEK_SET);

    return buf as *mut i8;
}

unsafe fn QTMfdi_init(window: i32, level: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let wndsize: u32 = 1u32 << window;
    let msz: i32 = window * 2;
    let mut i: i32;
    let mut j: u32;

    if window < 10 || window > 21 {
        return 1; /* DECR_DATAFORMAT */
    }
    if (*decomp_state).methods.qtm.actual_size < wndsize {
        if !(*decomp_state).methods.qtm.window.is_null() {
            (((*(*decomp_state).fdi).free).unwrap_unchecked())((*decomp_state).methods.qtm.window as *mut core::ffi::c_void);
        }
        (*decomp_state).methods.qtm.window = core::ptr::null_mut();
    }
    if (*decomp_state).methods.qtm.window.is_null() {
        let ptr = (((*(*decomp_state).fdi).alloc).unwrap_unchecked())(wndsize) as *mut u8;
        (*decomp_state).methods.qtm.window = ptr;
        if ptr.is_null() {
            return 3; /* DECR_NOMEMORY */
        }
        (*decomp_state).methods.qtm.actual_size = wndsize;
    }
    (*decomp_state).methods.qtm.window_size = wndsize;
    (*decomp_state).methods.qtm.window_posn = 0;

    /* initialize static slot/extrabits tables */
    j = 0;
    for i in 0..27 {
        let extra_val: u32 = if i == 26 {
            0
        } else if i < 2 {
            0
        } else {
            (i - 2) >> 2
        };
        (*decomp_state).q_length_extra[i as usize] = extra_val as u8;
        (*decomp_state).q_length_base[i as usize] = j as u8;
        j += 1u32 << if i == 26 { 5 } else { extra_val };
    }
    j = 0;
    for i in 0..42 {
        let extra_bits_val: u32 = if i < 2 { 0 } else { (i - 2) >> 1 };
        (*decomp_state).q_extra_bits[i as usize] = extra_bits_val as u8;
        (*decomp_state).q_position_base[i as usize] = j;
        j += 1u32 << extra_bits_val;
    }

    /* initialize arithmetic coding models */
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model7),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m7sym) as *mut QTMmodelsym,
        7,
        0,
    );
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model00),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m00sym) as *mut QTMmodelsym,
        0x40,
        0x00,
    );
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model40),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m40sym) as *mut QTMmodelsym,
        0x40,
        0x40,
    );
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model80),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m80sym) as *mut QTMmodelsym,
        0x40,
        0x80,
    );
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.modelC0),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.mC0sym) as *mut QTMmodelsym,
        0x40,
        0xC0,
    );
    /* model 4 depends on table size, ranges from 20 to 24 */
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model4),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m4sym) as *mut QTMmodelsym,
        if msz < 24 { msz } else { 24 },
        0,
    );
    /* model 5 depends on table size, ranges from 20 to 36 */
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model5),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m5sym) as *mut QTMmodelsym,
        if msz < 36 { msz } else { 36 },
        0,
    );
    /* model 6pos depends on table size, ranges from 20 to 42 */
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model6pos),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m6psym) as *mut QTMmodelsym,
        msz,
        0,
    );
    QTMfdi_initmodel(
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.model6len),
        core::ptr::addr_of_mut!((*decomp_state).methods.qtm.m6lsym) as *mut QTMmodelsym,
        27,
        0,
    );

    0 /* DECR_OK */
}

unsafe fn FDI_read_entries(
    fdi: *mut FDI_Int,
    hf: INT_PTR,
    pfdici: *mut FDICABINETINFO,
    pmii: *mut MORE_ISCAB_INFO,
) -> i32 {
    // set_error and FDI_read_string are defined at module scope (fdi.rs); the
    // DeepSeek output wrongly re-declared them `extern` here (→ undefined
    // _set_error@12). EndGetI32/EndGetI16 are wine little-endian read macros,
    // not real functions — define them locally. (Faithful to wine/cabinet.h.)
    unsafe fn EndGetI32(a: *const u8) -> i32 {
        ((*a.add(0) as u32)
            | ((*a.add(1) as u32) << 8)
            | ((*a.add(2) as u32) << 16)
            | ((*a.add(3) as u32) << 24)) as i32
    }
    unsafe fn EndGetI16(a: *const u8) -> i16 {
        ((*a.add(0) as u16) | ((*a.add(1) as u16) << 8)) as i16
    }

    let mut num_folders: i32;
    let mut num_files: i32;
    let mut header_resv: i32;
    let mut folder_resv: u8 = 0;
    let mut cabsize: i32;
    let mut setid: u16;
    let mut cabidx: u16;
    let mut flags: u16;
    let mut buf: [u8; 64] = [0u8; 64];
    let mut block_resv: u8;
    let mut prevname: *mut i8 = core::ptr::null_mut();
    let mut previnfo: *mut i8 = core::ptr::null_mut();
    let mut nextname: *mut i8 = core::ptr::null_mut();
    let mut nextinfo: *mut i8 = core::ptr::null_mut();

    winedebug_rust::TRACE!("(fdi == ^%p, hf == %ld, pfdici == ^%p)\n", fdi as *const core::ffi::c_void, hf, pfdici as *const core::ffi::c_void);

    if (((*fdi).read).unwrap_unchecked())(hf as isize, buf.as_mut_ptr() as *mut core::ffi::c_void, cfhead_SIZEOF as _)
        != cfhead_SIZEOF as _
    {
        if !pmii.is_null() {
            set_error(fdi, FDIERROR_NOT_A_CABINET, 0);
        }
        return 0;
    }

    if EndGetI32(buf.as_ptr().add(cfhead_Signature)) != 0x4643534d {
        if !pmii.is_null() {
            set_error(fdi, FDIERROR_NOT_A_CABINET, 0);
        }
        return 0;
    }

    cabsize = EndGetI32(buf.as_ptr().add(cfhead_CabinetSize));

    num_folders = EndGetI16(buf.as_ptr().add(cfhead_NumFolders)) as i32;

    num_files = EndGetI16(buf.as_ptr().add(cfhead_NumFiles)) as i32;

    setid = EndGetI16(buf.as_ptr().add(cfhead_SetID)) as u16;

    cabidx = EndGetI16(buf.as_ptr().add(cfhead_CabinetIndex)) as u16;

    if (buf[cfhead_MajorVersion] > 1)
        || (buf[cfhead_MajorVersion] == 1 && buf[cfhead_MinorVersion] > 3)
    {
        winedebug_rust::WARN!("cabinet format version > 1.3\n");
        if !pmii.is_null() {
            set_error(fdi, FDIERROR_UNKNOWN_CABINET_VERSION, 0);
        }
        return 0;
    }

    flags = EndGetI16(buf.as_ptr().add(cfhead_Flags)) as u16;

    if (flags & cfheadRESERVE_PRESENT as u16) != 0 {
        if (((*fdi).read).unwrap_unchecked())(
            hf as isize,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            cfheadext_SIZEOF as _,
        ) != cfheadext_SIZEOF as _
        {
            winedebug_rust::ERR!("bunk reserve-sizes?\n");
            if !pmii.is_null() {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
            }
            return 0;
        }

        header_resv = EndGetI16(buf.as_ptr().add(cfheadext_HeaderReserved)) as i32;
        if !pmii.is_null() {
            (*pmii).header_resv = header_resv;
        }
        folder_resv = buf[cfheadext_FolderReserved];
        if !pmii.is_null() {
            (*pmii).folder_resv = folder_resv as i32;
        }
        block_resv = buf[cfheadext_DataReserved];
        if !pmii.is_null() {
            (*pmii).block_resv = block_resv;
        }

        if header_resv > 60000 {
            winedebug_rust::WARN!("WARNING; header reserved space > 60000\n");
        }

        if (header_resv != 0) && ((((*fdi).seek).unwrap_unchecked())(hf as isize, header_resv, SEEK_CUR) == -1) {
            winedebug_rust::ERR!("seek failure: header_resv\n");
            if !pmii.is_null() {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
            }
            return 0;
        }
    }

    if (flags & cfheadPREV_CABINET as u16) != 0 {
        prevname = FDI_read_string(fdi, hf, cabsize);
        if prevname.is_null() {
            if !pmii.is_null() {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
            }
            return 0;
        } else {
            if !pmii.is_null() {
                (*pmii).prevname = prevname;
            } else {
                (((*fdi).free).unwrap_unchecked())(prevname as *mut core::ffi::c_void);
            }
        }
        previnfo = FDI_read_string(fdi, hf, cabsize);
        if !previnfo.is_null() {
            if !pmii.is_null() {
                (*pmii).previnfo = previnfo;
            } else {
                (((*fdi).free).unwrap_unchecked())(previnfo as *mut core::ffi::c_void);
            }
        }
    }

    if (flags & cfheadNEXT_CABINET as u16) != 0 {
        if !pmii.is_null() {
            (*pmii).hasnext = 1;
        }
        nextname = FDI_read_string(fdi, hf, cabsize);
        if nextname.is_null() {
            if ((flags & cfheadPREV_CABINET as u16) != 0) && !pmii.is_null() {
                if !(*pmii).prevname.is_null() {
                    (((*fdi).free).unwrap_unchecked())((*pmii).prevname as *mut core::ffi::c_void);
                }
                if !(*pmii).previnfo.is_null() {
                    (((*fdi).free).unwrap_unchecked())((*pmii).previnfo as *mut core::ffi::c_void);
                }
            }
            set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
            return 0;
        } else {
            if !pmii.is_null() {
                (*pmii).nextname = nextname;
            } else {
                (((*fdi).free).unwrap_unchecked())(nextname as *mut core::ffi::c_void);
            }
        }
        nextinfo = FDI_read_string(fdi, hf, cabsize);
        if !nextinfo.is_null() {
            if !pmii.is_null() {
                (*pmii).nextinfo = nextinfo;
            } else {
                (((*fdi).free).unwrap_unchecked())(nextinfo as *mut core::ffi::c_void);
            }
        }
    }

    (*pfdici).cbCabinet = cabsize;
    (*pfdici).cFolders = num_folders as u16;
    (*pfdici).cFiles = num_files as u16;
    (*pfdici).setID = setid;
    (*pfdici).iCabinet = cabidx;
    (*pfdici).fReserve = if (flags & cfheadRESERVE_PRESENT as u16) != 0 { 1 } else { 0 };
    (*pfdici).hasprev = if (flags & cfheadPREV_CABINET as u16) != 0 { 1 } else { 0 };
    (*pfdici).hasnext = if (flags & cfheadNEXT_CABINET as u16) != 0 { 1 } else { 0 };

    1
}

unsafe fn fdi_Zipinflate_fixed(decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut fixed_tl: *mut Ziphuft = core::ptr::null_mut();
    let mut fixed_td: *mut Ziphuft = core::ptr::null_mut();
    let mut fixed_bl: i32 = 0;
    let mut fixed_bd: i32 = 0;
    let mut i: i32 = 0;
    let l: *mut cab_ULONG;

    l = core::ptr::addr_of_mut!((*decomp_state).methods.zip.ll[0]);

    i = 0;
    while i < 144 {
        *l.add(i as usize) = 8;
        i += 1;
    }
    while i < 256 {
        *l.add(i as usize) = 9;
        i += 1;
    }
    while i < 280 {
        *l.add(i as usize) = 7;
        i += 1;
    }
    while i < 288 {
        *l.add(i as usize) = 8;
        i += 1;
    }
    fixed_bl = 7;
    i = fdi_Ziphuft_build(
        l,
        288,
        257,
        core::ptr::addr_of!(Zipcplens[0]),
        core::ptr::addr_of!(Zipcplext[0]),
        core::ptr::addr_of_mut!(fixed_tl),
        core::ptr::addr_of_mut!(fixed_bl),
        decomp_state,
    );
    if i != 0 {
        return i;
    }

    i = 0;
    while i < 30 {
        *l.add(i as usize) = 5;
        i += 1;
    }
    fixed_bd = 5;
    i = fdi_Ziphuft_build(
        l,
        30,
        0,
        core::ptr::addr_of!(Zipcpdist[0]),
        core::ptr::addr_of!(Zipcpdext[0]),
        core::ptr::addr_of_mut!(fixed_td),
        core::ptr::addr_of_mut!(fixed_bd),
        decomp_state,
    );
    if i > 1 {
        fdi_Ziphuft_free((*decomp_state).fdi, fixed_tl);
        return i;
    }

    i = fdi_Zipinflate_codes(fixed_tl, fixed_td, fixed_bl, fixed_bd, decomp_state);

    fdi_Ziphuft_free((*decomp_state).fdi, fixed_td);
    fdi_Ziphuft_free((*decomp_state).fdi, fixed_tl);
    i
}

unsafe fn fdi_Zipinflate_dynamic(decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut i: i32;
    let mut j: u32;
    let mut ll: *mut u32;
    let mut l: u32;
    let mut m: u32;
    let mut n: u32;
    let mut tl: *mut Ziphuft = core::ptr::null_mut();
    let mut td: *mut Ziphuft = core::ptr::null_mut();
    let mut bl: i32;
    let mut bd: i32;
    let mut nb: u32;
    let mut nl: u32;
    let mut nd: u32;
    let mut b: u32;
    let mut k: u32;

    b = (*decomp_state).methods.zip.bb;
    k = (*decomp_state).methods.zip.bk;
    ll = (*decomp_state).methods.zip.ll.as_mut_ptr();

    /* ZIPNEEDBITS(5) */
    while k < 5 {
        let c: u8 = *(*decomp_state).methods.zip.inpos;
        (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
        b |= (c as u32) << k;
        k += 8;
    }
    nl = 257 + (b & 0x1f);
    /* ZIPDUMPBITS(5) */
    b >>= 5;
    k -= 5;

    /* ZIPNEEDBITS(5) */
    while k < 5 {
        let c: u8 = *(*decomp_state).methods.zip.inpos;
        (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
        b |= (c as u32) << k;
        k += 8;
    }
    nd = 1 + (b & 0x1f);
    /* ZIPDUMPBITS(5) */
    b >>= 5;
    k -= 5;

    /* ZIPNEEDBITS(4) */
    while k < 4 {
        let c: u8 = *(*decomp_state).methods.zip.inpos;
        (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
        b |= (c as u32) << k;
        k += 8;
    }
    nb = 4 + (b & 0xf);
    /* ZIPDUMPBITS(4) */
    b >>= 4;
    k -= 4;

    if nl > 288 || nd > 32 {
        return 1;
    }

    j = 0;
    while j < nb {
        /* ZIPNEEDBITS(3) */
        while k < 3 {
            let c: u8 = *(*decomp_state).methods.zip.inpos;
            (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
            b |= (c as u32) << k;
            k += 8;
        }
        *ll.offset(Zipborder[j as usize] as isize) = b & 7;
        /* ZIPDUMPBITS(3) */
        b >>= 3;
        k -= 3;
        j += 1;
    }
    while j < 19 {
        *ll.offset(Zipborder[j as usize] as isize) = 0;
        j += 1;
    }

    bl = 7;
    i = fdi_Ziphuft_build(ll, 19, 19, core::ptr::null(), core::ptr::null(), &mut tl, &mut bl, decomp_state);
    if i != 0 {
        if i == 1 {
            fdi_Ziphuft_free((*decomp_state).fdi, tl);
        }
        return i;
    }

    n = nl + nd;
    m = Zipmask[bl as usize] as u32;
    i = 0;
    l = 0;
    while (i as u32) < n {
        /* ZIPNEEDBITS((cab_ULONG)bl) */
        while k < (bl as u32) {
            let c: u8 = *(*decomp_state).methods.zip.inpos;
            (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
            b |= (c as u32) << k;
            k += 8;
        }
        td = tl.offset((b & m) as isize);
        j = (*td).b as u32;
        /* ZIPDUMPBITS(j) */
        b >>= j;
        k -= j;
        j = (*td).v.n as u32;
        if j < 16 {
            *ll.offset(i as isize) = j;
            l = j;
            i += 1;
        } else if j == 16 {
            /* ZIPNEEDBITS(2) */
            while k < 2 {
                let c: u8 = *(*decomp_state).methods.zip.inpos;
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            j = 3 + (b & 3);
            /* ZIPDUMPBITS(2) */
            b >>= 2;
            k -= 2;
            if (i as u32) + j > n {
                return 1;
            }
            while j > 0 {
                *ll.offset(i as isize) = l;
                i += 1;
                j -= 1;
            }
        } else if j == 17 {
            /* ZIPNEEDBITS(3) */
            while k < 3 {
                let c: u8 = *(*decomp_state).methods.zip.inpos;
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            j = 3 + (b & 7);
            /* ZIPDUMPBITS(3) */
            b >>= 3;
            k -= 3;
            if (i as u32) + j > n {
                return 1;
            }
            while j > 0 {
                *ll.offset(i as isize) = 0;
                i += 1;
                j -= 1;
            }
            l = 0;
        } else {
            /* ZIPNEEDBITS(7) */
            while k < 7 {
                let c: u8 = *(*decomp_state).methods.zip.inpos;
                (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(1);
                b |= (c as u32) << k;
                k += 8;
            }
            j = 11 + (b & 0x7f);
            /* ZIPDUMPBITS(7) */
            b >>= 7;
            k -= 7;
            if (i as u32) + j > n {
                return 1;
            }
            while j > 0 {
                *ll.offset(i as isize) = 0;
                i += 1;
                j -= 1;
            }
            l = 0;
        }
    }

    fdi_Ziphuft_free((*decomp_state).fdi, tl);

    (*decomp_state).methods.zip.bb = b;
    (*decomp_state).methods.zip.bk = k;

    bl = ZIPLBITS;
    i = fdi_Ziphuft_build(ll, nl, 257, Zipcplens.as_ptr() as *const u16, Zipcplext.as_ptr() as *const u16, &mut tl, &mut bl, decomp_state);
    if i != 0 {
        if i == 1 {
            fdi_Ziphuft_free((*decomp_state).fdi, tl);
        }
        return i;
    }

    bd = ZIPDBITS;
    let _ = fdi_Ziphuft_build(ll.offset(nl as isize), nd, 0, Zipcpdist.as_ptr() as *const u16, Zipcpdext.as_ptr() as *const u16, &mut td, &mut bd, decomp_state);

    if fdi_Zipinflate_codes(tl, td, bl, bd, decomp_state) != 0 {
        return 1;
    }

    fdi_Ziphuft_free((*decomp_state).fdi, tl);
    fdi_Ziphuft_free((*decomp_state).fdi, td);
    return 0;
}

#[no_mangle] pub unsafe extern "C" fn FDIIsCabinet(hfdi: HFDI, hf: INT_PTR, pfdici: PFDICABINETINFO) -> i32 {
    let rv;
    let fdi: *mut FDI_Int = get_fdi_ptr(hfdi);
    winedebug_rust::TRACE!("(hfdi == ^%p, hf == ^%ld, pfdici == ^%p)\n", hfdi, hf, pfdici);
    if fdi.is_null() {
        return 0;
    }
    if pfdici.is_null() {
        SetLastError(ERROR_BAD_ARGUMENTS as u32);
        return 0;
    }
    rv = FDI_read_entries(fdi, hf, pfdici, core::ptr::null_mut());
    if rv != 0 {
        (*pfdici).hasnext = 0;
    }
    return rv;
}

unsafe fn fdi_Zipinflate_block(e: *mut i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut b: u32 = (*decomp_state).methods.zip.bb;
    let mut k: u32 = (*decomp_state).methods.zip.bk;

    // ZIPNEEDBITS(1)
    while k < 1 {
        let c: i32 = *((*decomp_state).methods.zip.inpos) as i32;
        (*decomp_state).methods.zip.inpos = ((*decomp_state).methods.zip.inpos).add(1);
        b |= (c as u32) << k;
        k += 8;
    }

    *e = (b as i32) & 1;

    // ZIPDUMPBITS(1)
    b >>= 1;
    k -= 1;

    // ZIPNEEDBITS(2)
    while k < 2 {
        let c: i32 = *((*decomp_state).methods.zip.inpos) as i32;
        (*decomp_state).methods.zip.inpos = ((*decomp_state).methods.zip.inpos).add(1);
        b |= (c as u32) << k;
        k += 8;
    }

    let t: u32 = b & 3;

    // ZIPDUMPBITS(2)
    b >>= 2;
    k -= 2;

    (*decomp_state).methods.zip.bb = b;
    (*decomp_state).methods.zip.bk = k;

    if t == 2 {
        return fdi_Zipinflate_dynamic(decomp_state);
    }
    if t == 0 {
        return fdi_Zipinflate_stored(decomp_state);
    }
    if t == 1 {
        return fdi_Zipinflate_fixed(decomp_state);
    }
    return 2;
}

unsafe extern "C" fn ZIPfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut e: i32 = 0; /* cab_LONG e */

    winedebug_rust::TRACE!("(inlen == %d, outlen == %d)\n", inlen as i32, outlen as i32);

    (*decomp_state).methods.zip.inpos = (*decomp_state).inbuf.as_mut_ptr();
    (*decomp_state).methods.zip.window_posn = 0;
    (*decomp_state).methods.zip.bk = 0;
    (*decomp_state).methods.zip.bb = 0;

    if outlen as u32 > ZIPWSIZE {
        return 1; /* DECR_DATAFORMAT */
    }

    if *((*decomp_state).methods.zip.inpos) != 0x43 || *((*decomp_state).methods.zip.inpos.add(1)) != 0x4B {
        return 2; /* DECR_ILLEGALDATA */
    }
    (*decomp_state).methods.zip.inpos = (*decomp_state).methods.zip.inpos.add(2);

    loop {
        if fdi_Zipinflate_block(&raw mut e, decomp_state) != 0 {
            return 2; /* DECR_ILLEGALDATA */
        }
        if e != 0 {
            break;
        }
    }

    return 0; /* DECR_OK */
}

// ---- spliced: fdi_Ziphuft_build (dropped during reassembly) ----
unsafe fn fdi_Ziphuft_build(
    b: *mut u32,
    n: u32,
    s: u32,
    d: *const u16,
    e: *const u16,
    t: *mut *mut Ziphuft,
    m: *mut i32,
    decomp_state: *mut fdi_decomp_state,
) -> i32 {
    let mut t = t;
    let mut a: u32;
    let mut el: u32;
    let mut f: u32;
    let mut g: i32;
    let mut h: i32;
    let mut i: u32;
    let mut j: u32;
    let mut k: i32;
    let mut l: *mut i32;
    let mut p: *mut u32;
    let mut q: *mut Ziphuft;
    let mut r: Ziphuft = core::mem::zeroed();
    let mut w: i32;
    let mut xp: *mut u32;
    let mut y: i32;
    let mut z: u32;

    l = core::ptr::addr_of_mut!((*decomp_state).methods.zip.lx) as *mut i32;
    l = l.add(1);

    /* Generate counts for each bit length */
    el = if n > 256 { *b.add(256) } else { ZIPBMAX };

    for i in 0..(ZIPBMAX + 1) {
        (*decomp_state).methods.zip.c[i as usize] = 0;
    }
    p = b;
    i = n;
    loop {
        let idx = *p;
        (*decomp_state).methods.zip.c[idx as usize] += 1;
        p = p.add(1);
        i -= 1;
        if i == 0 {
            break;
        }
    }
    if (*decomp_state).methods.zip.c[0] == n {
        *t = core::ptr::null_mut();
        *m = 0;
        return 0;
    }

    /* Find minimum and maximum length, bound *m by those */
    j = 1;
    while j <= ZIPBMAX {
        if (*decomp_state).methods.zip.c[j as usize] != 0 {
            break;
        }
        j += 1;
    }
    k = j as i32; /* minimum code length */
    if (*m as u32) < j {
        *m = j as i32;
    }
    i = ZIPBMAX;
    while i != 0 {
        if (*decomp_state).methods.zip.c[i as usize] != 0 {
            break;
        }
        i -= 1;
    }
    g = i as i32; /* maximum code length */
    if (*m as u32) > i {
        *m = i as i32;
    }

    /* Adjust last length count to fill out codes, if needed */
    y = (1i32 << j) as i32;
    let max_len: u32 = i;
    let mut j_adj: u32 = j;
    while j_adj < max_len {
        y -= (*decomp_state).methods.zip.c[j_adj as usize] as i32;
        if y < 0 {
            return 2;
        }
        j_adj += 1;
        y <<= 1;
    }
    y -= (*decomp_state).methods.zip.c[max_len as usize] as i32;
    if y < 0 {
        return 2;
    }
    (*decomp_state).methods.zip.c[max_len as usize] += y as u32;

    /* Generate starting offsets LONGo the value table for each length */
    (*decomp_state).methods.zip.x[1] = {
        j = 0;
        j
    };
    p = core::ptr::addr_of_mut!((*decomp_state).methods.zip.c[0]).add(1);
    xp = core::ptr::addr_of_mut!((*decomp_state).methods.zip.x[0]).add(2);
    i = max_len;
    loop {
        i -= 1;
        if i == 0 {
            break;
        }
        *xp = {
            j += *p;
            j
        };
        xp = xp.add(1);
        p = p.add(1);
    }

    /* Make a table of values in order of bit lengths */
    p = b;
    i = 0;
    loop {
        j = *p;
        p = p.add(1);
        if j != 0 {
            let x_idx = j as usize;
            let v_idx = (*decomp_state).methods.zip.x[x_idx];
            (*decomp_state).methods.zip.v[v_idx as usize] = i;
            (*decomp_state).methods.zip.x[x_idx] = v_idx + 1;
        }
        i += 1;
        if i >= n {
            break;
        }
    }

    /* Generate the Huffman codes and for each, make the table entries */
    (*decomp_state).methods.zip.x[0] = {
        i = 0;
        i
    };
    p = core::ptr::addr_of_mut!((*decomp_state).methods.zip.v[0]);
    h = -1;
    w = {
        l.sub(1).write(0);
        0
    };
    // No bits decoded yet
    (*decomp_state).methods.zip.u[0] = core::ptr::null_mut();
    q = core::ptr::null_mut();
    z = 0;

    /* go through the bit lengths (k already is bits in shortest code) */
    for k_val in k..=g {
        a = (*decomp_state).methods.zip.c[k_val as usize];
        while a > 0 {
            a -= 1;

            /* here i is the Huffman code of length k_val bits for value *p */
            /* make tables up to required level */
            while k_val as i32 > w + *l.offset(h as isize) {
                w += *l.offset(h as isize);
                h += 1; /* add bits already decoded */

                /* compute minimum size table less than or equal to *m bits */
                z = g as u32 - w as u32;
                if z > (*m as u32) {
                    z = (*m as u32);
                }
                j = (k_val as i32 - w) as u32;
                f = 1u32 << j;
                if f > a + 1 {
                    /* too few codes for k-w bit table */
                    f -= a + 1;           /* deduct codes from patterns left */
                    xp = (*decomp_state).methods.zip.c.as_mut_ptr().add(k_val as usize);
                    loop {
                        j += 1;
                        if j >= z {
                            break;
                        }
                        f <<= 1;
                        xp = xp.add(1);
                        if f <= *xp {
                            break;      /* enough codes to use up j bits */
                        }
                        f -= *xp;       /* else deduct codes from patterns */
                    }
                }
                if (w as u32 + j) > el && (w as u32) < el {
                    j = el - w as u32;   /* make EOB code end at table */
                }
                z = 1u32 << j;           /* table entries for j-bit table */
                *(l.add(h as usize)) = j as i32; /* set table size in stack */

                /* allocate and link in new table */
                let alloc_size = (z + 1) as usize * core::mem::size_of::<Ziphuft>();
                let alloc_ptr = (((*(*decomp_state).fdi).alloc).unwrap_unchecked())(alloc_size as u32);
                if alloc_ptr.is_null() {
                    if h != 0 {
                        fdi_Ziphuft_free((*decomp_state).fdi, (*decomp_state).methods.zip.u[0]);
                    }
                    return 3;             /* not enough memory */
                }
                q = alloc_ptr as *mut Ziphuft;
                *t = q.add(1);           /* link to list for Ziphuft_free() */
                t = core::ptr::addr_of_mut!((*q).v.t);
                *t = core::ptr::null_mut();
                q = q.add(1);
                (*decomp_state).methods.zip.u[h as usize] = q; /* table starts after link */

                /* connect to last table, if there is one */
                if h != 0 {
                    (*decomp_state).methods.zip.x[h as usize] = i;
                    r.b = (*l.add(h as usize - 1)) as u8;    /* bits to dump before this table */
                    r.e = (16 + j) as u8;                    /* bits in this table */
                    r.v.t = q;                               /* pointer to this table */
                    let tmp_w = w as u32;
                    j = (i & ((1u32 << tmp_w) - 1)) >> (tmp_w - (*l.add(h as usize - 1) as u32));
                    let tbl_ptr = (*decomp_state).methods.zip.u[h as usize - 1];
                    if !tbl_ptr.is_null() {
                        *tbl_ptr.add(j as usize) = r;        /* connect to last table */
                    }
                }
            }

            /* set up table entry in r */
            r.b = (k_val as i32 - w) as u8;
            let v_start = core::ptr::addr_of!((*decomp_state).methods.zip.v[0]) as *mut u32;
            if p >= v_start.add(n as usize) {
                r.e = 99;               /* out of values--invalid code */
            } else if *p < s {
                r.e = if *p < 256 { 16 } else { 15 };
                r.v.n = *p as u16;
                p = p.add(1);
            } else {
                let idx = (*p - s) as usize;
                r.e = *e.add(idx) as u8;   /* non-simple--look up in lists */
                r.v.n = *d.add(idx);
                p = p.add(1);
            }

            /* fill code-like entries with r */
            f = 1u32 << (k_val as i32 - w);
            j = i >> w;
            while j < z {
                *q.add(j as usize) = r;
                j += f;
            }

            /* backwards increment the k-bit code i */
            let mut mask: u32 = 1 << (k_val as i32 - 1);
            while (i & mask) != 0 {
                i ^= mask;
                mask >>= 1;
            }
            i ^= mask;

            /* backup over finished tables */
            while (i & ((1 << w) - 1)) != (*decomp_state).methods.zip.x[h as usize] {
                h -= 1;
                w -= *l.add(h as usize);
            }
        }
    }

    /* return actual size of base table */
    *m = *l;
    return if y != 0 && g != 1 { 1 } else { 0 };
}

// EndGetIxx: wine little-endian read helpers (macros in C), module-level so both
// FDIIsCabinet and fdi_decomp use them.
#[inline]
unsafe fn end_get_i32(a: *const u8) -> i32 {
    ((*a.add(0) as u32)
        | ((*a.add(1) as u32) << 8)
        | ((*a.add(2) as u32) << 16)
        | ((*a.add(3) as u32) << 24)) as i32
}
#[inline]
unsafe fn end_get_i16(a: *const u8) -> i16 {
    ((*a.add(0) as u16) | ((*a.add(1) as u16) << 8)) as i16
}

extern "C" {
    fn strcpy(dst: *mut i8, src: *const i8) -> *mut i8;
    fn strcat(dst: *mut i8, src: *const i8) -> *mut i8;
}
extern "system" {
    fn lstrcmpiA(s1: *const i8, s2: *const i8) -> i32;
}

// ===========================================================================
// fdi_decomp (fdi.c:1938) — DROPPED by the rustconv tree-sitter parser
// (multi-line static sig at L1938 was skipped), hand-ported faithfully.
//
// Decompress `bytes` worth of output; on savemode, write it out via the user
// callback. Handles split cabinets: when a data block's uncompressed size is 0
// the file continues in the NEXT cabinet, which we open (with NEXT_CABINET
// notifications), parse its folders/files, and resume from.
// ===========================================================================
unsafe fn fdi_decomp(
    fi: *const fdi_file,
    savemode: i32,
    decomp_state: *mut fdi_decomp_state,
    pszCabPath: *mut i8,
    pfnfdin: Option<unsafe extern "C" fn(i32, *mut FDINOTIFICATION) -> i32>,
    pvUser: *mut core::ffi::c_void,
) -> i32 {
    const cfdata_SIZEOF: u32 = 8;
    const cfdata_CheckSum: usize = 0;
    const cfdata_CompressedSize: usize = 4;
    const cfdata_UncompressedSize: usize = 6;
    const cffold_SIZEOF: u32 = 8;
    const cffold_DataOffset: usize = 0;
    const cffold_NumBlocks: usize = 4;
    const cffold_CompType: usize = 6;
    const cffile_SIZEOF: u32 = 16;
    const cffile_UncompressedSize: usize = 0;
    const cffile_FolderOffset: usize = 4;
    const cffile_FolderIndex: usize = 8;
    const cffile_Date: usize = 10;
    const cffile_Time: usize = 12;
    const cffile_Attribs: usize = 14;
    const cffileCONTINUED_FROM_PREV: u16 = 0xFFFD;
    const MAX_PATH: usize = 260;
    const SEEK_SET: i32 = 0;
    const SEEK_CUR: i32 = 1;
    // _O_RDONLY|_O_BINARY ; _S_IREAD|_S_IWRITE (msvcrt fcntl/stat flags)
    const OPEN_FLAGS: i32 = 0x0000 | 0x8000;
    const OPEN_MODE: i32 = 0x0100 | 0x0080;

    let fdi: *mut FDI_Int = (*decomp_state).fdi;
    let read = (*fdi).read;
    let seek = (*fdi).seek;
    let open = (*fdi).open;
    let close = (*fdi).close;
    let write = (*fdi).write;
    let alloc = (*fdi).alloc;

    let mut bytes: u32 = if savemode != 0 {
        (*fi).length
    } else {
        (*fi).offset - (*decomp_state).offset
    };
    let mut buf: [u8; 8] = [0; 8];
    let mut cab: *mut fdi_decomp_state =
        if savemode != 0 && !(*decomp_state).decomp_cab.is_null() {
            (*decomp_state).decomp_cab
        } else {
            decomp_state
        };

    while bytes > 0 {
        // cando = max bytes we can emit from the current output buffer
        let mut cando: u32 = (*decomp_state).outlen as u32;
        if cando > bytes {
            cando = bytes;
        }
        if cando != 0 && savemode != 0 {
            (write.unwrap_unchecked())((*decomp_state).filehf as isize, (*decomp_state).outpos as *const core::ffi::c_void, cando);
        }
        (*decomp_state).outpos = (*decomp_state).outpos.add(cando as usize);
        (*decomp_state).outlen -= cando as u16;
        bytes -= cando;
        if bytes == 0 {
            break;
        }

        // emptied the output buffer: read the next data block header + data
        let mut inlen: u16 = 0;
        let mut outlen: u16 = 0;
        while outlen == 0 {
            if (read.unwrap_unchecked())((*cab).cabhf as isize, buf.as_mut_ptr() as *mut core::ffi::c_void, cfdata_SIZEOF) != cfdata_SIZEOF {
                return DECR_INPUT;
            }
            if (seek.unwrap_unchecked())((*cab).cabhf as isize, (*cab).mii.block_resv as i32, SEEK_CUR) == -1 {
                return DECR_INPUT;
            }
            let data: *mut u8 = (*decomp_state).inbuf.as_mut_ptr().add(inlen as usize);
            let len: u16 = end_get_i16(buf.as_ptr().add(cfdata_CompressedSize)) as u16;
            inlen += len;
            if inlen as u32 > CAB_INPUTMAX {
                return DECR_INPUT;
            }
            if (read.unwrap_unchecked())((*cab).cabhf as isize, data as *mut core::ffi::c_void, len as u32) != len as u32 {
                return DECR_INPUT;
            }
            // clear two bytes after read-in data
            *data.add(len as usize + 1) = 0;
            *data.add(len as usize + 2) = 0;

            let cksum: u32 = end_get_i32(buf.as_ptr().add(cfdata_CheckSum)) as u32;
            if cksum != 0
                && cksum != checksum(buf.as_mut_ptr().add(4), 4, checksum(data, len, 0))
            {
                return DECR_CHECKSUM;
            }
            outlen = end_get_i16(buf.as_ptr().add(cfdata_UncompressedSize)) as u16;

            if outlen == 0 {
                // split block: the file continues in the next cabinet
                let mut fullpath: [i8; MAX_PATH] = [0; MAX_PATH];
                let mut userpath: [i8; 256] = [0; 256];
                let mut buf2: [u8; 64] = [0; 64];
                let mut fdin: FDINOTIFICATION = core::mem::zeroed();
                let mut fdici: FDICABINETINFO = core::mem::zeroed();
                let emptystring: i8 = 0;

                'tryanothercab: loop {
                    if (*cab).next.is_null() {
                        if (*cab).mii.hasnext == 0 {
                            return DECR_INPUT;
                        }
                        (*cab).next = (alloc.unwrap_unchecked())(core::mem::size_of::<fdi_decomp_state>() as u32)
                            as *mut fdi_decomp_state;
                        if (*cab).next.is_null() {
                            return DECR_NOMEMORY;
                        }
                        core::ptr::write_bytes((*cab).next as *mut u8, 0, core::mem::size_of::<fdi_decomp_state>());

                        // copy pszCabPath to userpath
                        core::ptr::write_bytes(userpath.as_mut_ptr() as *mut u8, 0, 256);
                        let pathlen0: i32 = if !pszCabPath.is_null() { strlen(pszCabPath) as i32 } else { 0 };
                        if pathlen0 != 0 && pathlen0 < 256 {
                            strcpy(userpath.as_mut_ptr(), pszCabPath);
                        }

                        // initial fdintNEXT_CABINET notification
                        core::ptr::write_bytes(&mut fdin as *mut FDINOTIFICATION as *mut u8, 0, core::mem::size_of::<FDINOTIFICATION>());
                        fdin.psz1 = if !(*cab).mii.nextname.is_null() { (*cab).mii.nextname } else { &emptystring as *const i8 as *mut i8 };
                        fdin.psz2 = if !(*cab).mii.nextinfo.is_null() { (*cab).mii.nextinfo } else { &emptystring as *const i8 as *mut i8 };
                        fdin.psz3 = userpath.as_mut_ptr();
                        fdin.fdie = 0; // FDIERROR_NONE
                        fdin.pv = pvUser;

                        if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                            return DECR_USERABORT;
                        }

                        let cabhf: isize = loop {
                            let pathlen: i32 = strlen(userpath.as_ptr()) as i32;
                            let filenamelen: i32 = if !(*cab).mii.nextname.is_null() {
                                strlen((*cab).mii.nextname) as i32
                            } else {
                                0
                            };
                            if (pathlen + filenamelen + 3) as usize > MAX_PATH {
                                winedebug_rust::ERR!("MAX_PATH exceeded.\n");
                                return DECR_ILLEGALDATA;
                            }
                            // paste path + filename, Wine's way (upstream 4681b3e473a dropped the
                            // ReactOS variant, which always inserted '\\' and so turned an empty
                            // user path into the root-relative "\\next.cab")
                            fullpath[0] = 0;
                            if pathlen != 0 {
                                strcpy(fullpath.as_mut_ptr(), userpath.as_ptr());
                                if fullpath[pathlen as usize - 1] != b'\\' as i8 {
                                    strcat(fullpath.as_mut_ptr(), b"\\\0".as_ptr() as *const i8);
                                }
                            }
                            if filenamelen != 0 {
                                strcat(fullpath.as_mut_ptr(), (*cab).mii.nextname);
                            } else if fullpath[0] != 0 {
                                // remove the trailing backslash
                                fullpath[strlen(fullpath.as_ptr()) - 1] = 0;
                            }

                            winedebug_rust::TRACE!("full cab path/file name: %s\n", debugstr_a(fullpath.as_ptr()));

                            let h: isize = (open.unwrap_unchecked())(fullpath.as_mut_ptr(), OPEN_FLAGS, OPEN_MODE);
                            if h == -1 {
                                fdin.fdie = FDIERROR_CABINET_NOT_FOUND;
                                if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                                    return DECR_USERABORT;
                                }
                                continue;
                            }
                            if h == 0 {
                                fdin.fdie = FDIERROR_CABINET_NOT_FOUND;
                                if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                                    return DECR_USERABORT;
                                }
                                continue;
                            }
                            if FDI_read_entries(fdi, h as INT_PTR, &mut fdici, &mut (*(*cab).next).mii) == 0 {
                                winedebug_rust::WARN!("FDIIsCabinet failed.\n");
                                (close.unwrap_unchecked())(h as isize);
                                fdin.fdie = FDIERROR_NOT_A_CABINET;
                                if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                                    return DECR_USERABORT;
                                }
                                continue;
                            }
                            if fdici.setID != (*cab).setID || fdici.iCabinet != (*cab).iCabinet + 1 {
                                winedebug_rust::WARN!("Wrong Cabinet.\n");
                                (close.unwrap_unchecked())(h as isize);
                                fdin.fdie = FDIERROR_WRONG_CABINET;
                                if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                                    return DECR_USERABORT;
                                }
                                continue;
                            }
                            break h;
                        };

                        // cabinet notification
                        core::ptr::write_bytes(&mut fdin as *mut FDINOTIFICATION as *mut u8, 0, core::mem::size_of::<FDINOTIFICATION>());
                        fdin.setID = fdici.setID;
                        fdin.iCabinet = fdici.iCabinet;
                        fdin.pv = pvUser;
                        fdin.psz1 = if !(*(*cab).next).mii.nextname.is_null() { (*(*cab).next).mii.nextname } else { &emptystring as *const i8 as *mut i8 };
                        fdin.psz2 = if !(*(*cab).next).mii.nextinfo.is_null() { (*(*cab).next).mii.nextinfo } else { &emptystring as *const i8 as *mut i8 };
                        fdin.psz3 = pszCabPath;
                        if (pfnfdin.unwrap())(fdintCABINET_INFO, &mut fdin) != 0 {
                            return DECR_USERABORT;
                        }

                        (*(*cab).next).setID = fdici.setID;
                        (*(*cab).next).iCabinet = fdici.iCabinet;
                        (*(*cab).next).fdi = fdi;
                        (*(*cab).next).filehf = (*decomp_state).filehf;
                        (*(*cab).next).cabhf = cabhf as INT_PTR;
                        (*(*cab).next).decompress = (*decomp_state).decompress;

                        cab = (*cab).next;

                        // read folders
                        let mut linkfol: *mut fdi_folder = core::ptr::null_mut();
                        let mut i: u32 = 0;
                        while i < fdici.cFolders as u32 {
                            if (read.unwrap_unchecked())((*cab).cabhf as isize, buf2.as_mut_ptr() as *mut core::ffi::c_void, cffold_SIZEOF) != cffold_SIZEOF {
                                return DECR_INPUT;
                            }
                            if (*cab).mii.folder_resv > 0 {
                                (seek.unwrap_unchecked())((*cab).cabhf as isize, (*cab).mii.folder_resv, SEEK_CUR);
                            }
                            let fol: *mut fdi_folder = (alloc.unwrap_unchecked())(core::mem::size_of::<fdi_folder>() as u32) as *mut fdi_folder;
                            if fol.is_null() {
                                winedebug_rust::ERR!("out of memory!\n");
                                return DECR_NOMEMORY;
                            }
                            core::ptr::write_bytes(fol as *mut u8, 0, core::mem::size_of::<fdi_folder>());
                            if (*cab).firstfol.is_null() {
                                (*cab).firstfol = fol;
                            }
                            (*fol).offset = end_get_i32(buf2.as_ptr().add(cffold_DataOffset)) as cab_off_t;
                            (*fol).num_blocks = end_get_i16(buf2.as_ptr().add(cffold_NumBlocks)) as cab_UWORD;
                            (*fol).comp_type = end_get_i16(buf2.as_ptr().add(cffold_CompType)) as cab_UWORD;
                            if !linkfol.is_null() {
                                (*linkfol).next = fol;
                            }
                            linkfol = fol;
                            i += 1;
                        }

                        // read files
                        let mut linkfile: *mut fdi_file = core::ptr::null_mut();
                        let mut i2: u32 = 0;
                        while i2 < fdici.cFiles as u32 {
                            if (read.unwrap_unchecked())((*cab).cabhf as isize, buf2.as_mut_ptr() as *mut core::ffi::c_void, cffile_SIZEOF) != cffile_SIZEOF {
                                return DECR_INPUT;
                            }
                            let file: *mut fdi_file = (alloc.unwrap_unchecked())(core::mem::size_of::<fdi_file>() as u32) as *mut fdi_file;
                            if file.is_null() {
                                winedebug_rust::ERR!("out of memory!\n");
                                return DECR_NOMEMORY;
                            }
                            core::ptr::write_bytes(file as *mut u8, 0, core::mem::size_of::<fdi_file>());
                            if (*cab).firstfile.is_null() {
                                (*cab).firstfile = file;
                            }
                            (*file).length = end_get_i32(buf2.as_ptr().add(cffile_UncompressedSize)) as cab_ULONG;
                            (*file).offset = end_get_i32(buf2.as_ptr().add(cffile_FolderOffset)) as cab_ULONG;
                            (*file).index = end_get_i16(buf2.as_ptr().add(cffile_FolderIndex)) as cab_UWORD;
                            (*file).time = end_get_i16(buf2.as_ptr().add(cffile_Time)) as cab_UWORD;
                            (*file).date = end_get_i16(buf2.as_ptr().add(cffile_Date)) as cab_UWORD;
                            (*file).attribs = end_get_i16(buf2.as_ptr().add(cffile_Attribs)) as cab_UWORD;
                            (*file).filename = FDI_read_string(fdi, (*cab).cabhf, fdici.cbCabinet);
                            if (*file).filename.is_null() {
                                return DECR_INPUT;
                            }
                            if !linkfile.is_null() {
                                (*linkfile).next = file;
                            }
                            linkfile = file;
                            i2 += 1;
                        }
                    } else {
                        cab = (*cab).next;
                    }

                    // iterate files: find the continued file and resume from it
                    let mut success = false;
                    let mut file: *mut fdi_file = (*cab).firstfile;
                    while !file.is_null() {
                        if ((*file).index & cffileCONTINUED_FROM_PREV) == cffileCONTINUED_FROM_PREV
                            && lstrcmpiA((*fi).filename, (*file).filename) == 0
                        {
                            success = true;
                            if (seek.unwrap_unchecked())((*cab).cabhf as isize, (*(*cab).firstfol).offset as i32, SEEK_SET) == -1 {
                                return DECR_INPUT;
                            }
                            break;
                        }
                        file = (*file).next;
                    }
                    if !success {
                        continue 'tryanothercab;
                    }
                    break;
                }
            }
        }

        // decompress block
        let err = ((*decomp_state).decompress.unwrap())(inlen as i32, outlen as i32, decomp_state);
        if err != 0 {
            return err;
        }
        (*decomp_state).outlen = outlen;
        (*decomp_state).outpos = (*decomp_state).outbuf.as_mut_ptr();
    }

    (*decomp_state).decomp_cab = cab;
    DECR_OK
}


/***********************************************************************
 *		FDICopy (CABINET.22)
 *
 * Iterates over the files in a cabinet, notifying the caller and (when
 * the callback asks for extraction) decompressing each into the handle
 * the callback returns. Faithful port of dll/win32/cabinet/fdi.c; the C
 * `goto bail_and_fail` cleanup is modeled with a labeled `'main` block
 * plus a `bail` flag so both exits run the same free path.
 */
#[no_mangle]
pub unsafe extern "C" fn FDICopy(
    hfdi: HFDI,
    pszCabinet: *mut i8,
    pszCabPath: *mut i8,
    _flags: i32,
    pfnfdin: PFNFDINOTIFY,
    _pfnfdid: *const core::ffi::c_void, // PFNFDIDECRYPT — unused (as in wine)
    pvUser: *mut core::ffi::c_void,
) -> i32 {
    use core::mem::size_of;
    use core::ptr::addr_of_mut;

    const FALSE: i32 = 0;
    const TRUE: i32 = 1;
    // fdierror.h enum values
    // winerror.h / dderror.h
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
    // crt open flags (cabinet.h)
    const _O_RDONLY: i32 = 0;
    const _O_BINARY: i32 = 0x8000;
    const _S_IREAD: i32 = 0x0100;
    const _S_IWRITE: i32 = 0x0080;
    // cffolder / cffile field offsets (cabinet.h)
    const cffold_SIZEOF: u32 = 8;
    const cffold_DataOffset: usize = 0;
    const cffold_NumBlocks: usize = 4;
    const cffold_CompType: usize = 6;
    const cffile_SIZEOF: u32 = 16;
    const cffile_UncompressedSize: usize = 0;
    const cffile_FolderOffset: usize = 4;
    const cffile_FolderIndex: usize = 8;
    const cffile_Date: usize = 10;
    const cffile_Time: usize = 12;
    const cffile_Attribs: usize = 14;
    const cffileCONTINUED_FROM_PREV: u16 = 0xFFFD;
    const cffileCONTINUED_TO_NEXT: u16 = 0xFFFE;
    const cffoldCOMPTYPE_MASK: u16 = 0x000F;
    const cffoldCOMPTYPE_NONE: i32 = 0x0000;
    const cffoldCOMPTYPE_MSZIP: i32 = 0x0001;
    const cffoldCOMPTYPE_QUANTUM: i32 = 0x0002;
    const cffoldCOMPTYPE_LZX: i32 = 0x0003;
    const cffile_A_EXEC: u16 = 0x40;
    const SEEK_SET: i32 = 0;
    const SEEK_CUR: i32 = 1;
    const MAX_PATH: usize = 260; // local: types::* and ntdll_sys::* both glob-export it

    let fdi: *mut FDI_Int = get_fdi_ptr(hfdi);

    winedebug_rust::TRACE!("FDICopy(hfdi == ^%p, pszCabinet == %s, pszCabPath == %s)\n", hfdi, debugstr_a(pszCabinet), debugstr_a(pszCabPath));

    if fdi.is_null() { return FALSE; }

    let alloc = (*fdi).alloc;
    let free = (*fdi).free;

    let decomp_state: *mut fdi_decomp_state =
        (alloc.unwrap_unchecked())(size_of::<fdi_decomp_state>() as u32) as *mut fdi_decomp_state;
    if decomp_state.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }
    core::ptr::write_bytes(decomp_state as *mut u8, 0, size_of::<fdi_decomp_state>());

    let pathlen: usize = if pszCabPath.is_null() { 0 } else { strlen(pszCabPath) };
    let filenamelen: usize = if pszCabinet.is_null() { 0 } else { strlen(pszCabinet) };

    // slight overestimation here to save CPU cycles in the developer's brain
    if (pathlen + filenamelen + 3) > MAX_PATH {
        winedebug_rust::ERR!("MAX_PATH exceeded.\n");
        (free.unwrap_unchecked())(decomp_state as *mut core::ffi::c_void);
        set_error(fdi, FDIERROR_CABINET_NOT_FOUND, ERROR_FILE_NOT_FOUND as i32);
        return FALSE;
    }

    // paste the path and filename together
    let mut fullpath: [i8; MAX_PATH] = [0; MAX_PATH];
    fullpath[0] = 0;
    if pathlen != 0 { strcpy(fullpath.as_mut_ptr(), pszCabPath); }
    if filenamelen != 0 { strcat(fullpath.as_mut_ptr(), pszCabinet); }

    // get a handle to the cabfile
    let cabhf: isize = (((*fdi).open).unwrap_unchecked())(fullpath.as_mut_ptr(), _O_RDONLY | _O_BINARY, _S_IREAD | _S_IWRITE);
    if cabhf == -1 {
        (free.unwrap_unchecked())(decomp_state as *mut core::ffi::c_void);
        set_error(fdi, FDIERROR_CABINET_NOT_FOUND, 0);
        SetLastError(ERROR_FILE_NOT_FOUND);
        return FALSE;
    }

    let mut fdici: FDICABINETINFO = core::mem::zeroed();

    // check if it's really a cabfile
    if FDI_read_entries(fdi, cabhf as INT_PTR, &mut fdici, addr_of_mut!((*decomp_state).mii)) == 0 {
        winedebug_rust::WARN!("FDI_read_entries failed\n");
        (free.unwrap_unchecked())(decomp_state as *mut core::ffi::c_void);
        (((*fdi).close).unwrap_unchecked())(cabhf);
        return FALSE;
    }

    // fdi_decomp wants the callback typed with an i32 (vs INT_PTR) return; it is
    // the same C function pointer — re-type it (identical ABI on i386).
    let pfn_for_decomp: Option<unsafe extern "C" fn(i32, *mut FDINOTIFICATION) -> i32> =
        Some(core::mem::transmute::<PFNFDINOTIFY, unsafe extern "C" fn(i32, *mut FDINOTIFICATION) -> i32>(pfnfdin));

    let mut fdin: FDINOTIFICATION = core::mem::zeroed();
    let mut emptystring: i8 = 0;
    let mut buf: [u8; 64] = [0; 64];
    let mut fol: *mut fdi_folder = core::ptr::null_mut();
    let mut linkfol: *mut fdi_folder = core::ptr::null_mut();
    let mut file: *mut fdi_file = core::ptr::null_mut();
    let mut linkfile: *mut fdi_file = core::ptr::null_mut();
    let mut filehf: isize = 0;
    let mut i: u32;

    let mut bail = false;
    'main: loop {
        // cabinet notification
        core::ptr::write_bytes(addr_of_mut!(fdin) as *mut u8, 0, size_of::<FDINOTIFICATION>());
        fdin.setID = fdici.setID;
        fdin.iCabinet = fdici.iCabinet;
        fdin.pv = pvUser;
        fdin.psz1 = if (*decomp_state).mii.nextname.is_null() { addr_of_mut!(emptystring) } else { (*decomp_state).mii.nextname };
        fdin.psz2 = if (*decomp_state).mii.nextinfo.is_null() { addr_of_mut!(emptystring) } else { (*decomp_state).mii.nextinfo };
        fdin.psz3 = pszCabPath;

        if (pfnfdin.unwrap_unchecked())(fdintCABINET_INFO, &mut fdin) == -1 {
            set_error(fdi, FDIERROR_USER_ABORT, 0);
            bail = true; break 'main;
        }

        (*decomp_state).setID = fdici.setID;
        (*decomp_state).iCabinet = fdici.iCabinet;
        (*decomp_state).cabhf = cabhf as INT_PTR;

        // read folders
        i = 0;
        while i < fdici.cFolders as u32 {
            if (((*fdi).read).unwrap_unchecked())(cabhf, buf.as_mut_ptr() as *mut core::ffi::c_void, cffold_SIZEOF) != cffold_SIZEOF {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                bail = true; break 'main;
            }
            if (*decomp_state).mii.folder_resv > 0 {
                (((*fdi).seek).unwrap_unchecked())(cabhf, (*decomp_state).mii.folder_resv, SEEK_CUR);
            }
            fol = (alloc.unwrap_unchecked())(size_of::<fdi_folder>() as u32) as *mut fdi_folder;
            if fol.is_null() {
                winedebug_rust::ERR!("out of memory!\n");
                set_error(fdi, FDIERROR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
                bail = true; break 'main;
            }
            core::ptr::write_bytes(fol as *mut u8, 0, size_of::<fdi_folder>());
            if (*decomp_state).firstfol.is_null() { (*decomp_state).firstfol = fol; }

            (*fol).offset = end_get_i32(buf.as_ptr().add(cffold_DataOffset)) as cab_off_t;
            (*fol).num_blocks = end_get_i16(buf.as_ptr().add(cffold_NumBlocks)) as u16;
            (*fol).comp_type = end_get_i16(buf.as_ptr().add(cffold_CompType)) as u16;

            if !linkfol.is_null() { (*linkfol).next = fol; }
            linkfol = fol;
            i += 1;
        }

        // read files
        i = 0;
        while i < fdici.cFiles as u32 {
            if (((*fdi).read).unwrap_unchecked())(cabhf, buf.as_mut_ptr() as *mut core::ffi::c_void, cffile_SIZEOF) != cffile_SIZEOF {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                bail = true; break 'main;
            }
            file = (alloc.unwrap_unchecked())(size_of::<fdi_file>() as u32) as *mut fdi_file;
            if file.is_null() {
                winedebug_rust::ERR!("out of memory!\n");
                set_error(fdi, FDIERROR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
                bail = true; break 'main;
            }
            core::ptr::write_bytes(file as *mut u8, 0, size_of::<fdi_file>());
            if (*decomp_state).firstfile.is_null() { (*decomp_state).firstfile = file; }

            (*file).length = end_get_i32(buf.as_ptr().add(cffile_UncompressedSize)) as cab_ULONG;
            (*file).offset = end_get_i32(buf.as_ptr().add(cffile_FolderOffset)) as cab_ULONG;
            (*file).index = end_get_i16(buf.as_ptr().add(cffile_FolderIndex)) as u16;
            (*file).time = end_get_i16(buf.as_ptr().add(cffile_Time)) as u16;
            (*file).date = end_get_i16(buf.as_ptr().add(cffile_Date)) as u16;
            (*file).attribs = end_get_i16(buf.as_ptr().add(cffile_Attribs)) as u16;
            (*file).filename = FDI_read_string(fdi, cabhf as INT_PTR, fdici.cbCabinet);

            if (*file).filename.is_null() {
                set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                bail = true; break 'main;
            }
            if !linkfile.is_null() { (*linkfile).next = file; }
            linkfile = file;
            i += 1;
        }

        // extract loop
        file = (*decomp_state).firstfile;
        while !file.is_null() {
            // partial-file notification
            if ((*file).index & cffileCONTINUED_FROM_PREV) == cffileCONTINUED_FROM_PREV {
                core::ptr::write_bytes(addr_of_mut!(fdin) as *mut u8, 0, size_of::<FDINOTIFICATION>());
                fdin.pv = pvUser;
                fdin.psz1 = (*file).filename;
                fdin.psz2 = if (*decomp_state).mii.prevname.is_null() { addr_of_mut!(emptystring) } else { (*decomp_state).mii.prevname };
                fdin.psz3 = if (*decomp_state).mii.previnfo.is_null() { addr_of_mut!(emptystring) } else { (*decomp_state).mii.previnfo };
                if (pfnfdin.unwrap_unchecked())(fdintPARTIAL_FILE, &mut fdin) == -1 {
                    set_error(fdi, FDIERROR_USER_ABORT, 0);
                    bail = true; break 'main;
                }
                // don't decompress partial files
                (*file).oppressed = TRUE;
            }
            if (*file).oppressed != 0 {
                filehf = 0;
            } else {
                core::ptr::write_bytes(addr_of_mut!(fdin) as *mut u8, 0, size_of::<FDINOTIFICATION>());
                fdin.pv = pvUser;
                fdin.psz1 = (*file).filename;
                fdin.cb = (*file).length as i32;
                fdin.date = (*file).date;
                fdin.time = (*file).time;
                fdin.attribs = (*file).attribs;
                fdin.iFolder = (*file).index;
                filehf = (pfnfdin.unwrap_unchecked())(fdintCOPY_FILE, &mut fdin);
                if filehf == -1 {
                    set_error(fdi, FDIERROR_USER_ABORT, 0);
                    filehf = 0;
                    bail = true; break 'main;
                }
            }

            // find the folder for this file if necessary
            if filehf != 0 {
                fol = (*decomp_state).firstfol;
                if ((*file).index & cffileCONTINUED_TO_NEXT) == cffileCONTINUED_TO_NEXT {
                    // pick the last folder
                    while !(*fol).next.is_null() { fol = (*fol).next; }
                } else {
                    let mut i2: u32 = 0;
                    while i2 < (*file).index as u32 {
                        if !(*fol).next.is_null() { fol = (*fol).next; } // bug resistance
                        i2 += 1;
                    }
                }
            }

            if filehf != 0 {
                let comptype: u16 = (*fol).comp_type;
                let ct1: i32 = (comptype & cffoldCOMPTYPE_MASK) as i32;
                let ct2: i32 = if !(*decomp_state).current.is_null() {
                    ((*(*decomp_state).current).comp_type & cffoldCOMPTYPE_MASK) as i32
                } else { 0 };
                let mut err: i32 = 0;

                winedebug_rust::TRACE!("Extracting file %s as requested by callee.\n", debugstr_a((*file).filename));

                // set up decomp_state
                (*decomp_state).fdi = fdi;
                (*decomp_state).filehf = filehf as INT_PTR;

                // change of folder / compression / went backwards?
                if (ct1 != ct2) || ((*decomp_state).current != fol) || ((*file).offset < (*decomp_state).offset) {
                    // free stuff for the old decompressor
                    if ct2 == cffoldCOMPTYPE_LZX {
                        if !(*decomp_state).methods.lzx.window.is_null() {
                            (free.unwrap_unchecked())((*decomp_state).methods.lzx.window as *mut core::ffi::c_void);
                            (*decomp_state).methods.lzx.window = core::ptr::null_mut();
                        }
                    } else if ct2 == cffoldCOMPTYPE_QUANTUM {
                        if !(*decomp_state).methods.qtm.window.is_null() {
                            (free.unwrap_unchecked())((*decomp_state).methods.qtm.window as *mut core::ffi::c_void);
                            (*decomp_state).methods.qtm.window = core::ptr::null_mut();
                        }
                    }

                    (*decomp_state).decomp_cab = core::ptr::null_mut();
                    (((*fdi).seek).unwrap_unchecked())(cabhf, (*fol).offset as i32, SEEK_SET);
                    (*decomp_state).offset = 0;
                    (*decomp_state).outlen = 0;

                    // initialize the new decompressor
                    if ct1 == cffoldCOMPTYPE_NONE {
                        (*decomp_state).decompress = Some(NONEfdi_decomp);
                    } else if ct1 == cffoldCOMPTYPE_MSZIP {
                        (*decomp_state).decompress = Some(ZIPfdi_decomp);
                    } else if ct1 == cffoldCOMPTYPE_QUANTUM {
                        (*decomp_state).decompress = Some(QTMfdi_decomp);
                        err = QTMfdi_init(((comptype >> 8) & 0x1f) as i32, ((comptype >> 4) & 0xF) as i32, decomp_state);
                    } else if ct1 == cffoldCOMPTYPE_LZX {
                        (*decomp_state).decompress = Some(LZXfdi_decomp);
                        err = LZXfdi_init(((comptype >> 8) & 0x1f) as i32, decomp_state);
                    } else {
                        err = DECR_DATAFORMAT;
                    }
                }

                (*decomp_state).current = fol;

                if err == DECR_OK {
                    // ok
                } else if err == DECR_NOMEMORY {
                    set_error(fdi, FDIERROR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
                    bail = true; break 'main;
                } else {
                    set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                    bail = true; break 'main;
                }

                if (*file).offset > (*decomp_state).offset {
                    // decode bytes and send them to /dev/null
                    let r = fdi_decomp(file, 0, decomp_state, pszCabPath, pfn_for_decomp, pvUser);
                    if r == DECR_OK {
                        // ok
                    } else if r == DECR_USERABORT {
                        set_error(fdi, FDIERROR_USER_ABORT, 0);
                        bail = true; break 'main;
                    } else if r == DECR_NOMEMORY {
                        set_error(fdi, FDIERROR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
                        bail = true; break 'main;
                    } else {
                        set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                        bail = true; break 'main;
                    }
                    (*decomp_state).offset = (*file).offset;
                }

                // now do the actual decompression
                let err2 = fdi_decomp(file, 1, decomp_state, pszCabPath, pfn_for_decomp, pvUser);
                if err2 != 0 { (*decomp_state).current = core::ptr::null_mut(); }
                else { (*decomp_state).offset += (*file).length; }

                // fdintCLOSE_FILE_INFO notification
                core::ptr::write_bytes(addr_of_mut!(fdin) as *mut u8, 0, size_of::<FDINOTIFICATION>());
                fdin.pv = pvUser;
                fdin.psz1 = (*file).filename;
                fdin.hf = filehf;
                fdin.cb = if ((*file).attribs & cffile_A_EXEC) != 0 { 1 } else { 0 };
                fdin.date = (*file).date;
                fdin.time = (*file).time;
                fdin.attribs = (*file).attribs;
                fdin.iFolder = (*file).index;
                (pfnfdin.unwrap_unchecked())(fdintCLOSE_FILE_INFO, &mut fdin);
                filehf = 0;

                if err2 == DECR_OK {
                    // ok
                } else if err2 == DECR_USERABORT {
                    set_error(fdi, FDIERROR_USER_ABORT, 0);
                    bail = true; break 'main;
                } else if err2 == DECR_NOMEMORY {
                    set_error(fdi, FDIERROR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
                    bail = true; break 'main;
                } else {
                    set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
                    bail = true; break 'main;
                }
            }

            file = (*file).next;
        }

        break 'main; // success
    }

    // cleanup (shared by success + bail, matching C bail_and_fail/return TRUE)
    if !fol.is_null() { free_decompression_temps(fdi, fol, decomp_state); }
    if bail && filehf != 0 { (((*fdi).close).unwrap_unchecked())(filehf); }
    free_decompression_mem(fdi, decomp_state);

    if bail { FALSE } else { TRUE }
}
