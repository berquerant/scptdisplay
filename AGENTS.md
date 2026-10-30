# Agent Guidelines for `scptdisplay`

This repository implements `scptdisplay`, a macOS command-line utility for displaying desktop notifications, alerts, and interactive dialogs via AppleScript (`osascript`) with structured JSON output.

All AI agents operating in this repository must adhere to the following rules and guidelines.

---

## 1. Keeping Documentation & AGENTS.md Up to Date

When making structural changes to the project (adding or modifying CLI subcommands, options, parsing logic, or architectural conventions), **update this file** and `README.md` so they accurately reflect the current state of the codebase. Treat `AGENTS.md` as living documentation.

---

## 2. Natural Language Policy

- **Language**: English is strictly required for all natural language content throughout this repository:
  - Code comments and docstrings.
  - Git commit messages and pull request descriptions (following Conventional Commits).
  - Markdown documentation (`README.md`, `AGENTS.md`).

---

## 3. Project Overview & Subcommands

`scptdisplay` wraps AppleScript UI commands and returns structured JSON to `stdout`:

| Subcommand | AppleScript Target | Purpose | Key Parameters |
|---|---|---|---|
| `notification` (`n`, `notify`) | `display notification` | macOS Notification Center banner | `text` (optional if title provided), `-t, --title`, `-s, --subtitle`, `--sound` |
| `alert` (`a`) | `display alert` | Standardized alert dialog with 1–3 buttons | `text`, `--message`, `-t, --as` (`AlertType`), `--buttons`, `--default_button`, `--cancel_button`, `-g, --giving_up_after` |
| `dialog` (`d`) | `display dialog` | Interactive dialog with text field and 1–3 buttons | `text`, `--default_answer`, `--hidden_answer`, `--buttons`, `--default_button`, `--cancel_button`, `-t, --title`, `--icon`, `-g, --giving_up_after` |
| `mcp` | — | Model Context Protocol (MCP) server over stdio | — |

### CLI Options
- `--osascript <OSASCRIPT>`: Path or name of the osascript binary (defaults to `/usr/bin/osascript`). Can be pointed to a mock or wrapper script for automated testing.

---

## 4. Repository Layout

```text
scptdisplay/
├── .github/
│   ├── dependabot.yml       # Dependabot configuration (cargo, github-actions)
│   └── workflows/
│       └── ci.yml           # GitHub Actions CI workflow (lint, test, build)
├── src/
│   ├── lib.rs               # Library root re-exporting modules for internal and external use
│   ├── main.rs              # Application entry point: logging, argument parsing, child process execution, exit status handling
│   ├── cli.rs               # Clap CLI definition (`Cli`, `Commands`, `AlertType`), AppleScript command synthesis, and response parsing
│   ├── cmd.rs               # `Cmd` abstraction managing process command vectors and argument construction
│   ├── mcp.rs               # Model Context Protocol (MCP) stdio server implementation using `rmcp`
│   ├── parse.rs             # AppleScript input escaping (`Input`) and record output parsing (`Output`)
│   └── response.rs          # Unified JSON `Response`, status `Code`, and subcommand `Data` models
├── Cargo.lock               # Cargo dependency lockfile
├── Cargo.toml               # Package manifest and dependencies
├── Makefile                 # Make task definitions (`test`, `lint`, `format`, `clippy`, `ci`)
├── README.md                # User-facing documentation and usage guide
└── AGENTS.md                # Agent operational guide and architecture documentation
```

---

## 5. Development Tasks & Verification Commands

All development tasks should be verified with Cargo or `make`:

```shell
# Run unit tests
cargo test
# or: make test

# Run linter checks (fail on warnings)
cargo clippy -- -D warnings
# or: make clippy

# Check code formatting
cargo fmt --all -- --check
# or: make format

# Format code
cargo fmt --all

# Run security audit on dependencies
cargo audit

# Run full CI suite via make
make ci
```

### Pre-Completion Rule
Always ensure `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt -- --check` pass with zero warnings or errors before concluding tasks.

---

## 6. Architecture & Design Principles

### 6.1 Process Exit Status Contract
- **Contract**:
  - `0`: Process successfully spawned, executed, and output parsed.
  - `1`: Any failure occurs (e.g., spawn failure, osascript non-zero exit, unparseable output).
- **Enforcement**:
  - In `src/main.rs`, always inspect `result.is_err()` and terminate via `std::process::exit(1)` when false/error.
  - Even on error, `scptdisplay` prints the error `Response` JSON to `stdout` before exiting.

### 6.2 Security & AppleScript Injection Prevention
- **AppleScript String Literals**:
  - Never interpolate raw user input into AppleScript command strings.
  - Always use `Input::quoted()` in `src/parse.rs`, which escapes backslashes (`\` → `\\`) and double quotes (`"` → `\"`) to prevent string boundary breakouts and arbitrary AppleScript command injection (`do shell script`).
- **Standard Input Payload Delivery**:
  - Always pass AppleScript command payloads to `osascript` via `stdin` piping rather than command-line arguments (`-e`). This prevents sensitive prompts and passwords from being exposed in process table listings (`ps aux`) and avoids OS command-line length limits (`ARG_MAX`).
- **Enumerations for Options**:
  - Arguments that map to unquoted AppleScript keywords (such as `--as` in `alert`) must be constrained using `clap::ValueEnum` (e.g. `AlertType`) rather than free-form `String`.

### 6.3 Record Output Parsing & Response Integrity
- AppleScript execution passes `-s s` to format record output in AppleScript source notation (`{button returned:"...", text returned:"...", gave up:false}`).
- `Output::record` in `src/parse.rs` lexes this quoted structure safely, ensuring embedded commas, colons, or injected keywords (e.g. `button returned:...`) inside user text cannot spoof or overwrite parsed fields.
- Legacy unquoted output parsing is retained as a fallback.

### 6.4 Defensive Design & Concurrency Protection
- Avoid unhandled `.unwrap()` on user input or process I/O.
- Conversions (`From<Cmd> for String`, `From<Cmd> for Command`) must safely handle empty command vectors without panicking.
- In the MCP server (`src/mcp.rs`), synchronous child process executions are offloaded to `tokio::task::spawn_blocking` to avoid starving the async Tokio runtime when awaiting user dialog interactions.
- Dialog and alert buttons are validated to enforce AppleScript's upper limit of 3 buttons.

---

## 7. macOS Environment Prerequisites

When testing or running `scptdisplay` locally on macOS:
- `/usr/bin/osascript` must be available on the system (or a custom command specified via `--osascript` must be resolvable in `PATH`).
- Terminal emulators or calling agents require macOS automation permissions under:
  `System Settings > Privacy & Security > Automation` (System Events).
- Banner notifications require notifications to be enabled under:
  `System Settings > Notifications`.
