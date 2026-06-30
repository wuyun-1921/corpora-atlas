# CLI Reference

```
corpora-atlas [FLAGS] [OPTIONS] <QUERY>...
```

## Positional

| Arg | Description |
|-----|-------------|
| `QUERY` | Words to look up. Multiple words joined with spaces. |

If no backend flag is given, runs GoldenDict in catalog mode (lists dictionary names).

Multiple backends can be combined; they execute sequentially.

## GoldenDict

| Flag | Description |
|------|-------------|
| `--gd` | Enable GoldenDict backend |
| `-g <GROUP>` | GD group name to look up |
| `-d <DICTS>` | Comma-separated dictionary names to filter |
| `-a` | Extract all dictionaries (full content) |
| `-m` | Multi-file mode: write each dict to a separate file |
| `-n` | Annotate output with `# From <dict>` headers |

## Kiwix

| Flag | Description |
|------|-------------|
| `--kiwix` | Enable Kiwix backend |
| `-z <ZIM>` | ZIM shorthand name (required with --kiwix) |

## Aard2

| Flag | Description |
|------|-------------|
| `--aard2` | Enable Aard2 backend |
| `-s <SLOB>` | SLOB shorthand name (resolved via config, or pass-through) |

## MediaWiki

| Flag | Description |
|------|-------------|
| `--mw <SITE>` | MediaWiki site key (from config) or any MediaWiki URL |
| `--mw-search` | Use search API instead of page parse |

`--mw` accepts either a config key (e.g. `en.wikipedia` mapped in `config.yaml`
under `mediawiki.sites`) or any full URL (e.g. `https://fr.wikipedia.org/w`).
URLs with `/api.php` suffix are normalized automatically.

## Output Formatting

| Flag | Description |
|------|-------------|
| `--html` | Output raw HTML instead of plain text |
| `--lean-toc` | Extract and display table of contents |
| `--lean-section <ID>` | Extract a specific section by heading ID |
| `--lean-text` | Convert HTML to plain text |

## Daemon Control

| Flag | Description |
|------|-------------|
| `--daemon` | Start the daemon process in the foreground |
| `--toggle-gd-auto-clip` | Toggle clipboard monitoring (auto-starts daemon if not running) |
| `--toggle-gd-auto-focus` | Toggle GoldenDict auto-focus feature (daemon must be running) |
| `--gd-clip` | Look up clipboard text in GD, cycling through dictionary groups defined by the fallback chain for the detected script (auto-starts daemon if not running) |
| `--clip <TEXT>` | Override clipboard content for `--gd-clip` |
| `--serve` | Start web UI server (not yet implemented) |

Note: `--toggle-gd-auto-clip` and `--gd-clip` will auto-start the daemon in the
background if it is not running. `--toggle-gd-auto-focus` requires the daemon to
already be running.

## Examples

### Basic lookups

```sh
# GD catalog (default)
corpora-atlas hello

# GD with group
corpora-atlas --gd -g "English" hello

# GD with specific dictionaries
corpora-atlas --gd -d "Oxford,Longman" hello

# Kiwix
corpora-atlas --kiwix -z wikisource-en hello

# Aard2
corpora-atlas --aard2 -s enwiki hello

# MediaWiki with config key
corpora-atlas --mw en.wikipedia Philosophy
corpora-atlas --mw en.wikipedia --mw-search "quantum physics"

# MediaWiki with any URL
corpora-atlas --mw https://fr.wikipedia.org/w Philosophy
```

### Multi-file extraction

```sh
# Extract all GD dictionaries, one file each, with headers
corpora-atlas --gd -a -m -n "hello"
# Output: <paths.output>/hello/gd/<dict>.txt
```

### Lean reading

```sh
# Table of contents
corpora-atlas --kiwix -z wikisource-en --lean-toc "Philosophy"

# Specific section
corpora-atlas --kiwix -z wikisource-en --lean-section Etymology "Philosophy"

# Plain text
corpora-atlas --kiwix -z wikisource-en --lean-text "Philosophy"

# Lead section (before first heading)
corpora-atlas --kiwix -z wikisource-en --lean-section _lead "Philosophy"
```

### Daemon

```sh
# Start daemon (monitoring starts disabled)
corpora-atlas --daemon &

# Enable clipboard monitoring
corpora-atlas --toggle-gd-auto-clip

# Cycle groups for current clipboard word
corpora-atlas --gd-clip

# Cycle with custom text
corpora-atlas --gd-clip --clip "test"

# Toggle auto-focus
corpora-atlas --toggle-gd-auto-focus
```

### Combining backends

```sh
# Query both Kiwix and MediaWiki
corpora-atlas --kiwix -z wikisource-en --mw en.wikipedia hello
```
