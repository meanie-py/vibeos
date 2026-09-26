/* The MS CRT names version.c expects its <string.h> to declare. Included BEFORE
   version.c's own `#define NONAMELESSUNION`, so it must not pull in any Windows header;
   under the host profile's -fshort-wchar, wchar_t is the 16-bit WCHAR. Defined by
   version_env.c, which includes this header so the two cannot disagree. */
#pragma once
#include <stddef.h>
size_t wcslen(const wchar_t *s);
int _wcsnicmp(const wchar_t *a, const wchar_t *b, size_t n);
int _wcsicmp(const wchar_t *a, const wchar_t *b);
int wcsicmp(const wchar_t *a, const wchar_t *b);
int _strnicmp(const char *a, const char *b, size_t n);
