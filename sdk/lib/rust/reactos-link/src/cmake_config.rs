//! Narrow, content-stable CMake inputs for Cargo build scripts.
use std::{fs, io, path::Path};

pub fn read(build: &Path, name: &str) -> io::Result<Option<bool>> {
    if !matches!(name, "ENABLE_CCACHE" | "_WINKD_" | "GDB") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unknown CMake Boolean projection",
        ));
    }
    let cache = build.join("CMakeCache.txt");
    if !cache.try_exists()? {
        // Pure Cargo builds have no configured CMake state. Watch its arrival
        // so a subsequent configure transitions to the projected inputs.
        println!("cargo:rerun-if-changed={}", cache.display());
        return Ok(None);
    }
    let projection = build.join("CMakeFiles/reactos-link-config").join(name);
    println!("cargo:rerun-if-changed={}", projection.display());
    let value = fs::read_to_string(&projection).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "cannot read {}: {e}; reconfigure the CMake build to generate reactos-link inputs",
                projection.display()
            ),
        )
    })?;
    match value.as_str() {
        "0\n" => Ok(Some(false)),
        "1\n" => Ok(Some(true)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid Boolean projection {}; reconfigure the CMake build",
                projection.display()
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_inputs_refuse_missing_or_corrupt_projections() {
        let root = std::env::temp_dir().join(format!("reactos-link-config-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let _cleanup = Cleanup(root.clone());
        assert_eq!(read(&root, "ENABLE_CCACHE").unwrap(), None);
        assert_eq!(
            read(&root, "../unknown").unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        fs::write(root.join("CMakeCache.txt"), "ENABLE_CCACHE:BOOL=ON\n").unwrap();
        let error = read(&root, "ENABLE_CCACHE").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.to_string().contains("reconfigure"));
        let input = root.join("CMakeFiles/reactos-link-config/ENABLE_CCACHE");
        fs::create_dir_all(input.parent().unwrap()).unwrap();
        for (value, expected) in [("0\n", false), ("1\n", true)] {
            fs::write(&input, value).unwrap();
            assert_eq!(read(&root, "ENABLE_CCACHE").unwrap(), Some(expected));
        }
        fs::write(&input, "ON\n").unwrap();
        assert_eq!(
            read(&root, "ENABLE_CCACHE").unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
