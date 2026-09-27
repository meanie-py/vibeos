# GDB preamble sourced by every inspect.sh invocation.
#
# Sets options for batch use: no pager (would block), no confirmation
# prompts, structured output. Architecture is set by inspect.sh before
# sourcing this — leaving it here would trigger before remote connect.

set pagination off
# The helpers are written in C expressions. With the cargo-linked (Rust) kernel, gdb's
# "auto" language becomes Rust once symbols load, and then `(unsigned long)&Sym` does not
# parse and 0xfffffa80... literals overflow ("integers of more than 8 bytes"). An explicit
# language is never switched by later symbol loads.
set language c
set confirm off
set print pretty on
set print elements 256
set print frame-arguments scalars
set print frame-info short-location
set logging redirect on
# Stale breakpoints from a prior run-until.sh would alter execution at unexpected
# points if we're attaching to a live kernel. Defense-in-depth: clear them now.
delete breakpoints
