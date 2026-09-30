
list(APPEND UCRT_CONVERT_SOURCES
    convert/cfout.cpp
    convert/cvt.cpp
    convert/fp_flags.cpp
    convert/wctype.cpp
    convert/_fptostr.cpp
)

# All multibyte string functions require the _MBCS macro to be defined
