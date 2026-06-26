//! objects/ — GDI object & drawing client bodies.
//! Wave 0: linedda. Wave 1a: gdiobj. Wave 1b: brush/pen/palette/colorspace/clientobj.
//! Wave 2: coord/painting/text/arc/region (the desktop render path).
//! Wave 2b: font (selection/enumeration/metrics — the largest single file).

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
