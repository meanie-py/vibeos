# Rewrite windmc's `<id> MESSAGETABLE "<file>.bin"` as `<id> 11 "<file>.bin"` (RT_MESSAGETABLE).
# llvm-rc has no MESSAGETABLE keyword: it takes the word as a user-defined type NAME and emits a
# string-named resource type, which RtlFindMessage (looking up integer type 11) never finds.
# Every message table in the OS was unreachable that way -- FormatMessage(FROM_SYSTEM) failed
# with ERROR_RESOURCE_TYPE_NOT_FOUND and notepad's error box read just "5".
# Usage: cmake -DRC_FILE=<generated .rc> -P mc_rc_numeric_type.cmake
file(READ "${RC_FILE}" _rc)
string(REGEX REPLACE "(\n[0-9]+)[ \t]+MESSAGETABLE[ \t]" "\\1 11 " _fixed "\n${_rc}")
string(SUBSTRING "${_fixed}" 1 -1 _fixed)
if(NOT _fixed STREQUAL _rc)
    file(WRITE "${RC_FILE}" "${_fixed}")
endif()
string(REGEX MATCH "MESSAGETABLE[ \t]+\"" _left "${_fixed}")
if(_left)
    message(FATAL_ERROR "${RC_FILE}: a MESSAGETABLE statement survived the rewrite")
endif()
