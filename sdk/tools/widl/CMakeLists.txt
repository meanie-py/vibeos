

FLEX_TARGET(p_scanner parser.l ${CMAKE_CURRENT_BINARY_DIR}/parser.yy.c)
BISON_TARGET(p_parser parser.y ${CMAKE_CURRENT_BINARY_DIR}/parser.tab.c)
ADD_FLEX_BISON_DEPENDENCY(p_scanner p_parser)

list(APPEND SOURCE
    attribute.c
    client.c
    expr.c
    hash.c
    header.c
    proxy.c
    register.c
    server.c
    typegen.c
    typelib.c
    typetree.c
    utils.c
    widl.c
    write_msft.c
    write_sltg.c
    ${FLEX_p_scanner_OUTPUTS}
    ${BISON_p_parser_OUTPUTS}
    ../port/getopt.c
    ../port/getopt1.c
    ../port/mkstemps.c)

add_definitions(-DINT16=SHORT -DLANG_SCOTTISH_GAELIC=0x91 -DINCLUDEDIR="unused" -DLIBDIR="unused")
include_directories(${REACTOS_SOURCE_DIR}/sdk/tools/wpp/)
add_host_tool(widl ${SOURCE})
target_link_libraries(widl PRIVATE host_includes wpphost)
