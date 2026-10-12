use crate::winhlp32::winhelp::WINHELP_CreateHelpWindow;
use crate::winhlp32::winhelp::WINHELP_CreateIndexWindow;
use crate::winhlp32::winhelp::WINHELP_DeleteBackSet;
use crate::winhlp32::winhelp::WINHELP_GetOpenFileName;
use crate::winhlp32::winhelp::WINHELP_GetWindowInfo;
use crate::winhlp32::winhelp::WINHELP_LayoutMainWindow;
use crate::winhlp32::winhelp::WINHELP_LookupHelpFile;
use crate::winhlp32::winhelp::WINHELP_OpenHelpWindow;
use crate::winhlp32::winhelp::WINHELP_ReleaseWindow;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut MACRO_Loaded: *mut MacroDesc = ::core::ptr::null::<MacroDesc>() as *mut MacroDesc;

static mut MACRO_NumLoaded: ::core::ffi::c_uint = 0;

static mut MACRO_Builtins: [MacroDesc; 92] = unsafe {
    [
        MacroDesc {
            name: b"About\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_About as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"AddAccelerator\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"AA\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"UUS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_AddAccelerator as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ALink\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"AL\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SUS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ALink as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Annotate\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Annotate as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"AppendItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_AppendItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Back\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Back as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"BackFlush\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"BF\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_BackFlush as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"BookmarkDefine\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_BookmarkDefine as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"BookmarkMore\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_BookmarkMore as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"BrowseButtons\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_BrowseButtons as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ChangeButtonBinding\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CBB\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ChangeButtonBinding as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ChangeEnable\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CE\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ChangeEnable as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ChangeItemBinding\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CIB\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ChangeItemBinding as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CheckItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CheckItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CloseSecondarys\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CS\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CloseSecondarys as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CloseWindow\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CW\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CloseWindow as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Compare\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Compare as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Contents\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Contents as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ControlPanel\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ControlPanel as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CopyDialog\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CopyDialog as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CopyTopic\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CT\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CopyTopic as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"CreateButton\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"CB\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_CreateButton as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_DeleteItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"DeleteMark\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_DeleteMark as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"DestroyButton\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_DestroyButton as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"DisableButton\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"DB\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_DisableButton as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"DisableItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"DI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_DisableItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"EnableButton\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"EB\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_EnableButton as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"EnableItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"EI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_EnableItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"EndMPrint\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_EndMPrint as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ExecFile\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"EF\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSUS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ExecFile as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ExecProgram\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"EP\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ExecProgram as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Exit\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Exit as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ExtAbleItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ExtAbleItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ExtInsertItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSSSUU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ExtInsertItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ExtInsertMenu\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSSUU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ExtInsertMenu as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"FileExist\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"FE\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 1 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_FileExist as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"FileOpen\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"FO\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_FileOpen as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Find\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Find as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Finder\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"FD\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Finder as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"FloatingMenu\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_FloatingMenu as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Flush\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"FH\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Flush as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"FocusWindow\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_FocusWindow as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Generate\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SUU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Generate as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"GotoMark\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_GotoMark as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"HelpOn\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_HelpOn as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"HelpOnTop\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_HelpOnTop as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"History\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_History as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"InitMPrint\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 1 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_InitMPrint as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"InsertItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSSSU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_InsertItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"InsertMenu\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SSU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_InsertMenu as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"IfThen\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"IF\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"BS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_IfThen as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"IfThenElse\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"IE\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"BSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_IfThenElse as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"IsBook\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 1 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_IsBook as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"IsMark\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 1 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_IsMark as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"IsNotMark\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"NM\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 1 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_IsNotMark as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpContents\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpContents as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpContext\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"JC\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpContext as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpHash\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"JH\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpHash as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpHelpOn\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpHelpOn as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpID\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"JI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpID as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"JumpKeyword\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"JK\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_JumpKeyword as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"KLink\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"KL\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SUSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_KLink as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Menu\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"MU\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Menu as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"MPrintHash\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_MPrintHash as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"MPrintID\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_MPrintID as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Next\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Next as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"NoShow\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"NS\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_NoShow as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"PopupContext\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"PC\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_PopupContext as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"PopupHash\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_PopupHash as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"PopupId\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"PI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_PopupId as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"PositionWindow\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"PW\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"IIUUUS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_PositionWindow as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Prev\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Prev as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Print\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Print as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"PrinterSetup\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_PrinterSetup as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"RegisterRoutine\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"RR\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_RegisterRoutine as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"RemoveAccelerator\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"RA\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"UU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_RemoveAccelerator as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ResetMenu\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ResetMenu as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"SaveMark\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_SaveMark as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Search\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Search as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"SetContents\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"SU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_SetContents as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"SetHelpOnFile\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_SetHelpOnFile as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"SetPopupColor\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"SPC\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"UUU\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_SetPopupColor as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ShellExecute\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"SE\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSUUSS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ShellExecute as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"ShortCut\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"SH\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SSUUS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_ShortCut as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"TCard\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_TCard as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"Test\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_Test as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"TestALink\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 1 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_TestALink as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"TestKLink\0" as *const u8 as *const ::core::ffi::c_char,
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 1 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_TestKLink as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"UncheckItem\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"UI\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"S\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_UncheckItem as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: b"UpdateWindow\0" as *const u8 as *const ::core::ffi::c_char,
            alias: b"UW\0" as *const u8 as *const ::core::ffi::c_char,
            isBool: 0 as BOOL,
            arguments: b"SS\0" as *const u8 as *const ::core::ffi::c_char,
            fn_: MACRO_UpdateWindow as *mut ::core::ffi::c_void,
        },
        MacroDesc {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            alias: ::core::ptr::null::<::core::ffi::c_char>(),
            isBool: 0 as BOOL,
            arguments: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_: NULL,
        },
    ]
};

#[repr(C)]
pub struct MacroDesc {
    pub name: *const i8,
    pub alias: *const i8,
    pub isBool: i32,
    pub arguments: *const i8,
    pub fn_: *mut core::ffi::c_void,
}

pub type PMacroDesc = *mut MacroDesc;

unsafe extern "C" fn StrDup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut dst: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    dst = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (strlen(str) as SIZE_T).wrapping_add(1 as SIZE_T),
    ) as *mut ::core::ffi::c_char;
    strcpy(dst, str);
    return dst;
}

unsafe extern "C" fn MACRO_LookupButton(win: *mut WINHELP_WINDOW, name: LPCSTR) -> *mut *mut WINHELP_BUTTON {
    let mut b: *mut *mut WINHELP_BUTTON = core::ptr::addr_of_mut!((*win).first_button);
    while !(*b).is_null() {
        if lstrcmpiA(name, (**b).lpszID) == 0 {
            break;
        }
        b = core::ptr::addr_of_mut!((**b).next);
    }
    return b;
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_CreateButton(id: LPCSTR, name: LPCSTR, macro_0: LPCSTR) {
    let mut win: *mut WINHELP_WINDOW = MACRO_CurrentWindow() as *mut WINHELP_WINDOW;
    let mut button: *mut WINHELP_BUTTON = core::ptr::null_mut();
    let mut b: *mut *mut WINHELP_BUTTON = core::ptr::null_mut();
    let mut size: LONG = 0;
    let mut ptr: LPSTR = core::ptr::null_mut();

    WINE_TRACE!(
        b"(%s, %s, %s)\n\0".as_ptr() as *const i8,
        debugstr_a(id),
        debugstr_a(name),
        debugstr_a(macro_0),
    );

    size = (core::mem::size_of::<WINHELP_BUTTON>() as usize)
        .wrapping_add(strlen(id) as usize)
        .wrapping_add(strlen(name) as usize)
        .wrapping_add(strlen(macro_0) as usize)
        .wrapping_add(3 as usize) as LONG;

    button = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut WINHELP_BUTTON;
    if button.is_null() {
        return;
    }

    (*button).next = core::ptr::null_mut();
    (*button).hWnd = core::ptr::null_mut();

    ptr = (button as *mut CHAR)
        .offset(core::mem::size_of::<WINHELP_BUTTON>() as usize as isize) as LPSTR;

    strcpy(ptr, id);
    (*button).lpszID = ptr as LPCSTR;
    ptr = ptr.offset(
        (strlen(id) as usize + 1 as usize) as isize,
    );

    strcpy(ptr, name);
    (*button).lpszName = ptr as LPCSTR;
    ptr = ptr.offset(
        (strlen(name) as usize + 1 as usize) as isize,
    );

    strcpy(ptr, macro_0);
    (*button).lpszMacro = ptr as LPCSTR;

    (*button).wParam = (WH_FIRST_BUTTON as _);
    b = core::ptr::addr_of_mut!((*win).first_button);
    while !(*b).is_null() {
        (*button).wParam = max!(
            (*button).wParam,
            (**b).wParam + 1,
        );
        b = core::ptr::addr_of_mut!((**b).next) as *mut *mut WINHELP_BUTTON;
    }
    *b = button;

    WINHELP_LayoutMainWindow(win);
}
unsafe extern "system" fn MACRO_DestroyButton(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_DisableButton(id: LPCSTR) {
    let mut b: *mut *mut WINHELP_BUTTON;

    WINE_TRACE!(
        b"(%s)\n\0".as_ptr() as *const i8,
        debugstr_a(id),
    );

    b = MACRO_LookupButton(MACRO_CurrentWindow() as *mut WINHELP_WINDOW, id);
    if (*b).is_null() {
        WINE_FIXME!(
            b"Couldn't find button %s\n\0".as_ptr() as *const i8,
            debugstr_a(id),
        );
        return;
    }

    EnableWindow((**b).hWnd, (FALSE) as BOOL);
}

unsafe extern "system" fn MACRO_EnableButton(id: LPCSTR) {
    let mut b: *mut *mut WINHELP_BUTTON = core::ptr::null_mut();

    WINE_TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(id));

    b = MACRO_LookupButton(MACRO_CurrentWindow() as *mut WINHELP_WINDOW, id);
    if (*b).is_null() {
        WINE_FIXME!(b"Couldn't find button %s\n\0".as_ptr() as *const i8, debugstr_a(id));
        return;
    }

    EnableWindow((**b).hWnd, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_JumpContents(lpszPath: LPCSTR, lpszWindow: LPCSTR) {
    let mut hlpfile: *mut HLPFILE = core::ptr::null_mut();
    WINE_TRACE!("(%s, %s)\n", debugstr_a(lpszPath), debugstr_a(lpszWindow));
    hlpfile = WINHELP_LookupHelpFile(lpszPath);
    if !hlpfile.is_null() {
        WINHELP_OpenHelpWindow(
            Some(HLPFILE_PageByHash as unsafe extern "C" fn(*mut HLPFILE, LONG, *mut ULONG) -> *mut HLPFILE_PAGE),
            hlpfile,
            0,
            WINHELP_GetWindowInfo(hlpfile, lpszWindow),
            (SW_NORMAL as i32),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_About() {
    let mut name: [WCHAR; 256] = [0; 256];
    let mut icon: HICON = LoadImageW(
        Globals.hInstance as HINSTANCE,
        MAKEINTRESOURCEW!(IDI_WINHELP),
        IMAGE_ICON as UINT,
        48 as ::core::ffi::c_int,
        48 as ::core::ffi::c_int,
        LR_SHARED as UINT,
    ) as HICON;
    LoadStringW(
        Globals.hInstance as HINSTANCE,
        STID_WINE_HELP as UINT,
        core::ptr::addr_of_mut!(name) as LPWSTR,
        ARRAY_SIZE!(name) as ::core::ffi::c_int,
    );
    ShellAboutW(
        (*MACRO_CurrentWindow()).hMainWnd,
        core::ptr::addr_of_mut!(name) as LPCWSTR,
        core::ptr::null_mut(),
        icon,
    );
}
unsafe extern "system" fn MACRO_AddAccelerator(u1: LONG, u2: LONG, str: LPCSTR) {
    WINE_FIXME!(b"(%u, %u, %s)\n\0".as_ptr() as *const i8, u1, u2, debugstr_a(str));
}
unsafe extern "system" fn MACRO_ALink(str1: LPCSTR, u: LONG, str2: LPCSTR) {
    WINE_FIXME!(b"(%s, %u, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), u, debugstr_a(str2));
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_Annotate() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_AppendItem(
    str1: LPCSTR,
    str2: LPCSTR,
    str3: LPCSTR,
    str4: LPCSTR,
) {
    WINE_FIXME!(b"(%s, %s, %s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), debugstr_a(str3), debugstr_a(str4));
}
unsafe extern "system" fn MACRO_Back() {
    let mut win: *mut WINHELP_WINDOW = MACRO_CurrentWindow() as *mut WINHELP_WINDOW;

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);

    if !win.is_null() && (*win).back.index >= 2 {
        (*win).back.index = (*win).back.index.wrapping_sub(1);
        WINHELP_CreateHelpWindow(
            (core::ptr::addr_of_mut!((*win).back.set) as *mut WINHELP_WNDPAGE)
                .offset((*win).back.index.wrapping_sub(1) as isize)
                as *mut WINHELP_WNDPAGE,
            (SW_SHOW as i32),
            (FALSE) as BOOL,
        );
    }
}
unsafe extern "system" fn MACRO_BackFlush() {
    let mut win: *mut WINHELP_WINDOW = MACRO_CurrentWindow() as *mut WINHELP_WINDOW;

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);

    if !win.is_null() {
        WINHELP_DeleteBackSet(win);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_BookmarkDefine() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_BookmarkMore() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn MACRO_BrowseButtons() {
    let mut page: *mut HLPFILE_PAGE = (*MACRO_CurrentWindow()).page;
    let mut relative: ULONG = 0;

    WINE_TRACE!(b"()\n\0".as_ptr() as LPCSTR);

    MACRO_CreateButton(
        b"BTN_PREV\0".as_ptr() as LPCSTR,
        b"&<<\0".as_ptr() as LPCSTR,
        b"Prev()\0".as_ptr() as LPCSTR,
    );
    MACRO_CreateButton(
        b"BTN_NEXT\0".as_ptr() as LPCSTR,
        b"&>>\0".as_ptr() as LPCSTR,
        b"Next()\0".as_ptr() as LPCSTR,
    );

    if HLPFILE_PageByOffset(
        (*page).file as *mut HLPFILE,
        (*page).browse_bwd as LONG,
        core::ptr::addr_of_mut!(relative),
    )
    .is_null()
    {
        MACRO_DisableButton(b"BTN_PREV\0".as_ptr() as LPCSTR);
    }
    if HLPFILE_PageByOffset(
        (*page).file as *mut HLPFILE,
        (*page).browse_fwd as LONG,
        core::ptr::addr_of_mut!(relative),
    )
    .is_null()
    {
        MACRO_DisableButton(b"BTN_NEXT\0".as_ptr() as LPCSTR);
    }
}

unsafe extern "system" fn MACRO_ChangeButtonBinding(id: LPCSTR, macro_: LPCSTR) {
    let mut win: *mut WINHELP_WINDOW = MACRO_CurrentWindow() as *mut WINHELP_WINDOW;
    let mut button: *mut WINHELP_BUTTON;
    let mut b: *mut *mut WINHELP_BUTTON;
    let mut size: LONG;
    let mut ptr: LPSTR;

    WINE_TRACE!(
        b"(%s, %s)\n\0".as_ptr() as LPCSTR,
        debugstr_a(id),
        debugstr_a(macro_),
    );

    b = MACRO_LookupButton(win, id);
    if (*b).is_null() {
        WINE_FIXME!(
            b"Couldn't find button %s\n\0".as_ptr() as LPCSTR,
            debugstr_a(id),
        );
        return;
    }

    size = (core::mem::size_of::<WINHELP_BUTTON>()
        .wrapping_add(strlen(id) as usize)
        .wrapping_add(strlen((**b).lpszName) as usize)
        .wrapping_add(strlen(macro_) as usize)
        .wrapping_add((3) as usize)) as LONG;

    button = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        size as SIZE_T,
    ) as *mut WINHELP_BUTTON;
    if button.is_null() {
        return;
    }

    (*button).next = (**b).next;
    (*button).hWnd = (**b).hWnd;
    (*button).wParam = (**b).wParam;

    ptr = (button as *mut CHAR)
        .offset(core::mem::size_of::<WINHELP_BUTTON>() as usize as isize) as LPSTR;

    strcpy(ptr, id);
    (*button).lpszID = ptr as LPCSTR;
    ptr = ptr.offset(
        (strlen(id) as usize).wrapping_add((1) as usize)
            as isize,
    );

    strcpy(ptr, (**b).lpszName);
    (*button).lpszName = ptr as LPCSTR;
    ptr = ptr.offset(
        (strlen((**b).lpszName) as usize)
            .wrapping_add((1) as usize) as isize,
    );

    strcpy(ptr, macro_);
    (*button).lpszMacro = ptr as LPCSTR;

    *b = button;

    WINHELP_LayoutMainWindow(win);
}

unsafe extern "system" fn MACRO_ChangeEnable(id: LPCSTR, macro_0: LPCSTR) {
    WINE_TRACE!(
        b"(%s, %s)\n\0".as_ptr() as *const i8,
        debugstr_a(id),
        debugstr_a(macro_0),
    );

    MACRO_ChangeButtonBinding(id, macro_0);
    MACRO_EnableButton(id);
}
unsafe extern "system" fn MACRO_ChangeItemBinding(str1: LPCSTR, str2: LPCSTR) {
    WINE_FIXME!(b"(%s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2));
}
unsafe extern "system" fn MACRO_CheckItem(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}

unsafe extern "system" fn MACRO_CloseSecondarys() {
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut();
    let mut next: *mut WINHELP_WINDOW = core::ptr::null_mut();

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);
    win = Globals.win_list;
    while !win.is_null() {
        next = (*win).next as *mut WINHELP_WINDOW;
        if lstrcmpiA(
            &raw mut (*(*win).info).name as *mut i8 as LPCSTR,
            b"main\0".as_ptr() as LPCSTR,
        ) != 0
        {
            WINHELP_ReleaseWindow(win);
        }
        win = next;
    }
}

unsafe extern "system" fn MACRO_CloseWindow(mut lpszWindow: LPCSTR) {
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut();
    let mut next: *mut WINHELP_WINDOW = core::ptr::null_mut();

    WINE_TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(lpszWindow));

    if lpszWindow.is_null() || *lpszWindow.offset(0 as isize) == 0 {
        lpszWindow = b"main\0".as_ptr() as *const i8 as LPCSTR;
    }

    win = Globals.win_list;
    while !win.is_null() {
        next = (*win).next as *mut WINHELP_WINDOW;
        if lstrcmpiA(
            core::ptr::addr_of_mut!((*(*win).info).name) as *mut i8 as LPCSTR,
            lpszWindow,
        ) == 0
        {
            WINHELP_ReleaseWindow(win);
        }
        win = next;
    }
}
unsafe extern "system" fn MACRO_Compare(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_Contents() {
    let mut page: *mut HLPFILE_PAGE = (*MACRO_CurrentWindow()).page;

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);

    if !page.is_null() {
        MACRO_JumpContents((*(*page).file).lpszPath as LPCSTR, core::ptr::null());
    }
}
unsafe extern "system" fn MACRO_ControlPanel(str1: LPCSTR, str2: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), u);
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_CopyDialog() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_CopyTopic() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_DeleteItem(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_DeleteMark(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_DisableItem(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_EnableItem(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_EndMPrint() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn MACRO_ExecFile(pgm: LPCSTR, args: LPCSTR, cmd_show: LONG, topic: LPCSTR) {
    let mut ret: HINSTANCE;

    WINE_TRACE!(
        b"(%s, %s, %u, %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
        debugstr_a(pgm as *const ::core::ffi::c_char),
        debugstr_a(args as *const ::core::ffi::c_char),
        cmd_show,
        debugstr_a(topic as *const ::core::ffi::c_char),
    );

    ret = ShellExecuteA(
        if !Globals.active_win.is_null() {
            (*Globals.active_win).hMainWnd
        } else {
            ::core::ptr::null_mut::<HWND__>()
        },
        b"open\0" as *const u8 as LPCSTR,
        pgm,
        args,
        b".\0" as *const u8 as LPCSTR,
        cmd_show as INT,
    );
    if (ret as DWORD_PTR) < 32 {
        WINE_WARN!(
            b"Failed with %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
        if !topic.is_null() {
            MACRO_JumpID(::core::ptr::null::<CHAR>(), topic);
        }
    }
}
unsafe extern "system" fn MACRO_ExecProgram(str: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str), u);
}
#[no_mangle]
pub unsafe extern "system" fn MACRO_Exit() {
    WINE_TRACE!("()\n\0".as_ptr() as *const i8);

    while !Globals.win_list.is_null() {
        WINHELP_ReleaseWindow(Globals.win_list);
    }
}
unsafe extern "system" fn MACRO_ExtAbleItem(str: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str), u);
}
unsafe extern "system" fn MACRO_ExtInsertItem(
    str1: LPCSTR,
    str2: LPCSTR,
    str3: LPCSTR,
    str4: LPCSTR,
    u1: LONG,
    u2: LONG,
) {
    WINE_FIXME!(b"(%s, %s, %s, %s, %u, %u)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), debugstr_a(str3), debugstr_a(str4), u1, u2);
}
unsafe extern "system" fn MACRO_ExtInsertMenu(
    str1: LPCSTR,
    str2: LPCSTR,
    str3: LPCSTR,
    u1: LONG,
    u2: LONG,
) {
    WINE_FIXME!(b"(%s, %s, %s, %u, %u)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), debugstr_a(str3), u1, u2);
}

unsafe extern "system" fn MACRO_FileExist(str: LPCSTR) -> BOOL {
    WINE_TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
    return (GetFileAttributesA(str) != INVALID_FILE_ATTRIBUTES) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn MACRO_FileOpen() {
    let mut szFile: [::core::ffi::c_char; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if WINHELP_GetOpenFileName(szFile.as_mut_ptr() as LPSTR, (MAX_PATH as i32)) != 0 {
        MACRO_JumpContents(
            szFile.as_mut_ptr() as LPCSTR,
            b"main\0".as_ptr() as LPCSTR,
        );
    }
}
unsafe extern "system" fn MACRO_Find() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn MACRO_Finder() {
    WINHELP_CreateIndexWindow((FALSE) as BOOL);
}
unsafe extern "system" fn MACRO_FloatingMenu() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_Flush() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_FocusWindow(mut lpszWindow: LPCSTR) {
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut();
    WINE_TRACE!("(%s)\n", debugstr_a(lpszWindow as *const i8));

    if lpszWindow.is_null() || *lpszWindow.offset(0 as isize) == 0 {
        lpszWindow = b"main\0".as_ptr() as *const i8 as LPCSTR;
    }

    win = Globals.win_list;
    while !win.is_null() {
        if lstrcmpiA(core::ptr::addr_of_mut!((*(*win).info).name) as *mut i8 as LPCSTR, lpszWindow) == 0 {
            SetFocus((*win).hMainWnd);
        }
        win = (*win).next as *mut WINHELP_WINDOW;
    }
}
unsafe extern "system" fn MACRO_Generate(str: LPCSTR, w: LONG, l: LONG) {
    WINE_FIXME!(b"(%s, %x, %x)\n\0".as_ptr() as *const i8, debugstr_a(str), w, l);
}
unsafe extern "system" fn MACRO_GotoMark(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
#[no_mangle]
pub unsafe extern "system" fn MACRO_HelpOn() {
    let mut win: *mut WINHELP_WINDOW = MACRO_CurrentWindow() as *mut WINHELP_WINDOW;
    let mut file: LPCSTR = core::ptr::null();

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);
    if !win.is_null() && !(*win).page.is_null() && !(*(*win).page).file.is_null() {
        file = (*(*(*win).page).file).help_on_file as LPCSTR;
    }

    if file.is_null() {
        file = if Globals.wVersion > 4 {
            b"winhlp32.hlp\0".as_ptr() as *const i8
        } else {
            b"winhelp.hlp\0".as_ptr() as *const i8
        };
    }

    MACRO_JumpContents(file, core::ptr::null());
}
#[no_mangle]
pub unsafe extern "system" fn MACRO_HelpOnTop() {
    static mut on_top: BOOL = (FALSE as i32);
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut::<WINHELP_WINDOW>();
    let mut main_wnd: HWND = core::ptr::null_mut::<HWND__>();
    let mut menu: HMENU = core::ptr::null_mut::<HMENU__>();
    win = Globals.win_list;
    while !win.is_null() {
        if lstrcmpiA(
            core::ptr::addr_of_mut!((*(*win).info).name) as *mut core::ffi::c_char as LPCSTR,
            b"main\0" as *const u8 as LPCSTR,
        ) == 0
        {
            main_wnd = (*win).hMainWnd;
        }
        win = (*win).next as *mut WINHELP_WINDOW;
    }
    if main_wnd.is_null() {
        WINE_ERR!("could not find the main window!\n");
        return;
    }
    menu = GetMenu(main_wnd);

    on_top = (on_top == 0) as BOOL;
    if on_top != 0 {
        CheckMenuItem(
            menu,
            MNID_HELP_HELPTOP as UINT,
            (MF_BYCOMMAND | MF_CHECKED) as UINT,
        );
        SetWindowPos(
            main_wnd,
            HWND_TOPMOST,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            (SWP_NOMOVE | SWP_NOSIZE) as UINT,
        );
    } else {
        CheckMenuItem(
            menu,
            MNID_HELP_HELPTOP as UINT,
            (MF_BYCOMMAND | MF_UNCHECKED) as UINT,
        );
        SetWindowPos(
            main_wnd,
            HWND_NOTOPMOST,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            (SWP_NOMOVE | SWP_NOSIZE) as UINT,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn MACRO_History() {
    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);

    if !Globals.active_win.is_null() && (*Globals.active_win).hHistoryWnd.is_null() {
        let mut hWnd: HWND = CreateWindowA!(
            HISTORY_WIN_CLASS_NAME.as_ptr(),
            b"History\0".as_ptr() as LPCSTR,
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            0,
            0,
            Globals.hInstance,
            Globals.active_win,
        );
        ShowWindow(hWnd, (SW_NORMAL as i32));
    }
}
unsafe extern "system" fn MACRO_IfThen(b: BOOL, t: LPCSTR) {
    if b != 0 {
        MACRO_ExecuteMacro(MACRO_CurrentWindow(), t);
    }
}

unsafe extern "system" fn MACRO_IfThenElse(b: BOOL, t: LPCSTR, f: LPCSTR) {
    if b != 0 {
        MACRO_ExecuteMacro(MACRO_CurrentWindow(), t);
    } else {
        MACRO_ExecuteMacro(MACRO_CurrentWindow(), f);
    }
}
unsafe extern "system" fn MACRO_InitMPrint() -> BOOL {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    (FALSE as i32)
}
unsafe extern "system" fn MACRO_InsertItem(
    str1: LPCSTR,
    str2: LPCSTR,
    str3: LPCSTR,
    str4: LPCSTR,
    u: LONG,
) {
    WINE_FIXME!(b"(%s, %s, %s, %s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), debugstr_a(str3), debugstr_a(str4), u);
}
unsafe extern "system" fn MACRO_InsertMenu(str1: LPCSTR, str2: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), u);
}

unsafe extern "system" fn MACRO_IsBook() -> BOOL {
    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);
    return Globals.isBook;
}
unsafe extern "system" fn MACRO_IsMark(str: LPCSTR) -> BOOL {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
    (FALSE as i32)
}
unsafe extern "system" fn MACRO_IsNotMark(str: LPCSTR) -> BOOL {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_JumpContext(lpszPath: LPCSTR, lpszWindow: LPCSTR, context: LONG) {
    let mut hlpfile: *mut HLPFILE = core::ptr::null_mut();

    WINE_TRACE!(
        b"(%s, %s, %d)\n\0".as_ptr() as *const i8,
        debugstr_a(lpszPath),
        debugstr_a(lpszWindow),
        context,
    );
    hlpfile = WINHELP_LookupHelpFile(lpszPath);
    if !hlpfile.is_null() {
        /* Some madness: what user calls 'context', hlpfile calls 'map' */
        WINHELP_OpenHelpWindow(
            Some(HLPFILE_PageByMap as unsafe extern "C" fn(*mut HLPFILE, LONG, *mut ULONG) -> *mut HLPFILE_PAGE),
            hlpfile,
            context,
            WINHELP_GetWindowInfo(hlpfile, lpszWindow),
            (SW_NORMAL as i32),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_JumpHash(lpszPath: LPCSTR, lpszWindow: LPCSTR, lHash: LONG) {
    let mut hlpfile: *mut HLPFILE = core::ptr::null_mut();
    WINE_TRACE!(
        b"(%s, %s, %u)\n\0".as_ptr() as *const i8,
        debugstr_a(lpszPath),
        debugstr_a(lpszWindow),
        lHash,
    );
    if lpszPath.is_null() || *lpszPath.offset(0 as isize) == 0 {
        hlpfile = (*(*MACRO_CurrentWindow()).page).file as *mut HLPFILE;
    } else {
        hlpfile = WINHELP_LookupHelpFile(lpszPath);
    }
    if !hlpfile.is_null() {
        WINHELP_OpenHelpWindow(
            Some(
                HLPFILE_PageByHash
                    as unsafe extern "C" fn(
                        *mut HLPFILE,
                        LONG,
                        *mut ULONG,
                    ) -> *mut HLPFILE_PAGE,
            ),
            hlpfile,
            lHash,
            WINHELP_GetWindowInfo(hlpfile, lpszWindow),
            (SW_NORMAL as i32),
        );
    }
}
unsafe extern "system" fn MACRO_JumpHelpOn() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn MACRO_JumpID(lpszPathWindow: LPCSTR, topic_id: LPCSTR) {
    let mut ptr: LPSTR = core::ptr::null_mut();
    WINE_TRACE!(
        b"(%s, %s)\n\0".as_ptr() as *const i8,
        debugstr_a(lpszPathWindow),
        debugstr_a(topic_id),
    );
    if !lpszPathWindow.is_null() && {
        ptr = strchr(lpszPathWindow, '>' as i32) as LPSTR;
        !ptr.is_null()
    } {
        let mut tmp: LPSTR = core::ptr::null_mut();
        let mut sz: size_t = 0;
        tmp = HeapAlloc(
            GetProcessHeap(),
            0,
            (strlen(lpszPathWindow) + 1) as SIZE_T,
        ) as LPSTR;
        if !tmp.is_null() {
            strcpy(tmp, lpszPathWindow);
            *tmp.offset(ptr.offset_from(lpszPathWindow) as isize) = 0 as CHAR;
            ptr = ptr.offset(tmp.offset_from(lpszPathWindow) as isize);
            sz = strlen(ptr.offset(1 as isize) as *const i8);
            while sz >= 1 && *ptr.offset(sz as isize) as i32 == ' ' as i32 {
                *ptr.offset(sz as isize) = 0 as CHAR;
                sz = sz.wrapping_sub(1);
            }
            MACRO_JumpHash(
                tmp as LPCSTR,
                ptr.offset(1 as isize) as LPCSTR,
                HLPFILE_Hash(topic_id),
            );
            HeapFree(
                GetProcessHeap(),
                0,
                tmp as PVOID,
            );
        }
    } else {
        MACRO_JumpHash(
            lpszPathWindow,
            core::ptr::null::<CHAR>(),
            HLPFILE_Hash(topic_id),
        );
    }
}
unsafe extern "system" fn MACRO_JumpKeyword(
    lpszPath: LPCSTR,
    lpszWindow: LPCSTR,
    keyword: LPCSTR,
) {
    WINE_FIXME!(b"(%s, %s, %s)\n\0".as_ptr() as *const i8, debugstr_a(lpszPath), debugstr_a(lpszWindow), debugstr_a(keyword));
}
unsafe extern "system" fn MACRO_KLink(
    str1: LPCSTR,
    u: LONG,
    str2: LPCSTR,
    str3: LPCSTR,
) {
    WINE_FIXME!(b"(%s, %u, %s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), u, debugstr_a(str2), debugstr_a(str3));
}
unsafe extern "system" fn MACRO_Menu() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_MPrintHash(u: LONG) {
    WINE_FIXME!(b"(%u)\n\0".as_ptr() as *const i8, u);
}
unsafe extern "system" fn MACRO_MPrintID(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_Next() {
    let mut wp: WINHELP_WNDPAGE = core::mem::zeroed();
    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);
    wp.page = (*MACRO_CurrentWindow()).page;
    wp.page = HLPFILE_PageByOffset(
        (*wp.page).file,
        ((*wp.page).browse_fwd) as LONG,
        core::ptr::addr_of_mut!(wp.relative),
    );
    if !wp.page.is_null() {
        (*(*wp.page).file).wRefCount += 1;
        wp.wininfo = (*MACRO_CurrentWindow()).info;
        WINHELP_CreateHelpWindow(core::ptr::addr_of_mut!(wp), (SW_NORMAL as i32), (TRUE) as BOOL);
    }
}
unsafe extern "system" fn MACRO_NoShow() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_PopupContext(str: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str), u);
}
unsafe extern "system" fn MACRO_PopupHash(str: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str), u);
}
unsafe extern "system" fn MACRO_PopupId(str1: LPCSTR, str2: LPCSTR) {
    WINE_FIXME!(b"(%s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2));
}
unsafe extern "system" fn MACRO_PositionWindow(
    i1: LONG,
    i2: LONG,
    u1: LONG,
    u2: LONG,
    u3: LONG,
    str: LPCSTR,
) {
    WINE_FIXME!(b"(%i, %i, %u, %u, %u, %s)\n\0".as_ptr() as *const i8, i1, i2, u1, u2, u3, debugstr_a(str));
}
unsafe extern "system" fn MACRO_Prev() {
    let mut wp: WINHELP_WNDPAGE = core::mem::zeroed();
    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);
    wp.page = (*MACRO_CurrentWindow()).page;
    wp.page = HLPFILE_PageByOffset((*wp.page).file, ((*wp.page).browse_bwd) as LONG, &mut wp.relative);
    if !wp.page.is_null() {
        (*(*wp.page).file).wRefCount += 1;
        wp.wininfo = (*MACRO_CurrentWindow()).info;
        WINHELP_CreateHelpWindow(&mut wp, (SW_NORMAL as i32), (TRUE) as BOOL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_Print() {
    let mut printer: PRINTDLGW = core::mem::zeroed();

    WINE_TRACE!(b"()\n\0".as_ptr() as *const i8);

    printer.lStructSize = core::mem::size_of::<PRINTDLGW>() as DWORD;
    printer.hwndOwner = (*MACRO_CurrentWindow()).hMainWnd;
    printer.hInstance = (Globals.hInstance).cast();
    printer.hDevMode = core::ptr::null_mut();
    printer.hDevNames = core::ptr::null_mut();
    printer.hDC = core::ptr::null_mut();
    printer.Flags = 0;
    printer.nFromPage = 0;
    printer.nToPage = 0;
    printer.nMinPage = 0;
    printer.nMaxPage = 0;
    printer.nCopies = 0;
    printer.lCustData = 0;
    printer.lpfnPrintHook = None;
    printer.lpfnSetupHook = None;
    printer.lpPrintTemplateName = core::ptr::null();
    printer.lpSetupTemplateName = core::ptr::null();
    printer.hPrintTemplate = core::ptr::null_mut();
    printer.hSetupTemplate = core::ptr::null_mut();

    if PrintDlgW(core::ptr::addr_of_mut!(printer)) != 0 {
        WINE_FIXME!(b"Print()\n\0".as_ptr() as *const i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_PrinterSetup() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn MACRO_RegisterRoutine(dll_name: LPCSTR, proc: LPCSTR, args: LPCSTR) {
    let mut fn_0: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut size: core::ffi::c_int = 0;
    let mut dll: *mut WINHELP_DLL = core::ptr::null_mut();

    WINE_TRACE!(
        b"(%s, %s, %s)\n\0" as *const u8 as *const core::ffi::c_char,
        debugstr_a(dll_name),
        debugstr_a(proc),
        debugstr_a(args),
    );

    /* FIXME: are the registered DLLs global or linked to the current file ???
     * We assume globals (as we did for macros, but is this really the case ???)
     */
    dll = Globals.dlls;
    while !dll.is_null() {
        if strcmp((*dll).name, dll_name) == 0 {
            break;
        }
        dll = (*dll).next as *mut WINHELP_DLL;
    }
    if dll.is_null() {
        let mut hLib: HANDLE = LoadLibraryA(dll_name) as HANDLE;

        /* FIXME: the library will not be unloaded until exit of program
         * We don't send the DW_TERM message
         */
        WINE_TRACE!(
            b"Loading %s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_a(dll_name),
        );
        /* FIXME: should look in the directory where current hlpfile
         * is loaded from
         */
        if hLib.is_null() {
            /* FIXME: internationalisation for error messages */
            WINE_FIXME!(
                b"Cannot find dll %s\n\0" as *const u8 as *const core::ffi::c_char,
                debugstr_a(dll_name),
            );
        } else {
            dll = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                core::mem::size_of::<WINHELP_DLL>() as SIZE_T,
            ) as *mut WINHELP_DLL;
            if !dll.is_null() {
                (*dll).hLib = hLib;
                (*dll).name = StrDup(dll_name); /* FIXME: never freed */
                (*dll).next = Globals.dlls as *mut tagDll;
                Globals.dlls = dll;
                (*dll).handler = ::core::mem::transmute::<FARPROC, WINHELP_LDLLHandler>(
                    GetProcAddress(
                        (*dll).hLib as HINSTANCE,
                        b"LDLLHandler\0" as *const u8 as LPCSTR,
                    ),
                );
                (*dll).class = (if (*dll).handler.is_some() {
                    (*dll).handler.unwrap_unchecked()(
                        DW_WHATMSG as WORD,
                        0 as LONG_PTR,
                        0 as LONG_PTR,
                    )
                } else {
                    DC_NOMSG as LONG
                }) as DWORD;
                WINE_TRACE!(
                    b"Got class %x for DLL %s\n\0" as *const u8 as *const core::ffi::c_char,
                    (*dll).class,
                    debugstr_a(dll_name),
                );
                if ((*dll).class & (DC_INITTERM as DWORD)) != 0 {
                    (*dll).handler.unwrap_unchecked()(
                        DW_INIT as WORD,
                        0 as LONG_PTR,
                        0 as LONG_PTR,
                    );
                }
                if ((*dll).class & (DC_CALLBACKS as DWORD)) != 0 {
                    (*dll).handler.unwrap_unchecked()(
                        DW_CALLBACKS as WORD,
                        core::ptr::addr_of!(Callbacks) as LONG_PTR,
                        0 as LONG_PTR,
                    );
                }
            } else {
                WINE_WARN!(b"OOM\n\0" as *const u8 as *const core::ffi::c_char);
            }
        }
    }
    if !dll.is_null() && {
        fn_0 = ::core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress((*dll).hLib as HINSTANCE, proc),
        );
        fn_0.is_null()
    } {
        /* FIXME: internationalisation for error messages */
        WINE_FIXME!(
            b"Cannot find proc %s in dll %s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_a(dll_name),
            debugstr_a(proc),
        );
    }

    MACRO_NumLoaded = MACRO_NumLoaded.wrapping_add(1);
    size = (MACRO_NumLoaded as usize).wrapping_mul(core::mem::size_of::<MacroDesc>() as usize)
        as core::ffi::c_int;
    if MACRO_Loaded.is_null() {
        MACRO_Loaded = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            size as SIZE_T,
        ) as *mut MacroDesc;
    } else {
        MACRO_Loaded = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            MACRO_Loaded as PVOID,
            size as SIZE_T,
        ) as *mut MacroDesc;
    }
    (*MACRO_Loaded.offset(
        MACRO_NumLoaded.wrapping_sub(1 as core::ffi::c_uint) as isize,
    ))
    .name = StrDup(proc); /* FIXME: never freed */
    (*MACRO_Loaded.offset(
        MACRO_NumLoaded.wrapping_sub(1 as core::ffi::c_uint) as isize,
    ))
    .alias = core::ptr::null_mut();
    (*MACRO_Loaded.offset(
        MACRO_NumLoaded.wrapping_sub(1 as core::ffi::c_uint) as isize,
    ))
    .isBool = FALSE as BOOL;
    (*MACRO_Loaded.offset(
        MACRO_NumLoaded.wrapping_sub(1 as core::ffi::c_uint) as isize,
    ))
    .arguments = StrDup(args); /* FIXME: never freed */
    (*MACRO_Loaded.offset(
        MACRO_NumLoaded.wrapping_sub(1 as core::ffi::c_uint) as isize,
    ))
    .fn_ = fn_0;
    WINE_TRACE!(
        b"Added %s(%s) at %p\n\0" as *const u8 as *const core::ffi::c_char,
        debugstr_a(proc),
        debugstr_a(args),
        fn_0,
    );
}
unsafe extern "system" fn MACRO_RemoveAccelerator(u1: LONG, u2: LONG) {
    WINE_FIXME!(b"(%u, %u)\n\0".as_ptr() as *const i8, u1, u2);
}
unsafe extern "system" fn MACRO_ResetMenu() {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn MACRO_SaveMark(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}

unsafe extern "system" fn MACRO_Search() {
    WINHELP_CreateIndexWindow((TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn MACRO_SetContents(str: LPCSTR, u: LONG) {
    WINE_FIXME!(b"(%s, %u)\n\0".as_ptr() as *const i8, debugstr_a(str), u);
}

unsafe extern "system" fn MACRO_SetHelpOnFile(str: LPCSTR) {
    let mut page: *mut HLPFILE_PAGE = (*MACRO_CurrentWindow()).page;

    WINE_TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str as *const i8));

    HeapFree(GetProcessHeap(), 0, (*(*page).file).help_on_file as PVOID);
    (*(*page).file).help_on_file = HeapAlloc(
        GetProcessHeap(),
        0,
        (strlen(str as *const i8) as SIZE_T) + 1,
    ) as LPSTR;
    if !(*(*page).file).help_on_file.is_null() {
        strcpy((*(*page).file).help_on_file as *mut i8, str as *const i8);
    }
}

unsafe extern "system" fn MACRO_SetPopupColor(r: LONG, g: LONG, b: LONG) {
    let page: *mut HLPFILE_PAGE = (*MACRO_CurrentWindow()).page;

    WINE_TRACE!(b"(%x, %x, %x)\n\0".as_ptr() as *const i8, r, g, b);
    (*(*page).file).has_popup_color = (TRUE as i32);
    (*(*page).file).popup_color = RGB!(r, g, b);
}
unsafe extern "system" fn MACRO_ShellExecute(
    str1: LPCSTR,
    str2: LPCSTR,
    u1: LONG,
    u2: LONG,
    str3: LPCSTR,
    str4: LPCSTR,
) {
    WINE_FIXME!(b"(%s, %s, %u, %u, %s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), u1, u2, debugstr_a(str3), debugstr_a(str4));
}
unsafe extern "system" fn MACRO_ShortCut(
    str1: LPCSTR,
    str2: LPCSTR,
    w: LONG,
    l: LONG,
    str: LPCSTR,
) {
    WINE_FIXME!(b"(%s, %s, %x, %x, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2), w, l, debugstr_a(str));
}
unsafe extern "system" fn MACRO_TCard(u: LONG) {
    WINE_FIXME!(b"(%u)\n\0".as_ptr() as *const i8, u);
}
unsafe extern "system" fn MACRO_Test(u: LONG) {
    WINE_FIXME!(b"(%u)\n\0".as_ptr() as *const i8, u);
}
unsafe extern "system" fn MACRO_TestALink(str: LPCSTR) -> BOOL {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
    (FALSE as i32)
}
unsafe extern "system" fn MACRO_TestKLink(str: LPCSTR) -> BOOL {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
    (FALSE as i32)
}
unsafe extern "system" fn MACRO_UncheckItem(str: LPCSTR) {
    WINE_FIXME!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_a(str));
}
unsafe extern "system" fn MACRO_UpdateWindow(str1: LPCSTR, str2: LPCSTR) {
    WINE_FIXME!(b"(%s, %s)\n\0".as_ptr() as *const i8, debugstr_a(str1), debugstr_a(str2));
}

unsafe extern "C" fn MACRO_DoLookUp(
    mut start: *mut MacroDesc,
    mut name: *const ::core::ffi::c_char,
    mut lr: *mut lexret,
    mut len: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut md: *mut MacroDesc = ::core::ptr::null_mut::<MacroDesc>();
    md = start;
    while !(*md).name.is_null() && len != 0 as ::core::ffi::c_uint {
        if strcasecmp((*md).name, name) == 0
            || !(*md).alias.is_null()
                && strcasecmp((*md).alias, name) == 0
        {
            (*lr).proto = (*md).arguments as LPCSTR;
            (*lr).function =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, FARPROC>((*md).fn_);
            return if (*md).isBool != 0 {
                BOOL_FUNCTION as ::core::ffi::c_int
            } else {
                VOID_FUNCTION as ::core::ffi::c_int
            };
        }
        md = md.offset(1);
        len = len.wrapping_sub(1);
    }
    return EMPTY as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MACRO_Lookup(
    name: *const ::core::ffi::c_char,
    lr: *mut lexret,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    ret = MACRO_DoLookUp(
        &raw mut MACRO_Builtins as *mut MacroDesc,
        name,
        lr,
        (-(1 as ::core::ffi::c_int)) as ::core::ffi::c_uint,
    );
    if ret != EMPTY as ::core::ffi::c_int {
        return ret;
    }
    if !MACRO_Loaded.is_null() && {
        ret = MACRO_DoLookUp(MACRO_Loaded, name, lr, MACRO_NumLoaded);
        ret != EMPTY as ::core::ffi::c_int
    } {
        return ret;
    }
    if strcmp(
        name,
        b"hwndApp\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        let mut win: *mut WINHELP_WINDOW = ::core::ptr::null_mut::<WINHELP_WINDOW>();
        (*lr).integer = 0 as LONG;
        win = Globals.win_list;
        while !win.is_null() {
            if strcmp(
                &raw mut (*(*win).info).name as *mut ::core::ffi::c_char,
                b"main\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                (*lr).integer = (*win).hMainWnd as LONG_PTR as LONG;
                break;
            } else {
                win = (*win).next as *mut WINHELP_WINDOW;
            }
        }
        return INTEGER as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"hwndContext\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        (*lr).integer = (if !Globals.active_win.is_null() {
            (*Globals.active_win).hMainWnd as LONG_PTR
        } else {
            0 as LONG_PTR
        }) as LONG;
        return INTEGER as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"qchPath\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
        || strcmp(
            name,
            b"qError\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        || strcmp(
            name,
            b"lTopicNo\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        || strcmp(
            name,
            b"hfs\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        || strcmp(
            name,
            b"coForeground\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        || strcmp(
            name,
            b"coBackground\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
    {
        WINE_FIXME!(
            b"keyword %s not substituted in macro parsing\n\0" as *const u8
                as *const ::core::ffi::c_char,
            debugstr_a(name),
        );
        return EMPTY as ::core::ffi::c_int;
    }

    (*lr).string = name as LPCSTR;
    return IDENTIFIER as ::core::ffi::c_int;
}
