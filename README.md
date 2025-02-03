# scprdisplay

Display a notification on macOS by

- [display notification](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW224)
- [display alert](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW11)
- [display dialog](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW12)

# Usage

``` shell
❯ scptdisplay -h
Display a notification, dialog or alert via AppleScript

Usage: scptdisplay [OPTIONS] <COMMAND>

Commands:
  notification  Posts a notification using the Notification Center, containing a title, subtitle, and explanation, and optionally playing a sound. [aliases: n, notify]
  alert         Displays a standardized alert containing a message, explanation, and from one to three buttons. [aliases: a]
  dialog        Displays a dialog containing a message, one to three buttons, and optionally an icon and a ﬁeld in which the user can enter text. [aliases: d]
  help          Print this message or the help of the given subcommand(s)

Options:
      --osascript <OSASCRIPT>  osascript command [default: osascript]
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

# Requirements

- osascript
