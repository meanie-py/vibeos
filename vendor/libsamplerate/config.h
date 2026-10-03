/*
** Copyright (c) 2002-2021, Erik de Castro Lopo <erikd@mega-nerd.com>
** All rights reserved.
**
** This code is released under 2-clause BSD license. Please see the
** file at : https://github.com/libsndfile/libsamplerate/blob/master/COPYING
*/

/*
** ReactOS: this is the hand-maintained stand-in for the file libsamplerate's own
** build generates from config.h.cmake. Upstream deleted the hand-edited
** Win32/config.h during 0.2.0, so there is no longer an upstream file for this one
** to be a patch against -- keep it in step with config.h.cmake on every version bump.
** Everything below mirrors that template's switch list, in its order.
*/

#ifndef LIBSAMPLERATE_CONFIG_H
#define LIBSAMPLERATE_CONFIG_H

/*
** MSVC++ assumes that all floating point constants without a trailing letter 'f'
** are double precision, and src_sinc.c has about 25000 of them. The pragma below
** stops that warning from dominating compile time.
*/
#ifdef _MSC_VER
#pragma warning(disable: 4305)
#pragma warning(disable: 4244)
#endif

/* Name of package */
#define PACKAGE "libsamplerate"

/* Version number of package */
#define VERSION "0.2.2"

/* Target processor clips on negative float to int conversion. */
#define CPU_CLIPS_NEGATIVE 0

/* Target processor clips on positive float to int conversion. */
#define CPU_CLIPS_POSITIVE 0

/* Target processor is big endian. */
#define CPU_IS_BIG_ENDIAN 0

/* Target processor is little endian. All three of our targets (i386, amd64,
** arm64) are little-endian; revisit if a big-endian port ever appears. */
#define CPU_IS_LITTLE_ENDIAN 1

/* Define to 1 if you have the `alarm' function. */
#define HAVE_ALARM 0

/* Define to 1 if you have the <alsa/asoundlib.h> header file. */
#define HAVE_ALSA 0

/* Set to 1 if you have libfftw3. */
#define HAVE_FFTW3 0

/* Define if you have C99's lrint function. */
#define HAVE_LRINT 1

/* Define if you have C99's lrintf function. */
#define HAVE_LRINTF 1

/* Define if you have signal SIGALRM. */
#define HAVE_SIGALRM 0

/* Define to 1 if you have the `signal' function. */
#define HAVE_SIGNAL 0

/* Set to 1 if you have libsndfile. */
#define HAVE_SNDFILE 0

/* Define to 1 if you have the <stdbool.h> header file. */
#define HAVE_STDBOOL_H

/* Define to 1 if you have the <stdint.h> header file. */
#define HAVE_STDINT_H 1

/* Define to 1 if you have the <sys/times.h> header file. */
#define HAVE_SYS_TIMES_H 0

/* Define to 1 if you have the <unistd.h> header file. */
/* #undef HAVE_UNISTD_H */

/* Define to 1 if the compiler supports simple visibility declarations.
** Left off deliberately: this is ELF visibility, and we build PE-COFF. */
/* #undef HAVE_VISIBILITY */

/*
** define fast / balanced / best samplerate convertors.
**
** ALL THREE MUST STAY DEFINED. 0.2.0 made the sinc converters optional and the
** #ifdefs are silent: undefine one and src_new() just returns SRC_ERR_BAD_CONVERTER
** for it at runtime, with nothing failing at build time. kmixer.sys and wdmaud.drv
** both ask for SRC_SINC_FASTEST. 0.1.9 had no such switches, so defining all three
** is what keeps this drop behaviour-identical to the one it replaces.
*/
#define ENABLE_SINC_FAST_CONVERTER
#define ENABLE_SINC_MEDIUM_CONVERTER
#define ENABLE_SINC_BEST_CONVERTER

/* The size of `int', as computed by sizeof. */
#define SIZEOF_INT 4

/* The size of `long', as computed by sizeof. Windows is LLP64, so `long' is 4
** bytes on amd64 and arm64 as well as i386. */
#define SIZEOF_LONG 4

/*
** THE KERNEL-MODE exit()/printf() SHIM IS GONE, and stays gone unless the check
** below says otherwise. libsamplerate is linked into kmixer.sys, where neither
** function exists, so 0.1.9 carried `#define exit(n) __debugbreak()` and
** `#define printf DbgPrint` here. In 0.2.2 every call to both is dead: src_linear.c
** guards its "Whoops!!!!" pair with `#define SRC_DEBUG 0`, and samplerate.c's three
** printf()s are themselves commented out. Verified by linking kmixer.sys without the
** shim on all three arches.
**
** ON EVERY BUMP, re-check with:  grep -n 'printf\|exit *(' *.c
** and confirm each hit is still dead. If upstream ever un-guards one, the shim has to
** come back -- and it must then be defined AFTER <stdio.h> and <stdlib.h> are included,
** or it rewrites the CRT's own declarations instead of the call sites: `int printf
** (const char*, ...)` becomes a second, conflicting declaration of DbgPrint (which
** returns ULONG), and `void exit (int)` becomes `void __debugbreak ()`. 0.1.9 got that
** order right by accident because it included config.h last; 0.2.x includes it first.
*/

#endif /* LIBSAMPLERATE_CONFIG_H */
