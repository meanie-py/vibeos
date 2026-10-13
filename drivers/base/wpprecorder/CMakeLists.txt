# ============================================================================
# wpprecorder.sys — cargo-driven link (sdk/lib/rust/wpprecorder-cargo)
# ============================================================================
# Not in the 2003 tree: the WPP in-flight recorder Windows 10 drivers import
# (128 of 478 vendor packages). The body (wpprecorder-rust) records to nowhere,
# which is correct with no trace consumer; see wpprecorder.spec for scope.
# 64-bit only (drivers/base/CMakeLists.txt).
spec2def(wpprecorder.sys wpprecorder.spec ADD_IMPORTLIB)
add_cargo_module(wpprecorder wpprecorder-cargo wpprecorder.sys KIND kerneldll
    BODY_CRATE wpprecorder-rust DESTINATION reactos/system32/drivers)
