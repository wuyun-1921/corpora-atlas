# corpora-atlas

Unified terminal-based lookup tool for GoldenDict-ng, Aard2-web, Kiwix-serve, and MediaWiki APIs.

Queries multiple offline/online backends from the command line, or runs as a daemon
that monitors your clipboard and auto-looks up words in GoldenDict.

**Platform:** Linux (Wayland) only. Requires `wl-paste` (clipboard) and `wlrctl` (window focus).

macOS and Windows support is feasible — the only platform-specific code is clipboard
access and window focus, which could be swapped with native equivalents (NSPasteboard
on macOS, Win32 clipboard APIs on Windows). Not planned, but low-effort ports for
anyone who needs them.

Pre-built x86_64 and aarch64 Linux binaries available in [releases](https://github.com/wuyun-1921/corpora-atlas/releases).

## Install

```sh
cargo build --release
cp target/release/corpora-atlas ~/.local/bin/
```

## Quick Start

### One-off lookups

```sh
corpora-atlas hello                          # GoldenDict catalog
corpora-atlas --gd -g "English" hello        # GoldenDict with group
corpora-atlas --kiwix -z wikisource-en hello # Kiwix search
corpora-atlas --aard2 -s enwiki hello        # Aard2 lookup
corpora-atlas --mw en.wikipedia Philosophy   # MediaWiki article
```

### Daemon mode

```sh
corpora-atlas --daemon &                     # start daemon
corpora-atlas --toggle-gd-auto-clip          # enable clipboard monitoring
corpora-atlas --toggle-gd-auto-focus         # auto-focus GoldenDict window
corpora-atlas --gd-clip                      # lookup clipboard text
```

### Lean mode (filtered reading)

```sh
corpora-atlas --kiwix -z wikisource-en --lean-toc "Philosophy"
corpora-atlas --kiwix -z wikisource-en --lean-section Etymology "Philosophy"
corpora-atlas --kiwix -z wikisource-en --lean-text "Philosophy"
```

## Architecture

```
corpora-atlas CLI
  ├── One-off query → --gd / --kiwix / --aard2 / --mw
  ├── Daemon mode  → clipboard poll loop → GoldenDict auto-lookup
  └── Lean mode    → filtered, plain-text extraction
```

## Dependencies

| Dependency | Purpose |
|-----------|---------|
| GoldenDict-ng | `--gd` and daemon mode |
| kiwix-serve | `--kiwix` backend |
| aard2-web | `--aard2` backend |
| `wl-paste` | Wayland clipboard access |
| `wlrctl` | Wayland window focus (auto-focus GoldenDict) |
| `notify-send` | desktop notifications |

## Backends

- **GoldenDict-ng** — Chrome DevTools Protocol (CDP) websocket. Group-aware, multi-dictionary.
- **Kiwix-serve** — HTTP API for ZIM archives (Wikipedia, Wikisource, Wiktionary, etc.).
- **Aard2-web** — HTTP API for SLOB dictionaries.
- **MediaWiki** — `api.php` endpoint for any MediaWiki wiki.

See [docs/backends.md](docs/backends.md) for details.

## Configuration

Copy the example config and customize:

```sh
mkdir -p ~/.config/corpora-atlas
cp tests/fixtures/config.yaml.example ~/.config/corpora-atlas/config.yaml
```

See [docs/config.md](docs/config.md) for the full reference.

## Documentation

- [CLI reference](docs/cli.md) — all flags and options
- [Configuration](docs/config.md) — config file reference
- [Backends](docs/backends.md) — backend details
- [Daemon & IPC](docs/daemon.md) — daemon mode, clipboard polling, cycle logic
