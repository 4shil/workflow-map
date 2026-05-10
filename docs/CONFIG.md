# Configuration Reference

## Config File Location

`~/.config/workflow-map/config.toml`

Created automatically on first run if it doesn't exist.

## Full Configuration

```toml
[parsers]
# Use regex-based Python parsing (v1 default)
use_regex_parser = true
# Maximum parse errors before aborting
max_parse_errors = 50

[render]
# Color scheme: "default", "dark", "light", "none"
color_scheme = "default"
# Respect NO_COLOR environment variable
respect_no_color = true

[render.status_markers]
ok = "OK"
error = "ERR"
running = "RUN"
waiting = "WAIT"
skipped = "SKIP"

[watch]
# Debounce interval in milliseconds
debounce_ms = 500

# Default export format: "text", "json", "md"
default_export_format = "text"
```

## Parser Configuration

### `[parsers]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `use_regex_parser` | bool | `true` | Use regex-based Python parsing (v1). Set to `false` when tree-sitter is added. |
| `max_parse_errors` | usize | `50` | Maximum number of parse errors before the parser stops. |
| `disable_cache` | bool | `false` | Disable in-memory parse caching. |

## Render Configuration

### `[render]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `color_scheme` | string | `"default"` | Color scheme for TUI rendering |
| `respect_no_color` | bool | `true` | Respect the `NO_COLOR` environment variable |

### `[render.status_markers]`
Customize the status markers shown in the TUI and text export.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ok` | string | `"OK"` | Marker for completed steps |
| `error` | string | `"ERR"` | Marker for failed steps |
| `running` | string | `"RUN"` | Marker for in-progress steps |
| `waiting` | string | `"WAIT"` | Marker for queued steps |
| `skipped` | string | `"SKIP"` | Marker for skipped steps |

## Watch Configuration

### `[watch]`
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `debounce_ms` | u64 | `500` | Debounce interval in milliseconds. Increase for slow file systems. |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NO_COLOR` | Disable colored output (per https://no-color.org) |
| `WORKFLOW_MAP_CONFIG` | Override config file path |

## CLI Flag Overrides

All config file values can be overridden by CLI flags:

| Config Key | CLI Flag |
|------------|----------|
| All | `--no-config` to ignore config file |
| N/A | `--config <path>` for custom config path |
| N/A | `--refresh` to re-download remote URLs |
| N/A | `--timing-file <path>` to load duration/resource data |
| `parsers.disable_cache` | `--no-cache` |
| `render.respect_no_color` | `--no-color` |
| `default_export_format` | `--format <format>` |
| `watch.debounce_ms` | N/A (config only) |
