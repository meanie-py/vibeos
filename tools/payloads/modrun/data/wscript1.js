// modrun wscript suite: wscript.exe's default is INTERACTIVE (cscript's is not), read back as
// an exit code because an interactive Echo would open a message box nobody closes.
WScript.Quit(WScript.Interactive ? 11 : 22);
