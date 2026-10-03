
include_directories(BEFORE
    ${REACTOS_SOURCE_DIR}/vendor/glu/include
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/include
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess)

add_definitions(
    -DBUILD_GLU32
    -DNDEBUG=
    -DLIBRARYBUILD
    -DRESOLVE_3D_TEXTURE_SUPPORT)

# we must use our own spec file
spec2def(glu32.dll glu32.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/error.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/glue.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/mipmap.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/project.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/quad.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libutil/registry.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/dict.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/geom.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/memalloc.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/mesh.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/normal.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/priorityq.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/render.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/sweep.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/tess.c
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libtess/tessmono.c)

list(APPEND LIBNURBS_SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/bezierEval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/bezierPatch.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/bezierPatchMesh.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/glcurveval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/glinterface.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/glrenderer.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/glsurfeval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/incurveeval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/interface/insurfeval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/arc.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/arcsorter.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/arctess.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/backend.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/basiccrveval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/basicsurfeval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/bin.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/bufpool.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/cachingeval.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/ccw.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/coveandtiler.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/curve.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/curvelist.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/curvesub.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/dataTransform.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/displaylist.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/flist.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/flistsorter.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/hull.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/intersect.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/knotvector.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/mapdesc.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/mapdescv.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/maplist.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/mesher.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/monoTriangulationBackend.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/monotonizer.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/mycode.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/nurbsinterfac.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/nurbstess.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/patch.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/patchlist.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/quilt.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/reader.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/renderhints.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/slicer.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/sorter.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/splitarcs.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/subdivider.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/tobezier.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/trimline.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/trimregion.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/trimvertpool.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/uarray.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/internals/varray.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/directedLine.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/gridWrap.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/monoChain.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/monoPolyPart.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/monoTriangulation.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/partitionX.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/partitionY.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/polyUtil.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/primitiveStream.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/quicksort.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/rectBlock.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampleComp.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampleCompBot.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampleCompRight.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampleCompTop.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampleMonoPoly.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/sampledLine.cc
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/searchTree.cc)

list(APPEND PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/glu/src/libnurbs/nurbtess/polyDBG.cc
    ${SOURCE})

add_library(glu32 MODULE
    ${LIBNURBS_SOURCE}
    ${PCH_SKIP_SOURCE}
    glu32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/glu32.def)
target_link_libraries(glu32 cpprt ${PSEH_LIB})

set_module_type(glu32 win32dll)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    target_compile_options(glu32 PRIVATE -Wno-write-strings)
    target_compile_options(glu32 PRIVATE -Wno-unused-but-set-variable)
    target_compile_options(glu32 PRIVATE -Wno-uninitialized)
    target_compile_options(glu32 PRIVATE -Wno-stringop-overflow)
    # Prevent a warning when comparing 'this' against 0
    set_source_files_properties(src/libnurbs/internals/arc.cc PROPERTIES COMPILE_OPTIONS "-Wno-nonnull-compare")
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(glu32 PRIVATE -Wno-write-strings)
    target_compile_options(glu32 PRIVATE -Wno-self-assign -Wno-unused-function -Wno-microsoft-include)
    target_compile_options(glu32 PRIVATE -Wno-deprecated-register -Wno-tautological-undefined-compare)
endif()

add_importlibs(glu32 opengl32 gdi32 msvcrt kernel32 ntdll)
add_pch(glu32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET glu32 DESTINATION reactos/system32 FOR all)
