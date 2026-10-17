//! Compile genuine mmixer C functions for the host and extract the shipping Rust ones.
use quote::ToTokens;
use std::{env, fs, path::PathBuf};

/// One graded unit: the C file, the span of it to compile (start of the first function, start
/// of the function after the last), the fixture header, the C names renamed `ref_`, the Rust
/// file and the shipping functions taken from it.
struct Unit {
    name: &'static str,
    c_file: &'static str,
    start: &'static str,
    next: &'static str,
    header: &'static str,
    c_names: &'static [&'static str],
    rust_file: &'static str,
    rust_names: &'static [&'static str],
}

const UNITS: [Unit; 2] = [
    Unit {
        name: "volume",
        c_file: "sdk/lib/drivers/sound/mmixer/sup.c",
        start: "MIXER_STATUS\nMMixerSetGetVolumeControlDetails(",
        next: "LPMIXER_DATA\nMMixerGetDataByDeviceId(",
        header: "volume.h",
        c_names: &["MMixerSetGetVolumeControlDetails"],
        rust_file: "sdk/lib/rust/mmixer-rust/src/mmixer/sup.rs",
        rust_names: &["MMixerSetGetVolumeControlDetails"],
    },
    Unit {
        name: "topology",
        c_file: "sdk/lib/drivers/sound/mmixer/topology.c",
        start: "VOID\nMMixerAddPinIndexToArray(",
        next: "MIXER_STATUS\nMMixerGetAllUpOrDownstreamNodesFromNodeIndex(",
        header: "topology.h",
        c_names: &[
            "MMixerAddPinIndexToArray",
            "MMixerGetUpOrDownStreamPins",
            "MMixerGetUpOrDownstreamNodes",
            "MMixerGetNodeIndexFromGuid",
            "MMixerGetAllUpOrDownstreamPinsFromNodeIndex",
        ],
        rust_file: "sdk/lib/rust/mmixer-rust/src/mmixer/topology.rs",
        rust_names: &[
            "MMixerAddPinIndexToArray",
            "MMixerGetUpOrDownStreamPins",
            "MMixerGetUpOrDownstreamNodes",
        ],
    },
];

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for unit in UNITS {
        let (c_path, rust_path) = (root.join(unit.c_file), root.join(unit.rust_file));
        for path in [&c_path, &rust_path] {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        println!("cargo:rerun-if-changed={}", unit.header);
        let c = fs::read_to_string(&c_path).unwrap();
        // Fail when the source boundaries move; never substitute a reimplementation.
        let tail = &c[c.find(unit.start).expect("original C function start")..];
        let body = &tail[..tail.find(unit.next).expect("next C function")];
        let reference = out.join(format!("{}.c", unit.name));
        fs::write(&reference, format!("#include \"{}\"\n{body}", unit.header)).unwrap();
        let mut build = cc::Build::new();
        build.file(&reference).include(".").flag_if_supported("-std=c11");
        for name in unit.c_names {
            build.define(name, format!("ref_{name}").as_str());
        }
        build.compile(&format!("mmixer_{}_reference", unit.name));

        let tree = syn::parse_file(&fs::read_to_string(&rust_path).unwrap()).unwrap();
        let mut rust = String::new();
        let mut found = 0;
        for item in tree.items {
            if let syn::Item::Fn(mut function) = item {
                if unit.rust_names.contains(&function.sig.ident.to_string().as_str()) {
                    function.attrs.clear(); // no process-global exported test symbol
                    rust.push_str(&function.into_token_stream().to_string());
                    found += 1;
                }
            }
        }
        assert_eq!(found, unit.rust_names.len(), "{}: shipping functions must exist", unit.name);
        fs::write(out.join(format!("{}_shipping.rs", unit.name)), rust).unwrap();
    }
}
