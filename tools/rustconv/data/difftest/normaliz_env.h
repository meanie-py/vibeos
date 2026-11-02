/* The msvcrt name normaliz's C expects its headers to declare: ReactOS's wine/unicode.h
   spells tolowerW as towlower, and msvcrt's wint_t is 16-bit. Defined by normaliz_env.c,
   which includes this header so the two cannot disagree. */
#pragma once
unsigned short towlower(unsigned short c);
