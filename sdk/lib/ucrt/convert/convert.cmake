
list(APPEND UCRT_CONVERT_SOURCES
    convert/atof.cpp
    convert/atoldbl.cpp
    convert/c16rtomb.cpp
    convert/c32rtomb.cpp
    convert/cfout.cpp
    convert/common_utf8.cpp
    convert/cvt.cpp
    convert/fcvt.cpp
    convert/fp_flags.cpp
    convert/gcvt.cpp
    convert/ismbstr.cpp
    convert/mblen.cpp
    convert/mbrtoc16.cpp
    convert/mbrtoc32.cpp
    convert/mbrtowc.cpp
    convert/mbstowcs.cpp
    convert/mbtowc.cpp
    convert/strtod.cpp
    convert/swab.cpp
    convert/wcrtomb.cpp
    convert/wcstombs.cpp
    convert/wctomb.cpp
    convert/wctrans.cpp
    convert/wctype.cpp
    convert/_fptostr.cpp
    convert/_mbslen.cpp
)

# All multibyte string functions require the _MBCS macro to be defined
set_source_files_properties(convert/ismbstr.cpp PROPERTIES COMPILE_DEFINITIONS _MBCS)
