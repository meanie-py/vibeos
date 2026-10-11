// PLACEHOLDER — the real bindgen island has not been generated yet.
// An empty island is a CORRECTNESS bug, not a missing convenience: constants absent from an
// island become silent catch-all match arms (field manual §9.112). So this fails the build
// loudly instead of compiling as an empty module.
compile_error!(
    "types_amd64.rs is a placeholder: run tools/rustconv/gen_driver_types.sh ftp \
     (add the derived profile first) to generate the real island."
);
