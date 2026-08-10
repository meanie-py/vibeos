"""Project plugins — the per-tree data and policy the translation engine runs against.

A "project" owns what is true of ONE codebase: its source roots, its translation targets,
how a source path maps to a target, and how its Rust crates and FFI substrate are named.
The engine (everything in `rustconv/` outside this package) owns translation itself and
must stay free of any one project's vocabulary.

Today there is exactly one project, `reactos`, and it is reached through `config`'s lazy
re-exports rather than a loader. That is deliberate: a plugin interface derived from a
single consumer is a guess. The interface gets defined once a second project exists to
argue with it.
"""
