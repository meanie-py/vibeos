Stub C++ standard library headers for the C++ bindgen ISLAND pass only (gen_driver_types.sh,
`-nostdinc++ -isystem` this dir). The island must merely PARSE a module that includes
`<string>`/`<vector>`/`<map>`: its classes holding STL members are laid out by cppapp on
`atl_rust::stl` (rustconv/cpp_core.py maps the spellings), never by bindgen. llvm-mingw's real
libc++ does not parse in bindgen's include order (UINT64_C, _sscanf_l). Add a name here only
when a module's island needs it to parse.
