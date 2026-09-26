/*
 * ReactOS: the config.h libxml2's build is expected to generate. On Windows
 * upstream's own CMakeLists does exactly this —
 *
 *     configure_file(include/win32config.h config.h COPYONLY)
 *
 * — so this file is that copy, expressed as an include, plus the two headers our
 * CRT wants pulled in early.
 *
 * WHY IT LIVES HERE. libxml.h does `#include "config.h"`, and a quoted include
 * resolves against the INCLUDING FILE'S directory first, which is this one. So the
 * file satisfies libxml.h without putting the dangerously generic name `config.h`
 * on any include path — sdk/include/wine ships one too. Until 2026-09-20 we patched
 * libxml.h to say "win32config.h" instead and patched win32config.h to add the two
 * includes; both files are now PRISTINE and this is the only thing standing in for
 * the build system.
 */

#ifndef REACTOS_LIBXML2_CONFIG_H
#define REACTOS_LIBXML2_CONFIG_H

#include "include/win32config.h"

#include <io.h>
#include <direct.h>

#endif /* REACTOS_LIBXML2_CONFIG_H */
