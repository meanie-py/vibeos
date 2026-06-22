//! controls/ — the standard window-class control bodies (button/static/edit/combo/
//! listbox/scrollbar). Wave-2 integrated. Each provides its `{Name}WndProc{A,W}` +
//! `{Name}WndProc_common` symbols (referenced cross-TU by the surviving C
//! `controls/regcontrol.c`), so those must be `#[no_mangle]` with CALLBACK (stdcall on
//! i386) decoration.

pub mod shared;

pub mod button;
pub mod combo;
pub mod edit;
pub mod listbox;
pub mod scrollbar;
pub mod static_;

// Wave 4 — secondary controls.
pub mod appswitch;
pub mod ghost;
pub mod icontitle;
pub mod regcontrol;
