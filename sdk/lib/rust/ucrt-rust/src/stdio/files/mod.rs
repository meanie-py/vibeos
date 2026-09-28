//! `stdio/{openfile,fopen,freopen,fdopen,fclose,closeall,fflush,fread,fwrite,
//! fseek,ftell,fgetpos,fsetpos,rewind,setbuf,setvbuf,clearerr,feoferr,fileno,
//! setmaxf,ncommode,printf_count_output}.cpp`, ported.
//!
//! This is one of three pieces over the stdio stream core (`super`, i.e.
//! `stdio/mod.rs` = `_file.cpp`/`stream.cpp`); see its module doc for the
//! shared design and `docs`/the batch brief for the piece-ownership table.
//! This piece owns: opening a stream over a name or an existing lowio
//! handle, closing it, flushing, buffered read/write, seeking/telling,
//! `fgetpos`/`fsetpos`/`rewind`, `setbuf`/`setvbuf`, and the small
//! accessors (`clearerr`/`feof`/`ferror`/`_fileno`/`_setmaxstdio`/
//! `_commode`/the `%n`-enable cookie).
//!
//! Submodules, one (or a small related group) of C source files each:
//! * [`parse_mode`] -- the header-inline `__acrt_stdio_parse_mode` template,
//!   ported once, generic over character width.
//! * [`raw_lowio`] -- lowio accessors `stdio_layout.rs` doesn't expose
//!   (`_startpos`, unchecked `_textmode`, `_utf8translations`, a writable
//!   `_osfile_safe`) plus the still-C, PTD-propagating lowio entry points
//!   this piece calls (`_lseek*_internal`, `_write_internal`,
//!   `_close_internal`, `ReadFile`).
//! * [`open`] -- `openfile.cpp`, `fopen.cpp`, `freopen.cpp`, `fdopen.cpp`.
//! * [`close`] -- `fclose.cpp`, `closeall.cpp`.
//! * [`flush`] -- `fflush.cpp` (also the cross-piece `__acrt_stdio_flush_nolock`).
//! * [`readwrite`] -- `fread.cpp`, `fwrite.cpp`.
//! * [`seek`] -- `fseek.cpp`, `ftell.cpp`, `fgetpos.cpp`, `fsetpos.cpp`,
//!   `rewind.cpp`.
//! * [`buf`] -- `setbuf.cpp`, `setvbuf.cpp`.
//! * [`misc`] -- `clearerr.cpp`, `feoferr.cpp`, `fileno.cpp`, `setmaxf.cpp`,
//!   `ncommode.cpp`, `printf_count_output.cpp`.
//!
//! Cross-piece calls (declared `extern "C"` in whichever submodule needs
//! them, transcribed from the header prototype -- see AGENTS.md and the
//! batch brief): `__acrt_stdio_refill_and_read_narrow_nolock`,
//! `__acrt_stdio_flush_and_write_narrow_nolock`,
//! `__acrt_stdio_free_buffer_nolock`,
//! `__acrt_stdio_begin_temporary_buffering_nolock`,
//! `__acrt_stdio_end_temporary_buffering_nolock` -- owned by the `bufio`
//! piece. `_sopen_s`/`_wsopen_s`, `_lseeki64_nolock`, `_lseeki64_internal`,
//! `_lseeki64_nolock_internal`, `_lseek_internal`, `_close_internal`,
//! `_write_internal`, `_read_nolock`, `_lookuptrailbytes` -- still C
//! elsewhere in the tree (lowio, a different batch).
//!
//! `FILE*` is represented at every boundary here as `*mut c_void`, matching
//! `stdio/mod.rs`'s `Stream::from_file`/`public_stream` and `output.rs`'s own
//! stream parameters -- never a typed `*mut StreamData`, so the ABI is
//! identical to the C `FILE*` regardless of which side of a call is Rust.

mod buf;
mod close;
mod flush;
mod misc;
mod open;
mod parse_mode;
mod raw_lowio;
mod readwrite;
mod seek;
