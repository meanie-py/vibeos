//! winnls/string/ — the NLS / locale / string family of kernel32.
//! Each .c → one module here (rustconv-translated, Sonnet+c2rust). locale.c first.
pub mod nls_subst;
pub mod locale;
pub mod nls;
pub mod lcformat;
pub mod casemap;
pub mod collation;
pub mod digitmap;
pub mod fold;
pub mod lstring;
pub mod sortkey;
pub mod japanese;
pub mod format_msg;
