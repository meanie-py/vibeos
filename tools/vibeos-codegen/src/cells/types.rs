use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
pub const ARCHES: [&str; 3] = ["i386", "amd64", "arm64"];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub arch: String,
    pub output: String,
    pub kind: String,
    pub entry: String,
    pub subsystem: Option<String>,
    pub image_base: Option<u64>,
    pub stack: Option<u64>,
    pub inputs: Vec<String>,
    pub libs: Vec<String>,
    #[serde(default)]
    pub require_defined: Vec<String>,
    /// The link sets a PE OS version (set_module_type_toolchain's 5.01); without one lld
    /// writes its default (6.0).
    #[serde(default)]
    pub os_versioned: bool,
}
impl Default for Link {
    fn default() -> Self {
        Self {
            arch: Default::default(),
            output: Default::default(),
            kind: Default::default(),
            entry: Default::default(),
            subsystem: Default::default(),
            image_base: Default::default(),
            stack: Default::default(),
            inputs: Default::default(),
            libs: Default::default(),
            require_defined: Default::default(),
            os_versioned: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub target: String,
    pub cmake_dir: PathBuf,
    pub output_name: String,
    pub module_kind: String,
    pub entries: BTreeMap<String, String>,
    pub subsystem: Option<String>,
    pub image_base: BTreeMap<String, Option<u64>>,
    pub stack: Option<u64>,
    pub body_crate: String,
    pub body_features: Vec<String>,
    pub body_defaults: bool,
    pub import_libs: Vec<(String, String, String)>,
    pub msvcrtex: Option<String>,
    pub static_libs: BTreeMap<String, Vec<String>>,
    pub spec: Option<(String, String)>,
    pub spec_stubs: bool,
    pub c_shims: Vec<String>,
    pub rc: Option<String>,
    /// Further `.rc` files the target links beside `rc` (wined3d: resource.rc's HLSL RCDATA
    /// next to version.rc). cmake links one windres object per `.rc`; so does the cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_rc: Vec<String>,
    pub rc_defines: Vec<String>,
    pub destinations: Vec<String>,
    pub name_on_cd: Option<String>,
    #[serde(default = "default_panics")]
    pub panics: BTreeMap<String, bool>,
    #[serde(default)]
    pub body_var: String,
    #[serde(default)]
    pub late_libs: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub delay_import_libs: Vec<(String, String, String)>,
    #[serde(default)]
    pub generated: Vec<String>,
    #[serde(default)]
    pub foreign: Vec<String>,
    #[serde(default)]
    pub import_order: Vec<(String, bool)>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub body_rustflags: BTreeMap<String, String>,
    #[serde(default)]
    pub body_env_block: String,
    #[serde(default)]
    pub arch_destinations: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub link_order: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub no_cab: bool,
    #[serde(default)]
    pub kernel_kind: String,
    #[serde(default = "default_empty_object")]
    pub custom_body: serde_json::Value,
    #[serde(default)]
    pub arch_shims: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub subsystems: BTreeMap<String, String>,
    #[serde(default)]
    pub extra_cd: Vec<String>,
    #[serde(default)]
    pub rc_flags: BTreeMap<String, RcFlags>,
    #[serde(default)]
    pub require_defined: BTreeMap<String, Vec<String>>,
    /// The arches cmake builds the module on, when an `if(ARCH ...)` gate in its own
    /// CMakeLists leaves some out (wow64.dll: amd64 and arm64). Empty means all of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arches: Vec<String>,
    /// Files the body's build waits on (rust_staticlib_variant's DEPENDS), passed on as
    /// add_cargo_module EXTRA_DEPENDS.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_depends: Vec<String>,
    /// A body-less module with no C of its own (a forwarder like rsabase, a resource
    /// DLL like shdoclc, comcat's .def over a static lib): its link takes nothing from
    /// rustc's bin, so the PE has no `.text` or ntdll import cmake's did not
    /// (reactos-link `Linker::shell_less`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shell_less: bool,
    /// A module with no set_module_type (genincdata, a data DLL native-geninc reads): no
    /// link of it sets set_module_type_toolchain's 5.01 image/OS versions, so lld's
    /// defaults (image 0.0, OS 6.0) stand.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_module_type: bool,
    /// (spec, dll) whose spec2def stubs a linked static library carries in its archive
    /// (fusion_common holds fusion.spec's): reactos-link `Linker::spec_stubs_from`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stub_specs: Vec<(String, String)>,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            target: Default::default(),
            cmake_dir: Default::default(),
            output_name: Default::default(),
            module_kind: Default::default(),
            entries: Default::default(),
            subsystem: Default::default(),
            image_base: Default::default(),
            stack: Default::default(),
            body_crate: Default::default(),
            body_features: Default::default(),
            body_defaults: true,
            import_libs: Default::default(),
            msvcrtex: Default::default(),
            static_libs: Default::default(),
            spec: Default::default(),
            spec_stubs: Default::default(),
            c_shims: Default::default(),
            rc: Default::default(),
            extra_rc: Default::default(),
            rc_defines: Default::default(),
            destinations: Default::default(),
            name_on_cd: Default::default(),
            panics: default_panics(),
            body_var: Default::default(),
            late_libs: Default::default(),
            delay_import_libs: Default::default(),
            generated: Default::default(),
            foreign: Default::default(),
            import_order: Default::default(),
            notes: Default::default(),
            body_rustflags: Default::default(),
            body_env_block: Default::default(),
            arch_destinations: Default::default(),
            link_order: Default::default(),
            no_cab: Default::default(),
            kernel_kind: Default::default(),
            custom_body: default_empty_object(),
            arch_shims: Default::default(),
            subsystems: Default::default(),
            extra_cd: Default::default(),
            rc_flags: Default::default(),
            require_defined: Default::default(),
            arches: Default::default(),
            body_depends: Default::default(),
            shell_less: false,
            no_module_type: false,
            stub_specs: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RcParts {
    pub windres: String,
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub defs: Vec<String>,
    pub defines: Vec<String>,
    pub src: String,
}
impl Default for RcParts {
    fn default() -> Self {
        Self {
            windres: Default::default(),
            includes: Default::default(),
            flags: Default::default(),
            defs: Default::default(),
            defines: Default::default(),
            src: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RcTemplate {
    pub windres: String,
    pub std: Vec<String>,
    pub flags: Vec<String>,
    pub defs: Vec<String>,
    pub defines: Vec<String>,
}
impl Default for RcTemplate {
    fn default() -> Self {
        Self {
            windres: Default::default(),
            std: Default::default(),
            flags: Default::default(),
            defs: Default::default(),
            defines: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RcFlags {
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub defines: Vec<String>,
}
impl Default for RcFlags {
    fn default() -> Self {
        Self {
            includes: Default::default(),
            flags: Default::default(),
            defines: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CParts {
    pub cc: String,
    pub target: String,
    pub defines: Vec<String>,
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub obj: String,
    pub src: String,
}
impl Default for CParts {
    fn default() -> Self {
        Self {
            cc: Default::default(),
            target: Default::default(),
            defines: Default::default(),
            includes: Default::default(),
            flags: Default::default(),
            obj: Default::default(),
            src: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CTemplate {
    pub cc: String,
    pub target: String,
    pub std: Vec<String>,
    pub core: Vec<String>,
    pub defines: Vec<String>,
}
impl Default for CTemplate {
    fn default() -> Self {
        Self {
            cc: Default::default(),
            target: Default::default(),
            std: Default::default(),
            core: Default::default(),
            defines: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CFlags {
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub defines: Vec<String>,
}
impl Default for CFlags {
    fn default() -> Self {
        Self {
            includes: Default::default(),
            flags: Default::default(),
            defines: Default::default(),
        }
    }
}
pub fn default_panics() -> BTreeMap<String, bool> {
    ARCHES.into_iter().map(|a| (a.to_owned(), true)).collect()
}
pub fn default_empty_object() -> serde_json::Value {
    serde_json::json!({})
}
impl Cell {
    /// The arches this cell links on, in `ARCHES` order.
    pub fn arches(&self) -> Vec<&'static str> {
        ARCHES
            .into_iter()
            .filter(|a| self.arches.is_empty() || self.arches.iter().any(|x| x == a))
            .collect()
    }
    pub fn crate_name(&self) -> String {
        if self
            .target
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
        {
            format!(
                "{}-cargo",
                self.body_crate
                    .strip_suffix("-rust")
                    .unwrap_or(&self.body_crate)
            )
        } else {
            // A crate name cannot hold a `.`: activeds.tlb is activeds_tlb-cargo.
            format!("{}-cargo", self.target.replace('.', "_"))
        }
    }
}
