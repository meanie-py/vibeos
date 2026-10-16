' modrun cscript suite: the same host surface from VBScript.
WScript.Echo "vbs", 3 * 7, True
WScript.Echo WScript.ScriptName
Dim i
For i = 0 To WScript.Arguments.Count - 1
    WScript.Echo "arg" & i & "=" & WScript.Arguments.Item(i)
Next
WScript.Quit 3
