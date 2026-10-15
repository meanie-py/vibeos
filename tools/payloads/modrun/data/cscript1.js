// modrun cscript suite: the IHost surface a console script sees (not FullName/Path,
// which name the host binary and so differ between the C and Rust variants).
WScript.Echo("hello", 42, true, null, 1.5);
WScript.Echo(WScript.Name);
WScript.Echo(WScript.ScriptName);
WScript.Echo(WScript.Version, WScript.BuildVersion);
WScript.Echo("interactive=" + WScript.Interactive, "timeout=" + WScript.Timeout);
var args = WScript.Arguments;
WScript.Echo("argc=" + args.Count());
for (var i = 0; i < args.Count(); i++)
    WScript.Echo("arg" + i + "=" + args.Item(i));
WScript.Interactive = false;
WScript.Sleep(10);
WScript.Quit(7);
