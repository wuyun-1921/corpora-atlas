# Daemon Mode

The daemon runs in the foreground, monitoring the clipboard and auto-looking up
words in GoldenDict.

## Start

```sh
corpora-atlas --daemon &
```

## Clipboard Polling

Every `poll_interval` seconds (default 0.5s):

1. `wl-paste --no-newline` reads clipboard
2. First line only. Strips markdown formatting (`*`, `_`, `~`, backtick, `#`, `=`)
   and surrounding punctuation
3. Rejects if: empty, exceeds `max_query_len`, not alphabetic text, starts with
   URL scheme, or contains `/`
4. Detects script via Unicode codepoint ranges (see lang detection below)
5. Looks up word in GD using the first group from the script's fallback chain
6. Optionally focuses GD window via `wlrctl`

## IPC

Unix socket at `config.paths.socket`. JSON protocol, newline-delimited.

### Commands

| Action | Extra Fields | Description |
|--------|-------------|-------------|
| `cycle` | `clip` (optional) | Cycle to next GD dictionary group |
| `toggle` | | Toggle clipboard monitoring on/off |
| `toggle_focus` | | Toggle auto-focus GD window |

### Client Usage

```sh
# Toggle monitoring
corpora-atlas --toggle

# Cycle groups
corpora-atlas --cycle

# Cycle with custom text
corpora-atlas --cycle --clip "test"

# Toggle focus
corpora-atlas --toggle-focus
```

### Responses

Success: `{"status": "ok"}`
Error: `{"error": "..."}`

## State

JSON file at `config.paths.state`, protected by `flock`:

```json
{
  "prev_query": "hello",
  "repeat": 1,
  "group": "English",
  "monitoring": true,
  "focus_gd": true
}
```

| Field | Description |
|-------|-------------|
| `prev_query` | Last looked-up word |
| `repeat` | Cycle counter for current word |
| `group` | Current GD group name |
| `monitoring` | Clipboard polling enabled |
| `focus_gd` | Auto-focus GD window after lookup |

## Cycle Logic

The cycle command (`--cycle` or IPC `cycle`) implements smart group switching:

### English text
- Looks up in the **currently active** GD group (read from CDP)
- Does not cycle - English words use whatever group GD is already on

### Non-English text
1. Reads current GD group from CDP
2. If GD's group differs from daemon state: respects GD's group, advances to next
3. If same as daemon state: advances through the fallback chain
4. `advance()` increments `repeat` counter, selects `chain[repeat % chain.len()]`

Example with `chinese: [文, CE, 中2, J, J2]`:

| Cycle | Group |
|-------|-------|
| 0 | 文 |
| 1 | CE |
| 2 | 中2 |
| 3 | J |
| 4 | J2 |
| 5 | 文 (wraps) |

## Language Detection

Script detection by Unicode codepoint ranges:

| Script | Key | Unicode Ranges |
|--------|-----|---------------|
| Japanese | `japanese` | 0x3040-0x30FF (Hiragana + Katakana) |
| Chinese | `chinese` | 0x4E00-0x9FFF, 0x3400-0x4DBF (CJK) |
| Cyrillic | `cyrillic` | 0x0400-0x052F |
| Greek | `greek` | 0x0370-0x03FF, 0x1F00-0x1FFF |
| Hangul | `hangul` | 0xAC00-0xD7AF |
| Semitic | `semitic` | 0x0600-0x06FF (Arabic), 0x0590-0x05FF (Hebrew) |
| Brahmic | `brahmic` | 0x0900-0x097F, 0x0980-0x09FF |
| English | `english` | 0x0041-0x005A, 0x0061-0x007A (Latin) |
| Other | `other` | Everything else |

## Systemd

Example service files for the daemon and backend services:

```ini
# ~/.config/systemd/user/corpora-atlas.service
[Unit]
Description=Corpora Atlas daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/corpora-atlas --daemon
Restart=on-failure
RestartSec=3

[Install]
WantedBy=graphical-session.target
```

```sh
systemctl --user daemon-reload
systemctl --user enable --now corpora-atlas
```
