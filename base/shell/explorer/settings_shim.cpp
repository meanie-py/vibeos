/*
 * ReactOS Explorer — Rust cohabitation shim for settings.cpp
 *
 * settings.cpp's bodies (TaskbarSettings::Save/Load) and the g_TaskbarSettings
 * global live in Rust now (sdk/lib/rust/explorer-rust/src/explorer/settings.rs).
 * The global is a #[no_mangle] Rust static (Itanium ABI leaves namespace-scope
 * globals unmangled, so the `extern TaskbarSettings g_TaskbarSettings;` in
 * precomp.h binds to it directly). But Save()/Load() are C++ *methods* —
 * still-C++ callers (explorer.cpp, traywnd.cpp) invoke the mangled
 * TaskbarSettings::Save / ::Load, which Rust's flat #[no_mangle]
 * TaskbarSettings_Save(this) cannot satisfy. This shim provides the mangled
 * method entry points, forwarding to the Rust workers (same pattern as
 * util_vararg.cpp / the wine-inline shims).
 */

#include "precomp.h"

extern "C" BOOL TaskbarSettings_Save(TaskbarSettings *self);
extern "C" BOOL TaskbarSettings_Load(TaskbarSettings *self);

BOOL TaskbarSettings::Save()
{
    return TaskbarSettings_Save(this);
}

BOOL TaskbarSettings::Load()
{
    return TaskbarSettings_Load(this);
}
