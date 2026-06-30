# CLI Reference

```
corpora-atlas [FLAGS] [OPTIONS] <QUERY>...
```

## Positional

| Arg | Description |
|-----|-------------|
| `QUERY` | Words to look up. Multiple words joined with spaces. |

## Backend Flags

| Flag | Description |
|------|-------------|
| `--gd` | Use GoldenDict backend |
| `--kiwix` | Use Kiwix backend |
| `--aard2` | Use Aard2 backend |
| `--mw <SITE>` | Use MediaWiki backend (site key from config, or any URL) |

If no backend flag is given, runs GoldenDict in catalog mode (lists dictionary names).

Multiple backends can be combined; they execute sequentially.

## GoldenDict Options

| Flag | Short | Description |
|------|-------|-------------|
| `--gd` | | Enable GoldenDict backend |
| `-g <GROUP>` | | GD group name to look up in |
| `-d <DICTS>` | | Comma-separated dictionary names to filter |
| `-a` | | Extract all dictionaries (full content) |
| `-m` | | Multi-file mode: write each dict to a separate file |
| `-n` | | Annotate output with `# From <dict>` headers |

## Kiwix Options

| Flag | Short | Description |
|------|-------|-------------|
| `--kiwix` | | Enable Kiwix backend |
| `-z <ZIM>` | | ZIM shorthand name (required with --kiwix) |
| `--kiwix-page <N>` | | Page number (default: 1) |

## Aard2 Options

| Flag | Short | Description |
|------|-------|-------------|
| `--aard2` | | Enable Aard2 backend |
| `-s <SLOB>` | | SLOB shorthand name |

## MediaWiki Options

| Flag | Description |
|------|-------------|
| `--mw <SITE>` | MediaWiki site key (from config) or any MediaWiki URL |
| `--mw-search` | Use search API instead of page parse |
| `--mw-page <N>` | Page number for search results (default: 1) |

## Output Formatting

| Flag | Description |
|------|-------------|
| `--html` | Output raw HTML instead of plain text |
| `--lean-toc` | Extract and display table of contents |
| `--lean-section <ID>` | Extract a specific section by ID |
| `--lean-text` | Convert HTML to plain text (lean mode) |

## Daemon Control

| Flag | Description |
|------|-------------|
| `--daemon` | Start the daemon process (foreground) |
| `--toggle-gd-auto-clip` | Toggle clipboard monitoring (auto-starts daemon) |
| `--toggle-gd-auto-focus` | Toggle GoldenDict auto-focus feature |
| `--gd-clip` | Cycle to next GD dictionary group for current word |
| `--clip <TEXT>` | Override clipboard content for --gd-clip |

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
# Output: /tmp/corpus/hello/gd/<dict_name>.txt
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
# Start daemon
corpora-atlas --daemon &

# Toggle clipboard monitoring
corpora-atlas --toggle-gd-auto-clip

# Cycle groups
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
