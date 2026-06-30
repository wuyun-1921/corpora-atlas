# Configuration

Config file: `~/.config/corpora-atlas/config.yaml`

Copy the example from `tests/fixtures/config.yaml.example`:

```sh
mkdir -p ~/.config/corpora-atlas
cp tests/fixtures/config.yaml.example ~/.config/corpora-atlas/config.yaml
```

## Paths

Runtime paths for daemon IPC and state.

```yaml
paths:
  socket: /tmp/corpora-atlas.sock    # Unix socket for IPC
  state:  /tmp/corpora-atlas.state   # JSON state file (flock-protected)
  lock:   /tmp/corpora-atlas.lock    # File lock for state access
  pid:    /tmp/corpora-atlas.pid     # Daemon PID file
  output: /tmp/corpus                # Multi-file extraction output dir
```

## Servers

Backend service endpoints. All three must be running for full functionality.

```yaml
servers:
  gd_cdp: localhost:18123       # GoldenDict Chrome DevTools Protocol port
  kiwix:  http://localhost:8077  # kiwix-serve base URL
  aard2:  http://127.0.0.1:8013  # aard2-web base URL
```

## Web UI

Local HTTP server with side-by-side iframes (not yet wired to the CLI).

```yaml
webui:
  port: 8090       # HTTP port
  host: 127.0.0.1  # Bind address
```

## Daemon

Clipboard polling settings.

```yaml
daemon:
  poll_interval: 0.5   # Seconds between clipboard checks (must be > 0)
  max_query_len: 50    # Max characters to process from clipboard
```

## GoldenDict

GD integration settings.

```yaml
gd:
  binary: goldendict                    # Executable name or path
  config_path: ~/.goldendict/config     # GD XML config (for group ID mapping)
  cdp_timeout: 3                        # HTTP timeout (seconds) for CDP target discovery (must be > 0)
  window_app_id: goldendict             # WM class for wlrctl focus (optional, default: goldendict)
```

## Fallback Chains

Language script detection routes words through GoldenDict group chains.

Key names must match the script detection output in `lang.rs`:
`chinese`, `japanese`, `english`, `cyrillic`, `greek`, `hangul`, `semitic`, `brahmic`, `other`

Values are ordered lists of GD group names. The daemon starts at `[0]` and cycles
through on repeated lookups of the same word.

```yaml
fallback:
  chinese:  [ZH, Z2, Z3]
  japanese: [JA, J2, Z2]
  english:  [EN, EN2, EN3]
  cyrillic: [RU]
  greek:    [GR]
  hangul:   [KR]
  semitic:  [AR, HE]
  brahmic:  [DV, BN, TM]
  other:    [OT]
```

## Kiwix

ZIM shorthand names for `-z` flag.

```yaml
kiwix:
  wikisource-zh: wikisource_zh_all_maxi_2026-02
  gutenberg-phil: gutenberg_en_lcc-b_2026-03
  iep: internet-encyclopedia-philosophy_en_all_2025-11
  wikipedia-mini: wikipedia_en_all_mini_2026-03
```

## Aard2

SLOB shorthand names for `-s` flag.

```yaml
aard2:
  enwiki: enwiki-20260401.slob
  enwiktionary: enwiktionary-20260301.slob
  jawiki: jawiki-20251001.slob
```

## MediaWiki

Live MediaWiki API sites for `--mw` flag.

```yaml
mediawiki:
  sites:
    en.wikipedia: https://en.wikipedia.org/w
    zh.wikipedia: https://zh.wikipedia.org/w
    en.wiktionary: https://en.wiktionary.org/w
```

Base URL is the `/w` directory (parent of `api.php`).
