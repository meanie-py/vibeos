//! objects/ — GDI object & drawing client bodies.
//! Wave 0: linedda. Wave 1a: gdiobj. Wave 1b: brush/pen/palette/colorspace/clientobj.
//! Wave 2: coord/painting/text/arc/region (the desktop render path).
//! Wave 2b: font (selection/enumeration/metrics — the largest single file).
//! Wave 3a: path/bitmap/icm (paths, DIB/bitmap creation, color management).
//! Wave 4: printdrv (spooler), eng (engine entry) — closes gdi32 to 100% Rust.
//! Wave 3b: dc/utils/efloat(i386)/enhmfile/metafile (DC lifecycle, utils, EMF/WMF).

pub mod linedda;
pub mod gdiobj;
pub mod brush;
pub mod pen;
pub mod palette;
pub mod colorspace;
pub mod clientobj;
pub mod coord;
pub mod painting;
pub mod text;
pub mod arc;
pub mod region;
pub mod font;
pub mod path;
pub mod bitmap;
pub mod icm;
pub mod dc;
pub mod utils;
pub mod enhmfile;
pub mod metafile;
// efloat.c is i386-only (CMake `if(ARCH STREQUAL i386)`) — FLOAT↔EFLOAT bit conversion.
#[cfg(target_arch = "x86")]
pub mod efloat;
pub mod printdrv;
pub mod eng;
