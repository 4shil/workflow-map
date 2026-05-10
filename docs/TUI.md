# TUI Renderer Internals

## Overview

The TUI renderer (`src/renderer.rs`) uses ratatui 0.29 and crossterm 0.28 to provide an interactive terminal interface. It's ~810 lines and handles all user interaction, rendering, and state management.

## Architecture

```
TuiApp
├── workflow: Workflow          # Parsed workflow model
├── cursor: usize               # Selected step index
├── scroll_offset: usize        # Viewport scroll position
├── filter_mode: FilterMode     # All | FailedOnly | RunningOnly
├── search: SearchFilter        # Optional search query
├── selected_step: Option<usize> # Expanded step detail
├── show_help: bool             # Help overlay toggle
├── search_mode: bool           # Inline search input
├── search_buffer: String       # Search input text
├── config: AppConfig           # Rendering configuration
├── last_reload: Option<String> # Watch mode timestamp
├── should_quit: bool           # Exit flag
├── export_requested: bool      # Export trigger
└── reload_requested: bool      # Reload trigger
```

## Main Loop

```
loop {
    draw(frame);                    # Render current state
    if should_quit -> break;
    if export_requested -> export();
    poll_event(200ms);              # Non-blocking input
    if key_event -> handle_input();
}
```

## Rendering Pipeline

1. **Filtered steps** -- Apply filter mode + search to flattened step list
2. **Clamp cursor** -- Ensure cursor is within bounds
3. **Adjust scroll** -- Keep cursor visible in viewport
4. **Layout** -- Split terminal into title, content, status areas
5. **Draw steps** -- Render ASCII diagram with status markers
6. **Draw detail** -- If step selected, show detail panel
7. **Draw status bar** -- Keybindings hint
7. **Draw overlays** -- Help and search prompt on top

## Step Rendering

Each step is rendered as a single line:

```
{collapse} {error_prefix}{step_num}. {name}
```

Where:
- `{collapse}` = `[+]` (collapsed), `[-]` (expanded), `[ ]` (no children)
- `{error_prefix}` = `*` if error, ` ` otherwise
- `{step_num}` = Sequential number in filtered list
- `{name}` = Step name, truncated to fit terminal width

Indentation: 2 spaces per nesting depth.

## Color Scheme

| Element | Color | Style |
|---------|-------|-------|
| Title | Cyan | Bold |
| OK steps | Green | Normal |
| Error steps | Red | Bold + `*` prefix |
| Running steps | Yellow | Normal |
| Waiting steps | White | Dim |
| Skipped steps | DarkGray | Normal |
| Cursor | Inverted | Reverse video |
| Status bar | White on Blue | Normal |
| Help overlay | White on Black | Normal |
| Detail panel border | Cyan | Normal |

## Input Handling

### Normal Mode
| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `Up` / `Down` | Move cursor |
| `Left` / `Right` | Collapse / expand |
| `Enter` | Toggle detail view |
| `f` | Cycle filter mode |
| `/` | Enter search mode |
| `e` | Export current view (toast confirmation) |
| `r` | Trigger reload |
| `?` / `h` | Toggle help |
| `PageUp` / `PageDown` | Scroll by 10 |
| `Home` / `End` | Jump to first / last |
| Mouse click | Select step |
| Mouse wheel | Move cursor |

### Search Mode
| Key | Action |
|-----|--------|
| `Enter` | Apply search |
| `Esc` | Cancel search |
| `Backspace` | Delete character |
| Any char | Append to query |

## Filtering

Three filter modes cycle with `f`:
1. **All** -- Show all steps
2. **FailedOnly** -- Show only steps with Status::Error
3. **RunningOnly** -- Show only steps with Status::Running

Search is applied on top of filter. When search is active, only matching steps from the filtered set are shown.

## Pagination

For workflows with more steps than viewport height:
- `scroll_offset` tracks the first visible step
- Cursor movement adjusts scroll to keep cursor visible
- PageUp/PageDown scroll by 10 steps
- Home/End jump to first/last step

## Detail Panel

When a step is selected (Enter key):
- Terminal splits 60/40 between step list and detail panel
- Detail panel shows: name, type, source location, config snippet (up to 6 lines), error message + suggestion
- Panel has a border with " Detail " title

## Help Overlay

Centered popup showing all keybindings:
```
+------------------------------------------+
| Help                                     |
|                                          |
| q          Quit                          |
| Up/Down    Navigate steps                |
| Left/Right Collapse/expand               |
| Enter      Toggle detail view            |
| f          Cycle filter mode             |
| /          Search steps                  |
| e          Export current view           |
| r          Trigger reload                |
| ?          Toggle this help              |
| PgUp/PgDn  Scroll by 10                 |
| Home/End   Jump to first/last            |
|                                          |
| Press ? to close                         |
+------------------------------------------+
```
