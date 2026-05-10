# Keyboard Shortcuts

## Normal Mode

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `Up` / `Down` | Move cursor |
| `Left` / `Right` | Collapse / expand step |
| `Enter` | Toggle detail view |
| `f` | Cycle filter mode (All → Failed → Running) |
| `/` | Enter search mode |
| `e` | Export current view |
| `r` | Trigger reload |
| `?` / `h` | Toggle help overlay |
| `PageUp` / `PageDown` | Scroll by 10 steps |
| `Home` / `End` | Jump to first / last step |
| `y` | Yank (copy) step name to clipboard |
| `Y` | Yank full step details to clipboard |
| `v` | Toggle view mode (list / tree / split) |
| `Tab` | Switch panel in split view |

## Search Mode

| Key | Action |
|-----|--------|
| `Enter` | Apply search |
| `Esc` | Cancel search |
| `Backspace` | Delete character |
| Any char | Append to query |

## Mouse

| Action | Result |
|--------|--------|
| Click step | Select and show detail |
| Scroll up | Move cursor up |
| Scroll down | Move cursor down |

## Filter Modes

Press `f` to cycle:

1. **All** — Show all steps
2. **FailedOnly** — Show only steps with `ERR` status
3. **RunningOnly** — Show only steps with `RUN` status

## View Modes

Press `v` to cycle:

1. **List** — Single list view (default)
2. **Tree** — Hierarchical tree view
3. **Split** — Tree (left) + list (right) + detail (bottom)

## Status Bar

The status bar shows:
- Current keybindings
- Active filter mode
- Search status
- Toast notifications (export confirmations, copy confirmations)

## Help Overlay

Press `?` or `h` to show the help overlay with all available shortcuts.
