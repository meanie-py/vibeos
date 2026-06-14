//! winnls/string/ — the NLS / locale / string family of kernel32.
//! Each .c → one module here (rustconv-translated, Sonnet+c2rust). locale.c first.
pub mod nls_subst;
pub mod locale;
pub mod nls;
pub mod lcformat;
pub mod casemap;
pub mod collation;
