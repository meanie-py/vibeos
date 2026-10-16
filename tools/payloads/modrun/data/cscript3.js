// modrun cscript suite: a runtime error, reported through the host's OnScriptError.
WScript.Echo("before");
undefinedFunction();
WScript.Echo("after");
