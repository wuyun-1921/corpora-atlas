# Backends

## GoldenDict

Communicates with GoldenDict via Chrome DevTools Protocol (CDP) over WebSocket.

### Flow

1. Spawns GD binary with query word and optional group (`-g <group>`)
2. Waits 1.5s for GD to render (3s for group "H")
3. Discovers CDP target via `http://localhost:{port}/json/list`
4. Connects via WebSocket to extract `document.body.innerHTML`
5. Parses HTML: splits on `<article class="gdarticle">`, extracts dict titles and content

### Output Modes

| Flags | Behavior |
|-------|----------|
| (none) | Lists dictionary names |
| `-a` | Full content from all dictionaries |
| `-d <dicts>` | Full content from specified dictionaries |
| `-m` | Multi-file: writes each dict to `{output}/{query}/gd/{safe_name}.txt` |
| `-n` | Adds `# From <dictname>` headers |

Note: without `-a`, `-d`, or `-m`, GD still runs in multi-file mode internally
but prints results to stdout rather than writing files.

### Group Resolution

GD group IDs in CDP URLs are numeric. The backend reads GD's XML config
(configurable via `gd.config_path`, default `~/.goldendict/config`) to map
IDs back to human-readable names.

### Dependencies

- GoldenDict binary (CDP enabled)
- `wlrctl` (optional, for window focus in daemon mode)

---

## Kiwix

Queries a local `kiwix-serve` HTTP API.

### Flow

1. Resolves ZIM shorthand via config (`-z wikisource-en` -> real filename).
   If shorthand is not found in config, used as-is (pass-through).
2. HTTP GET: `{kiwix_base}/search?pattern={query}&books.name={zim}&pageLength=25&page=1`
3. Parses results from HTML response

### Output Modes

| Mode | Behavior |
|------|----------|
| Text | Header line + numbered list of results with 200-char snippet truncation |
| HTML | Raw HTML from kiwix-serve |

### Dependencies

- kiwix-serve running and serving ZIM files

---

## Aard2

Queries a local `aard2-web` HTTP API.

### Flow

1. Resolves SLOB shorthand via config (`-s enwiki` -> real filename).
   If shorthand is not found in config, used as-is (pass-through).
2. HTTP GET: `{aard2_base}/find/?key={query}`
3. Response: JSON array with `dictLabel`, `label`, `url` fields

### Output Modes

| Mode | Behavior |
|------|----------|
| Text | Header line + `[dictLabel] title -> url` per result |
| HTML | Fetches full article HTML from first result's URL |

### Dependencies

- aard2-web running and serving SLOB files

---

## MediaWiki

Queries MediaWiki sites via their `api.php` action API.

### Flow

1. Resolves site: if `--mw` value contains `://`, uses it directly as base URL
   (strips trailing `/` and `/api.php` suffix if present); otherwise looks up
   config key (`en.wikipedia` -> `https://en.wikipedia.org/w`).
2. Depending on mode:
   - **Parse** (default): `action=parse&prop=text&redirects&page={query}` - full article HTML
   - **Search** (`--mw-search`): `action=query&list=search` - up to 50 results

### Output Modes

| Flags | Behavior |
|-------|----------|
| (default) | Rendered article HTML, converted to plain text |
| `--html` | Raw parsoid HTML |
| `--mw-search` | Search results with titles and snippets |
| `--lean-toc` | Table of contents from article |
| `--lean-section <id>` | Specific section by heading ID |
| `--lean-text` | Convert HTML to plain text |

Note: the `redirects` parameter is included in parse requests, so redirects
are followed automatically.

### User-Agent

`corpora-atlas/1.0`

### Dependencies

- Internet access (or local MediaWiki instance)
