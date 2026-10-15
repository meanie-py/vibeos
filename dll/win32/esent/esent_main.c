/*
 * ESENT (Extensible Storage Engine) entry points that are not implemented.
 *
 * This tree has no ESE database engine. Each function reports JET_errFeatureNotAvailable
 * (-1001) so a caller sees "the engine is absent", and never a fabricated success. Handle and
 * output parameters are cleared first, so a caller that ignores the error reads only zeros.
 * JetTerm and JetCloseTable return JET_errInvalidParameter instead: no instance or table handle
 * can ever have been created here, so there is nothing to release and the call is invalid.
 */

#include <windef.h>
#include <string.h>

#ifndef JET_API
#define JET_API __stdcall
#endif

typedef long JET_ERR;
typedef unsigned long JET_GRBIT;
typedef unsigned long JET_DBID;
typedef unsigned long JET_COLUMNID;
typedef ULONG_PTR JET_API_PTR;
typedef JET_API_PTR JET_INSTANCE;
typedef JET_API_PTR JET_SESID;
typedef JET_API_PTR JET_TABLEID;
typedef const WCHAR *JET_PCWSTR;
struct _JET_RETINFO;

#define JET_errSuccess              0
#define JET_errFeatureNotAvailable  -1001
#define JET_errInvalidParameter     -1003

static JET_ERR JET_API jet_not_available(void)
{
    return JET_errFeatureNotAvailable;
}

JET_ERR JET_API JetAttachDatabase2W(JET_SESID sesid, const WCHAR *szFilename, const unsigned long cpgDatabaseSizeMax, JET_GRBIT grbit)
{
    return jet_not_available();
}

JET_ERR JET_API JetBeginSessionW(JET_INSTANCE instance, JET_SESID *psesid, JET_PCWSTR szUserName, JET_PCWSTR szPassword)
{
    if (psesid)
        *psesid = 0;
    return jet_not_available();
}

JET_ERR JET_API JetCloseTable(JET_SESID sesid, JET_TABLEID tableid)
{
    return JET_errInvalidParameter;
}

JET_ERR JET_API JetCreateInstanceW(JET_INSTANCE *pinstance, const WCHAR *szInstanceName)
{
    if (pinstance)
        *pinstance = 0;
    return jet_not_available();
}

JET_ERR JET_API JetGetTableColumnInfoW(JET_SESID sesid, JET_TABLEID tableid, const WCHAR *szColumnName, void *pvResult, unsigned long cbMax, unsigned long InfoLevel)
{
    if (pvResult && cbMax)
        memset(pvResult, 0, cbMax);
    return jet_not_available();
}

JET_ERR JET_API JetInit(JET_INSTANCE *pinstance)
{
    if (pinstance)
        *pinstance = 0;
    return jet_not_available();
}

JET_ERR JET_API JetMove(JET_SESID sesid, JET_TABLEID tableid, long cRow, JET_GRBIT grbit)
{
    return jet_not_available();
}

JET_ERR JET_API JetOpenDatabaseW(JET_SESID sesid, const WCHAR *szFilename, const WCHAR *szConnect, JET_DBID *pdbid, JET_GRBIT grbit)
{
    if (pdbid)
        *pdbid = 0;
    return jet_not_available();
}

JET_ERR JET_API JetOpenTableW(JET_SESID sesid, JET_DBID dbid, const WCHAR *szTableName, const void *pvParameters, unsigned long cbParameters, JET_GRBIT grbit, JET_TABLEID *ptableid)
{
    if (ptableid)
        *ptableid = 0;
    return jet_not_available();
}

JET_ERR JET_API JetRetrieveColumn(JET_SESID sesid, JET_TABLEID tableid, JET_COLUMNID columnid, void *pvData, unsigned long cbData, unsigned long *pcbActual, JET_GRBIT grbit, struct _JET_RETINFO *pretinfo)
{
    if (pcbActual)
        *pcbActual = 0;
    return jet_not_available();
}

JET_ERR JET_API JetSetSystemParameterW(JET_INSTANCE *pinstance, JET_SESID sesid, unsigned long paramid, JET_API_PTR lParam, JET_PCWSTR szParam)
{
    /* pinstance is in/out here (it names the instance the parameter applies to), so it is not
     * written: clearing it would replace a valid caller handle with zero. */
    return jet_not_available();
}

JET_ERR JET_API JetTerm(JET_INSTANCE instance)
{
    return JET_errInvalidParameter;
}
