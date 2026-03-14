# Steam Proton Browser — UX & Navigation Design

## Design Philosophy

Take direct inspiration from **k9s** (Kubernetes TUI):

- **Drill-down hierarchy** — start broad, narrow focus with each keypress.
- **Breadcrumb trail** — always know where you are.
- **Fuzzy filter** — type `/` to instantly filter the current list.
- **Vim-style navigation** — `j`/`k` or arrow keys, `Enter` to dive in, `Esc`/`Backspace` to go back.
- **Contextual hotkeys** — shown in the status bar, change per view.
- **Speed** — instant response, no loading spinners for local data.

## View Hierarchy

```
┌───────────────────────────────────────────────┐
│ [1] Library View                              │
│                                               │
│  All installed Steam games, sorted by name.   │
│  Shows: Game Name, App ID, Install Size,      │
│         Proton Version (if applicable)         │
│                                               │
│  ┌───────────────────────────────────────────┐ │
│  │ > The Witcher 3          [292030]   45 GB │ │
│  │   Cyberpunk 2077         [1091500]  65 GB │ │
│  │   Hades                  [1145360]   8 GB │ │
│  │   Elden Ring             [1245620]  50 GB │ │
│  │   ...                                     │ │
│  └───────────────────────────────────────────┘ │
│                                               │
│  / Filter  Enter Select  q Quit  ? Help       │
└───────────────────────────────────────────────┘
         │
         │ Enter
         ▼
┌───────────────────────────────────────────────┐
│ [2] Game Detail View                          │
│                                               │
│  Folder categories for the selected game.     │
│  Only categories with existing paths shown.   │
│                                               │
│  ┌───────────────────────────────────────────┐ │
│  │  The Witcher 3  [292030]                  │ │
│  │ ─────────────────────────────────────────  │ │
│  │ > 📁 Install Directory                    │ │
│  │   📁 Proton Prefix (compatdata)           │ │
│  │   📁 Proton Prefix — drive_c              │ │
│  │   📁 Shader Cache                         │ │
│  │   📁 Workshop Content                     │ │
│  │   📁 Cloud Saves (remote)                 │ │
│  │   📁 Proton Logs                          │ │
│  └───────────────────────────────────────────┘ │
│                                               │
│  / Filter  Enter Open  o Explorer  Esc Back   │
└───────────────────────────────────────────────┘
         │
         │ Enter
         ▼
┌───────────────────────────────────────────────┐
│ [3] Folder Browser View                       │
│                                               │
│  File listing within the selected folder.     │
│  Basic info: name, size, modified date.       │
│                                               │
│  ┌───────────────────────────────────────────┐ │
│  │  The Witcher 3 > Proton Prefix > drive_c  │ │
│  │ ─────────────────────────────────────────  │ │
│  │ > 📂 Program Files/                       │ │
│  │   📂 users/                               │ │
│  │   📂 windows/                             │ │
│  │   📄 dosdevices                            │ │
│  └───────────────────────────────────────────┘ │
│                                               │
│  / Filter  Enter Dive  o Explorer  Esc Back   │
└───────────────────────────────────────────────┘
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit application |
| `?` | Toggle help overlay |
| `/` | Enter fuzzy filter mode |
| `Esc` | Exit filter / go back one level |
| `Backspace` | Go back one level |
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### Library View

| Key | Action |
|-----|--------|
| `Enter` | Open game detail view |
| `o` | Open game's install directory in file explorer |
| `s` | Cycle sort (name / app id / size) |

### Game Detail View

| Key | Action |
|-----|--------|
| `Enter` | Browse into selected folder category |
| `o` | Open selected folder in file explorer |
| `y` | Copy folder path to clipboard |

### Folder Browser View

| Key | Action |
|-----|--------|
| `Enter` | Dive into subdirectory |
| `o` | Open current directory in file explorer |
| `y` | Copy current path to clipboard |

## UI Layout

```
┌─ Breadcrumb Bar ────────────────────────────────────┐
│  Library > The Witcher 3 > Proton Prefix > drive_c  │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Main Content Area                                  │
│  (scrollable list with selection highlight)         │
│                                                     │
│                                                     │
│                                                     │
│                                                     │
│                                                     │
├─ Status Bar ────────────────────────────────────────┤
│  / Filter  Enter Dive  o Explorer  y Copy  ? Help   │
└─────────────────────────────────────────────────────┘
```

### Optional: Split Pane Mode (Future)

For wider terminals, a two-pane layout like `ranger` or `midnight commander`:

```
┌─────────────────────┬───────────────────────────────┐
│  Game List          │  Folder Categories            │
│                     │                               │
│ > The Witcher 3     │  📁 Install Directory         │
│   Cyberpunk 2077    │  📁 Proton Prefix             │
│   Hades             │  📁 Shader Cache              │
│                     │  📁 Workshop Content          │
└─────────────────────┴───────────────────────────────┘
```

## Search & Filtering

- **Fuzzy matching** — pressing `/` opens an inline filter input at the bottom of the current list. As you type, items are filtered in real-time using fuzzy substring matching (similar to fzf).
- **Highlight matches** — matching characters in list items are highlighted.
- **Esc clears** — pressing `Esc` exits filter mode and restores the full list.
- **Persistent filter** — filter state is preserved per-view in the navigation stack, so going back restores the previous filter.

## Color Scheme

Default dark terminal theme with ANSI 256 colors (works in most terminals):

| Element | Color |
|---------|-------|
| Selected row background | Blue (ANSI 4) |
| Breadcrumb text | Cyan (ANSI 6) |
| Folder icons/names | Yellow (ANSI 3) |
| File names | Default foreground |
| Status bar background | Dark gray |
| Status bar hotkeys | Bold + underline first char |
| Filter input | Green (ANSI 2) |
| App ID | Dim / gray |

User-configurable theme override via config file.

## Accessibility

- Respect `NO_COLOR` environment variable.
- All information conveyed by color is also conveyed by text/position.
- Keyboard-only navigation (no mouse required, but mouse click-to-select is a nice-to-have).
