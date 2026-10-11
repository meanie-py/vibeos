use crate::winegdi::emfdc::CloseEnhMetaFile;
use crate::winegdi::emfdc::CreateEnhMetaFileW;
use crate::winegdi::metafile::DeleteMetaFile;
use crate::winegdi::rosglue::GDI_GetObjPtr;
use crate::winegdi::rosglue::GDI_ReleaseObj;
use crate::winegdi::metafile::PlayMetaFile;
use crate::winegdi::metafile::SetMetaFileBitsEx;
use crate::winegdi::rosglue::alloc_gdi_handle;
use crate::winegdi::rosglue::free_gdi_handle;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetVersion() -> DWORD;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_0,
}

// C's `(DWORD_PTR)(__FILE__ ": enhmetafile_cs")` initialiser: see critsect_debug_init (prelude).
static mut critsect_debug: critsect_debug_init = critsect_debug_init {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut enhmetafile_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: unsafe { &raw mut critsect_debug.ProcessLocksList },
            Blink: unsafe { &raw mut critsect_debug.ProcessLocksList },
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        u: critsect_debug_string {
            WineDebugString: b"win32ss/gdi/gdi32/wine/enhmetafile.c: enhmetafile_cs\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
    };

static mut enhmetafile_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut emr_names: [emr_name; 121] = [
    emr_name {
        type_0: 1 as DWORD,
        name: b"EMR_HEADER\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 2 as DWORD,
        name: b"EMR_POLYBEZIER\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 3 as DWORD,
        name: b"EMR_POLYGON\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 4 as DWORD,
        name: b"EMR_POLYLINE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 5 as DWORD,
        name: b"EMR_POLYBEZIERTO\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 6 as DWORD,
        name: b"EMR_POLYLINETO\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 7 as DWORD,
        name: b"EMR_POLYPOLYLINE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 8 as DWORD,
        name: b"EMR_POLYPOLYGON\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 9 as DWORD,
        name: b"EMR_SETWINDOWEXTEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 10 as DWORD,
        name: b"EMR_SETWINDOWORGEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 11 as DWORD,
        name: b"EMR_SETVIEWPORTEXTEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 12 as DWORD,
        name: b"EMR_SETVIEWPORTORGEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 13 as DWORD,
        name: b"EMR_SETBRUSHORGEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 14 as DWORD,
        name: b"EMR_EOF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 15 as DWORD,
        name: b"EMR_SETPIXELV\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 16 as DWORD,
        name: b"EMR_SETMAPPERFLAGS\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 17 as DWORD,
        name: b"EMR_SETMAPMODE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 18 as DWORD,
        name: b"EMR_SETBKMODE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 19 as DWORD,
        name: b"EMR_SETPOLYFILLMODE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 20 as DWORD,
        name: b"EMR_SETROP2\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 21 as DWORD,
        name: b"EMR_SETSTRETCHBLTMODE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 22 as DWORD,
        name: b"EMR_SETTEXTALIGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 23 as DWORD,
        name: b"EMR_SETCOLORADJUSTMENT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 24 as DWORD,
        name: b"EMR_SETTEXTCOLOR\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 25 as DWORD,
        name: b"EMR_SETBKCOLOR\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 26 as DWORD,
        name: b"EMR_OFFSETCLIPRGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 27 as DWORD,
        name: b"EMR_MOVETOEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 28 as DWORD,
        name: b"EMR_SETMETARGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 29 as DWORD,
        name: b"EMR_EXCLUDECLIPRECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 30 as DWORD,
        name: b"EMR_INTERSECTCLIPRECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 31 as DWORD,
        name: b"EMR_SCALEVIEWPORTEXTEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 32 as DWORD,
        name: b"EMR_SCALEWINDOWEXTEX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 33 as DWORD,
        name: b"EMR_SAVEDC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 34 as DWORD,
        name: b"EMR_RESTOREDC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 35 as DWORD,
        name: b"EMR_SETWORLDTRANSFORM\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 36 as DWORD,
        name: b"EMR_MODIFYWORLDTRANSFORM\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 37 as DWORD,
        name: b"EMR_SELECTOBJECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 38 as DWORD,
        name: b"EMR_CREATEPEN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 39 as DWORD,
        name: b"EMR_CREATEBRUSHINDIRECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 40 as DWORD,
        name: b"EMR_DELETEOBJECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 41 as DWORD,
        name: b"EMR_ANGLEARC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 42 as DWORD,
        name: b"EMR_ELLIPSE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 43 as DWORD,
        name: b"EMR_RECTANGLE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 44 as DWORD,
        name: b"EMR_ROUNDRECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 45 as DWORD,
        name: b"EMR_ARC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 46 as DWORD,
        name: b"EMR_CHORD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 47 as DWORD,
        name: b"EMR_PIE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 48 as DWORD,
        name: b"EMR_SELECTPALETTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 49 as DWORD,
        name: b"EMR_CREATEPALETTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 50 as DWORD,
        name: b"EMR_SETPALETTEENTRIES\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 51 as DWORD,
        name: b"EMR_RESIZEPALETTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 52 as DWORD,
        name: b"EMR_REALIZEPALETTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 53 as DWORD,
        name: b"EMR_EXTFLOODFILL\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 54 as DWORD,
        name: b"EMR_LINETO\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 55 as DWORD,
        name: b"EMR_ARCTO\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 56 as DWORD,
        name: b"EMR_POLYDRAW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 57 as DWORD,
        name: b"EMR_SETARCDIRECTION\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 58 as DWORD,
        name: b"EMR_SETMITERLIMIT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 59 as DWORD,
        name: b"EMR_BEGINPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 60 as DWORD,
        name: b"EMR_ENDPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 61 as DWORD,
        name: b"EMR_CLOSEFIGURE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 62 as DWORD,
        name: b"EMR_FILLPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 63 as DWORD,
        name: b"EMR_STROKEANDFILLPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 64 as DWORD,
        name: b"EMR_STROKEPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 65 as DWORD,
        name: b"EMR_FLATTENPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 66 as DWORD,
        name: b"EMR_WIDENPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 67 as DWORD,
        name: b"EMR_SELECTCLIPPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 68 as DWORD,
        name: b"EMR_ABORTPATH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 70 as DWORD,
        name: b"EMR_GDICOMMENT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 71 as DWORD,
        name: b"EMR_FILLRGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 72 as DWORD,
        name: b"EMR_FRAMERGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 73 as DWORD,
        name: b"EMR_INVERTRGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 74 as DWORD,
        name: b"EMR_PAINTRGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 75 as DWORD,
        name: b"EMR_EXTSELECTCLIPRGN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 76 as DWORD,
        name: b"EMR_BITBLT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 77 as DWORD,
        name: b"EMR_STRETCHBLT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 78 as DWORD,
        name: b"EMR_MASKBLT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 79 as DWORD,
        name: b"EMR_PLGBLT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 80 as DWORD,
        name: b"EMR_SETDIBITSTODEVICE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 81 as DWORD,
        name: b"EMR_STRETCHDIBITS\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 82 as DWORD,
        name: b"EMR_EXTCREATEFONTINDIRECTW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 83 as DWORD,
        name: b"EMR_EXTTEXTOUTA\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 84 as DWORD,
        name: b"EMR_EXTTEXTOUTW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 85 as DWORD,
        name: b"EMR_POLYBEZIER16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 86 as DWORD,
        name: b"EMR_POLYGON16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 87 as DWORD,
        name: b"EMR_POLYLINE16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 88 as DWORD,
        name: b"EMR_POLYBEZIERTO16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 89 as DWORD,
        name: b"EMR_POLYLINETO16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 90 as DWORD,
        name: b"EMR_POLYPOLYLINE16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 91 as DWORD,
        name: b"EMR_POLYPOLYGON16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 92 as DWORD,
        name: b"EMR_POLYDRAW16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 93 as DWORD,
        name: b"EMR_CREATEMONOBRUSH\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 94 as DWORD,
        name: b"EMR_CREATEDIBPATTERNBRUSHPT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 95 as DWORD,
        name: b"EMR_EXTCREATEPEN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 96 as DWORD,
        name: b"EMR_POLYTEXTOUTA\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 97 as DWORD,
        name: b"EMR_POLYTEXTOUTW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 98 as DWORD,
        name: b"EMR_SETICMMODE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 99 as DWORD,
        name: b"EMR_CREATECOLORSPACE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 100 as DWORD,
        name: b"EMR_SETCOLORSPACE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 101 as DWORD,
        name: b"EMR_DELETECOLORSPACE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 102 as DWORD,
        name: b"EMR_GLSRECORD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 103 as DWORD,
        name: b"EMR_GLSBOUNDEDRECORD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 104 as DWORD,
        name: b"EMR_PIXELFORMAT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 105 as DWORD,
        name: b"EMR_DRAWESCAPE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 106 as DWORD,
        name: b"EMR_EXTESCAPE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 107 as DWORD,
        name: b"EMR_STARTDOC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 108 as DWORD,
        name: b"EMR_SMALLTEXTOUT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 109 as DWORD,
        name: b"EMR_FORCEUFIMAPPING\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 110 as DWORD,
        name: b"EMR_NAMEDESCAPE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 111 as DWORD,
        name: b"EMR_COLORCORRECTPALETTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 112 as DWORD,
        name: b"EMR_SETICMPROFILEA\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 113 as DWORD,
        name: b"EMR_SETICMPROFILEW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 114 as DWORD,
        name: b"EMR_ALPHABLEND\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 115 as DWORD,
        name: b"EMR_SETLAYOUT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 116 as DWORD,
        name: b"EMR_TRANSPARENTBLT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 117 as DWORD,
        name: b"EMR_RESERVED_117\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 118 as DWORD,
        name: b"EMR_GRADIENTFILL\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 119 as DWORD,
        name: b"EMR_SETLINKEDUFI\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 120 as DWORD,
        name: b"EMR_SETTEXTJUSTIFICATION\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 121 as DWORD,
        name: b"EMR_COLORMATCHTOTARGETW\0" as *const u8 as *const ::core::ffi::c_char,
    },
    emr_name {
        type_0: 122 as DWORD,
        name: b"EMR_CREATECOLORSPACEW\0" as *const u8 as *const ::core::ffi::c_char,
    },
];

// WINE_DEFAULT_DEBUG_CHANNEL(enhmetafile) is a function-like macro that declares the debug channel; skipped.

#[repr(C)]
pub struct ENHMETAFILEOBJ {
    pub emh: *mut ENHMETAHEADER,
    pub on_disk: BOOL, // true if metafile is on disk
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct emr_name {
    pub type_0: DWORD, // C: `DWORD type;` (a Rust keyword)
    pub name: *const ::core::ffi::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EMF_dc_state {
    pub mode: INT,
    pub world_transform: XFORM,
    pub wndOrgX: INT,
    pub wndOrgY: INT,
    pub wndExtX: INT,
    pub wndExtY: INT,
    pub vportOrgX: INT,
    pub vportOrgY: INT,
    pub vportExtX: INT,
    pub vportExtY: INT,
    pub next: *mut EMF_dc_state,
}

#[repr(C)]
pub struct enum_emh_data {
    pub init_transform: XFORM,
    pub state: EMF_dc_state,
    pub save_level: INT,
    pub saved_state: *mut EMF_dc_state,
}

// ENUM_GET_PRIVATE_DATA, WIDTH, HEIGHT, IS_WIN9X are function-like macros; skipped.

// C: `typedef struct tagEMF_PaletteCopy {...} EMF_PaletteCopy;` (layout in the prelude).
pub type EMF_PaletteCopy = tagEMF_PaletteCopy;

#[repr(C)]
pub struct wmf_in_emf_comment {
    pub ident: DWORD,
    pub iComment: DWORD,
    pub nVersion: DWORD,
    pub nChecksum: DWORD,
    pub fFlags: DWORD,
    pub cbWinMetaFile: DWORD,
}

unsafe extern "C" fn get_emr_name(mut type_0: DWORD) -> *const ::core::ffi::c_char {
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (core::mem::size_of_val(&emr_names) as usize)
            .wrapping_div(core::mem::size_of_val(&emr_names[0]) as usize)
    {
        if type_0 == emr_names[i as usize].type_0 {
            return emr_names[i as usize].name;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<::core::ffi::c_char>();
}

unsafe extern "C" fn is_dib_monochrome(mut info: *const BITMAPINFO) -> BOOL {
    if (*info).bmiHeader.biBitCount as ::core::ffi::c_int != 1 {
        return (FALSE) as BOOL;
    }
    if (*info).bmiHeader.biSize as usize == ::core::mem::size_of::<BITMAPCOREHEADER>() as usize {
        let mut rgb: *const RGBTRIPLE =
            &raw const (*(info as *const BITMAPCOREINFO)).bmciColors as *const RGBTRIPLE;
        if (*rgb).rgbtRed as ::core::ffi::c_int == 0
            && (*rgb).rgbtGreen as ::core::ffi::c_int == 0
            && (*rgb).rgbtBlue as ::core::ffi::c_int == 0
        {
            rgb = rgb.offset(1);
            return ((*rgb).rgbtRed as ::core::ffi::c_int == 0xff
                && (*rgb).rgbtGreen as ::core::ffi::c_int == 0xff
                && (*rgb).rgbtBlue as ::core::ffi::c_int == 0xff)
                as ::core::ffi::c_int;
        } else {
            return (FALSE) as BOOL;
        }
    } else {
        let mut rgb_0: *const RGBQUAD = &raw const (*info).bmiColors as *const RGBQUAD;
        if (*rgb_0).rgbRed as ::core::ffi::c_int == 0
            && (*rgb_0).rgbGreen as ::core::ffi::c_int == 0
            && (*rgb_0).rgbBlue as ::core::ffi::c_int == 0
            && (*rgb_0).rgbReserved as ::core::ffi::c_int == 0
        {
            rgb_0 = rgb_0.offset(1);
            return ((*rgb_0).rgbRed as ::core::ffi::c_int == 0xff
                && (*rgb_0).rgbGreen as ::core::ffi::c_int == 0xff
                && (*rgb_0).rgbBlue as ::core::ffi::c_int == 0xff
                && (*rgb_0).rgbReserved as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int;
        } else {
            return (FALSE) as BOOL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn EMF_Create_HENHMETAFILE(
    mut emh: *mut ENHMETAHEADER,
    mut filesize: DWORD,
    mut on_disk: BOOL,
) -> HENHMETAFILE {
    let mut hmf: HENHMETAFILE = core::ptr::null_mut();
    let mut metaObj: *mut ENHMETAFILEOBJ = core::ptr::null_mut();

    if (*emh).iType != EMR_HEADER {
        SetLastError(ERROR_INVALID_DATA);
        return core::ptr::null_mut();
    }
    if (*emh).dSignature != ENHMETA_SIGNATURE || ((*emh).nBytes & 3) != 0 {
        return core::ptr::null_mut();
    }
    if filesize < (*emh).nBytes {
        return core::ptr::null_mut();
    }

    metaObj = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ENHMETAFILEOBJ>() as SIZE_T,
    ) as *mut ENHMETAFILEOBJ;
    if metaObj.is_null() {
        return core::ptr::null_mut();
    }

    (*metaObj).emh = emh;
    (*metaObj).on_disk = on_disk;

    hmf = alloc_gdi_handle(
        metaObj as *mut core::ffi::c_void,
        (OBJ_ENHMETAFILE) as WORD,
        core::ptr::null::<gdi_obj_funcs>(),
    ) as HENHMETAFILE;
    if hmf.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            metaObj as PVOID,
        );
    }
    return hmf;
}

unsafe extern "C" fn EMF_Delete_HENHMETAFILE(hmf: HENHMETAFILE) -> BOOL {
    let mut metaObj: *mut ENHMETAFILEOBJ;
    let mut Ret: BOOL = (FALSE) as i32;

    EnterCriticalSection(core::ptr::addr_of_mut!(enhmetafile_cs));
    metaObj = free_gdi_handle(hmf as HGDIOBJ) as *mut ENHMETAFILEOBJ;
    if !metaObj.is_null() {
        if (*metaObj).on_disk != 0 {
            UnmapViewOfFile((*metaObj).emh as LPCVOID);
        } else {
            HeapFree(GetProcessHeap(), 0, (*metaObj).emh as PVOID);
        }
        HeapFree(GetProcessHeap(), 0, metaObj as PVOID);
        Ret = (TRUE) as i32;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(enhmetafile_cs));
    return Ret;
}

unsafe extern "C" fn EMF_GetEnhMetaHeader(mut hmf: HENHMETAFILE) -> *mut ENHMETAHEADER {
    let mut ret: *mut ENHMETAHEADER = core::ptr::null_mut::<ENHMETAHEADER>();
    let mut metaObj: *mut ENHMETAFILEOBJ = core::ptr::null_mut::<ENHMETAFILEOBJ>();
    EnterCriticalSection(&raw mut enhmetafile_cs);
    metaObj = GDI_GetObjPtr(hmf as HGDIOBJ, OBJ_ENHMETAFILE as WORD) as *mut ENHMETAFILEOBJ;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/enhmetafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EMF_GetEnhMetaHeader\0" as *const u8 as *const ::core::ffi::c_char,
            327 as ::core::ffi::c_int,
            b"hmf %p -> enhmetaObj %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            hmf,
            metaObj,
        );
    }
    if !metaObj.is_null() {
        ret = (*metaObj).emh;
        GDI_ReleaseObj(hmf as HGDIOBJ);
    }
    LeaveCriticalSection(&raw mut enhmetafile_cs);
    return ret;
}

unsafe extern "C" fn EMF_GetEnhMetaFile(mut hFile: HANDLE) -> HENHMETAFILE {
    let mut emh: *mut ENHMETAHEADER = core::ptr::null_mut();
    let mut hMapping: HANDLE = core::ptr::null_mut();
    let mut hemf: HENHMETAFILE = core::ptr::null_mut();
    let mut filesize: DWORD = 0;

    filesize = GetFileSize(hFile, core::ptr::null_mut());

    hMapping = CreateFileMappingA(
        hFile,
        core::ptr::null_mut(),
        PAGE_READONLY,
        0,
        0,
        core::ptr::null(),
    );
    emh = MapViewOfFile(
        hMapping,
        FILE_MAP_READ,
        0,
        0,
        0,
    ) as *mut ENHMETAHEADER;
    CloseHandle(hMapping);

    if emh.is_null() {
        return core::ptr::null_mut();
    }

    hemf = EMF_Create_HENHMETAFILE(emh, filesize, (TRUE) as BOOL);
    if hemf.is_null() {
        UnmapViewOfFile(emh as LPCVOID);
    }
    return hemf;
}
#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileA(lpszMetaFile: LPCSTR) -> HENHMETAFILE {
    let hmf: HENHMETAFILE;
    let hFile: HANDLE;

    hFile = CreateFileA(lpszMetaFile, GENERIC_READ, FILE_SHARE_READ, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if hFile == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut();
    }
    hmf = EMF_GetEnhMetaFile(hFile);
    CloseHandle(hFile);
    return hmf;
}

#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileW(lpszMetaFile: LPCWSTR) -> HENHMETAFILE {
    let mut hmf: HENHMETAFILE;
    let mut hFile: HANDLE;

    hFile = CreateFileW(lpszMetaFile, GENERIC_READ, FILE_SHARE_READ, core::ptr::null_mut(),
        OPEN_EXISTING, 0, core::ptr::null_mut());
    if hFile == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut();
    }
    hmf = EMF_GetEnhMetaFile(hFile);
    CloseHandle(hFile);
    return hmf;
}
#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileHeader(
    hmf: HENHMETAFILE,
    bufsize: UINT,
    buf: LPENHMETAHEADER,
) -> UINT {
    let mut emh: LPENHMETAHEADER = core::ptr::null_mut();
    let mut size: UINT = 0;
    emh = EMF_GetEnhMetaHeader(hmf) as LPENHMETAHEADER;
    if emh.is_null() {
        return FALSE;
    }
    size = (*emh).nSize as UINT;
    if buf.is_null() {
        return size;
    }
    size = min(size, bufsize);
    memmove(
        buf as *mut core::ffi::c_void,
        emh as *const core::ffi::c_void,
        size as usize,
    );
    return size;
}

#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileDescriptionA(
    mut hmf: HENHMETAFILE,
    mut size: UINT,
    mut buf: LPSTR,
) -> UINT {
    let mut emh: LPENHMETAHEADER = EMF_GetEnhMetaHeader(hmf) as LPENHMETAHEADER;
    let mut len: DWORD = 0;
    let mut descrW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    if emh.is_null() {
        return FALSE as UINT;
    }
    if (*emh).nDescription == 0 as DWORD || (*emh).offDescription == 0 as DWORD {
        return 0 as UINT;
    }
    descrW = (emh as *mut ::core::ffi::c_char).offset((*emh).offDescription as isize) as *mut WCHAR;
    len = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        descrW as LPCWSTR,
        (*emh).nDescription as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;

    if buf.is_null() || size == 0 {
        return len as UINT;
    }

    len = min(size as DWORD, len);
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        descrW as LPCWSTR,
        (*emh).nDescription as ::core::ffi::c_int,
        buf,
        len as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    return len as UINT;
}
#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileDescriptionW(
    hmf: HENHMETAFILE,
    size: UINT,
    buf: LPWSTR,
) -> UINT {
    let emh: LPENHMETAHEADER = EMF_GetEnhMetaHeader(hmf) as LPENHMETAHEADER;

    if emh.is_null() {
        return FALSE;
    }
    if (*emh).nDescription == 0 || (*emh).offDescription == 0 {
        return 0;
    }
    if buf.is_null() || size == 0 {
        return (*emh).nDescription as UINT;
    }

    memmove(
        buf as *mut core::ffi::c_void,
        (emh as *mut u8).add((*emh).offDescription as usize) as *const core::ffi::c_void,
        (min(size as DWORD, (*emh).nDescription) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    return min(size as DWORD, (*emh).nDescription) as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn SetEnhMetaFileBits(bufsize: UINT, buf: *const BYTE) -> HENHMETAFILE {
    let mut emh: *mut ENHMETAHEADER = HeapAlloc(
        GetProcessHeap(),
        0,
        bufsize as SIZE_T,
    ) as *mut ENHMETAHEADER;
    let mut hmf: HENHMETAFILE = core::ptr::null_mut();
    memmove(
        emh as *mut core::ffi::c_void,
        buf as *const core::ffi::c_void,
        bufsize as usize,
    );
    hmf = EMF_Create_HENHMETAFILE(emh, bufsize as DWORD, (FALSE) as BOOL);
    if hmf.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            emh as PVOID,
        );
    }
    return hmf;
}

#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFileBits(
    hmf: HENHMETAFILE,
    bufsize: UINT,
    buf: LPBYTE,
) -> UINT {
    let emh: LPENHMETAHEADER = EMF_GetEnhMetaHeader(hmf) as LPENHMETAHEADER;
    let mut size: UINT;

    if emh.is_null() {
        return 0;
    }

    size = (*emh).nBytes as UINT;
    if buf.is_null() {
        return size;
    }

    size = min(size, bufsize);
    memmove(buf as *mut core::ffi::c_void, emh as *const core::ffi::c_void, size as usize);
    return size;
}

unsafe extern "C" fn EMF_Update_MF_Xform(mut hdc: HDC, mut info: *const enum_emh_data) {
    let mut mapping_mode_trans: XFORM = XFORM {
        eM11: 0.0,
        eM12: 0.0,
        eM21: 0.0,
        eM22: 0.0,
        eDx: 0.0,
        eDy: 0.0,
    };
    let mut final_trans: XFORM = XFORM {
        eM11: 0.0,
        eM12: 0.0,
        eM21: 0.0,
        eM22: 0.0,
        eDx: 0.0,
        eDy: 0.0,
    };
    let mut scaleX: f64 = 0.0;
    let mut scaleY: f64 = 0.0;

    scaleX = (*info).state.vportExtX as f64 / (*info).state.wndExtX as f64;
    scaleY = (*info).state.vportExtY as f64 / (*info).state.wndExtY as f64;
    mapping_mode_trans.eM11 = scaleX as FLOAT;
    mapping_mode_trans.eM12 = 0.0;
    mapping_mode_trans.eM21 = 0.0;
    mapping_mode_trans.eM22 = scaleY as FLOAT;
    mapping_mode_trans.eDx = ((*info).state.vportOrgX as f64 - scaleX * (*info).state.wndOrgX as f64) as FLOAT;
    mapping_mode_trans.eDy = ((*info).state.vportOrgY as f64 - scaleY * (*info).state.wndOrgY as f64) as FLOAT;

    CombineTransform(
        core::ptr::addr_of_mut!(final_trans),
        core::ptr::addr_of!((*info).state.world_transform),
        core::ptr::addr_of_mut!(mapping_mode_trans),
    );
    CombineTransform(
        core::ptr::addr_of_mut!(final_trans),
        core::ptr::addr_of_mut!(final_trans),
        core::ptr::addr_of!((*info).init_transform),
    );

    if SetWorldTransform(hdc, core::ptr::addr_of_mut!(final_trans)) == 0 {
        if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_ERR as i32 != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/enhmetafile.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"EMF_Update_MF_Xform\0" as *const u8 as *const core::ffi::c_char,
                578 as i32,
                b"World transform failed!\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
    }
}

unsafe extern "C" fn EMF_RestoreDC(info: *mut enum_emh_data, mut level: INT) {
    if abs(level) > (*info).save_level || level == 0 {
        return;
    }

    if level < 0 {
        level = (*info).save_level + level + 1;
    }

    while (*info).save_level >= level {
        let mut state: *mut EMF_dc_state = (*info).saved_state;
        (*info).saved_state = (*state).next;
        (*state).next = core::ptr::null_mut();
        (*info).save_level -= 1;
        if (*info).save_level < level {
            (*info).state = *state;
        }
        HeapFree(GetProcessHeap(), 0 as DWORD, state as PVOID);
    }
}

unsafe extern "C" fn EMF_SaveDC(info: *mut enum_emh_data) {
    let state: *mut EMF_dc_state = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EMF_dc_state>() as SIZE_T,
    ) as *mut EMF_dc_state;
    if !state.is_null() {
        *state = (*info).state;
        (*state).next = (*info).saved_state;
        (*info).saved_state = state;
        (*info).save_level += 1;
    }
}

unsafe extern "C" fn EMF_SetMapMode(hdc: HDC, info: *mut enum_emh_data) {
    let horzSize: INT = GetDeviceCaps(hdc, (HORZSIZE as i32)) as INT;
    let vertSize: INT = GetDeviceCaps(hdc, (VERTSIZE as i32)) as INT;
    let horzRes: INT = GetDeviceCaps(hdc, (HORZRES as i32)) as INT;
    let vertRes: INT = GetDeviceCaps(hdc, (VERTRES as i32)) as INT;

    match ((*info).state.mode) as u32 {
        MM_TEXT => {
            (*info).state.wndExtX = 1;
            (*info).state.wndExtY = 1;
            (*info).state.vportExtX = 1;
            (*info).state.vportExtY = 1;
        }
        MM_LOMETRIC | MM_ISOTROPIC => {
            (*info).state.wndExtX = horzSize * 10;
            (*info).state.wndExtY = vertSize * 10;
            (*info).state.vportExtX = horzRes;
            (*info).state.vportExtY = -vertRes;
        }
        MM_HIMETRIC => {
            (*info).state.wndExtX = horzSize * 100;
            (*info).state.wndExtY = vertSize * 100;
            (*info).state.vportExtX = horzRes;
            (*info).state.vportExtY = -vertRes;
        }
        MM_LOENGLISH => {
            (*info).state.wndExtX = MulDiv(1000, horzSize, 254) as INT;
            (*info).state.wndExtY = MulDiv(1000, vertSize, 254) as INT;
            (*info).state.vportExtX = horzRes;
            (*info).state.vportExtY = -vertRes;
        }
        MM_HIENGLISH => {
            (*info).state.wndExtX = MulDiv(10000, horzSize, 254) as INT;
            (*info).state.wndExtY = MulDiv(10000, vertSize, 254) as INT;
            (*info).state.vportExtX = horzRes;
            (*info).state.vportExtY = -vertRes;
        }
        MM_TWIPS => {
            (*info).state.wndExtX = MulDiv(14400, horzSize, 254) as INT;
            (*info).state.wndExtY = MulDiv(14400, vertSize, 254) as INT;
            (*info).state.vportExtX = horzRes;
            (*info).state.vportExtY = -vertRes;
        }
        MM_ANISOTROPIC => {}
        _ => return,
    }
}

unsafe extern "C" fn EMF_FixIsotropic(hdc: HDC, info: *mut enum_emh_data) {
    let xdim: f64 = fabs((*info).state.vportExtX as f64 * GetDeviceCaps(hdc, (HORZSIZE as i32)) as f64 /
                  (GetDeviceCaps(hdc, (HORZRES as i32)) as f64 * (*info).state.wndExtX as f64));
    let ydim: f64 = fabs((*info).state.vportExtY as f64 * GetDeviceCaps(hdc, (VERTSIZE as i32)) as f64 /
                  (GetDeviceCaps(hdc, (VERTRES as i32)) as f64 * (*info).state.wndExtY as f64));

    if xdim > ydim {
        let mincx: INT = if (*info).state.vportExtX >= 0 { 1 } else { -1 };
        (*info).state.vportExtX = floor((*info).state.vportExtX as f64 * ydim / xdim + 0.5) as INT;
        if (*info).state.vportExtX == 0 {
            (*info).state.vportExtX = mincx;
        }
    } else {
        let mincy: INT = if (*info).state.vportExtY >= 0 { 1 } else { -1 };
        (*info).state.vportExtY = floor((*info).state.vportExtY as f64 * xdim / ydim + 0.5) as INT;
        if (*info).state.vportExtY == 0 {
            (*info).state.vportExtY = mincy;
        }
    }
}

unsafe extern "C" fn emr_produces_output(type_0: ::core::ffi::c_int) -> BOOL {
    match (type_0) as u32 {
        EMR_POLYBEZIER
        | EMR_POLYGON
        | EMR_POLYLINE
        | EMR_POLYBEZIERTO
        | EMR_POLYLINETO
        | EMR_POLYPOLYLINE
        | EMR_POLYPOLYGON
        | EMR_SETPIXELV
        | EMR_MOVETOEX
        | EMR_EXCLUDECLIPRECT
        | EMR_INTERSECTCLIPRECT
        | EMR_SELECTOBJECT
        | EMR_ANGLEARC
        | EMR_ELLIPSE
        | EMR_RECTANGLE
        | EMR_ROUNDRECT
        | EMR_ARC
        | EMR_CHORD
        | EMR_PIE
        | EMR_EXTFLOODFILL
        | EMR_LINETO
        | EMR_ARCTO
        | EMR_POLYDRAW
        | EMR_GDICOMMENT
        | EMR_FILLRGN
        | EMR_FRAMERGN
        | EMR_INVERTRGN
        | EMR_PAINTRGN
        | EMR_BITBLT
        | EMR_STRETCHBLT
        | EMR_MASKBLT
        | EMR_PLGBLT
        | EMR_SETDIBITSTODEVICE
        | EMR_STRETCHDIBITS
        | EMR_EXTTEXTOUTA
        | EMR_EXTTEXTOUTW
        | EMR_POLYBEZIER16
        | EMR_POLYGON16
        | EMR_POLYLINE16
        | EMR_POLYBEZIERTO16
        | EMR_POLYLINETO16
        | EMR_POLYPOLYLINE16
        | EMR_POLYPOLYGON16
        | EMR_POLYDRAW16
        | EMR_POLYTEXTOUTA
        | EMR_POLYTEXTOUTW
        | EMR_SMALLTEXTOUT
        | EMR_ALPHABLEND
        | EMR_TRANSPARENTBLT => return (TRUE) as i32,
        _ => return (FALSE) as i32,
    }
}

#[no_mangle]
pub unsafe extern "system" fn EnumEnhMetaFile(
    hdc: HDC,
    hmf: HENHMETAFILE,
    callback: ENHMFENUMPROC,
    data: LPVOID,
    lpRect: *const RECT,
) -> BOOL {
    let mut ret: BOOL;
    let mut emh: *mut ENHMETAHEADER;
    let mut emr: *mut ENHMETARECORD;
    let mut offset: DWORD;
    let mut i: UINT;
    let mut ht: *mut HANDLETABLE;
    let mut savedMode: INT = 0;
    let mut savedXform: XFORM = core::mem::zeroed();
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hRgn: HRGN = core::ptr::null_mut();
    let mut info: *mut enum_emh_data;
    let mut vp_size: SIZE = core::mem::zeroed();
    let mut win_size: SIZE = core::mem::zeroed();
    let mut vp_org: POINT = core::mem::zeroed();
    let mut win_org: POINT = core::mem::zeroed();
    let mut mapMode: INT = (MM_TEXT as i32);
    let mut old_align: INT = 0;
    let mut old_rop2: INT = 0;
    let mut old_arcdir: INT = 0;
    let mut old_polyfill: INT = 0;
    let mut old_stretchblt: INT = 0;
    let mut old_text_color: COLORREF = 0;
    let mut old_bk_color: COLORREF = 0;

    if lpRect.is_null() && !hdc.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    emh = EMF_GetEnhMetaHeader(hmf);
    if emh.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }

    info = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<enum_emh_data>()
            + core::mem::size_of::<HANDLETABLE>() * ((*emh).nHandles) as usize) as SIZE_T,
    ) as *mut enum_emh_data;
    if info.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }
    (*info).state.mode = (MM_TEXT as i32);
    (*info).state.wndOrgX = 0;
    (*info).state.wndOrgY = 0;
    (*info).state.wndExtX = 1;
    (*info).state.wndExtY = 1;
    (*info).state.vportOrgX = 0;
    (*info).state.vportOrgY = 0;
    (*info).state.vportExtX = 1;
    (*info).state.vportExtY = 1;
    (*info).state.world_transform.eM22 = 1.0;
    (*info).state.world_transform.eM11 = (*info).state.world_transform.eM22;
    (*info).state.world_transform.eM21 = 0.0;
    (*info).state.world_transform.eM12 = (*info).state.world_transform.eM21;
    (*info).state.world_transform.eDy = 0.0;
    (*info).state.world_transform.eDx = (*info).state.world_transform.eDy;

    (*info).state.next = core::ptr::null_mut();
    (*info).save_level = 0;
    (*info).saved_state = core::ptr::null_mut();
    (*info).init_transform = (*info).state.world_transform;

    ht = info.offset(1 as isize) as *mut HANDLETABLE;
    *(core::ptr::addr_of_mut!((*ht).objectHandle) as *mut HGDIOBJ)
        .offset(0 as isize) = hmf as HGDIOBJ;
    i = 1 as UINT;
    while i < (*emh).nHandles as UINT {
        *(core::ptr::addr_of_mut!((*ht).objectHandle) as *mut HGDIOBJ).offset(i as isize) =
            core::ptr::null_mut();
        i = i.wrapping_add(1);
    }

    if !hdc.is_null() && is_meta_dc(hdc) == 0 {
        savedMode = SetGraphicsMode(hdc, (GM_ADVANCED as i32)) as INT;
        GetWorldTransform(hdc, core::ptr::addr_of_mut!(savedXform));
        GetViewportExtEx(hdc, core::ptr::addr_of_mut!(vp_size));
        GetWindowExtEx(hdc, core::ptr::addr_of_mut!(win_size));
        GetViewportOrgEx(hdc, core::ptr::addr_of_mut!(vp_org));
        GetWindowOrgEx(hdc, core::ptr::addr_of_mut!(win_org));
        mapMode = GetMapMode(hdc) as INT;

        /* save DC */
        hPen = GetCurrentObject(hdc, OBJ_PEN) as HPEN;
        hBrush = GetCurrentObject(hdc, OBJ_BRUSH) as HBRUSH;
        hFont = GetCurrentObject(hdc, OBJ_FONT) as HFONT;

        hRgn = CreateRectRgn(
            0,
            0,
            0,
            0,
        );
        if GetClipRgn(hdc, hRgn) == 0 {
            DeleteObject(hRgn as HGDIOBJ);
            hRgn = core::ptr::null_mut();
        }

        old_text_color = SetTextColor(
            hdc,
            ((0 as BYTE as WORD)
                | ((0 as BYTE as WORD) << 8)) as DWORD
                | ((0 as BYTE as DWORD) << 16),
        );
        old_bk_color = SetBkColor(
            hdc,
            ((0xff as BYTE as WORD)
                | ((0xff as BYTE as WORD) << 8)) as DWORD
                | ((0xff as BYTE as DWORD) << 16),
        );
        old_align = SetTextAlign(hdc, 0) as INT;
        old_rop2 = SetROP2(hdc, (R2_COPYPEN as i32)) as INT;
        old_arcdir = SetArcDirection(hdc, (AD_COUNTERCLOCKWISE as i32)) as INT;
        old_polyfill = SetPolyFillMode(hdc, (ALTERNATE as i32)) as INT;
        old_stretchblt = SetStretchBltMode(hdc, (BLACKONWHITE as i32)) as INT;

        if (GetVersion() & 0x80000000) == 0 {
            /* WinNT combines the vp/win ext/org info into a transform */
            let mut xscale: f64;
            let mut yscale: f64;
            xscale = vp_size.cx as f64 / win_size.cx as f64;
            yscale = vp_size.cy as f64 / win_size.cy as f64;
            (*info).init_transform.eM11 = xscale as FLOAT;
            (*info).init_transform.eM12 = 0.0;
            (*info).init_transform.eM21 = 0.0;
            (*info).init_transform.eM22 = yscale as FLOAT;
            (*info).init_transform.eDx =
                (vp_org.x as f64 - xscale * win_org.x as f64) as FLOAT;
            (*info).init_transform.eDy =
                (vp_org.y as f64 - yscale * win_org.y as f64) as FLOAT;

            CombineTransform(
                core::ptr::addr_of_mut!((*info).init_transform),
                core::ptr::addr_of_mut!(savedXform),
                core::ptr::addr_of_mut!((*info).init_transform),
            );
        }

        if !lpRect.is_null()
            && (*emh).rclFrame.right - (*emh).rclFrame.left != 0
            && (*emh).rclFrame.bottom - (*emh).rclFrame.top != 0
        {
            let mut xSrcPixSize: f64;
            let mut ySrcPixSize: f64;
            let mut xscale: f64;
            let mut yscale: f64;
            let mut xform: XFORM = core::mem::zeroed();

            xSrcPixSize = (*emh).szlMillimeters.cx as f64 / (*emh).szlDevice.cx as f64;
            ySrcPixSize = (*emh).szlMillimeters.cy as f64 / (*emh).szlDevice.cy as f64;
            xscale = ((*lpRect).right - (*lpRect).left) as f64
                * 100.0
                / ((*emh).rclFrame.right - (*emh).rclFrame.left) as f64
                * xSrcPixSize;
            yscale = ((*lpRect).bottom - (*lpRect).top) as f64
                * 100.0
                / ((*emh).rclFrame.bottom - (*emh).rclFrame.top) as f64
                * ySrcPixSize;

            xform.eM11 = xscale as FLOAT;
            xform.eM12 = 0.0;
            xform.eM21 = 0.0;
            xform.eM22 = yscale as FLOAT;
            xform.eDx = ((*lpRect).left as f64
                - ((*lpRect).right - (*lpRect).left) as f64
                    / ((*emh).rclFrame.right - (*emh).rclFrame.left) as f64
                    * (*emh).rclFrame.left as f64) as FLOAT;
            xform.eDy = ((*lpRect).top as f64
                - ((*lpRect).bottom - (*lpRect).top) as f64
                    / ((*emh).rclFrame.bottom - (*emh).rclFrame.top) as f64
                    * (*emh).rclFrame.top as f64) as FLOAT;

            CombineTransform(
                core::ptr::addr_of_mut!((*info).init_transform),
                core::ptr::addr_of_mut!(xform),
                core::ptr::addr_of_mut!((*info).init_transform),
            );
        }

        /* WinNT resets the current vp/win org/ext */
        if (GetVersion() & 0x80000000) == 0 {
            SetMapMode(hdc, (MM_TEXT as i32));
            SetWindowOrgEx(
                hdc,
                0,
                0,
                core::ptr::null_mut(),
            );
            SetViewportOrgEx(
                hdc,
                0,
                0,
                core::ptr::null_mut(),
            );
            EMF_Update_MF_Xform(hdc, info);
        }
    }

    ret = (TRUE) as i32;
    offset = 0;
    while ret != 0 && offset < (*emh).nBytes {
        emr = (emh as *mut core::ffi::c_char).offset(offset as isize) as *mut ENHMETARECORD;

        if offset.wrapping_add(8) > (*emh).nBytes
            || offset > offset.wrapping_add((*emr).nSize)
            || offset.wrapping_add((*emr).nSize) > (*emh).nBytes
        {
            /* WINE_WARN: record truncated (debug logging omitted) */
            break;
        } else {
            /* In Win9x mode we update the xform if the record will produce output */
            if !hdc.is_null()
                && (GetVersion() & 0x80000000) != 0
                && emr_produces_output((*emr).iType as INT) != 0
            {
                EMF_Update_MF_Xform(hdc, info);
            }
            /* WINE_TRACE: Calling EnumFunc (debug logging omitted) */
            ret = callback.unwrap_unchecked()(
                hdc,
                ht,
                emr,
                (*emh).nHandles as INT,
                data as LPARAM,
            ) as BOOL;
            offset = offset.wrapping_add((*emr).nSize);
        }
    }

    if !hdc.is_null() && is_meta_dc(hdc) == 0 {
        SetStretchBltMode(hdc, old_stretchblt);
        SetPolyFillMode(hdc, old_polyfill);
        SetArcDirection(hdc, old_arcdir);
        SetROP2(hdc, old_rop2);
        SetTextAlign(hdc, old_align as UINT);
        SetBkColor(hdc, old_bk_color);
        SetTextColor(hdc, old_text_color);

        /* restore DC */
        SelectObject(hdc, hBrush as HGDIOBJ);
        SelectObject(hdc, hPen as HGDIOBJ);
        SelectObject(hdc, hFont as HGDIOBJ);
        ExtSelectClipRgn(hdc, hRgn, (RGN_COPY as i32));
        DeleteObject(hRgn as HGDIOBJ);

        SetWorldTransform(hdc, core::ptr::addr_of_mut!(savedXform));
        if savedMode != 0 {
            SetGraphicsMode(hdc, savedMode);
        }
        SetMapMode(hdc, mapMode);
        SetWindowOrgEx(
            hdc,
            win_org.x,
            win_org.y,
            core::ptr::null_mut(),
        );
        SetWindowExtEx(
            hdc,
            win_size.cx,
            win_size.cy,
            core::ptr::null_mut(),
        );
        SetViewportOrgEx(
            hdc,
            vp_org.x,
            vp_org.y,
            core::ptr::null_mut(),
        );
        SetViewportExtEx(
            hdc,
            vp_size.cx,
            vp_size.cy,
            core::ptr::null_mut(),
        );
    }

    i = 1 as UINT;
    while i < (*emh).nHandles as UINT {
        /* Don't delete element 0 (hmf) */
        if !(*(core::ptr::addr_of_mut!((*ht).objectHandle) as *mut HGDIOBJ).offset(i as isize))
            .is_null()
        {
            DeleteObject(
                *(core::ptr::addr_of_mut!((*ht).objectHandle) as *mut HGDIOBJ).offset(i as isize),
            );
        }
        i = i.wrapping_add(1);
    }

    while !(*info).saved_state.is_null() {
        let mut state: *mut EMF_dc_state = (*info).saved_state;
        (*info).saved_state = (*(*info).saved_state).next;
        HeapFree(
            GetProcessHeap(),
            0,
            state as LPVOID,
        );
    }
    HeapFree(
        GetProcessHeap(),
        0,
        info as LPVOID,
    );
    return ret;
}

unsafe extern "system" fn EMF_PlayEnhMetaFileCallback(hdc: HDC, ht: *mut HANDLETABLE, emr: *const ENHMETARECORD, handles: INT, data: LPARAM) -> INT {
    return PlayEnhMetaFileRecord(hdc, ht as LPHANDLETABLE, emr, handles as UINT) as INT;
}

#[no_mangle]
pub unsafe extern "system" fn PlayEnhMetaFile(
    hdc: HDC,
    hmf: HENHMETAFILE,
    lpRect: *const RECT,
) -> BOOL {
    return EnumEnhMetaFile(
        hdc,
        hmf,
        Some(EMF_PlayEnhMetaFileCallback),
        core::ptr::null_mut(),
        lpRect,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DeleteEnhMetaFile(hmf: HENHMETAFILE) -> BOOL {
    EMF_Delete_HENHMETAFILE(hmf)
}

#[no_mangle]
pub unsafe extern "system" fn CopyEnhMetaFileA(
    hmfSrc: HENHMETAFILE,
    file: LPCSTR,
) -> HENHMETAFILE {
    let mut emrSrc: *mut ENHMETAHEADER = EMF_GetEnhMetaHeader(hmfSrc);
    let mut emrDst: *mut ENHMETAHEADER = core::ptr::null_mut::<ENHMETAHEADER>();
    let mut hmfDst: HENHMETAFILE = core::ptr::null_mut::<HENHMETAFILE__>();
    if emrSrc.is_null() {
        return core::ptr::null_mut();
    }
    if file.is_null() {
        emrDst = HeapAlloc(
            GetProcessHeap(),
            0,
            (*emrSrc).nBytes as SIZE_T,
        ) as *mut ENHMETAHEADER;
        memcpy(
            emrDst as *mut ::core::ffi::c_void,
            emrSrc as *const ::core::ffi::c_void,
            (*emrSrc).nBytes as usize,
        );
        hmfDst = EMF_Create_HENHMETAFILE(emrDst, (*emrSrc).nBytes, (FALSE) as BOOL);
        if hmfDst.is_null() {
            HeapFree(
                GetProcessHeap(),
                0,
                emrDst as PVOID,
            );
        }
    } else {
        let mut hFile: HANDLE = core::ptr::null_mut::<::core::ffi::c_void>();
        let mut w: DWORD = 0;
        hFile = CreateFileA(
            file,
            GENERIC_WRITE | GENERIC_READ,
            0,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
        WriteFile(
            hFile,
            emrSrc as LPCVOID,
            (*emrSrc).nBytes,
            core::ptr::addr_of_mut!(w),
            core::ptr::null_mut::<_OVERLAPPED>(),
        );
        CloseHandle(hFile);
        /* Reopen file for reading only, so that apps can share
           read access to the file while hmf is still valid */
        hFile = CreateFileA(
            file,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/gdi/gdi32/wine/enhmetafile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"CopyEnhMetaFileA\0" as *const u8 as *const ::core::ffi::c_char,
                    2655 as ::core::ffi::c_int,
                    b"Can't reopen emf for reading\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return core::ptr::null_mut();
        }
        hmfDst = EMF_GetEnhMetaFile(hFile);
        CloseHandle(hFile);
    }
    return hmfDst;
}

#[no_mangle]
pub unsafe extern "system" fn CopyEnhMetaFileW(
    hmfSrc: HENHMETAFILE,
    file: LPCWSTR,
) -> HENHMETAFILE {
    let mut emrSrc: *mut ENHMETAHEADER = EMF_GetEnhMetaHeader( hmfSrc );
    let mut emrDst: *mut ENHMETAHEADER = core::ptr::null_mut();
    let mut hmfDst: HENHMETAFILE = core::ptr::null_mut();

    if emrSrc.is_null() { return core::ptr::null_mut(); }
    if file.is_null() {
        emrDst = HeapAlloc( GetProcessHeap(), 0, ((*emrSrc).nBytes) as SIZE_T ) as *mut ENHMETAHEADER;
        memcpy( emrDst as *mut core::ffi::c_void, emrSrc as *const core::ffi::c_void, (*emrSrc).nBytes as usize );
        hmfDst = EMF_Create_HENHMETAFILE( emrDst, (*emrSrc).nBytes, (FALSE) as BOOL );
        if hmfDst.is_null() {
            HeapFree( GetProcessHeap(), 0, emrDst as PVOID );
        }
    } else {
        let mut hFile: HANDLE = core::ptr::null_mut();
        let mut w: DWORD = 0;
        hFile = CreateFileW( file, GENERIC_WRITE | GENERIC_READ, 0,
            core::ptr::null_mut(), CREATE_ALWAYS, 0, core::ptr::null_mut() );
        WriteFile( hFile, emrSrc as LPCVOID, (*emrSrc).nBytes, core::ptr::addr_of_mut!(w), core::ptr::null_mut() );
        CloseHandle( hFile );
        /* Reopen file for reading only, so that apps can share
           read access to the file while hmf is still valid */
        hFile = CreateFileW( file, GENERIC_READ, FILE_SHARE_READ,
            core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut() );
        if hFile == INVALID_HANDLE_VALUE {
            return core::ptr::null_mut();
        }
        hmfDst = EMF_GetEnhMetaFile( hFile );
        CloseHandle( hFile );
    }
    return hmfDst;
}

unsafe extern "system" fn cbEnhPaletteCopy(
    a: HDC,
    b: *mut HANDLETABLE,
    lpEMR: *const ENHMETARECORD,
    c: INT,
    lpData: LPARAM,
) -> INT {
    if (*lpEMR).iType == EMR_EOF {
        let lpEof: *const EMREOF = lpEMR as *const EMREOF;
        let info: *mut EMF_PaletteCopy = lpData as *mut EMF_PaletteCopy;
        let dwNumPalToCopy: DWORD = min((*lpEof).nPalEntries, (*info).cEntries as DWORD);

        memcpy(
            (*info).lpPe as *mut core::ffi::c_void,
            (lpEof as LPCSTR).offset((*lpEof).offPalEntries as isize) as *const core::ffi::c_void,
            core::mem::size_of::<PALETTEENTRY>().wrapping_mul(dwNumPalToCopy as usize),
        );

        /* Update the passed data as a return code */
        (*info).lpPe = core::ptr::null_mut();
        (*info).cEntries = dwNumPalToCopy as UINT;

        return (FALSE) as INT; /* That's all we need */
    }

    return (TRUE) as INT;
}
#[no_mangle]
pub unsafe extern "system" fn GetEnhMetaFilePaletteEntries(
    mut hEmf: HENHMETAFILE,
    mut cEntries: UINT,
    mut lpPe: LPPALETTEENTRY,
) -> UINT {
    let mut enhHeader: *mut ENHMETAHEADER = EMF_GetEnhMetaHeader(hEmf);
    let mut infoForCallBack: EMF_PaletteCopy = tagEMF_PaletteCopy {
        cEntries: 0,
        lpPe: ::core::ptr::null_mut::<tagPALETTEENTRY>(),
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/enhmetafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetEnhMetaFilePaletteEntries\0" as *const u8 as *const ::core::ffi::c_char,
            2765 as ::core::ffi::c_int,
            b"(%p,%d,%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hEmf,
            cEntries,
            lpPe,
        );
    }
    if enhHeader.is_null() {
        return 0;
    }
    if (*enhHeader).nPalEntries == 0 {
        return 0;
    }
    if lpPe.is_null() {
        return (*enhHeader).nPalEntries as UINT;
    }
    infoForCallBack.cEntries = cEntries;
    infoForCallBack.lpPe = lpPe;
    if EnumEnhMetaFile(
        core::ptr::null_mut(),
        hEmf,
        Some(
            cbEnhPaletteCopy,
        ),
        &raw mut infoForCallBack as LPVOID,
        core::ptr::null(),
    ) == 0
    {
        return GDI_ERROR;
    }
    if !infoForCallBack.lpPe.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/gdi/gdi32/wine/enhmetafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"GetEnhMetaFilePaletteEntries\0" as *const u8
                    as *const ::core::ffi::c_char,
                2788 as ::core::ffi::c_int,
                b"cbEnhPaletteCopy didn't execute correctly\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return GDI_ERROR;
    }
    return infoForCallBack.cEntries;
}

unsafe extern "C" fn extract_emf_from_comment(buf: *const BYTE, mf_size: UINT) -> HENHMETAFILE {
    let mut mh: *mut METAHEADER = buf as *mut METAHEADER;
    let mut mr: *mut METARECORD = ::core::ptr::null_mut::<METARECORD>();
    let mut chunk: *mut emf_in_wmf_comment = ::core::ptr::null_mut::<emf_in_wmf_comment>();
    let mut checksum: WORD = 0 as WORD;
    let mut size: DWORD = 0 as DWORD;
    let mut remaining: DWORD = 0;
    let mut chunks: DWORD = 0;
    let mut emf_bits: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut ptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut offset: UINT = 0;
    let mut emf: HENHMETAFILE = ::core::ptr::null_mut::<HENHMETAFILE__>();

    if (mf_size as usize) < ::core::mem::size_of::<METAHEADER>() as usize {
        return ::core::ptr::null_mut::<HENHMETAFILE__>();
    }

    'done: {
        offset = ((*mh).mtHeaderSize as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as UINT;
        loop {
            if !(offset < mf_size) {
                break;
            }
            mr = (mh as *mut ::core::ffi::c_char).offset(offset as isize) as *mut METARECORD;
            chunk = (&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize)
                as *mut emf_in_wmf_comment;

            if (*mr).rdFunction as ::core::ffi::c_int != (META_ESCAPE as i32)
                || *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize)
                    as ::core::ffi::c_int
                    != (MFCOMMENT as i32)
            {
                break 'done;
            }
            if (*chunk).magic != WMFC_MAGIC as DWORD {
                break 'done;
            }

            if emf_bits.is_null() {
                remaining = (*chunk).emf_size;
                size = remaining;
                chunks = (*chunk).num_chunks;
                ptr = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    size as SIZE_T,
                ) as *mut BYTE;
                emf_bits = ptr;
                if emf_bits.is_null() {
                    break 'done;
                }
            }
            if (*chunk).chunk_size > remaining {
                break 'done;
            }
            remaining = remaining.wrapping_sub((*chunk).chunk_size);
            if (*chunk).remaining_size != remaining {
                break 'done;
            }
            memcpy(
                ptr as *mut ::core::ffi::c_void,
                &raw mut (*chunk).emf_data as *mut BYTE as *const ::core::ffi::c_void,
                (*chunk).chunk_size as size_t,
            );
            ptr = ptr.offset((*chunk).chunk_size as isize);
            chunks = chunks.wrapping_sub(1);
            if chunks == 0 as DWORD {
                break;
            }
            offset = (offset as DWORD)
                .wrapping_add((*mr).rdSize.wrapping_mul(2 as DWORD)) as UINT;
        }

        offset = 0 as UINT;
        while offset < mf_size / (2 as UINT) {
            checksum = (checksum as ::core::ffi::c_int
                + *(buf as *mut WORD).offset(offset as isize) as ::core::ffi::c_int)
                as WORD;
            offset = offset.wrapping_add(1);
        }
        if checksum != 0 {
            break 'done;
        }

        emf = SetEnhMetaFileBits(size as UINT, emf_bits);
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        emf_bits as PVOID,
    );
    return emf;
}

#[no_mangle]
pub unsafe extern "system" fn SetWinMetaFileBits(
    cbBuffer: UINT,
    lpbBuffer: *const BYTE,
    mut hdcRef: HDC,
    lpmfp: *const METAFILEPICT,
) -> HENHMETAFILE {
    static mut szDisplayW: [u16; 8] = [0x0044u16, 0x0049, 0x0053, 0x0050, 0x004C, 0x0041, 0x0059, 0x0000];
    let mut hmf: HMETAFILE = core::ptr::null_mut();
    let mut ret: HENHMETAFILE = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hdcdisp: HDC = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    let mut prcFrame: *mut RECT = core::ptr::null_mut();
    let mut mm: LONG = 0;
    let mut xExt: LONG = 0;
    let mut yExt: LONG = 0;
    let mut horzsize: INT = 0;
    let mut vertsize: INT = 0;
    let mut horzres: INT = 0;
    let mut vertres: INT = 0;

    hmf = SetMetaFileBitsEx(cbBuffer, lpbBuffer);
    if hmf.is_null() {
        return core::ptr::null_mut();
    }

    ret = extract_emf_from_comment(lpbBuffer, cbBuffer);
    if !ret.is_null() {
        return ret;
    }

    if hdcRef.is_null() {
        hdcdisp = CreateDCW(
            core::ptr::addr_of!(szDisplayW) as LPCWSTR,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
        );
        hdcRef = hdcdisp;
    }

    if !lpmfp.is_null() {
        mm = (*lpmfp).mm;
        xExt = (*lpmfp).xExt;
        yExt = (*lpmfp).yExt;
    } else {
        /* Use the whole device surface */
        mm = (MM_ANISOTROPIC as i32);
        xExt = 0;
        yExt = 0;
    }

    if mm == (MM_ISOTROPIC as i32) || mm == (MM_ANISOTROPIC as i32) {
        if xExt < 0 || yExt < 0 {
            /* Use the whole device surface */
            xExt = 0;
            yExt = 0;
        }

        /* Use the x and y extents as the frame box */
        if xExt != 0 && yExt != 0 {
            rc.left = 0;
            rc.top = rc.left;
            rc.right = xExt;
            rc.bottom = yExt;
            prcFrame = core::ptr::addr_of_mut!(rc);
        }
    }

    'end: {
        hdc = CreateEnhMetaFileW(hdcRef, core::ptr::null(), prcFrame, core::ptr::null());
        if hdc.is_null() {
            break 'end;
        }

        /*
         * Write the original METAFILE into the enhanced metafile.
         * It is encapsulated in a GDICOMMENT_WINDOWS_METAFILE record.
         */
        if mm != (MM_TEXT as i32) {
            let mut mfcomment: *mut wmf_in_emf_comment = core::ptr::null_mut();
            let mut mfcomment_size: UINT = 0;

            mfcomment_size = (core::mem::size_of::<wmf_in_emf_comment>() as UINT).wrapping_add(cbBuffer);
            mfcomment = HeapAlloc(
                GetProcessHeap(),
                0,
                mfcomment_size as SIZE_T,
            ) as *mut wmf_in_emf_comment;
            if !mfcomment.is_null() {
                (*mfcomment).ident = GDICOMMENT_IDENTIFIER;
                (*mfcomment).iComment = (GDICOMMENT_WINDOWS_METAFILE) as u32;
                (*mfcomment).nVersion = 0x00000300;
                (*mfcomment).nChecksum = 0; /* FIXME */
                (*mfcomment).fFlags = 0;
                (*mfcomment).cbWinMetaFile = cbBuffer;
                memcpy(
                    mfcomment.offset(1 as isize) as *mut core::ffi::c_void,
                    lpbBuffer as *const core::ffi::c_void,
                    cbBuffer as usize,
                );
                GdiComment(hdc, mfcomment_size, mfcomment as *mut BYTE);
                HeapFree(
                    GetProcessHeap(),
                    0,
                    mfcomment as PVOID,
                );
            }
            SetMapMode(hdc, mm as INT);
        }

        horzsize = GetDeviceCaps(hdcRef, (HORZSIZE as i32));
        vertsize = GetDeviceCaps(hdcRef, (VERTSIZE as i32));
        horzres = GetDeviceCaps(hdcRef, (HORZRES as i32));
        vertres = GetDeviceCaps(hdcRef, (VERTRES as i32));

        if xExt == 0 || yExt == 0 {
            /* Use the whole device surface */
            xExt = horzres as LONG;
            yExt = vertres as LONG;
        } else {
            xExt = MulDiv(
                xExt as INT,
                horzres,
                100 * horzsize,
            ) as LONG;
            yExt = MulDiv(
                yExt as INT,
                vertres,
                100 * vertsize,
            ) as LONG;
        }

        /* set the initial viewport:window ratio as 1:1 */
        SetViewportExtEx(hdc, xExt as INT, yExt as INT, core::ptr::null_mut());
        SetWindowExtEx(hdc, xExt as INT, yExt as INT, core::ptr::null_mut());

        PlayMetaFile(hdc, hmf);

        ret = CloseEnhMetaFile(hdc);
    }
    if !hdcdisp.is_null() {
        DeleteDC(hdcdisp);
    }
    DeleteMetaFile(hmf);
    return ret;
}

// Hand port of PlayEnhMetaFileRecord from win32ss/gdi/gdi32/wine/enhmetafile.c (C lines
// 777-2304). The pipeline could not emit it: the Rust exceeds one 64K-token reply.
//
// Integration: paste into sdk/lib/rust/winegdi-rust/src/winegdi/enhmetafile.rs. It relies on
// that module's scope for the file-static helpers (EMF_Update_MF_Xform, EMF_SaveDC,
// EMF_RestoreDC, EMF_SetMapMode, EMF_FixIsotropic, is_dib_monochrome), the private structs
// (enum_emh_data) and the `GetVersion` extern, and on crate::prelude for the island types,
// the EMR_* constants, `memcpy`, and the TRACE!/WARN!/FIXME!/ERR! macros. The island carries
// no gdi32/kernel32 prototypes, so the GDI calls below need the same extern declarations the
// rest of enhmetafile.rs already needs (SetBkColor, SelectObject, CombineTransform, ...).
//
// Every EMR_* constant is a u32 in all three islands while the C's `type` is an `int`, so the
// switch scrutinee is `type_0 as DWORD`; MM_* / GM_* constants are u32 too and are cast to the
// INT the C compares them against.

// C: #define ENUM_GET_PRIVATE_DATA(ht) \
//        ((enum_emh_data*)(((unsigned char*)(ht))-sizeof (enum_emh_data)))
// EnumEnhMetaFile allocates enum_emh_data immediately before the handle table.
// wrapping_sub: the C evaluates this before checking anything, so it must not be UB in Rust.
macro_rules! ENUM_GET_PRIVATE_DATA {
    ($ht:expr) => {
        ($ht as *mut u8).wrapping_sub(core::mem::size_of::<enum_emh_data>()) as *mut enum_emh_data
    };
}

// C: #define IS_WIN9X() (GetVersion()&0x80000000)
macro_rules! IS_WIN9X {
    () => {
        (GetVersion() & 0x80000000) != 0
    };
}

// C: (handletable->objectHandle)[i] -- objectHandle is declared [HGDIOBJ; 1] but is really
// `handles` entries long, so index through a raw pointer. Expands to a place expression, so it
// can be read or assigned.
macro_rules! HT_OBJ {
    ($ht:expr, $i:expr) => {
        *((&raw mut (*$ht).objectHandle) as *mut HGDIOBJ).add(($i) as usize)
    };
}

#[no_mangle]
pub unsafe extern "system" fn PlayEnhMetaFileRecord(
    hdc: HDC,                   /* [in] device context in which to render EMF record */
    handletable: LPHANDLETABLE, /* [in] array of handles to be used in rendering record */
    mr: *const ENHMETARECORD,   /* [in] EMF record to render */
    handles: UINT,              /* [in] size of handle array */
) -> BOOL {
    let type_0: ::core::ffi::c_int;
    let mut tmprc: RECT = core::mem::zeroed();
    let info: *mut enum_emh_data = ENUM_GET_PRIVATE_DATA!(handletable);

    TRACE!(
        "hdc = %p, handletable = %p, record = %p, numHandles = %d\n",
        hdc,
        handletable,
        mr,
        handles
    );
    if mr.is_null() {
        return FALSE as BOOL;
    }

    type_0 = (*mr).iType as ::core::ffi::c_int;

    TRACE!("record %s\n", get_emr_name(type_0 as DWORD));
    match type_0 as DWORD {
        EMR_HEADER => {}
        EMR_EOF => {}
        EMR_GDICOMMENT => {
            let lpGdiComment = mr as *const EMRGDICOMMENT;
            /* In an enhanced metafile, there can be both public and private GDI comments */
            GdiComment(
                hdc,
                (*lpGdiComment).cbData as UINT,
                (&raw const (*lpGdiComment).Data) as *const BYTE,
            );
        }
        EMR_SETMAPMODE => 'case: {
            let pSetMapMode = mr as *const EMRSETMAPMODE;

            if (*info).state.mode as DWORD == (*pSetMapMode).iMode
                && ((*info).state.mode == MM_ISOTROPIC as INT
                    || (*info).state.mode == MM_ANISOTROPIC as INT)
            {
                break 'case;
            }
            (*info).state.mode = (*pSetMapMode).iMode as INT;
            EMF_SetMapMode(hdc, info);

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_SETBKMODE => {
            let pSetBkMode = mr as *const EMRSETBKMODE;
            SetBkMode(hdc, (*pSetBkMode).iMode as INT);
        }
        EMR_SETBKCOLOR => {
            let pSetBkColor = mr as *const EMRSETBKCOLOR;
            SetBkColor(hdc, (*pSetBkColor).crColor);
        }
        EMR_SETPOLYFILLMODE => {
            let pSetPolyFillMode = mr as *const EMRSETPOLYFILLMODE;
            SetPolyFillMode(hdc, (*pSetPolyFillMode).iMode as INT);
        }
        EMR_SETROP2 => {
            let pSetROP2 = mr as *const EMRSETROP2;
            SetROP2(hdc, (*pSetROP2).iMode as INT);
        }
        EMR_SETSTRETCHBLTMODE => {
            let pSetStretchBltMode = mr as *const EMRSETSTRETCHBLTMODE;
            SetStretchBltMode(hdc, (*pSetStretchBltMode).iMode as INT);
        }
        EMR_SETTEXTALIGN => {
            let pSetTextAlign = mr as *const EMRSETTEXTALIGN;
            SetTextAlign(hdc, (*pSetTextAlign).iMode as UINT);
        }
        EMR_SETTEXTCOLOR => {
            let pSetTextColor = mr as *const EMRSETTEXTCOLOR;
            SetTextColor(hdc, (*pSetTextColor).crColor);
        }
        EMR_SAVEDC => {
            if SaveDC(hdc) != 0 {
                EMF_SaveDC(info);
            }
        }
        EMR_RESTOREDC => {
            let pRestoreDC = mr as *const EMRRESTOREDC;
            TRACE!("EMR_RESTORE: %d\n", (*pRestoreDC).iRelative);
            if RestoreDC(hdc, (*pRestoreDC).iRelative as INT) != 0 {
                EMF_RestoreDC(info, (*pRestoreDC).iRelative as INT);
            }
        }
        EMR_INTERSECTCLIPRECT => {
            let pClipRect = mr as *const EMRINTERSECTCLIPRECT;
            TRACE!(
                "EMR_INTERSECTCLIPRECT: rect %d,%d - %d, %d\n",
                (*pClipRect).rclClip.left,
                (*pClipRect).rclClip.top,
                (*pClipRect).rclClip.right,
                (*pClipRect).rclClip.bottom
            );
            IntersectClipRect(
                hdc,
                (*pClipRect).rclClip.left as INT,
                (*pClipRect).rclClip.top as INT,
                (*pClipRect).rclClip.right as INT,
                (*pClipRect).rclClip.bottom as INT,
            );
        }
        EMR_SELECTOBJECT => {
            let pSelectObject = mr as *const EMRSELECTOBJECT;
            if ((*pSelectObject).ihObject & 0x80000000) != 0 {
                /* High order bit is set - it's a stock object
                 * Strip the high bit to get the index.
                 * See MSDN article Q142319
                 */
                SelectObject(
                    hdc,
                    GetStockObject(((*pSelectObject).ihObject & 0x7fffffff) as INT),
                );
            } else {
                /* High order bit wasn't set - not a stock object
                 */
                SelectObject(hdc, HT_OBJ!(handletable, (*pSelectObject).ihObject));
            }
        }
        EMR_DELETEOBJECT => {
            let pDeleteObject = mr as *const EMRDELETEOBJECT;
            DeleteObject(HT_OBJ!(handletable, (*pDeleteObject).ihObject));
            HT_OBJ!(handletable, (*pDeleteObject).ihObject) = core::ptr::null_mut();
        }
        EMR_SETWINDOWORGEX => {
            let pSetWindowOrgEx = mr as *const EMRSETWINDOWORGEX;

            (*info).state.wndOrgX = (*pSetWindowOrgEx).ptlOrigin.x as INT;
            (*info).state.wndOrgY = (*pSetWindowOrgEx).ptlOrigin.y as INT;

            TRACE!(
                "SetWindowOrgEx: %d,%d\n",
                (*info).state.wndOrgX,
                (*info).state.wndOrgY
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_SETWINDOWEXTEX => 'case: {
            let pSetWindowExtEx = mr as *const EMRSETWINDOWEXTEX;

            if (*info).state.mode != MM_ISOTROPIC as INT
                && (*info).state.mode != MM_ANISOTROPIC as INT
            {
                break 'case;
            }
            (*info).state.wndExtX = (*pSetWindowExtEx).szlExtent.cx as INT;
            (*info).state.wndExtY = (*pSetWindowExtEx).szlExtent.cy as INT;
            if (*info).state.mode == MM_ISOTROPIC as INT {
                EMF_FixIsotropic(hdc, info);
            }

            TRACE!(
                "SetWindowExtEx: %d,%d\n",
                (*info).state.wndExtX,
                (*info).state.wndExtY
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_SETVIEWPORTORGEX => {
            let pSetViewportOrgEx = mr as *const EMRSETVIEWPORTORGEX;

            (*info).state.vportOrgX = (*pSetViewportOrgEx).ptlOrigin.x as INT;
            (*info).state.vportOrgY = (*pSetViewportOrgEx).ptlOrigin.y as INT;
            TRACE!(
                "SetViewportOrgEx: %d,%d\n",
                (*info).state.vportOrgX,
                (*info).state.vportOrgY
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_SETVIEWPORTEXTEX => 'case: {
            let pSetViewportExtEx = mr as *const EMRSETVIEWPORTEXTEX;

            if (*info).state.mode != MM_ISOTROPIC as INT
                && (*info).state.mode != MM_ANISOTROPIC as INT
            {
                break 'case;
            }
            (*info).state.vportExtX = (*pSetViewportExtEx).szlExtent.cx as INT;
            (*info).state.vportExtY = (*pSetViewportExtEx).szlExtent.cy as INT;
            if (*info).state.mode == MM_ISOTROPIC as INT {
                EMF_FixIsotropic(hdc, info);
            }
            TRACE!(
                "SetViewportExtEx: %d,%d\n",
                (*info).state.vportExtX,
                (*info).state.vportExtY
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_CREATEPEN => {
            let pCreatePen = mr as *const EMRCREATEPEN;
            HT_OBJ!(handletable, (*pCreatePen).ihPen) =
                CreatePenIndirect(&raw const (*pCreatePen).lopn) as HGDIOBJ;
        }
        EMR_EXTCREATEPEN => {
            let pPen = mr as *const EMREXTCREATEPEN;
            let lb: LOGBRUSH = LOGBRUSH {
                lbStyle: (*pPen).elp.elpBrushStyle,
                lbColor: (*pPen).elp.elpColor,
                lbHatch: (*pPen).elp.elpHatch,
            };

            if (*pPen).offBmi != 0 || (*pPen).offBits != 0 {
                FIXME!("EMR_EXTCREATEPEN: Need to copy brush bitmap\n");
            }

            HT_OBJ!(handletable, (*pPen).ihPen) = ExtCreatePen(
                (*pPen).elp.elpPenStyle,
                (*pPen).elp.elpWidth,
                &lb,
                (*pPen).elp.elpNumEntries,
                if (*pPen).elp.elpNumEntries != 0 {
                    (&raw const (*pPen).elp.elpStyleEntry) as *const DWORD
                } else {
                    core::ptr::null()
                },
            ) as HGDIOBJ;
        }
        EMR_CREATEBRUSHINDIRECT => {
            let pBrush = mr as *const EMRCREATEBRUSHINDIRECT;
            let brush: LOGBRUSH = LOGBRUSH {
                lbStyle: (*pBrush).lb.lbStyle,
                lbColor: (*pBrush).lb.lbColor,
                // LOGBRUSH32.lbHatch is a ULONG; LOGBRUSH.lbHatch is a ULONG_PTR.
                lbHatch: (*pBrush).lb.lbHatch as ULONG_PTR,
            };
            HT_OBJ!(handletable, (*pBrush).ihBrush) = CreateBrushIndirect(&brush) as HGDIOBJ;
        }
        EMR_EXTCREATEFONTINDIRECTW => {
            let pFont = mr as *const EMREXTCREATEFONTINDIRECTW;
            HT_OBJ!(handletable, (*pFont).ihFont) =
                CreateFontIndirectW(&raw const (*pFont).elfw.elfLogFont) as HGDIOBJ;
        }
        EMR_MOVETOEX => {
            let pMoveToEx = mr as *const EMRMOVETOEX;
            MoveToEx(
                hdc,
                (*pMoveToEx).ptl.x as INT,
                (*pMoveToEx).ptl.y as INT,
                core::ptr::null_mut(),
            );
        }
        EMR_LINETO => {
            let pLineTo = mr as *const EMRLINETO;
            LineTo(hdc, (*pLineTo).ptl.x as INT, (*pLineTo).ptl.y as INT);
        }
        EMR_RECTANGLE => {
            let pRect = mr as *const EMRRECTANGLE;
            Rectangle(
                hdc,
                (*pRect).rclBox.left as INT,
                (*pRect).rclBox.top as INT,
                (*pRect).rclBox.right as INT,
                (*pRect).rclBox.bottom as INT,
            );
        }
        EMR_ELLIPSE => {
            let pEllipse = mr as *const EMRELLIPSE;
            Ellipse(
                hdc,
                (*pEllipse).rclBox.left as INT,
                (*pEllipse).rclBox.top as INT,
                (*pEllipse).rclBox.right as INT,
                (*pEllipse).rclBox.bottom as INT,
            );
        }
        EMR_POLYGON16 => {
            let pPoly = mr as *const EMRPOLYGON16;
            /* Shouldn't use Polygon16 since pPoly->cpts is DWORD */
            // The C does not NULL-check this allocation; kept as in the C.
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let apts = (&raw const (*pPoly).apts) as *const POINTS;
            let mut i: DWORD = 0;
            while i < (*pPoly).cpts {
                (*pts.add(i as usize)).x = (*apts.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*apts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            Polygon(hdc, pts, (*pPoly).cpts as INT);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_POLYLINE16 => {
            let pPoly = mr as *const EMRPOLYLINE16;
            /* Shouldn't use Polyline16 since pPoly->cpts is DWORD */
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let apts = (&raw const (*pPoly).apts) as *const POINTS;
            let mut i: DWORD = 0;
            while i < (*pPoly).cpts {
                (*pts.add(i as usize)).x = (*apts.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*apts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            Polyline(hdc, pts, (*pPoly).cpts as INT);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_POLYLINETO16 => {
            let pPoly = mr as *const EMRPOLYLINETO16;
            /* Shouldn't use PolylineTo16 since pPoly->cpts is DWORD */
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let apts = (&raw const (*pPoly).apts) as *const POINTS;
            let mut i: DWORD = 0;
            while i < (*pPoly).cpts {
                (*pts.add(i as usize)).x = (*apts.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*apts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            PolylineTo(hdc, pts, (*pPoly).cpts as DWORD);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_POLYBEZIER16 => {
            let pPoly = mr as *const EMRPOLYBEZIER16;
            /* Shouldn't use PolyBezier16 since pPoly->cpts is DWORD */
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let apts = (&raw const (*pPoly).apts) as *const POINTS;
            let mut i: DWORD = 0;
            while i < (*pPoly).cpts {
                (*pts.add(i as usize)).x = (*apts.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*apts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            PolyBezier(hdc, pts, (*pPoly).cpts as DWORD);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_POLYBEZIERTO16 => {
            let pPoly = mr as *const EMRPOLYBEZIERTO16;
            /* Shouldn't use PolyBezierTo16 since pPoly->cpts is DWORD */
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let apts = (&raw const (*pPoly).apts) as *const POINTS;
            let mut i: DWORD = 0;
            while i < (*pPoly).cpts {
                (*pts.add(i as usize)).x = (*apts.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*apts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            PolyBezierTo(hdc, pts, (*pPoly).cpts as DWORD);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_POLYPOLYGON16 => {
            let pPolyPoly = mr as *const EMRPOLYPOLYGON16;
            /* NB POINTS array doesn't start at pPolyPoly->apts it's actually
            pPolyPoly->aPolyCounts + pPolyPoly->nPolys */
            let aPolyCounts = (&raw const (*pPolyPoly).aPolyCounts) as *const DWORD;
            let pts = aPolyCounts.add((*pPolyPoly).nPolys as usize) as *const POINTS;
            let pt: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPolyPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let mut i: DWORD = 0;
            while i < (*pPolyPoly).cpts {
                (*pt.add(i as usize)).x = (*pts.add(i as usize)).x as LONG;
                (*pt.add(i as usize)).y = (*pts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            PolyPolygon(
                hdc,
                pt,
                aPolyCounts as *const INT,
                (*pPolyPoly).nPolys as INT,
            );
            HeapFree(GetProcessHeap(), 0, pt as LPVOID);
        }
        EMR_POLYPOLYLINE16 => {
            let pPolyPoly = mr as *const EMRPOLYPOLYLINE16;
            /* NB POINTS array doesn't start at pPolyPoly->apts it's actually
            pPolyPoly->aPolyCounts + pPolyPoly->nPolys */
            let aPolyCounts = (&raw const (*pPolyPoly).aPolyCounts) as *const DWORD;
            let pts = aPolyCounts.add((*pPolyPoly).nPolys as usize) as *const POINTS;
            let pt: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPolyPoly).cpts as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;
            let mut i: DWORD = 0;
            while i < (*pPolyPoly).cpts {
                (*pt.add(i as usize)).x = (*pts.add(i as usize)).x as LONG;
                (*pt.add(i as usize)).y = (*pts.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }
            PolyPolyline(hdc, pt, aPolyCounts, (*pPolyPoly).nPolys as DWORD);
            HeapFree(GetProcessHeap(), 0, pt as LPVOID);
        }
        EMR_POLYDRAW16 => 'case: {
            let pPolyDraw16 = mr as *const EMRPOLYDRAW16;
            let ptl = (&raw const (*pPolyDraw16).apts) as *const POINTS;
            let pts: *mut POINT = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPolyDraw16).cpts as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
            ) as *mut POINT;

            /* NB abTypes array doesn't start at pPolyDraw16->abTypes. It's actually
            pPolyDraw16->apts + pPolyDraw16->cpts. */
            let types = ptl.add((*pPolyDraw16).cpts as usize) as *const BYTE;

            if pts.is_null() {
                break 'case;
            }

            let mut i: DWORD = 0;
            while i < (*pPolyDraw16).cpts {
                (*pts.add(i as usize)).x = (*ptl.add(i as usize)).x as LONG;
                (*pts.add(i as usize)).y = (*ptl.add(i as usize)).y as LONG;
                i = i.wrapping_add(1);
            }

            PolyDraw(hdc, pts, types, (*pPolyDraw16).cpts as INT);
            HeapFree(GetProcessHeap(), 0, pts as LPVOID);
        }
        EMR_STRETCHDIBITS => {
            let pStretchDIBits = mr as *const EMRSTRETCHDIBITS;

            StretchDIBits(
                hdc,
                (*pStretchDIBits).xDest as INT,
                (*pStretchDIBits).yDest as INT,
                (*pStretchDIBits).cxDest as INT,
                (*pStretchDIBits).cyDest as INT,
                (*pStretchDIBits).xSrc as INT,
                (*pStretchDIBits).ySrc as INT,
                (*pStretchDIBits).cxSrc as INT,
                (*pStretchDIBits).cySrc as INT,
                (mr as *const BYTE).add((*pStretchDIBits).offBitsSrc as usize) as *const c_void,
                (mr as *const BYTE).add((*pStretchDIBits).offBmiSrc as usize) as *const BITMAPINFO,
                (*pStretchDIBits).iUsageSrc as UINT,
                (*pStretchDIBits).dwRop,
            );
        }
        EMR_EXTTEXTOUTA => {
            let pExtTextOutA = mr as *const EMREXTTEXTOUTA;
            let mut rc: RECT = core::mem::zeroed();
            let mut dx: *const INT = core::ptr::null();
            let old_mode: ::core::ffi::c_int;

            rc.left = (*pExtTextOutA).emrtext.rcl.left;
            rc.top = (*pExtTextOutA).emrtext.rcl.top;
            rc.right = (*pExtTextOutA).emrtext.rcl.right;
            rc.bottom = (*pExtTextOutA).emrtext.rcl.bottom;
            TRACE!(
                "EMR_EXTTEXTOUTA: x,y = %d, %d. rect = %s. flags %08x\n",
                (*pExtTextOutA).emrtext.ptlReference.x,
                (*pExtTextOutA).emrtext.ptlReference.y,
                wine_dbgstr_rect(&rc),
                (*pExtTextOutA).emrtext.fOptions
            );

            old_mode = SetGraphicsMode(hdc, (*pExtTextOutA).iGraphicsMode as INT);
            /* Reselect the font back into the dc so that the transformation
            gets updated. */
            SelectObject(hdc, GetCurrentObject(hdc, OBJ_FONT as UINT));

            /* Linux version of pstoedit produces EMFs with offDx set to 0.
             * These files can be enumerated and played under Win98 just
             * fine, but at least Win2k chokes on them.
             */
            if (*pExtTextOutA).emrtext.offDx != 0 {
                dx = (mr as *const BYTE).add((*pExtTextOutA).emrtext.offDx as usize) as *const INT;
            }

            ExtTextOutA(
                hdc,
                (*pExtTextOutA).emrtext.ptlReference.x as INT,
                (*pExtTextOutA).emrtext.ptlReference.y as INT,
                (*pExtTextOutA).emrtext.fOptions as UINT,
                &rc,
                (mr as *const BYTE).add((*pExtTextOutA).emrtext.offString as usize) as LPCSTR,
                (*pExtTextOutA).emrtext.nChars as UINT,
                dx,
            );

            SetGraphicsMode(hdc, old_mode);
        }
        EMR_EXTTEXTOUTW => {
            let pExtTextOutW = mr as *const EMREXTTEXTOUTW;
            let mut rc: RECT = core::mem::zeroed();
            let mut dx: *const INT = core::ptr::null();
            let old_mode: ::core::ffi::c_int;

            rc.left = (*pExtTextOutW).emrtext.rcl.left;
            rc.top = (*pExtTextOutW).emrtext.rcl.top;
            rc.right = (*pExtTextOutW).emrtext.rcl.right;
            rc.bottom = (*pExtTextOutW).emrtext.rcl.bottom;
            TRACE!(
                "EMR_EXTTEXTOUTW: x,y = %d, %d.  rect = %s. flags %08x\n",
                (*pExtTextOutW).emrtext.ptlReference.x,
                (*pExtTextOutW).emrtext.ptlReference.y,
                wine_dbgstr_rect(&rc),
                (*pExtTextOutW).emrtext.fOptions
            );

            old_mode = SetGraphicsMode(hdc, (*pExtTextOutW).iGraphicsMode as INT);
            /* Reselect the font back into the dc so that the transformation
            gets updated. */
            SelectObject(hdc, GetCurrentObject(hdc, OBJ_FONT as UINT));

            /* Linux version of pstoedit produces EMFs with offDx set to 0.
             * These files can be enumerated and played under Win98 just
             * fine, but at least Win2k chokes on them.
             */
            if (*pExtTextOutW).emrtext.offDx != 0 {
                dx = (mr as *const BYTE).add((*pExtTextOutW).emrtext.offDx as usize) as *const INT;
            }

            ExtTextOutW(
                hdc,
                (*pExtTextOutW).emrtext.ptlReference.x as INT,
                (*pExtTextOutW).emrtext.ptlReference.y as INT,
                (*pExtTextOutW).emrtext.fOptions as UINT,
                &rc,
                (mr as *const BYTE).add((*pExtTextOutW).emrtext.offString as usize) as LPCWSTR,
                (*pExtTextOutW).emrtext.nChars as UINT,
                dx,
            );

            SetGraphicsMode(hdc, old_mode);
        }
        EMR_CREATEPALETTE => {
            let lpCreatePal = mr as *const EMRCREATEPALETTE;

            HT_OBJ!(handletable, (*lpCreatePal).ihPal) =
                CreatePalette(&raw const (*lpCreatePal).lgpl) as HGDIOBJ;
        }
        EMR_SELECTPALETTE => {
            let lpSelectPal = mr as *const EMRSELECTPALETTE;

            if ((*lpSelectPal).ihPal & 0x80000000) != 0 {
                SelectPalette(
                    hdc,
                    GetStockObject(((*lpSelectPal).ihPal & 0x7fffffff) as INT) as HPALETTE,
                    TRUE as BOOL,
                );
            } else {
                SelectPalette(
                    hdc,
                    HT_OBJ!(handletable, (*lpSelectPal).ihPal) as HPALETTE,
                    TRUE as BOOL,
                );
            }
        }
        EMR_REALIZEPALETTE => {
            RealizePalette(hdc);
        }
        EMR_EXTSELECTCLIPRGN => {
            let lpRgn = mr as *const EMREXTSELECTCLIPRGN;
            /* #ifdef __REACTOS__ arm (the only one a ReactOS build compiles). */
            let pRgnData = (&raw const (*lpRgn).RgnData) as *const RGNDATA;
            let mut hRgn: HRGN = core::ptr::null_mut();

            if (*mr).nSize as usize
                >= core::mem::size_of::<EMREXTSELECTCLIPRGN>()
                    + core::mem::size_of::<RGNDATAHEADER>()
            {
                // DIVERGENCE: the C computes `dwSize` from pRgnData->rdh.nCount at the top of
                // the case, BEFORE this size check. A legitimate RGN_COPY-with-NULL-region
                // record is only 16 bytes, so that read lands past the end of the record (and
                // past the mapping when the record is last). dwSize is only ever used inside
                // this branch, so reading it here gives the identical value whenever it is
                // used and drops the out-of-bounds read when it is not.
                let dwSize: DWORD = (core::mem::size_of::<RGNDATAHEADER>() as DWORD).wrapping_add(
                    (*pRgnData)
                        .rdh
                        .nCount
                        .wrapping_mul(core::mem::size_of::<RECT>() as DWORD),
                );
                hRgn = ExtCreateRegion(&raw const (*info).init_transform, dwSize, pRgnData);
            }
            ExtSelectClipRgn(hdc, hRgn, (*lpRgn).iMode as INT);
            /* ExtSelectClipRgn created a copy of the region */
            DeleteObject(hRgn as HGDIOBJ);
        }
        EMR_SETMETARGN => {
            SetMetaRgn(hdc);
        }
        EMR_SETWORLDTRANSFORM => {
            let lpXfrm = mr as *const EMRSETWORLDTRANSFORM;
            (*info).state.world_transform = (*lpXfrm).xform;

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_POLYBEZIER => {
            let lpPolyBez = mr as *const EMRPOLYBEZIER;
            PolyBezier(
                hdc,
                (&raw const (*lpPolyBez).aptl) as *const POINT,
                (*lpPolyBez).cptl as UINT as DWORD,
            );
        }
        EMR_POLYGON => {
            let lpPoly = mr as *const EMRPOLYGON;
            Polygon(
                hdc,
                (&raw const (*lpPoly).aptl) as *const POINT,
                (*lpPoly).cptl as UINT as INT,
            );
        }
        EMR_POLYLINE => {
            let lpPolyLine = mr as *const EMRPOLYLINE;
            Polyline(
                hdc,
                (&raw const (*lpPolyLine).aptl) as *const POINT,
                (*lpPolyLine).cptl as UINT as INT,
            );
        }
        EMR_POLYBEZIERTO => {
            let lpPolyBezierTo = mr as *const EMRPOLYBEZIERTO;
            PolyBezierTo(
                hdc,
                (&raw const (*lpPolyBezierTo).aptl) as *const POINT,
                (*lpPolyBezierTo).cptl as UINT as DWORD,
            );
        }
        EMR_POLYLINETO => {
            let lpPolyLineTo = mr as *const EMRPOLYLINETO;
            PolylineTo(
                hdc,
                (&raw const (*lpPolyLineTo).aptl) as *const POINT,
                (*lpPolyLineTo).cptl as UINT as DWORD,
            );
        }
        EMR_POLYPOLYLINE => {
            let pPolyPolyline = mr as *const EMRPOLYPOLYLINE;
            /* NB Points at pPolyPolyline->aPolyCounts + pPolyPolyline->nPolys */
            let aPolyCounts = (&raw const (*pPolyPolyline).aPolyCounts) as *const DWORD;

            PolyPolyline(
                hdc,
                aPolyCounts.add((*pPolyPolyline).nPolys as usize) as *const POINT,
                aPolyCounts,
                (*pPolyPolyline).nPolys,
            );
        }
        EMR_POLYPOLYGON => {
            let pPolyPolygon = mr as *const EMRPOLYPOLYGON;
            /* NB Points at pPolyPolygon->aPolyCounts + pPolyPolygon->nPolys */
            let aPolyCounts = (&raw const (*pPolyPolygon).aPolyCounts) as *const DWORD;

            PolyPolygon(
                hdc,
                aPolyCounts.add((*pPolyPolygon).nPolys as usize) as *const POINT,
                aPolyCounts as *const INT,
                (*pPolyPolygon).nPolys as INT,
            );
        }
        EMR_SETBRUSHORGEX => {
            let lpSetBrushOrgEx = mr as *const EMRSETBRUSHORGEX;

            SetBrushOrgEx(
                hdc,
                (*lpSetBrushOrgEx).ptlOrigin.x as INT,
                (*lpSetBrushOrgEx).ptlOrigin.y as INT,
                core::ptr::null_mut(),
            );
        }
        EMR_SETPIXELV => {
            let lpSetPixelV = mr as *const EMRSETPIXELV;

            SetPixelV(
                hdc,
                (*lpSetPixelV).ptlPixel.x as INT,
                (*lpSetPixelV).ptlPixel.y as INT,
                (*lpSetPixelV).crColor,
            );
        }
        EMR_SETMAPPERFLAGS => {
            let lpSetMapperFlags = mr as *const EMRSETMAPPERFLAGS;

            SetMapperFlags(hdc, (*lpSetMapperFlags).dwFlags);
        }
        EMR_SETCOLORADJUSTMENT => {
            let lpSetColorAdjust = mr as *const EMRSETCOLORADJUSTMENT;

            SetColorAdjustment(hdc, &raw const (*lpSetColorAdjust).ColorAdjustment);
        }
        EMR_OFFSETCLIPRGN => {
            let lpOffsetClipRgn = mr as *const EMROFFSETCLIPRGN;

            OffsetClipRgn(
                hdc,
                (*lpOffsetClipRgn).ptlOffset.x as INT,
                (*lpOffsetClipRgn).ptlOffset.y as INT,
            );
            FIXME!("OffsetClipRgn\n");
        }
        EMR_EXCLUDECLIPRECT => {
            let lpExcludeClipRect = mr as *const EMREXCLUDECLIPRECT;

            ExcludeClipRect(
                hdc,
                (*lpExcludeClipRect).rclClip.left as INT,
                (*lpExcludeClipRect).rclClip.top as INT,
                (*lpExcludeClipRect).rclClip.right as INT,
                (*lpExcludeClipRect).rclClip.bottom as INT,
            );
            FIXME!("ExcludeClipRect\n");
        }
        EMR_SCALEVIEWPORTEXTEX => 'case: {
            let lpScaleViewportExtEx = mr as *const EMRSCALEVIEWPORTEXTEX;

            if (*info).state.mode != MM_ISOTROPIC as INT
                && (*info).state.mode != MM_ANISOTROPIC as INT
            {
                break 'case;
            }
            if (*lpScaleViewportExtEx).xNum == 0
                || (*lpScaleViewportExtEx).xDenom == 0
                || (*lpScaleViewportExtEx).yNum == 0
                || (*lpScaleViewportExtEx).yDenom == 0
            {
                break 'case;
            }
            (*info).state.vportExtX = MulDiv(
                (*info).state.vportExtX,
                (*lpScaleViewportExtEx).xNum as INT,
                (*lpScaleViewportExtEx).xDenom as INT,
            );
            (*info).state.vportExtY = MulDiv(
                (*info).state.vportExtY,
                (*lpScaleViewportExtEx).yNum as INT,
                (*lpScaleViewportExtEx).yDenom as INT,
            );
            if (*info).state.vportExtX == 0 {
                (*info).state.vportExtX = 1;
            }
            if (*info).state.vportExtY == 0 {
                (*info).state.vportExtY = 1;
            }
            if (*info).state.mode == MM_ISOTROPIC as INT {
                EMF_FixIsotropic(hdc, info);
            }

            TRACE!(
                "EMRSCALEVIEWPORTEXTEX %d/%d %d/%d\n",
                (*lpScaleViewportExtEx).xNum,
                (*lpScaleViewportExtEx).xDenom,
                (*lpScaleViewportExtEx).yNum,
                (*lpScaleViewportExtEx).yDenom
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_SCALEWINDOWEXTEX => 'case: {
            let lpScaleWindowExtEx = mr as *const EMRSCALEWINDOWEXTEX;

            if (*info).state.mode != MM_ISOTROPIC as INT
                && (*info).state.mode != MM_ANISOTROPIC as INT
            {
                break 'case;
            }
            if (*lpScaleWindowExtEx).xNum == 0
                || (*lpScaleWindowExtEx).xDenom == 0
                || (*lpScaleWindowExtEx).yNum == 0
                || (*lpScaleWindowExtEx).yDenom == 0
            {
                break 'case;
            }
            (*info).state.wndExtX = MulDiv(
                (*info).state.wndExtX,
                (*lpScaleWindowExtEx).xNum as INT,
                (*lpScaleWindowExtEx).xDenom as INT,
            );
            (*info).state.wndExtY = MulDiv(
                (*info).state.wndExtY,
                (*lpScaleWindowExtEx).yNum as INT,
                (*lpScaleWindowExtEx).yDenom as INT,
            );
            if (*info).state.wndExtX == 0 {
                (*info).state.wndExtX = 1;
            }
            if (*info).state.wndExtY == 0 {
                (*info).state.wndExtY = 1;
            }
            if (*info).state.mode == MM_ISOTROPIC as INT {
                EMF_FixIsotropic(hdc, info);
            }

            TRACE!(
                "EMRSCALEWINDOWEXTEX %d/%d %d/%d\n",
                (*lpScaleWindowExtEx).xNum,
                (*lpScaleWindowExtEx).xDenom,
                (*lpScaleWindowExtEx).yNum,
                (*lpScaleWindowExtEx).yDenom
            );

            if !IS_WIN9X!() {
                EMF_Update_MF_Xform(hdc, info);
            }
        }
        EMR_MODIFYWORLDTRANSFORM => {
            let lpModifyWorldTrans = mr as *const EMRMODIFYWORLDTRANSFORM;

            match (*lpModifyWorldTrans).iMode {
                MWT_IDENTITY => {
                    (*info).state.world_transform.eM22 = 1.0;
                    (*info).state.world_transform.eM11 = (*info).state.world_transform.eM22;
                    (*info).state.world_transform.eM21 = 0.0;
                    (*info).state.world_transform.eM12 = (*info).state.world_transform.eM21;
                    (*info).state.world_transform.eDy = 0.0;
                    (*info).state.world_transform.eDx = (*info).state.world_transform.eDy;
                    if !IS_WIN9X!() {
                        EMF_Update_MF_Xform(hdc, info);
                    }
                }
                MWT_LEFTMULTIPLY => {
                    CombineTransform(
                        &raw mut (*info).state.world_transform,
                        &raw const (*lpModifyWorldTrans).xform,
                        &raw const (*info).state.world_transform,
                    );
                    if !IS_WIN9X!() {
                        ModifyWorldTransform(
                            hdc,
                            &raw const (*lpModifyWorldTrans).xform,
                            MWT_LEFTMULTIPLY as DWORD,
                        );
                    }
                }
                MWT_RIGHTMULTIPLY => {
                    CombineTransform(
                        &raw mut (*info).state.world_transform,
                        &raw const (*info).state.world_transform,
                        &raw const (*lpModifyWorldTrans).xform,
                    );
                    if !IS_WIN9X!() {
                        EMF_Update_MF_Xform(hdc, info);
                    }
                }
                _ => {
                    FIXME!("Unknown imode %d\n", (*lpModifyWorldTrans).iMode);
                }
            }
        }
        EMR_ANGLEARC => {
            let lpAngleArc = mr as *const EMRANGLEARC;

            AngleArc(
                hdc,
                (*lpAngleArc).ptlCenter.x as INT,
                (*lpAngleArc).ptlCenter.y as INT,
                (*lpAngleArc).nRadius,
                (*lpAngleArc).eStartAngle,
                (*lpAngleArc).eSweepAngle,
            );
        }
        EMR_ROUNDRECT => {
            let lpRoundRect = mr as *const EMRROUNDRECT;

            RoundRect(
                hdc,
                (*lpRoundRect).rclBox.left as INT,
                (*lpRoundRect).rclBox.top as INT,
                (*lpRoundRect).rclBox.right as INT,
                (*lpRoundRect).rclBox.bottom as INT,
                (*lpRoundRect).szlCorner.cx as INT,
                (*lpRoundRect).szlCorner.cy as INT,
            );
        }
        EMR_ARC => {
            let lpArc = mr as *const EMRARC;

            Arc(
                hdc,
                (*lpArc).rclBox.left as INT,
                (*lpArc).rclBox.top as INT,
                (*lpArc).rclBox.right as INT,
                (*lpArc).rclBox.bottom as INT,
                (*lpArc).ptlStart.x as INT,
                (*lpArc).ptlStart.y as INT,
                (*lpArc).ptlEnd.x as INT,
                (*lpArc).ptlEnd.y as INT,
            );
        }
        EMR_CHORD => {
            let lpChord = mr as *const EMRCHORD;

            Chord(
                hdc,
                (*lpChord).rclBox.left as INT,
                (*lpChord).rclBox.top as INT,
                (*lpChord).rclBox.right as INT,
                (*lpChord).rclBox.bottom as INT,
                (*lpChord).ptlStart.x as INT,
                (*lpChord).ptlStart.y as INT,
                (*lpChord).ptlEnd.x as INT,
                (*lpChord).ptlEnd.y as INT,
            );
        }
        EMR_PIE => {
            let lpPie = mr as *const EMRPIE;

            Pie(
                hdc,
                (*lpPie).rclBox.left as INT,
                (*lpPie).rclBox.top as INT,
                (*lpPie).rclBox.right as INT,
                (*lpPie).rclBox.bottom as INT,
                (*lpPie).ptlStart.x as INT,
                (*lpPie).ptlStart.y as INT,
                (*lpPie).ptlEnd.x as INT,
                (*lpPie).ptlEnd.y as INT,
            );
        }
        EMR_ARCTO => {
            let lpArcTo = mr as *const EMRARC;

            ArcTo(
                hdc,
                (*lpArcTo).rclBox.left as INT,
                (*lpArcTo).rclBox.top as INT,
                (*lpArcTo).rclBox.right as INT,
                (*lpArcTo).rclBox.bottom as INT,
                (*lpArcTo).ptlStart.x as INT,
                (*lpArcTo).ptlStart.y as INT,
                (*lpArcTo).ptlEnd.x as INT,
                (*lpArcTo).ptlEnd.y as INT,
            );
        }
        EMR_EXTFLOODFILL => {
            let lpExtFloodFill = mr as *const EMREXTFLOODFILL;

            ExtFloodFill(
                hdc,
                (*lpExtFloodFill).ptlStart.x as INT,
                (*lpExtFloodFill).ptlStart.y as INT,
                (*lpExtFloodFill).crColor,
                (*lpExtFloodFill).iMode as UINT,
            );
        }
        EMR_POLYDRAW => {
            let lpPolyDraw = mr as *const EMRPOLYDRAW;
            let aptl = (&raw const (*lpPolyDraw).aptl) as *const POINTL;
            // DIVERGENCE: the C passes `lpPolyDraw->abTypes`, the declared field, which sits
            // after exactly ONE POINTL. The types really start at aptl + cptl: the writer in
            // this same tree (emfdc.c, EMR_POLYDRAW) memcpy's them to the end of store_points(),
            // and the EMR_POLYDRAW16 case above reads them from apts + cpts for that reason.
            // With the C's field, any record with cptl != 1 plays its point coordinates back
            // as PT_* type bytes. Upstream Wine reads aptl + cptl too.
            let types = aptl.add((*lpPolyDraw).cptl as usize) as *const BYTE;
            PolyDraw(hdc, aptl as *const POINT, types, (*lpPolyDraw).cptl as INT);
        }
        EMR_SETARCDIRECTION => {
            let lpSetArcDirection = mr as *const EMRSETARCDIRECTION;
            SetArcDirection(hdc, (*lpSetArcDirection).iArcDirection as INT);
        }
        EMR_SETMITERLIMIT => {
            let lpSetMiterLimit = mr as *const EMRSETMITERLIMIT;
            SetMiterLimit(hdc, (*lpSetMiterLimit).eMiterLimit, core::ptr::null_mut());
        }
        EMR_BEGINPATH => {
            BeginPath(hdc);
        }
        EMR_ENDPATH => {
            EndPath(hdc);
        }
        EMR_CLOSEFIGURE => {
            CloseFigure(hdc);
        }
        EMR_FILLPATH => {
            /*const EMRFILLPATH lpFillPath = (const EMRFILLPATH *)mr;*/
            FillPath(hdc);
        }
        EMR_STROKEANDFILLPATH => {
            /*const EMRSTROKEANDFILLPATH lpStrokeAndFillPath = (const EMRSTROKEANDFILLPATH *)mr;*/
            StrokeAndFillPath(hdc);
        }
        EMR_STROKEPATH => {
            /*const EMRSTROKEPATH lpStrokePath = (const EMRSTROKEPATH *)mr;*/
            StrokePath(hdc);
        }
        EMR_FLATTENPATH => {
            FlattenPath(hdc);
        }
        EMR_WIDENPATH => {
            WidenPath(hdc);
        }
        EMR_SELECTCLIPPATH => {
            let lpSelectClipPath = mr as *const EMRSELECTCLIPPATH;
            SelectClipPath(hdc, (*lpSelectClipPath).iMode as INT);
        }
        EMR_ABORTPATH => {
            AbortPath(hdc);
        }
        EMR_CREATECOLORSPACE => {
            let lpCreateColorSpace: PEMRCREATECOLORSPACE = mr as PEMRCREATECOLORSPACE;
            HT_OBJ!(handletable, (*lpCreateColorSpace).ihCS) =
                CreateColorSpaceA(&raw mut (*lpCreateColorSpace).lcs) as HGDIOBJ;
        }
        EMR_SETCOLORSPACE => {
            let lpSetColorSpace = mr as *const EMRSETCOLORSPACE;
            SetColorSpace(
                hdc,
                HT_OBJ!(handletable, (*lpSetColorSpace).ihCS) as HCOLORSPACE,
            );
        }
        EMR_DELETECOLORSPACE => {
            let lpDeleteColorSpace = mr as *const EMRDELETECOLORSPACE;
            DeleteColorSpace(HT_OBJ!(handletable, (*lpDeleteColorSpace).ihCS) as HCOLORSPACE);
        }
        EMR_SETICMMODE => {
            let lpSetICMMode = mr as *const EMRSETICMMODE;
            SetICMMode(hdc, (*lpSetICMMode).iMode as INT);
        }
        EMR_PIXELFORMAT => {
            let iPixelFormat: INT;
            let lpPixelFormat = mr as *const EMRPIXELFORMAT;

            iPixelFormat = ChoosePixelFormat(hdc, &raw const (*lpPixelFormat).pfd);
            SetPixelFormat(hdc, iPixelFormat, &raw const (*lpPixelFormat).pfd);
        }
        EMR_SETPALETTEENTRIES => {
            let lpSetPaletteEntries = mr as *const EMRSETPALETTEENTRIES;

            SetPaletteEntries(
                HT_OBJ!(handletable, (*lpSetPaletteEntries).ihPal) as HPALETTE,
                (*lpSetPaletteEntries).iStart as UINT,
                (*lpSetPaletteEntries).cEntries as UINT,
                (&raw const (*lpSetPaletteEntries).aPalEntries) as *const PALETTEENTRY,
            );
        }
        EMR_RESIZEPALETTE => {
            let lpResizePalette = mr as *const EMRRESIZEPALETTE;

            ResizePalette(
                HT_OBJ!(handletable, (*lpResizePalette).ihPal) as HPALETTE,
                (*lpResizePalette).cEntries as UINT,
            );
        }
        EMR_CREATEDIBPATTERNBRUSHPT => 'case: {
            let lpCreate = mr as *const EMRCREATEDIBPATTERNBRUSHPT;
            let lpPackedStruct: LPVOID;

            /* Check that offsets and data are contained within the record
             * (including checking for wrap-arounds).
             */
            // DWORD arithmetic: the wrap-around half of this check depends on the sums
            // wrapping exactly as C unsigned arithmetic does.
            if (*lpCreate).offBmi.wrapping_add((*lpCreate).cbBmi) > (*mr).nSize
                || (*lpCreate).offBits.wrapping_add((*lpCreate).cbBits) > (*mr).nSize
                || (*lpCreate).offBmi.wrapping_add((*lpCreate).cbBmi) < (*lpCreate).offBmi
                || (*lpCreate).offBits.wrapping_add((*lpCreate).cbBits) < (*lpCreate).offBits
            {
                ERR!("Invalid EMR_CREATEDIBPATTERNBRUSHPT record\n");
                break 'case;
            }

            /* This is a BITMAPINFO struct followed directly by bitmap bits */
            lpPackedStruct = HeapAlloc(
                GetProcessHeap(),
                0,
                (*lpCreate).cbBmi.wrapping_add((*lpCreate).cbBits) as SIZE_T,
            ) as LPVOID;
            if lpPackedStruct.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'case;
            }

            /* Now pack this structure */
            memcpy(
                lpPackedStruct as *mut c_void,
                (lpCreate as *const BYTE).add((*lpCreate).offBmi as usize) as *const c_void,
                (*lpCreate).cbBmi as usize,
            );
            memcpy(
                (lpPackedStruct as *mut BYTE).add((*lpCreate).cbBmi as usize) as *mut c_void,
                (lpCreate as *const BYTE).add((*lpCreate).offBits as usize) as *const c_void,
                (*lpCreate).cbBits as usize,
            );

            HT_OBJ!(handletable, (*lpCreate).ihBrush) = CreateDIBPatternBrushPt(
                lpPackedStruct as *const c_void,
                (*lpCreate).iUsage as UINT,
            ) as HGDIOBJ;

            HeapFree(GetProcessHeap(), 0, lpPackedStruct);
        }
        EMR_CREATEMONOBRUSH => {
            let pCreateMonoBrush = mr as *const EMRCREATEMONOBRUSH;
            let pbi =
                (mr as *const BYTE).add((*pCreateMonoBrush).offBmi as usize) as *const BITMAPINFO;
            let hBmp: HBITMAP;

            /* Need to check if the bitmap is monochrome, and if the
            two colors are really black and white */
            if (*pCreateMonoBrush).iUsage == DIB_PAL_MONO as DWORD {
                let mut bm: BITMAP = core::mem::zeroed();

                /* Undocumented iUsage indicates a mono bitmap with no palette table,
                 * aligned to 32 rather than 16 bits.
                 */
                bm.bmType = 0;
                bm.bmWidth = (*pbi).bmiHeader.biWidth;
                bm.bmHeight = abs((*pbi).bmiHeader.biHeight) as LONG;
                bm.bmWidthBytes = 4 * ((*pbi).bmiHeader.biWidth.wrapping_add(31) / 32);
                bm.bmPlanes = (*pbi).bmiHeader.biPlanes;
                bm.bmBitsPixel = (*pbi).bmiHeader.biBitCount;
                bm.bmBits = (mr as *mut BYTE).add((*pCreateMonoBrush).offBits as usize) as LPVOID;
                hBmp = CreateBitmapIndirect(&bm);
            } else if is_dib_monochrome(pbi) != 0 {
                /* Top-down DIBs have a negative height */
                let height: LONG = (*pbi).bmiHeader.biHeight;

                hBmp = CreateBitmap(
                    (*pbi).bmiHeader.biWidth as INT,
                    abs(height) as INT,
                    1,
                    1,
                    core::ptr::null(),
                );
                SetDIBits(
                    hdc,
                    hBmp,
                    0,
                    (*pbi).bmiHeader.biHeight as UINT,
                    (mr as *const BYTE).add((*pCreateMonoBrush).offBits as usize) as *const c_void,
                    pbi,
                    (*pCreateMonoBrush).iUsage as UINT,
                );
            } else {
                hBmp = CreateDIBitmap(
                    hdc,
                    pbi as *const BITMAPINFOHEADER,
                    CBM_INIT as DWORD,
                    (mr as *const BYTE).add((*pCreateMonoBrush).offBits as usize) as *const c_void,
                    pbi,
                    (*pCreateMonoBrush).iUsage as UINT,
                );
            }

            HT_OBJ!(handletable, (*pCreateMonoBrush).ihBrush) = CreatePatternBrush(hBmp) as HGDIOBJ;

            /* CreatePatternBrush created a copy of the bitmap */
            DeleteObject(hBmp as HGDIOBJ);
        }
        EMR_BITBLT => {
            let pBitBlt = mr as *const EMRBITBLT;

            if (*pBitBlt).offBmiSrc == 0 {
                /* Record is a PatBlt */
                PatBlt(
                    hdc,
                    (*pBitBlt).xDest as INT,
                    (*pBitBlt).yDest as INT,
                    (*pBitBlt).cxDest as INT,
                    (*pBitBlt).cyDest as INT,
                    (*pBitBlt).dwRop,
                );
            } else {
                /* BitBlt */
                let hdcSrc: HDC = CreateCompatibleDC(hdc);
                let hBrush: HBRUSH;
                let hBrushOld: HBRUSH;
                let mut hBmp: HBITMAP = core::ptr::null_mut();
                let mut hBmpOld: HBITMAP = core::ptr::null_mut();
                let pbi =
                    (mr as *const BYTE).add((*pBitBlt).offBmiSrc as usize) as *const BITMAPINFO;

                SetGraphicsMode(hdcSrc, GM_ADVANCED as INT);
                SetWorldTransform(hdcSrc, &raw const (*pBitBlt).xformSrc);

                hBrush = CreateSolidBrush((*pBitBlt).crBkColorSrc);
                hBrushOld = SelectObject(hdcSrc, hBrush as HGDIOBJ) as HBRUSH;
                PatBlt(
                    hdcSrc,
                    (*pBitBlt).rclBounds.left as INT,
                    (*pBitBlt).rclBounds.top as INT,
                    (*pBitBlt)
                        .rclBounds
                        .right
                        .wrapping_sub((*pBitBlt).rclBounds.left) as INT,
                    (*pBitBlt)
                        .rclBounds
                        .bottom
                        .wrapping_sub((*pBitBlt).rclBounds.top) as INT,
                    PATCOPY as DWORD,
                );
                SelectObject(hdcSrc, hBrushOld as HGDIOBJ);
                DeleteObject(hBrush as HGDIOBJ);

                hBmp = CreateDIBitmap(
                    hdc,
                    pbi as *const BITMAPINFOHEADER,
                    CBM_INIT as DWORD,
                    (mr as *const BYTE).add((*pBitBlt).offBitsSrc as usize) as *const c_void,
                    pbi,
                    (*pBitBlt).iUsageSrc as UINT,
                );
                hBmpOld = SelectObject(hdcSrc, hBmp as HGDIOBJ) as HBITMAP;

                BitBlt(
                    hdc,
                    (*pBitBlt).xDest as INT,
                    (*pBitBlt).yDest as INT,
                    (*pBitBlt).cxDest as INT,
                    (*pBitBlt).cyDest as INT,
                    hdcSrc,
                    (*pBitBlt).xSrc as INT,
                    (*pBitBlt).ySrc as INT,
                    (*pBitBlt).dwRop,
                );

                SelectObject(hdcSrc, hBmpOld as HGDIOBJ);
                DeleteObject(hBmp as HGDIOBJ);
                DeleteDC(hdcSrc);
            }
        }
        EMR_STRETCHBLT => {
            let pStretchBlt = mr as *const EMRSTRETCHBLT;

            TRACE!(
                "EMR_STRETCHBLT: %d, %d %dx%d -> %d, %d %dx%d. rop %08x offBitsSrc %d\n",
                (*pStretchBlt).xSrc,
                (*pStretchBlt).ySrc,
                (*pStretchBlt).cxSrc,
                (*pStretchBlt).cySrc,
                (*pStretchBlt).xDest,
                (*pStretchBlt).yDest,
                (*pStretchBlt).cxDest,
                (*pStretchBlt).cyDest,
                (*pStretchBlt).dwRop,
                (*pStretchBlt).offBitsSrc
            );

            if (*pStretchBlt).offBmiSrc == 0 {
                /* Record is a PatBlt */
                PatBlt(
                    hdc,
                    (*pStretchBlt).xDest as INT,
                    (*pStretchBlt).yDest as INT,
                    (*pStretchBlt).cxDest as INT,
                    (*pStretchBlt).cyDest as INT,
                    (*pStretchBlt).dwRop,
                );
            } else {
                /* StretchBlt */
                let hdcSrc: HDC = CreateCompatibleDC(hdc);
                let hBrush: HBRUSH;
                let hBrushOld: HBRUSH;
                let mut hBmp: HBITMAP = core::ptr::null_mut();
                let mut hBmpOld: HBITMAP = core::ptr::null_mut();
                let pbi =
                    (mr as *const BYTE).add((*pStretchBlt).offBmiSrc as usize) as *const BITMAPINFO;

                SetGraphicsMode(hdcSrc, GM_ADVANCED as INT);
                SetWorldTransform(hdcSrc, &raw const (*pStretchBlt).xformSrc);

                hBrush = CreateSolidBrush((*pStretchBlt).crBkColorSrc);
                hBrushOld = SelectObject(hdcSrc, hBrush as HGDIOBJ) as HBRUSH;
                PatBlt(
                    hdcSrc,
                    (*pStretchBlt).rclBounds.left as INT,
                    (*pStretchBlt).rclBounds.top as INT,
                    (*pStretchBlt)
                        .rclBounds
                        .right
                        .wrapping_sub((*pStretchBlt).rclBounds.left) as INT,
                    (*pStretchBlt)
                        .rclBounds
                        .bottom
                        .wrapping_sub((*pStretchBlt).rclBounds.top) as INT,
                    PATCOPY as DWORD,
                );
                SelectObject(hdcSrc, hBrushOld as HGDIOBJ);
                DeleteObject(hBrush as HGDIOBJ);

                hBmp = CreateDIBitmap(
                    hdc,
                    pbi as *const BITMAPINFOHEADER,
                    CBM_INIT as DWORD,
                    (mr as *const BYTE).add((*pStretchBlt).offBitsSrc as usize) as *const c_void,
                    pbi,
                    (*pStretchBlt).iUsageSrc as UINT,
                );
                hBmpOld = SelectObject(hdcSrc, hBmp as HGDIOBJ) as HBITMAP;

                StretchBlt(
                    hdc,
                    (*pStretchBlt).xDest as INT,
                    (*pStretchBlt).yDest as INT,
                    (*pStretchBlt).cxDest as INT,
                    (*pStretchBlt).cyDest as INT,
                    hdcSrc,
                    (*pStretchBlt).xSrc as INT,
                    (*pStretchBlt).ySrc as INT,
                    (*pStretchBlt).cxSrc as INT,
                    (*pStretchBlt).cySrc as INT,
                    (*pStretchBlt).dwRop,
                );

                SelectObject(hdcSrc, hBmpOld as HGDIOBJ);
                DeleteObject(hBmp as HGDIOBJ);
                DeleteDC(hdcSrc);
            }
        }
        EMR_ALPHABLEND => {
            let pAlphaBlend = mr as *const EMRALPHABLEND;

            TRACE!(
                "EMR_ALPHABLEND: %d, %d %dx%d -> %d, %d %dx%d. blendfn %08x offBitsSrc %d\n",
                (*pAlphaBlend).xSrc,
                (*pAlphaBlend).ySrc,
                (*pAlphaBlend).cxSrc,
                (*pAlphaBlend).cySrc,
                (*pAlphaBlend).xDest,
                (*pAlphaBlend).yDest,
                (*pAlphaBlend).cxDest,
                (*pAlphaBlend).cyDest,
                (*pAlphaBlend).dwRop,
                (*pAlphaBlend).offBitsSrc
            );

            if (*pAlphaBlend).offBmiSrc == 0 {
                FIXME!("EMR_ALPHABLEND: offBmiSrc == 0\n");
            } else {
                let hdcSrc: HDC = CreateCompatibleDC(hdc);
                let mut hBmp: HBITMAP = core::ptr::null_mut();
                let mut hBmpOld: HBITMAP = core::ptr::null_mut();
                let pbi =
                    (mr as *const BYTE).add((*pAlphaBlend).offBmiSrc as usize) as *const BITMAPINFO;
                let mut bits: *mut c_void = core::ptr::null_mut();

                SetGraphicsMode(hdcSrc, GM_ADVANCED as INT);
                SetWorldTransform(hdcSrc, &raw const (*pAlphaBlend).xformSrc);

                hBmp = CreateDIBSection(
                    hdc,
                    pbi,
                    (*pAlphaBlend).iUsageSrc as UINT,
                    &mut bits,
                    core::ptr::null_mut(),
                    0,
                );
                // The C does not check `bits` (CreateDIBSection failure); kept as in the C.
                memcpy(
                    bits,
                    (mr as *const BYTE).add((*pAlphaBlend).offBitsSrc as usize) as *const c_void,
                    (*pAlphaBlend).cbBitsSrc as usize,
                );
                hBmpOld = SelectObject(hdcSrc, hBmp as HGDIOBJ) as HBITMAP;

                GdiAlphaBlend(
                    hdc,
                    (*pAlphaBlend).xDest as INT,
                    (*pAlphaBlend).yDest as INT,
                    (*pAlphaBlend).cxDest as INT,
                    (*pAlphaBlend).cyDest as INT,
                    hdcSrc,
                    (*pAlphaBlend).xSrc as INT,
                    (*pAlphaBlend).ySrc as INT,
                    (*pAlphaBlend).cxSrc as INT,
                    (*pAlphaBlend).cySrc as INT,
                    // C: *(BLENDFUNCTION *)&pAlphaBlend->dwRop
                    core::ptr::read((&raw const (*pAlphaBlend).dwRop) as *const BLENDFUNCTION),
                );

                SelectObject(hdcSrc, hBmpOld as HGDIOBJ);
                DeleteObject(hBmp as HGDIOBJ);
                DeleteDC(hdcSrc);
            }
        }
        EMR_MASKBLT => {
            let pMaskBlt = mr as *const EMRMASKBLT;
            let hdcSrc: HDC = CreateCompatibleDC(hdc);
            let hBrush: HBRUSH;
            let hBrushOld: HBRUSH;
            let hBmp: HBITMAP;
            let hBmpOld: HBITMAP;
            let hBmpMask: HBITMAP;
            let mut pbi: *const BITMAPINFO;

            SetGraphicsMode(hdcSrc, GM_ADVANCED as INT);
            SetWorldTransform(hdcSrc, &raw const (*pMaskBlt).xformSrc);

            hBrush = CreateSolidBrush((*pMaskBlt).crBkColorSrc);
            hBrushOld = SelectObject(hdcSrc, hBrush as HGDIOBJ) as HBRUSH;
            PatBlt(
                hdcSrc,
                (*pMaskBlt).rclBounds.left as INT,
                (*pMaskBlt).rclBounds.top as INT,
                (*pMaskBlt)
                    .rclBounds
                    .right
                    .wrapping_sub((*pMaskBlt).rclBounds.left) as INT,
                (*pMaskBlt)
                    .rclBounds
                    .bottom
                    .wrapping_sub((*pMaskBlt).rclBounds.top) as INT,
                PATCOPY as DWORD,
            );
            SelectObject(hdcSrc, hBrushOld as HGDIOBJ);
            DeleteObject(hBrush as HGDIOBJ);

            pbi = (mr as *const BYTE).add((*pMaskBlt).offBmiMask as usize) as *const BITMAPINFO;
            hBmpMask = CreateBitmap(
                (*pbi).bmiHeader.biWidth as INT,
                (*pbi).bmiHeader.biHeight as INT,
                1,
                1,
                core::ptr::null(),
            );
            SetDIBits(
                hdc,
                hBmpMask,
                0,
                (*pbi).bmiHeader.biHeight as UINT,
                (mr as *const BYTE).add((*pMaskBlt).offBitsMask as usize) as *const c_void,
                pbi,
                (*pMaskBlt).iUsageMask as UINT,
            );

            pbi = (mr as *const BYTE).add((*pMaskBlt).offBmiSrc as usize) as *const BITMAPINFO;
            hBmp = CreateDIBitmap(
                hdc,
                pbi as *const BITMAPINFOHEADER,
                CBM_INIT as DWORD,
                (mr as *const BYTE).add((*pMaskBlt).offBitsSrc as usize) as *const c_void,
                pbi,
                (*pMaskBlt).iUsageSrc as UINT,
            );
            hBmpOld = SelectObject(hdcSrc, hBmp as HGDIOBJ) as HBITMAP;
            MaskBlt(
                hdc,
                (*pMaskBlt).xDest as INT,
                (*pMaskBlt).yDest as INT,
                (*pMaskBlt).cxDest as INT,
                (*pMaskBlt).cyDest as INT,
                hdcSrc,
                (*pMaskBlt).xSrc as INT,
                (*pMaskBlt).ySrc as INT,
                hBmpMask,
                (*pMaskBlt).xMask as INT,
                (*pMaskBlt).yMask as INT,
                (*pMaskBlt).dwRop,
            );
            SelectObject(hdcSrc, hBmpOld as HGDIOBJ);
            DeleteObject(hBmp as HGDIOBJ);
            DeleteObject(hBmpMask as HGDIOBJ);
            DeleteDC(hdcSrc);
        }
        EMR_PLGBLT => {
            let pPlgBlt = mr as *const EMRPLGBLT;
            let hdcSrc: HDC = CreateCompatibleDC(hdc);
            let hBrush: HBRUSH;
            let hBrushOld: HBRUSH;
            let hBmp: HBITMAP;
            let hBmpOld: HBITMAP;
            let hBmpMask: HBITMAP;
            let mut pbi: *const BITMAPINFO;
            let mut pts: [POINT; 3] = core::mem::zeroed();

            SetGraphicsMode(hdcSrc, GM_ADVANCED as INT);
            SetWorldTransform(hdcSrc, &raw const (*pPlgBlt).xformSrc);

            pts[0].x = (*pPlgBlt).aptlDest[0].x;
            pts[0].y = (*pPlgBlt).aptlDest[0].y;
            pts[1].x = (*pPlgBlt).aptlDest[1].x;
            pts[1].y = (*pPlgBlt).aptlDest[1].y;
            pts[2].x = (*pPlgBlt).aptlDest[2].x;
            pts[2].y = (*pPlgBlt).aptlDest[2].y;

            hBrush = CreateSolidBrush((*pPlgBlt).crBkColorSrc);
            hBrushOld = SelectObject(hdcSrc, hBrush as HGDIOBJ) as HBRUSH;
            PatBlt(
                hdcSrc,
                (*pPlgBlt).rclBounds.left as INT,
                (*pPlgBlt).rclBounds.top as INT,
                (*pPlgBlt)
                    .rclBounds
                    .right
                    .wrapping_sub((*pPlgBlt).rclBounds.left) as INT,
                (*pPlgBlt)
                    .rclBounds
                    .bottom
                    .wrapping_sub((*pPlgBlt).rclBounds.top) as INT,
                PATCOPY as DWORD,
            );
            SelectObject(hdcSrc, hBrushOld as HGDIOBJ);
            DeleteObject(hBrush as HGDIOBJ);

            pbi = (mr as *const BYTE).add((*pPlgBlt).offBmiMask as usize) as *const BITMAPINFO;
            hBmpMask = CreateBitmap(
                (*pbi).bmiHeader.biWidth as INT,
                (*pbi).bmiHeader.biHeight as INT,
                1,
                1,
                core::ptr::null(),
            );
            SetDIBits(
                hdc,
                hBmpMask,
                0,
                (*pbi).bmiHeader.biHeight as UINT,
                (mr as *const BYTE).add((*pPlgBlt).offBitsMask as usize) as *const c_void,
                pbi,
                (*pPlgBlt).iUsageMask as UINT,
            );

            pbi = (mr as *const BYTE).add((*pPlgBlt).offBmiSrc as usize) as *const BITMAPINFO;
            hBmp = CreateDIBitmap(
                hdc,
                pbi as *const BITMAPINFOHEADER,
                CBM_INIT as DWORD,
                (mr as *const BYTE).add((*pPlgBlt).offBitsSrc as usize) as *const c_void,
                pbi,
                (*pPlgBlt).iUsageSrc as UINT,
            );
            hBmpOld = SelectObject(hdcSrc, hBmp as HGDIOBJ) as HBITMAP;
            PlgBlt(
                hdc,
                pts.as_ptr(),
                hdcSrc,
                (*pPlgBlt).xSrc as INT,
                (*pPlgBlt).ySrc as INT,
                (*pPlgBlt).cxSrc as INT,
                (*pPlgBlt).cySrc as INT,
                hBmpMask,
                (*pPlgBlt).xMask as INT,
                (*pPlgBlt).yMask as INT,
            );
            SelectObject(hdcSrc, hBmpOld as HGDIOBJ);
            DeleteObject(hBmp as HGDIOBJ);
            DeleteObject(hBmpMask as HGDIOBJ);
            DeleteDC(hdcSrc);
        }
        EMR_SETDIBITSTODEVICE => {
            let pSetDIBitsToDevice = mr as *const EMRSETDIBITSTODEVICE;

            SetDIBitsToDevice(
                hdc,
                (*pSetDIBitsToDevice).xDest as INT,
                (*pSetDIBitsToDevice).yDest as INT,
                (*pSetDIBitsToDevice).cxSrc as DWORD,
                (*pSetDIBitsToDevice).cySrc as DWORD,
                (*pSetDIBitsToDevice).xSrc as INT,
                (*pSetDIBitsToDevice).ySrc as INT,
                (*pSetDIBitsToDevice).iStartScan as UINT,
                (*pSetDIBitsToDevice).cScans as UINT,
                (mr as *const BYTE).add((*pSetDIBitsToDevice).offBitsSrc as usize) as *const c_void,
                (mr as *const BYTE).add((*pSetDIBitsToDevice).offBmiSrc as usize)
                    as *const BITMAPINFO,
                (*pSetDIBitsToDevice).iUsageSrc as UINT,
            );
        }
        EMR_POLYTEXTOUTA => {
            let pPolyTextOutA = mr as *const EMRPOLYTEXTOUTA;
            // `LONG * sizeof(...)`: the LONG converts (sign-extends) to size_t first, as here.
            // The C does not NULL-check this allocation; kept as in the C.
            let polytextA: *mut POLYTEXTA = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPolyTextOutA).cStrings as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<POLYTEXTA>() as SIZE_T),
            ) as *mut POLYTEXTA;
            let aemrtext = (&raw const (*pPolyTextOutA).aemrtext) as *const EMRTEXT;
            let mut i: LONG;
            let mut xform: XFORM = core::mem::zeroed();
            let mut xformOld: XFORM = core::mem::zeroed();
            let gModeOld: ::core::ffi::c_int;

            gModeOld = SetGraphicsMode(hdc, (*pPolyTextOutA).iGraphicsMode as INT);
            GetWorldTransform(hdc, &mut xformOld);

            xform.eM11 = (*pPolyTextOutA).exScale;
            xform.eM12 = 0.0;
            xform.eM21 = 0.0;
            xform.eM22 = (*pPolyTextOutA).eyScale;
            xform.eDx = 0.0;
            xform.eDy = 0.0;
            SetWorldTransform(hdc, &xform);

            /* Set up POLYTEXTA structures */
            i = 0;
            while i < (*pPolyTextOutA).cStrings {
                let src = aemrtext.add(i as usize);
                let dst = polytextA.add(i as usize);
                (*dst).x = (*src).ptlReference.x as ::core::ffi::c_int;
                (*dst).y = (*src).ptlReference.y as ::core::ffi::c_int;
                (*dst).n = (*src).nChars as UINT;
                (*dst).lpstr = (mr as *const BYTE).add((*src).offString as usize) as LPCSTR;
                (*dst).uiFlags = (*src).fOptions as UINT;
                (*dst).rcl.left = (*src).rcl.left;
                (*dst).rcl.right = (*src).rcl.right;
                (*dst).rcl.top = (*src).rcl.top;
                (*dst).rcl.bottom = (*src).rcl.bottom;
                (*dst).pdx =
                    (mr as *mut BYTE).add((*src).offDx as usize) as *mut ::core::ffi::c_int;
                i += 1;
            }
            PolyTextOutA(hdc, polytextA, (*pPolyTextOutA).cStrings as INT);
            HeapFree(GetProcessHeap(), 0, polytextA as LPVOID);

            SetWorldTransform(hdc, &xformOld);
            SetGraphicsMode(hdc, gModeOld);
        }
        EMR_POLYTEXTOUTW => {
            let pPolyTextOutW = mr as *const EMRPOLYTEXTOUTW;
            // `LONG * sizeof(...)`: the LONG converts (sign-extends) to size_t first, as here.
            // The C does not NULL-check this allocation; kept as in the C.
            let polytextW: *mut POLYTEXTW = HeapAlloc(
                GetProcessHeap(),
                0,
                ((*pPolyTextOutW).cStrings as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<POLYTEXTW>() as SIZE_T),
            ) as *mut POLYTEXTW;
            let aemrtext = (&raw const (*pPolyTextOutW).aemrtext) as *const EMRTEXT;
            let mut i: LONG;
            let mut xform: XFORM = core::mem::zeroed();
            let mut xformOld: XFORM = core::mem::zeroed();
            let gModeOld: ::core::ffi::c_int;

            gModeOld = SetGraphicsMode(hdc, (*pPolyTextOutW).iGraphicsMode as INT);
            GetWorldTransform(hdc, &mut xformOld);

            xform.eM11 = (*pPolyTextOutW).exScale;
            xform.eM12 = 0.0;
            xform.eM21 = 0.0;
            xform.eM22 = (*pPolyTextOutW).eyScale;
            xform.eDx = 0.0;
            xform.eDy = 0.0;
            SetWorldTransform(hdc, &xform);

            /* Set up POLYTEXTW structures */
            i = 0;
            while i < (*pPolyTextOutW).cStrings {
                let src = aemrtext.add(i as usize);
                let dst = polytextW.add(i as usize);
                (*dst).x = (*src).ptlReference.x as ::core::ffi::c_int;
                (*dst).y = (*src).ptlReference.y as ::core::ffi::c_int;
                (*dst).n = (*src).nChars as UINT;
                (*dst).lpstr = (mr as *const BYTE).add((*src).offString as usize) as LPCWSTR;
                (*dst).uiFlags = (*src).fOptions as UINT;
                (*dst).rcl.left = (*src).rcl.left;
                (*dst).rcl.right = (*src).rcl.right;
                (*dst).rcl.top = (*src).rcl.top;
                (*dst).rcl.bottom = (*src).rcl.bottom;
                (*dst).pdx =
                    (mr as *mut BYTE).add((*src).offDx as usize) as *mut ::core::ffi::c_int;
                i += 1;
            }
            PolyTextOutW(hdc, polytextW, (*pPolyTextOutW).cStrings as INT);
            HeapFree(GetProcessHeap(), 0, polytextW as LPVOID);

            SetWorldTransform(hdc, &xformOld);
            SetGraphicsMode(hdc, gModeOld);
        }
        EMR_FILLRGN => {
            let pFillRgn = mr as *const EMRFILLRGN;
            let hRgn: HRGN = ExtCreateRegion(
                core::ptr::null(),
                (*pFillRgn).cbRgnData,
                (&raw const (*pFillRgn).RgnData) as *const RGNDATA,
            );
            FillRgn(
                hdc,
                hRgn,
                HT_OBJ!(handletable, (*pFillRgn).ihBrush) as HBRUSH,
            );
            DeleteObject(hRgn as HGDIOBJ);
        }
        EMR_FRAMERGN => {
            let pFrameRgn = mr as *const EMRFRAMERGN;
            let hRgn: HRGN = ExtCreateRegion(
                core::ptr::null(),
                (*pFrameRgn).cbRgnData,
                (&raw const (*pFrameRgn).RgnData) as *const RGNDATA,
            );
            FrameRgn(
                hdc,
                hRgn,
                HT_OBJ!(handletable, (*pFrameRgn).ihBrush) as HBRUSH,
                (*pFrameRgn).szlStroke.cx as INT,
                (*pFrameRgn).szlStroke.cy as INT,
            );
            DeleteObject(hRgn as HGDIOBJ);
        }
        EMR_INVERTRGN => {
            let pInvertRgn = mr as *const EMRINVERTRGN;
            let hRgn: HRGN = ExtCreateRegion(
                core::ptr::null(),
                (*pInvertRgn).cbRgnData,
                (&raw const (*pInvertRgn).RgnData) as *const RGNDATA,
            );
            InvertRgn(hdc, hRgn);
            DeleteObject(hRgn as HGDIOBJ);
        }
        EMR_PAINTRGN => {
            let pPaintRgn = mr as *const EMRPAINTRGN;
            let hRgn: HRGN = ExtCreateRegion(
                core::ptr::null(),
                (*pPaintRgn).cbRgnData,
                (&raw const (*pPaintRgn).RgnData) as *const RGNDATA,
            );
            PaintRgn(hdc, hRgn);
            DeleteObject(hRgn as HGDIOBJ);
        }
        EMR_SETTEXTJUSTIFICATION => {
            let pSetTextJust = mr as *const EMRSETTEXTJUSTIFICATION;
            SetTextJustification(
                hdc,
                (*pSetTextJust).nBreakExtra as INT,
                (*pSetTextJust).nBreakCount as INT,
            );
        }
        EMR_SETLAYOUT => {
            let pSetLayout = mr as *const EMRSETLAYOUT;
            SetLayout(hdc, (*pSetLayout).iMode as DWORD);
        }
        EMR_GRADIENTFILL => {
            let grad = mr as *mut EMRGRADIENTFILL;
            let ver = (&raw mut (*grad).Ver) as *mut TRIVERTEX;
            GdiGradientFill(
                hdc,
                ver,
                (*grad).nVer as ULONG,
                ver.add((*grad).nVer as usize) as PVOID,
                (*grad).nTri as ULONG,
                (*grad).ulMode,
            );
        }
        EMR_DRAWESCAPE => {
            let pemr: PEMRESCAPE = mr as PEMRESCAPE;
            DrawEscape(
                hdc,
                (*pemr).iEsc,
                (*pemr).cjIn,
                (&raw const (*pemr).Data) as LPCSTR,
            );
        }
        EMR_EXTESCAPE => {
            let pemr: PEMRESCAPE = mr as PEMRESCAPE;
            ExtEscape(
                hdc,
                (*pemr).iEsc,
                (*pemr).cjIn,
                (&raw const (*pemr).Data) as LPCSTR,
                0,
                core::ptr::null_mut(),
            );
        }
        EMR_NAMEDESCAPE => {
            let pemr: PEMRNAMEDESCAPE = mr as PEMRNAMEDESCAPE;
            let rounded_size: INT = (*pemr).cjIn.wrapping_add(3) & !3;
            let data = (&raw mut (*pemr).Data) as *mut BYTE;
            NamedEscape(
                hdc,
                data.offset(rounded_size as isize) as PWCHAR,
                (*pemr).iEsc,
                (*pemr).cjIn,
                data as LPSTR,
                0,
                core::ptr::null_mut(),
            );
        }
        // C: these twelve labels have no body and fall through into `default:`.
        EMR_GLSRECORD
        | EMR_GLSBOUNDEDRECORD
        | EMR_STARTDOC
        | EMR_SMALLTEXTOUT
        | EMR_FORCEUFIMAPPING
        | EMR_COLORCORRECTPALETTE
        | EMR_SETICMPROFILEA
        | EMR_SETICMPROFILEW
        | EMR_TRANSPARENTBLT
        | EMR_SETLINKEDUFI
        | EMR_COLORMATCHTOTARGETW
        | EMR_CREATECOLORSPACEW => {
            /* From docs: If PlayEnhMetaFileRecord doesn't recognize a
            record then ignore and return TRUE. */
            FIXME!("type %d is unimplemented\n", type_0);
        }
        _ => {
            /* From docs: If PlayEnhMetaFileRecord doesn't recognize a
            record then ignore and return TRUE. */
            FIXME!("type %d is unimplemented\n", type_0);
        }
    }
    tmprc.top = 0;
    tmprc.left = tmprc.top;
    tmprc.bottom = 1000;
    tmprc.right = tmprc.bottom;
    LPtoDP(hdc, (&raw mut tmprc) as *mut POINT, 2);
    TRACE!("L:0,0 - 1000,1000 -> D:%s\n", wine_dbgstr_rect(&tmprc));

    return TRUE as BOOL;
}
