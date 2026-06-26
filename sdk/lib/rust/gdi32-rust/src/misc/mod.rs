//! misc/ — gdi32 client substrate. Wave 1a: misc.rs (handle-table primitives +
//! GdiHandleTable/CurrentProcessId/GDI_BatchLimit globals, escapes, batch-limit, lang-pack).
//! Wave 3b: heap (private heap), historic (legacy APIs), wingl (OpenGL/pixel-format).

pub mod misc;
pub mod heap;
pub mod historic;
pub mod wingl;
