//! d3dwine — Rust translation of dll/directx/wine/wined3d.
//! staticlib → libd3dwine_rust.a, cohab-linked into the cmake-built d3dwine
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 29 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh d3dwine)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod d3dwine {
    pub mod adapter_gl;
    pub mod adapter_vk;
    pub mod buffer;
    pub mod context;
    pub mod context_gl;
    pub mod context_vk;
    pub mod cs;
    pub mod device;
    pub mod directx;
    pub mod ffp_gl;
    pub mod ffp_hlsl;
    pub mod gl_compat;
    pub mod glsl_shader;
    pub mod palette;
    pub mod query;
    pub mod resource;
    pub mod sampler;
    pub mod shader;
    pub mod shader_sm1;
    pub mod shader_sm4;
    pub mod shader_spirv;
    pub mod stateblock;
    pub mod surface;
    pub mod swapchain;
    pub mod texture;
    pub mod utils;
    pub mod vertexdeclaration;
    pub mod view;
    pub mod wined3d_main;
}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
