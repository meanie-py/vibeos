//! wine/ — wine-derived pure-C bodies of shell32.
//! Wave 0: shellreg (SHReg* registry access wrappers + SHCreateSessionKey).
//! Wave 1: shpolicy (system-policy table + SHRestricted), shellstring (StrRet*/CheckEscapes*),
//! clipboard (Render* shell clipboard-format builders).
//! Wave 2: classes (HCR_* HKEY_CLASSES_ROOT helpers), shellole (OLE/COM glue + drag-drop),
//! control (Control Panel applet loader).
//! Wave 3: shell32_main (SHGetFileInfo/ExtractIcon/CommandLineToArgvW/ShellAbout),
//! shlmenu (FileMenu_* PIDL owner-draw menus + Shell_MergeMenus).

pub mod shellreg;
pub mod shpolicy;
pub mod shellstring;
pub mod clipboard;
pub mod classes;
pub mod shellole;
pub mod control;
pub mod shell32_main;
pub mod shlmenu;
// Wave 4: the PIDL core. pidl (foundation) uses crate::shellcom.
pub mod pidl;
pub mod shellpath;
pub mod shellord;
pub mod knownfolders;
// shcore DPI/scaling exports (api-ms-win-shcore-* routes to shell32; this tree builds no shcore).
pub mod shcore_dpi;
