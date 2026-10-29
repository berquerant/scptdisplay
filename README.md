# scptdisplay

Display notifications, alerts, and dialogs on macOS via AppleScript commands:

- [display notification](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW224)
- [display alert](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW11)
- [display dialog](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW12)

All subcommands output structured JSON to standard output, making `scptdisplay` easy to integrate with scripts, pipelines, and tools like `jq`.

---

# Usage

```shell
❯ scptdisplay -h
Display a notification, dialog or alert via AppleScript

Usage: scptdisplay [OPTIONS] <COMMAND>

Commands:
  notification  Posts a notification using the Notification Center, containing a title, subtitle, and explanation, and optionally playing a sound. [aliases: n, notify]
  alert         Displays a standardized alert containing a message, explanation, and from one to three buttons. [aliases: a]
  dialog        Displays a dialog containing a message, one to three buttons, and optionally an icon and a ﬁeld in which the user can enter text. [aliases: d]
  mcp           Start Model Context Protocol (MCP) server over stdio
  help          Print this message or the help of the given subcommand(s)

Options:
      --osascript <OSASCRIPT>  osascript command [default: /usr/bin/osascript]
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

## Examples

### 1. Notification

Post a notification banner to macOS Notification Center:

```shell
# With text and title
scptdisplay notification "Build completed successfully." --title "CI" --subtitle "scptdisplay" --sound "Glass"

# Title only
scptdisplay notification --title "Quick Reminder"
```

### 2. Alert

Show an alert dialog with customizable buttons:

```shell
scptdisplay alert "File not found" \
  --message "Would you like to search again?" \
  --as warning \
  --buttons "Cancel" "Search" \
  --default_button 2 \
  --cancel_button 1
```

Alert types (`--as`):
- `informational` (default)
- `warning`
- `critical`

### 3. Dialog

Show an interactive text input or prompt dialog:

```shell
# Text input prompt
scptdisplay dialog "Enter your name:" --default_answer "Alice" --title "User Profile"

# Password prompt (hidden answer)
scptdisplay dialog "Enter token:" --default_answer "" --hidden_answer --title "Authentication"

# Auto-dismiss after 10 seconds
scptdisplay dialog "Are you still there?" --buttons "No" "Yes" --default_button 2 --giving_up_after 10
```

### 4. Model Context Protocol (MCP) Server

Run `scptdisplay` as a Model Context Protocol (MCP) server over stdio:

```shell
scptdisplay mcp
```

#### MCP Tools Provided:
- `notification`: Posts a macOS desktop notification banner using Notification Center.
  - Parameters: `text` (string), `title` (string), `subtitle` (string), `sound_name` (string).
- `alert`: Displays a standardized alert dialog with 1 to 3 buttons.
  - Parameters: `text` (string, required), `message` (string), `alert_type` ("informational" | "warning" | "critical"), `buttons` (array of strings), `default_button` (string), `cancel_button` (string), `giving_up_after` (integer).
- `dialog`: Displays an interactive dialog with buttons and optional text field.
  - Parameters: `text` (string, required), `default_answer` (string), `hidden_answer` (boolean), `buttons` (array of strings), `default_button` (string), `cancel_button` (string), `title` (string), `icon` (string), `giving_up_after` (integer).

#### Claude Desktop / MCP Configuration Example

Add `scptdisplay` to your `claude_desktop_config.json` or MCP settings:

```json
{
  "mcpServers": {
    "scptdisplay": {
      "command": "scptdisplay",
      "args": ["mcp"]
    }
  }
}
```

---

# Output Format

The command outputs a JSON object to `stdout`:

### Successful Dialog Response

```json
{
  "result": "ok",
  "code": 0,
  "error": null,
  "data": {
    "dialog": {
      "raw": "{button returned:\"OK\", text returned:\"Alice\", gave up:false}\n",
      "record": {
        "button returned": "OK",
        "text returned": "Alice",
        "gave up": "false"
      },
      "text": "Alice",
      "button": "OK",
      "gave_up": false
    }
  }
}
```

### Error Response

```json
{
  "result": "error",
  "code": 1,
  "error": "execution error: User canceled.",
  "data": null
}
```

---

# Exit Status

- `0`: Successfully executed and processed.
- `1`: Failed to process (e.g. process execution failure, script error).

---

# Requirements & Permissions

- macOS with `osascript` installed.
- **Automation Permission**: When running from terminal applications (e.g. Terminal, iTerm2), macOS may prompt for permission to control "System Events". Ensure permission is granted under:
  `System Settings > Privacy & Security > Automation`
- **Notification Permission**: Notification banners require notifications to be enabled for the terminal or executing application under:
  `System Settings > Notifications`
