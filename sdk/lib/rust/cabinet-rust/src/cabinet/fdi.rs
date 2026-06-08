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
const FDIERROR_NONE: i32 = 0;
const FDIERROR_ALLOC_FAIL: i32 = 2;
const TRUE: i32 = 1;
const CAB_BLOCKMAX: i32 = 32768; // typical value

extern "C" {
    fn READ_BITS(bitbuf: *mut u32, bitsleft: *mut i32, inpos: *mut *mut u8, n: u32) -> u32;
    fn LENTABLE(decomp_state: *mut fdi_decomp_state, table: u32) -> *mut u8;
    fn BUILD_TABLE(decomp_state: *mut fdi_decomp_state, table: u32);
    fn READ_HUFFSYM(decomp_state: *mut fdi_decomp_state, table: u32, bitbuf: *mut u32, bitsleft: *mut i32, inpos: *mut *mut u8) -> i32;
    fn TRACE(format: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    fn strlen(s: *const i8) -> usize;
}

extern "system" {
    fn SetLastError(dwErrCode: u32);
    fn FIXME(format: *const i8, ...);
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
    fn TRACE(format: *const i8, ...);
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
    let fdi = get_fdi_ptr(hfdi as *mut FDI_Int);
    FIXME(
        b"(hfdi == ^%p, pszCabinetName == %s, iFolderToDelete == %hu): stub\n\0".as_ptr() as *const i8,
        hfdi as *const core::ffi::c_void,
        debugstr_a(pszCabinetName),
        iFolderToDelete,
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
    extern "system" {
        fn TRACE(format: *const i8, ...);
        fn ERR(format: *const i8, ...);
        fn WARN(format: *const i8, ...);
        fn set_error(fdi: *mut FDI_Int, error: i32, param: i32);
        fn EndGetI32(ptr: *const u8) -> i32;
        fn EndGetI16(ptr: *const u8) -> i16;
        fn FDI_read_string(fdi: *mut FDI_Int, hf: INT_PTR, cabsize: i32) -> *mut i8;
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
            hf,
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
            (*pmii).folder_resv = folder_resv;
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
    let mut tl: *mut Ziphuft;
    let mut td: *mut Ziphuft;
    let mut bl: i32;
    let mut bd: i32;
    let mut nb: u32;
    let mut nl: u32;
    let mut nd: u32;
    let mut b: u32;
    let mut k: u32;

    b = (*decomp_state).methods.zip.bb;
    k = (*decomp_state).methods.zip.bk;
    ll = (*decomp_state).methods.zip.ll;

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
    i = fdi_Ziphuft_build(ll, 19, 19, core::ptr::null_mut(), core::ptr::null_mut(), &mut tl, &mut bl, decomp_state);
    if i != 0 {
        if i == 1 {
            fdi_Ziphuft_free((*decomp_state).fdi, tl);
        }
        return i;
    }

    n = nl + nd;
    m = Zipmask[bl as usize];
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
        j = (*td).b;
        /* ZIPDUMPBITS(j) */
        b >>= j;
        k -= j;
        j = (*td).v.n;
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
    i = fdi_Ziphuft_build(ll, nl, 257, Zipcplens.as_ptr() as *mut u32, Zipcplext.as_ptr() as *mut u32, &mut tl, &mut bl, decomp_state);
    if i != 0 {
        if i == 1 {
            fdi_Ziphuft_free((*decomp_state).fdi, tl);
        }
        return i;
    }

    bd = ZIPDBITS;
    let _ = fdi_Ziphuft_build(ll.offset(nl as isize), nd, 0, Zipcpdist.as_ptr() as *mut u32, Zipcpdext.as_ptr() as *mut u32, &mut td, &mut bd, decomp_state);

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

    if outlen > ZIPWSIZE {
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

