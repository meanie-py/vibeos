//! wine/ — wine-derived pure-C bodies of shell32.
//! Wave 0: shellreg (SHReg* registry access wrappers + SHCreateSessionKey).
//! Wave 1: shpolicy (system-policy table + SHRestricted), shellstring (StrRet*/CheckEscapes*),
//! clipboard (Render* shell clipboard-format builders).
//! Wave 2: classes (HCR_* HKEY_CLASSES_ROOT helpers), shellole (OLE/COM glue + drag-drop),
//! control (Control Panel applet loader).

pub mod shellreg;
pub mod shpolicy;
pub mod shellstring;
pub mod clipboard;
pub mod classes;
pub mod shellole;
pub mod control;
