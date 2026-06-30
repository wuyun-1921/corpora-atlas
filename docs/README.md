# corpora-atlas

Unified terminal-based lookup tool for dictionaries, encyclopedias, and wikis.

Queries multiple offline/online backends from the command line, or runs as a daemon
that monitors your clipboard and auto-looks up words in GoldenDict.

**Backends:** GoldenDict (CDP), Kiwix, Aard2, MediaWiki

**Platform:** Linux (Wayland) - uses `wl-paste` and `wlrctl`.

## Install

```sh
cargo build --release
# binary at target/release/corpora-atlas
```

Copy to PATH:

```sh
cp target/release/corpora-atlas ~/.local/bin/
```

## Dependencies

- GoldenDict (for `--gd` and daemon mode)
- kiwix-serve (for `--kiwix`)
- aard2-web (for `--aard2`)
- `wl-paste` / `wlrctl` (daemon clipboard + window focus)
- `notify-send` (desktop notifications)

## Quick Start

### One-off lookups

```sh
# Default: GoldenDict catalog (lists dictionaries for a word)
corpora-atlas hello

# GoldenDict with group
corpora-atlas --gd -g "English" hello

# Kiwix search
corpora-atlas --kiwix -z wikisource-en hello

# Aard2 lookup
corpora-atlas --aard2 -s enwiki hello

# MediaWiki article
corpora-atlas --mw en.wikipedia Philosophy
```

### Daemon mode

```sh
# Start daemon (clipboard polling + auto GD lookup)
corpora-atlas --daemon &

# Toggle clipboard monitoring on/off
corpora-atlas --toggle-clipboard

# Cycle through dictionary groups for current word
corpora-atlas --cycle

# Toggle auto-focus GoldenDict window
corpora-atlas --toggle-focus
```

### Lean mode (filtered reading)

```sh
# Show table of contents
corpora-atlas --kiwix -z wikisource-en --lean-toc "Philosophy"

# Extract specific section
corpora-atlas --kiwix -z wikisource-en --lean-section Etymology "Philosophy"

# Plain text output
corpora-atlas --kiwix -z wikisource-en --lean-text "Philosophy"
```

### Multi-file extraction

```sh
# Extract all dictionaries to separate files
corpora-atlas --gd -a -m -n "hello"
# Writes to /tmp/corpus/hello/gd/<dict>.txt
```

## Architecture

```
User
  |
  v
corpora-atlas CLI
  |-- Config::init() loads ~/.config/corpora-atlas/config.yaml
  |
  +-- One-off query
  |     +-- --gd       -> GoldenDict via CDP websocket
  |     +-- --kiwix    -> kiwix-serve HTTP API
  |     +-- --aard2    -> aard2-web HTTP API
  |     +-- --mw       -> MediaWiki api.php
  |     +-- (default)  -> GD catalog mode
  |
  +-- Daemon mode
  |     +-- Unix socket IPC (toggle/cycle)
  |     +-- Clipboard poll loop (wl-paste)
  |     +-- Language detection -> group chain routing
  |     +-- Auto GD lookup + optional window focus
  |
  +-- Web UI (--serve, not yet implemented)
```

## Configuration

See [config.md](config.md) for the full reference.

## CLI Reference

See [cli.md](cli.md) for all flags and options.

## Backends

See [backends.md](backends.md) for details on each backend.

## Daemon & IPC

See [daemon.md](daemon.md) for daemon mode, clipboard polling, and cycle logic.
