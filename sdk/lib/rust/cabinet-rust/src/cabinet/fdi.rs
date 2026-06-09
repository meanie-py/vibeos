use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};
use crate::types::*;

extern "C" {
    fn ERR(format: *const i8, ...);
    fn FIXME(format: *const i8, ...);
    fn TRACE(format: *const i8, ...);
    fn WARN(format: *const i8, ...);
}

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
const FDIERROR_NONE: i32 = 0;
const FDIERROR_ALLOC_FAIL: i32 = 2;
const TRUE: i32 = 1;
const CAB_BLOCKMAX: i32 = 32768; // typical value

extern "C" {
    fn READ_BITS(bitbuf: *mut u32, bitsleft: *mut i32, inpos: *mut *mut u8, n: u32) -> u32;
    fn LENTABLE(decomp_state: *mut fdi_decomp_state, table: u32) -> *mut u8;
    fn BUILD_TABLE(decomp_state: *mut fdi_decomp_state, table: u32);
    fn READ_HUFFSYM(decomp_state: *mut fdi_decomp_state, table: u32, bitbuf: *mut u32, bitsleft: *mut i32, inpos: *mut *mut u8) -> i32;
    fn strlen(s: *const i8) -> usize;
}

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
    TRACE(
        b"(pfnalloc == ^%p, pfnfree == ^%p, pfnopen == ^%p, pfnread == ^%p, pfnwrite == ^%p, pfnclose == ^%p, pfnseek == ^%p, cpuType == %d, perf == ^%p)\n\0"
            .as_ptr() as *const i8,
        pfnalloc,
        pfnfree,
        pfnopen,
        pfnread,
        pfnwrite,
        pfnclose,
        pfnseek,
        cpuType,
        perf,
    );

    if (pfnalloc as *const ()).is_null() || (pfnfree as *const ()).is_null() {
        (*perf).erfOper = FDIERROR_NONE;
        (*perf).erfType = ERROR_BAD_ARGUMENTS as i32;
        (*perf).fError = TRUE;

        SetLastError(ERROR_BAD_ARGUMENTS);
        return core::ptr::null_mut();
    }

    let fdi = pfnalloc(core::mem::size_of::<FDI_Int>() as u32) as *mut FDI_Int;
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
    ((*fdi).seek)(hf as isize, 0, SEEK_CUR)
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
        const LZX_NUM_CHARS: u32 = 0; // Placeholder, value not critical for init
        const LZX_BLOCKTYPE_INVALID: i32 = 0;

        let wndsize: u32 = 1u32 << (window as u32);
        let mut posn_slots: i32;

        if window < 15 || window > 21 {
            return DECR_DATAFORMAT;
        }

        if (*decomp_state).methods.lzx.actual_size < wndsize {
            if !(*decomp_state).methods.lzx.window.is_null() {
                ((*(*decomp_state).fdi).free)((*decomp_state).methods.lzx.window as *mut core::ffi::c_void);
            }
            (*decomp_state).methods.lzx.window = core::ptr::null_mut();
        }

        if (*decomp_state).methods.lzx.window.is_null() {
            let mem = ((*(*decomp_state).fdi).alloc)(wndsize as u32) as *mut u8;
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

unsafe fn NONEfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
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
        ((*fdi).free)(p as *mut core::ffi::c_void);
        p = q;
    }
}

unsafe fn fdi_Zipinflate_codes(tl: *const Ziphuft, td: *const Ziphuft, bl: i32, bd: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    extern "C" {
        static Zipmask: [u32; 100]; // approximate size, caller defines
    }
    extern "C" {
        fn max(a: u32, b: u32) -> u32;
        fn min(a: u32, b: u32) -> u32;
    }
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

extern "C" {
    fn Q_INIT_BITSTREAM();
    fn Q_READ_BITS(val: *mut i32, n: u32);
    fn GET_SYMBOL(model: i32, sym: *mut u8);
    static model7: i32;
    static model00: i32;
    static model40: i32;
    static model80: i32;
    static modelC0: i32;
    static model4: i32;
    static model5: i32;
    static model6len: i32;
    static model6pos: i32;
    }


unsafe extern "C" fn QTMfdi_decomp(mut inlen: i32, mut outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut inpos = (*decomp_state).inbuf.as_mut_ptr();
    let mut window = (*decomp_state).methods.qtm.window;
    let mut window_posn = (*decomp_state).methods.qtm.window_posn;
    let window_size = (*decomp_state).methods.qtm.window_size;
    let mut bitsleft: i32 = 0;
    let mut bitrun: i32 = 0;
    let mut bitsneed: i32 = 0;
    let mut bitbuf: u32 = 0;
    let mut range: u32;
    let mut symf: u16;
    let mut i: i32;
    let mut extra: i32 = 0;
    let mut togo = outlen;
    let mut match_length: i32 = 0;
    let mut copy_length: i32 = 0;
    let mut selector: u8 = 0;
    let mut sym: u8 = 0;
    let mut match_offset: u32 = 0;
    let mut H: u16 = 0xFFFF;
    let mut L: u16 = 0;
    let mut C: i32 = 0;
    TRACE(b"(inlen == %d, outlen == %d)\n\0".as_ptr() as *const i8, inlen, outlen);
    Q_INIT_BITSTREAM();
    Q_READ_BITS(&raw mut C, 16);
    window_posn &= window_size - 1;
    if (window_posn + togo as u32) > window_size {
        TRACE(b"straddled run\n\0".as_ptr() as *const i8);
        return 1;
    }
    while togo > 0 {
        GET_SYMBOL(model7, &raw mut selector);
        match selector {
            0 => {
                GET_SYMBOL(model00, &raw mut sym);
                *window.offset(window_posn as isize) = sym;
                window_posn = window_posn.wrapping_add(1);
                togo -= 1;
            }
            1 => {
                GET_SYMBOL(model40, &raw mut sym);
                *window.offset(window_posn as isize) = sym;
                window_posn = window_posn.wrapping_add(1);
                togo -= 1;
            }
            2 => {
                GET_SYMBOL(model80, &raw mut sym);
                *window.offset(window_posn as isize) = sym;
                window_posn = window_posn.wrapping_add(1);
                togo -= 1;
            }
            3 => {
                GET_SYMBOL(modelC0, &raw mut sym);
                *window.offset(window_posn as isize) = sym;
                window_posn = window_posn.wrapping_add(1);
                togo -= 1;
            }
            4 => {
                GET_SYMBOL(model4, &raw mut sym);
                Q_READ_BITS(&raw mut extra, (*decomp_state).q_extra_bits[sym as usize] as u32);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
                match_length = 3;
            }
            5 => {
                GET_SYMBOL(model5, &raw mut sym);
                Q_READ_BITS(&raw mut extra, (*decomp_state).q_extra_bits[sym as usize] as u32);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
                match_length = 4;
            }
            6 => {
                GET_SYMBOL(model6len, &raw mut sym);
                Q_READ_BITS(&raw mut extra, (*decomp_state).q_length_extra[sym as usize] as u32);
                match_length = (*decomp_state).q_length_base[sym as usize] as i32 + extra + 5;
                GET_SYMBOL(model6pos, &raw mut sym);
                Q_READ_BITS(&raw mut extra, (*decomp_state).q_extra_bits[sym as usize] as u32);
                match_offset = (*decomp_state).q_position_base[sym as usize] + extra as u32 + 1;
            }
            _ => {
                TRACE(b"Selector is bogus\n\0".as_ptr() as *const i8);
                return 2;
            }
        }
        if selector >= 4 {
            let mut rundest = window.offset(window_posn as isize);
            togo -= match_length;
            let mut runsrc;
            if window_posn >= match_offset {
                runsrc = rundest.offset(-(match_offset as isize));
            } else {
                runsrc = rundest.offset((window_size - match_offset) as isize);
                copy_length = (match_offset - window_posn) as i32;
                if copy_length < match_length {
                    match_length -= copy_length;
                    window_posn += copy_length as u32;
                    while copy_length > 0 {
                        *rundest = *runsrc;
                        rundest = rundest.offset(1);
                        runsrc = runsrc.offset(1);
                        copy_length -= 1;
                    }
                    runsrc = window;
                }
            }
            window_posn += match_length as u32;
            while match_length > 0 {
                *rundest = *runsrc;
                rundest = rundest.offset(1);
                runsrc = runsrc.offset(1);
                match_length -= 1;
            }
        }
    }
    if togo != 0 {
        TRACE(b"Frame overflow, this_run = %d\n\0".as_ptr() as *const i8, togo);
        return 2;
    }
    let src_offset = (if window_posn == 0 { window_size } else { window_posn }) - outlen as u32;
    core::ptr::copy_nonoverlapping(
        window.offset(src_offset as isize),
        (*decomp_state).outbuf.as_mut_ptr(),
        outlen as usize,
    );
    (*decomp_state).methods.qtm.window_posn = window_posn;
    return 0;
}

unsafe fn fdi_lzx_read_lens(lens: *mut u8, first: u32, last: u32, lb: *mut lzx_bits, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut bitbuf = (*lb).bb;
    let mut bitsleft = (*lb).bl;
    let mut inpos = (*lb).ip;
    let mut x;
    let mut y;
    let mut z;

    for x in 0..20 {
        y = READ_BITS(&mut bitbuf, &mut bitsleft, &mut inpos, 4);
        let table_ptr = LENTABLE(decomp_state, PRETREE);
        *((table_ptr as *mut u8).add(x as usize)) = y as u8;
    }
    BUILD_TABLE(decomp_state, PRETREE);

    x = first;
    while x < last {
        z = READ_HUFFSYM(decomp_state, PRETREE, &mut bitbuf, &mut bitsleft, &mut inpos);
        if z == 17 {
            y = READ_BITS(&mut bitbuf, &mut bitsleft, &mut inpos, 4);
            y += 4;
            while y > 0 {
                *lens.add(x as usize) = 0;
                x += 1;
                y -= 1;
            }
        } else if z == 18 {
            y = READ_BITS(&mut bitbuf, &mut bitsleft, &mut inpos, 5);
            y += 20;
            while y > 0 {
                *lens.add(x as usize) = 0;
                x += 1;
                y -= 1;
            }
        } else if z == 19 {
            y = READ_BITS(&mut bitbuf, &mut bitsleft, &mut inpos, 1);
            y += 4;
            z = READ_HUFFSYM(decomp_state, PRETREE, &mut bitbuf, &mut bitsleft, &mut inpos);
            {
                let cur = *lens.add(x as usize);
                z = cur as i32 - z;
                if z < 0 {
                    z += 17;
                }
            }
            while y > 0 {
                *lens.add(x as usize) = z as u8;
                x += 1;
                y -= 1;
            }
        } else {
            {
                let cur = *lens.add(x as usize);
                z = cur as i32 - z;
                if z < 0 {
                    z += 17;
                }
            }
            *lens.add(x as usize) = z as u8;
            x += 1;
        }
    }

    (*lb).bb = bitbuf;
    (*lb).bl = bitsleft;
    (*lb).ip = inpos;
    0
}

unsafe extern "C" fn free_decompression_temps(fdi: *mut FDI_Int, fol: *const fdi_folder, decomp_state: *mut fdi_decomp_state) {
    match (*fol).comp_type & 0x000F {
        // cffoldCOMPTYPE_LZX
        0x0003 => {
            if !(*decomp_state).methods.lzx.window.is_null() {
                ((*fdi).free)((*decomp_state).methods.lzx.window as *mut core::ffi::c_void);
                (*decomp_state).methods.lzx.window = core::ptr::null_mut();
            }
        }
        // cffoldCOMPTYPE_QUANTUM
        0x0004 => {
            if !(*decomp_state).methods.qtm.window.is_null() {
                ((*fdi).free)((*decomp_state).methods.qtm.window as *mut core::ffi::c_void);
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
        ((*fdi).close)((*decomp_state).cabhf as isize);
        if !(*decomp_state).mii.nextname.is_null() {
            ((*fdi).free)((*decomp_state).mii.nextname as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.nextinfo.is_null() {
            ((*fdi).free)((*decomp_state).mii.nextinfo as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.prevname.is_null() {
            ((*fdi).free)((*decomp_state).mii.prevname as *mut core::ffi::c_void);
        }
        if !(*decomp_state).mii.previnfo.is_null() {
            ((*fdi).free)((*decomp_state).mii.previnfo as *mut core::ffi::c_void);
        }
        while !(*decomp_state).firstfol.is_null() {
            fol = (*decomp_state).firstfol;
            (*decomp_state).firstfol = (*fol).next;
            ((*fdi).free)(fol as *mut core::ffi::c_void);
        }
        while !(*decomp_state).firstfile.is_null() {
            let file = (*decomp_state).firstfile;
            if !(*file).filename.is_null() {
                ((*fdi).free)((*file).filename as *mut core::ffi::c_void);
            }
            (*decomp_state).firstfile = (*file).next;
            ((*fdi).free)(file as *mut core::ffi::c_void);
        }
        prev_fds = decomp_state;
        decomp_state = (*decomp_state).next;
        ((*fdi).free)(prev_fds as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FDIDestroy(hfdi: isize) -> i32 {
    let fdi: *mut FDI_Int = get_fdi_ptr(hfdi as HFDI);
    let format = b"(hfdi == ^%p)\n\0".as_ptr() as *const i8;
    TRACE(format, hfdi);
    if fdi.is_null() {
        return 0;
    }
    (*fdi).magic = 0;
    ((*fdi).free)(fdi as *mut core::ffi::c_void);
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn FDITruncateCabinet(
    hfdi: HFDI,
    pszCabinetName: *mut i8,
    iFolderToDelete: u16,
) -> i32 {
    let fdi = get_fdi_ptr(hfdi as *mut core::ffi::c_void);
    FIXME(
        b"(hfdi == ^%p, pszCabinetName == %s, iFolderToDelete == %hu): stub\n\0".as_ptr() as *const i8,
        hfdi as *const core::ffi::c_void,
        debugstr_a(pszCabinetName),
        iFolderToDelete as u32,
    );
    if fdi.is_null() {
        return 0;
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

unsafe fn FDI_read_string(fdi: *mut FDI_Int, hf: i32, cabsize: i32) -> *mut i8 {
    let mut len: usize = 256;
    let base: i32 = FDI_getoffset(fdi, hf as INT_PTR);
    let maxlen: i32 = cabsize - base;
    let mut ok: i32 = 0;
    let mut buf: *mut u8 = core::ptr::null_mut();

    TRACE(
        b"(fdi == %p, hf == %ld, cabsize == %ld)\n\0".as_ptr() as *const i8,
        fdi as *const core::ffi::c_void,
        hf,
        cabsize,
    );

    loop {
        if len > (maxlen as usize) {
            len = maxlen as usize;
        }
        buf = ((*fdi).alloc)(len as u32) as *mut u8;
        if buf.is_null() {
            break;
        }
        if ((*fdi).read)(hf as isize, buf as *mut core::ffi::c_void, len as u32) == 0 {
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
                ERR(b"cabinet is truncated\n\0".as_ptr() as *const i8);
                break;
            }
            ((*fdi).seek)(hf as isize, base, SEEK_SET);
            ((*fdi).free)(buf as *mut core::ffi::c_void);
            buf = core::ptr::null_mut();
            len *= 2;
        }

        if ok != 0 {
            break;
        }
    }

    if ok == 0 {
        if !buf.is_null() {
            ((*fdi).free)(buf as *mut core::ffi::c_void);
        } else {
            ERR(b"out of memory!\n\0".as_ptr() as *const i8);
        }
        return core::ptr::null_mut();
    }

    let seek_pos = (base as i32).wrapping_add((strlen(buf as *const i8) + 1) as i32);
    ((*fdi).seek)(hf as isize, seek_pos, SEEK_SET);

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
            ((*(*decomp_state).fdi).free)((*decomp_state).methods.qtm.window as *mut core::ffi::c_void);
        }
        (*decomp_state).methods.qtm.window = core::ptr::null_mut();
    }
    if (*decomp_state).methods.qtm.window.is_null() {
        let ptr = ((*(*decomp_state).fdi).alloc)(wndsize) as *mut u8;
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

    TRACE(
        b"(fdi == ^%p, hf == %ld, pfdici == ^%p)\n\0".as_ptr() as *const i8,
        fdi as *const core::ffi::c_void,
        hf,
        pfdici as *const core::ffi::c_void,
    );

    if ((*fdi).read)(hf as isize, buf.as_mut_ptr() as *mut core::ffi::c_void, cfhead_SIZEOF as _)
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
        WARN(b"cabinet format version > 1.3\n\0".as_ptr() as *const i8);
        if !pmii.is_null() {
            set_error(fdi, FDIERROR_UNKNOWN_CABINET_VERSION, 0);
        }
        return 0;
    }

    flags = EndGetI16(buf.as_ptr().add(cfhead_Flags)) as u16;

    if (flags & cfheadRESERVE_PRESENT as u16) != 0 {
        if ((*fdi).read)(
            hf as isize,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            cfheadext_SIZEOF as _,
        ) != cfheadext_SIZEOF as _
        {
            ERR(b"bunk reserve-sizes?\n\0".as_ptr() as *const i8);
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
            WARN(b"WARNING; header reserved space > 60000\n\0".as_ptr() as *const i8);
        }

        if (header_resv != 0) && (((*fdi).seek)(hf as isize, header_resv, SEEK_CUR) == -1) {
            ERR(b"seek failure: header_resv\n\0".as_ptr() as *const i8);
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
                ((*fdi).free)(prevname as *mut core::ffi::c_void);
            }
        }
        previnfo = FDI_read_string(fdi, hf, cabsize);
        if !previnfo.is_null() {
            if !pmii.is_null() {
                (*pmii).previnfo = previnfo;
            } else {
                ((*fdi).free)(previnfo as *mut core::ffi::c_void);
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
                    ((*fdi).free)((*pmii).prevname as *mut core::ffi::c_void);
                }
                if !(*pmii).previnfo.is_null() {
                    ((*fdi).free)((*pmii).previnfo as *mut core::ffi::c_void);
                }
            }
            set_error(fdi, FDIERROR_CORRUPT_CABINET, 0);
            return 0;
        } else {
            if !pmii.is_null() {
                (*pmii).nextname = nextname;
            } else {
                ((*fdi).free)(nextname as *mut core::ffi::c_void);
            }
        }
        nextinfo = FDI_read_string(fdi, hf, cabsize);
        if !nextinfo.is_null() {
            if !pmii.is_null() {
                (*pmii).nextinfo = nextinfo;
            } else {
                ((*fdi).free)(nextinfo as *mut core::ffi::c_void);
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
    TRACE(b"(hfdi == ^%p, hf == ^%ld, pfdici == ^%p)\n\0".as_ptr() as *const i8, hfdi, hf, pfdici);
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

unsafe fn ZIPfdi_decomp(inlen: i32, outlen: i32, decomp_state: *mut fdi_decomp_state) -> i32 {
    let mut e: i32 = 0; /* cab_LONG e */

    TRACE(b"(inlen == %d, outlen == %d)\n\0".as_ptr() as *const i8, inlen as i32, outlen as i32);

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
                let alloc_ptr = ((*(*decomp_state).fdi).alloc)(alloc_size as u32);
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
    const FDIERROR_NOT_A_CABINET: i32 = 5;
    const FDIERROR_WRONG_CABINET: i32 = 7;
    const FDIERROR_CABINET_NOT_FOUND: i32 = 1;
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
            write((*decomp_state).filehf as isize, (*decomp_state).outpos as *const core::ffi::c_void, cando);
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
            if read((*cab).cabhf as isize, buf.as_mut_ptr() as *mut core::ffi::c_void, cfdata_SIZEOF) != cfdata_SIZEOF {
                return DECR_INPUT;
            }
            if seek((*cab).cabhf as isize, (*cab).mii.block_resv as i32, SEEK_CUR) == -1 {
                return DECR_INPUT;
            }
            let data: *mut u8 = (*decomp_state).inbuf.as_mut_ptr().add(inlen as usize);
            let len: u16 = end_get_i16(buf.as_ptr().add(cfdata_CompressedSize)) as u16;
            inlen += len;
            if inlen as u32 > CAB_INPUTMAX {
                return DECR_INPUT;
            }
            if read((*cab).cabhf as isize, data as *mut core::ffi::c_void, len as u32) != len as u32 {
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
                        (*cab).next = alloc(core::mem::size_of::<fdi_decomp_state>() as u32)
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
                                ERR(b"MAX_PATH exceeded.\n\0".as_ptr() as *const i8);
                                return DECR_ILLEGALDATA;
                            }
                            // paste path + filename (ReactOS always uses '\\' separators)
                            fullpath[0] = 0;
                            if pathlen != 0 {
                                strcpy(fullpath.as_mut_ptr(), userpath.as_ptr());
                                if fullpath[pathlen as usize - 1] == b'\\' as i8 {
                                    fullpath[pathlen as usize - 1] = 0;
                                }
                            }
                            if filenamelen != 0 {
                                strcat(fullpath.as_mut_ptr(), b"\\\0".as_ptr() as *const i8);
                                strcat(fullpath.as_mut_ptr(), (*cab).mii.nextname);
                            }

                            TRACE(b"full cab path/file name: %s\n\0".as_ptr() as *const i8, debugstr_a(fullpath.as_ptr()));

                            let h: isize = open(fullpath.as_mut_ptr(), OPEN_FLAGS, OPEN_MODE);
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
                            if FDI_read_entries(fdi, h as i32, &mut fdici, &mut (*(*cab).next).mii) == 0 {
                                WARN(b"FDIIsCabinet failed.\n\0".as_ptr() as *const i8);
                                close(h as isize);
                                fdin.fdie = FDIERROR_NOT_A_CABINET;
                                if (pfnfdin.unwrap())(fdintNEXT_CABINET, &mut fdin) != 0 {
                                    return DECR_USERABORT;
                                }
                                continue;
                            }
                            if fdici.setID != (*cab).setID || fdici.iCabinet != (*cab).iCabinet + 1 {
                                WARN(b"Wrong Cabinet.\n\0".as_ptr() as *const i8);
                                close(h as isize);
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
                        (*(*cab).next).cabhf = cabhf as i32;
                        (*(*cab).next).decompress = (*decomp_state).decompress;

                        cab = (*cab).next;

                        // read folders
                        let mut linkfol: *mut fdi_folder = core::ptr::null_mut();
                        let mut i: u32 = 0;
                        while i < fdici.cFolders as u32 {
                            if read((*cab).cabhf as isize, buf2.as_mut_ptr() as *mut core::ffi::c_void, cffold_SIZEOF) != cffold_SIZEOF {
                                return DECR_INPUT;
                            }
                            if (*cab).mii.folder_resv > 0 {
                                seek((*cab).cabhf as isize, (*cab).mii.folder_resv, SEEK_CUR);
                            }
                            let fol: *mut fdi_folder = alloc(core::mem::size_of::<fdi_folder>() as u32) as *mut fdi_folder;
                            if fol.is_null() {
                                ERR(b"out of memory!\n\0".as_ptr() as *const i8);
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
                            if read((*cab).cabhf as isize, buf2.as_mut_ptr() as *mut core::ffi::c_void, cffile_SIZEOF) != cffile_SIZEOF {
                                return DECR_INPUT;
                            }
                            let file: *mut fdi_file = alloc(core::mem::size_of::<fdi_file>() as u32) as *mut fdi_file;
                            if file.is_null() {
                                ERR(b"out of memory!\n\0".as_ptr() as *const i8);
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
                            if seek((*cab).cabhf as isize, (*(*cab).firstfol).offset as i32, SEEK_SET) == -1 {
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

