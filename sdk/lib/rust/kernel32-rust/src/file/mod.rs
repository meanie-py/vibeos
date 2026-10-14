//! client/file/ — the kernel32 file-I/O API family, ported to Rust (rustconv c2rust).
pub mod backup;
pub mod cnotify;
pub mod copy;
pub mod create;
pub mod delete;
pub mod deviceio;
pub mod dir;
pub mod disk;
pub mod fileinfo;
pub mod filemap;
pub mod filename;
pub mod find;
pub mod hardlink;
pub mod iocompl;
pub mod io_ex;
pub mod lfile;
pub mod lock;
pub mod mailslot;
pub mod mntpoint;
#[path = "move.rs"]
pub mod move_;
pub mod npipe;
pub mod rw;
pub mod volume;
// DEFERRED (IOCTL_TAPE_* type hierarchy not yet defined; .rs preserved, tape.c stays C):
pub mod tape;
