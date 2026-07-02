# corpora-atlas

Terminal-based lookup for GoldenDict-ng, Aard2-web, Kiwix-serve, and MediaWiki APIs.
Query offline/online backends from CLI, or run as daemon to auto-lookup clipboard text in GoldenDict.

Linux (Wayland). Requires `wl-paste`, `wlrctl`. macOS/Windows feasible, not planned.

Pre-built x86_64, aarch64 binaries: [releases](https://github.com/wuyun-1921/corpora-atlas/releases).

## Install

```sh
cargo build --release
cp target/release/corpora-atlas ~/.local/bin/
```

## Quick Start

### One-off lookups

```sh
corpora-atlas hello                          # GoldenDict catalog
corpora-atlas --gd -g "English" hello        # group-aware
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
| `wlrctl` | Wayland window focus |
| `notify-send` | desktop notifications |

## Backends

| Backend | Protocol | Details |
|---------|----------|---------|
| GoldenDict-ng | CDP websocket | Group-aware, multi-dictionary |
| Kiwix-serve | HTTP API | ZIM archives (Wikipedia, Wikisource, Wiktionary, etc.) |
| Aard2-web | HTTP API | SLOB dictionaries |
| MediaWiki | api.php | Any MediaWiki wiki |

See [docs/backends.md](docs/backends.md) for details.

## Configuration

```sh
mkdir -p ~/.config/corpora-atlas
cp tests/fixtures/config.yaml.example ~/.config/corpora-atlas/config.yaml
```

See [docs/config.md](docs/config.md) for full reference.

## Documentation

- [CLI reference](docs/cli.md) — flags and options
- [Configuration](docs/config.md) — config file reference
- [Backends](docs/backends.md) — backend details
- [Daemon & IPC](docs/daemon.md) — daemon mode, clipboard polling, cycle logic
