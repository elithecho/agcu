# agcu — Agent Computer Use

[![Stars](https://img.shields.io/github/stars/elithecho/agcu?style=flat&logo=github&color=a8d420)](https://github.com/elithecho/agcu/stargazers)
[![License](https://img.shields.io/github/license/elithecho/agcu?color=a8d420)](./LICENSE)
[![skills.sh](https://img.shields.io/badge/skills.sh-agcu-a8d420)](https://skills.sh)

> **One CLI. Any desktop app. Like a human would.**
> Click buttons, type text, read screens on macOS / Linux / Windows / Electron — no vision tokens, no screenshots, deterministic. For AI agents (Claude, Cursor, Codex, local models) and humans alike.

<p align="center">
  <img src="assets/demo.gif" alt="Desktop automation demo: opening Music, playing a song, launching Calculator, computing 7^8" width="720" />
</p>

```bash
# One prompt: "open Music, play Espresso by Sabrina Carpenter, then open Calculator and compute 7^8"
# Claude drives agcu end-to-end — no app-specific code, no vision, just a11y primitives
```

## Why agcu

|                     | **agcu**      | Anthropic Computer Use | OpenAI CUA | pyautogui |
| ------------------- | ------------- | ---------------------- | ---------- | --------- |
| Approach            | accessibility | vision                 | vision     | pixels    |
| Tokens per click    | **0**         | ~1500                  | ~1200      | 0         |
| Deterministic       | ✅            | ❌                     | ❌         | ✅        |
| Reads element state | ✅            | limited                | limited    | ❌        |
| Works on any app    | ✅ (a11y)     | ✅                     | ✅         | ✅        |
| Open source, local  | ✅            | ❌                     | ❌         | ✅        |

Built in Rust. Runs locally. Zero per-action cost.

## Origins

agcu derives from the original **[Kortix](https://github.com/kortix-ai)** computer-use
project. Original contributors and historical release notes retain their attribution.
The repository includes a skill for Claude Code, Cursor, Codex and other supported agents.

## Installation

### npm

`agcu@0.1.0` is published on npm with the complete CLI binaries.
`agcu@0.0.1` remains a reservation preview, not the complete CLI.

```bash
npm install -g agcu@0.1.0       # npm
pnpm add -g agcu@0.1.0          # pnpm
yarn global add agcu@0.1.0      # yarn
bun add -g agcu@0.1.0           # bun
```

Release packaging supports precompiled binaries for macOS (Apple Silicon / Intel),
Linux (x64 / arm64) and Windows (x64). Published bundles require no Rust toolchain.
The Linux binaries use GNU libc and dynamically link OpenSSL 3; they are not
Alpine/musl builds. CLI startup is checked in Debian 12 containers; this does not
verify desktop accessibility or input delivery on Linux or Windows.

After install, grant accessibility permissions:

```bash
agcu check-permissions
```

### Uninstall

```bash
npm uninstall -g agcu     # npm
pnpm remove -g agcu       # pnpm
yarn global remove agcu   # yarn
bun remove -g agcu        # bun
```

### From source

```bash
git clone https://github.com/elithecho/agcu
cd agcu
cargo install --path cli
```

Install Rust first from [rustup.rs](https://rustup.rs). Cargo installs the `agcu`
executable to `~/.cargo/bin` (ensure it is on `PATH`). `./scripts/setup.sh` is a
development helper: it checks Rust/Node, installs JS dependencies and builds
`./cli/target/release/agcu`; it does not install the executable on `PATH` or grant
permissions.

### Cargo

```bash
cargo install --git https://github.com/elithecho/agcu agcu
```

## Use with Claude Code, Cursor, Codex, Copilot, …

The repository includes the `agcu` skill in `skills/agcu/SKILL.md`. Install it
with the [skills.sh](https://skills.sh) CLI from a local checkout:

```bash
npx skills add . --skill agcu -a claude-code -g
```

The `-a claude-code` flag installs for Claude Code; drop it to pick interactively from [40+ supported agents](https://skills.sh) (Cursor, Codex, Copilot, OpenCode, Cline, VS Code, etc.). `-g` installs globally — available in every project.

### First-run setup (optional)

By default, Claude Code asks approval on every `agcu` command. To run without prompts, pick one of:

```bash
agcu setup    # interactive wizard — writes the allow rule for you
```

<details>
<summary>Or configure manually</summary>

Add to `~/.claude/settings.json`:

```json
{
  "permissions": {
    "allow": ["Bash(agcu *)"]
  }
}
```

Or, in the first approval prompt, pick _"Yes, and don't ask again for: `agcu *`"_ — covers that specific subcommand.

The rename installs only `agcu`, not an old-command alias. Existing old installs
are not removed automatically. Private Rust crate names and existing `.agent-cu`
ref/CDP caches are retained; old command allow rules do not approve `agcu`.

</details>

## Quick start

```bash
agcu apps                              # what's running?
agcu snapshot -a Calculator -i -c      # see interactive elements
agcu click @e5                         # click by ref
agcu type "hello" -s @e3               # type into a field
agcu text -a Music                     # read all visible text
```

The workflow: **snapshot → identify refs → act → re-snapshot to verify**.

### Target one process (macOS)

When multiple running app instances share a name, use the PID reported by `agcu apps`
or `agcu windows` instead of `--app`:

```bash
agcu apps
agcu snapshot --pid 12345 -i -c
agcu click @e5                         # refs retain the snapshot's PID
agcu type "hello" --pid 12345
agcu windows --pid 12345
agcu screenshot --pid 12345 --grid --path /absolute/path/shot.png
```

`--pid` is mutually exclusive with `--app` and requires a positive, running macOS
GUI process ID. It uses native accessibility even with `--cdp`/`--cdp-port`: a CDP
port does not establish which OS process owns the page. It does not relaunch apps.
Invalid or no-longer-running targets fail rather than falling back to another app.
Refs from another app/PID are rejected when combined with `--pid`; take a fresh
snapshot after the app restarts. PIDs identify processes, not individual windows
within one process; use window selectors for those.

Supported on tree, find, get-value, snapshot, observe, text, click, type, key,
scroll, scroll-to, drag, windows, move-window, resize-window, and screenshot.
Workflow-file `run --pid` and non-macOS PID targeting are not supported.
Coordinate/keyboard fallback can bring the target app forward and uses existing
Accessibility and System Events Automation permissions; no permissions are granted
automatically.

Reproduce process isolation with two disposable same-name Cocoa fixtures:
`python3 scripts/smoke-pid-targeting.py` (after `cd cli && cargo build -p agcu`).

## Commands

### Discovery

```bash
agcu apps                              # list all running apps
agcu snapshot -a Music -i -c           # interactive elements, compact
agcu snapshot -a Safari -d 8           # deeper tree
agcu tree -a Finder                    # raw accessibility tree (JSON)
agcu find 'role=button' -a Calculator  # find matching elements
agcu find 'id="play"' -a Music        # find by id
agcu find 'name~="Submit"' -a Safari  # find by partial name
agcu get-value @e5                     # read element value/state
agcu text -a Calculator                # all visible text
agcu focused                           # currently focused element
agcu windows -a Finder                 # list windows with positions
```

### Click

```bash
agcu click @e5                         # click by ref (AXPress, no focus steal)
agcu click 'name="Login"' -a Safari   # click by selector
agcu click 'id~="track-123"' -a Music # partial id match
agcu click @e5 --count 2              # double-click
agcu click @e5 --button right         # right-click
agcu click --x 500 --y 300 -a Finder  # coordinate click (last resort)
agcu click @e5 --expect 'name="Done"' # click then verify element appeared
```

### Type

```bash
agcu type "hello" -s @e3               # type into element (AXSetValue)
agcu type "hello" -a Safari            # type into focused field (keyboard sim)
agcu type "hello" -s @e3 --append      # append without clearing
agcu type "hello" -s @e3 --submit      # type then press Return
```

### Key

```bash
agcu key Return -a Calculator          # press a key
agcu key cmd+c -a TextEdit             # key combo
agcu key cmd+shift+p -a "VS Code"      # complex combo
agcu key Escape -a Slack               # escape
agcu key space -a Music                # play/pause
```

### Scroll

```bash
agcu scroll down -a Music              # scroll the main content area
agcu scroll down --amount 10 -a Music  # scroll more
agcu scroll-to @e42                    # scroll element into view
```

### Drag

```bash
agcu drag @e5 @e10 -a Finder                           # drag by refs
agcu drag 'name="file.txt"' 'name="Desktop"' -a Finder # drag by name
agcu drag --from-x 200 --from-y 55 --to-x 900 --to-y 300 -a Finder  # by coordinates
```

Drag uses smooth 20-step interpolation with easing — mimics natural mouse movement.

### Window management

```bash
agcu move-window -a Notes --x 100 --y 100    # move window (instant)
agcu resize-window -a Notes --width 800 --height 600  # resize (instant)
agcu open Calculator --wait                   # launch and wait for ready
agcu screenshot -a Music --path shot.png      # screenshot an app
agcu screenshot --path full.png               # full screen
agcu screenshot -a Music --path grid.png --grid # coordinate grid (macOS)
```

`--grid` overlays absolute screen coordinates every 100 points on macOS. Use the
top x labels and left y labels directly with `agcu click --x X --y Y -a App`;
the app flag does **not** make click coordinates relative to its window. The grid
accounts for the captured window's position and Retina image scale. Without
`-a`, grid capture includes only the main display. Plain screenshots are unchanged;
Linux and Windows return an unsupported-action error for `--grid`.

For example, a window at `(125, 80)` captured at 2× resolution places screen point
`(400, 300)` at image pixel `(550, 440)`, but the click still uses `--x 400 --y 300`.
Use numeric labels rather than resized image-viewer pixels. Recapture after moving
or resizing a window, and verify after clicking: app activation or UI changes can
invalidate the intended target. Coordinate clicks may move the mouse and steal
focus. Screenshots require the usual Screen Recording permission; no permissions
are granted automatically.

### Wait and verify

```bash
agcu wait-for 'name="Dashboard"'              # poll until element appears
agcu wait-for 'role=button' --timeout 15      # custom timeout
agcu ensure-text @e3 "hello"                  # only types if value differs
```

### Batch and workflow

```bash
echo '[["click","@e5"],["key","Return","-a","Music"]]' | agcu batch
echo '[["click","@e5"]]' | agcu batch --bail  # stop on first error
agcu run workflow.yaml                         # execute YAML workflow
```

### System

```bash
agcu check-permissions                 # verify accessibility access
agcu observe -a Calculator             # interactive TUI explorer
```

## Selectors

### Refs (fastest — from latest snapshot)

```bash
@e1, @e2, @e3
```

### Selector DSL

```bash
'role=button'                          # by role
'name="Login"'                         # exact name
'name~="Log"'                          # name contains (case-insensitive)
'id="submit-btn"'                      # exact id
'id~="track-123"'                      # id contains
'button "Submit"'                      # shorthand: role name
'"Login"'                              # shorthand: just name
'role=button index=2'                  # 3rd button (0-based)
'css=".my-button"'                     # CSS selector (CDP only)
```

### Chains (parent → child)

```bash
'id=sidebar >> role=button index=0'    # first button inside sidebar
'name="Form" >> button "Submit"'       # submit inside form
```

## CDP (Electron apps)

Electron apps (Slack, Cursor, VS Code, Postman, Discord, Notion) get automatic CDP support. agcu detects Electron apps, auto-relaunches them with a debug port, and connects via WebSocket.

```bash
agcu text -a Slack                     # just works — auto-detects Electron
agcu snapshot -a Slack -i -c           # DOM tree merged with native shell
agcu click @e5                         # JS element.click() via CDP
agcu key cmd+k -a Slack                # CDP Input.dispatchKeyEvent
agcu type "hello" -a Slack             # CDP Input.insertText
agcu scroll down -a Slack              # JS scrollBy()
```

First run auto-relaunches the Electron app with CDP (~5s). Every subsequent run uses cached connection (~15ms).

Override auto-detection:

```bash
agcu snapshot -a MyApp --cdp --cdp-port 9222   # force CDP with specific port
agcu snapshot -a Slack --no-cdp                 # disable CDP, use native only
```

## Output

All output is JSON by default.

```bash
agcu click @e5                         # {"success": true, "message": "pressed ..."}
agcu click @e5 --human                 # human-readable output
agcu click @e5 --compact               # single-line JSON
```

## Architecture

```
agcu/
├── cli/                    Rust CLI
│   ├── src/                Binary, handlers, actions, snapshot, wait, DSL parser
│   └── crates/
│       ├── agent-computer-use-core/      Platform trait, AccessibilityNode, Selector, element utils
│       ├── agent-computer-use-macos/     macOS backend (AXUIElement, CGEvent, batch attribute fetch)
│       ├── agent-computer-use-cdp/       CDP bridge (WebSocket, DOM walker, JS interaction)
│       ├── agent-computer-use-linux/     Linux backend (AT-SPI2)
│       └── agent-computer-use-windows/   Windows backend (UIAutomation)
├── docs/                   Next.js docs site
├── benchmark/              E2E benchmarks, stress tests, comparison tools
├── docker/                 Cross-compilation Dockerfiles
└── scripts/                Setup, build, bench
```

## Development

```bash
pnpm build          # build CLI (release)
pnpm test           # Rust workspace + package/launcher tests
pnpm lint           # cargo fmt --check + clippy
pnpm format         # auto-format everything
pnpm bench          # criterion micro-benchmarks
pnpm bench:e2e      # real-world app benchmarks
pnpm bench:stress   # reliability stress test
pnpm bench:diff     # compare benchmark runs
```

### Screenshot grid smoke test (macOS)

Open a harmless isolated app window, such as a local HTML fixture in Chrome.
From the repository root, build the debug CLI and use a fresh absolute evidence
directory outside the repository:

```sh
cargo build --manifest-path cli/Cargo.toml
node scripts/test-screenshot-grid.mjs 'Google Chrome' /absolute/path/to/fresh-safe-evidence
```

This checks plain/grid screenshot responses, PNG dimensions and changed output,
default paths, and missing-app/invalid-path errors. It saves screenshots and
`smoke-results.json`; it does **not** reproduce the blind accuracy pilot below
or test native click delivery.
Keep proof artifacts outside Git and review screenshots for private content before
sharing. The pilot fixtures and evaluation utilities are not bundled with the CLI.

### Screenshot grid accuracy pilot

Ten fresh seeded harmless HTML scenes were captured with the source-built macOS
CLI. Five blind observers used GPT-6 Luna (high), and five used GPT-6.1 Sol
(medium), each on a distinct full-grid image with one requested labelled square
and four distractors. Requested target sizes were 8, 12, 12, 20 and 40 CSS pixels
per model. Observers received no actual target bounds; one central GPT-6.1 Sol
(medium) scorer checked frozen absolute-coordinate estimates against withheld,
calibrated actual DOM bounds.

```sh
./cli/target/debug/agcu screenshot --no-cdp --app 'Google Chrome' --grid --path /absolute/path/to/safe-evidence/trial-grid.png
```

**All 10 frozen predictions were in bounds; zero misses.** Center error is the
Euclidean distance from the prediction to the calibrated actual target center,
in native screen points:

| Dispatch-selected observer | In-bounds predictions | Median error | p90 error | Error range       |
| -------------------------- | --------------------- | ------------ | --------- | ----------------- |
| GPT-6 Luna, high           | 5/5                   | 0.398361     | 0.538779  | 0.118996–0.538779 |
| GPT-6.1 Sol, medium        | 5/5                   | 0.325135     | 0.456346  | 0.200098–0.456346 |

The p90 uses nearest rank; with five trials per model it equals the maximum.
Setup: macOS 26.6.2 / Chrome 155, source images 1100×1050 pixels at absolute
origin `(74,210)`, one pixel per screen point. Independently calibrated CSS
content origin was `(74,297)` with unit axes and held-out residual `(0,0)` points.
In-bounds scoring uses `left <= x < right` and `top <= y < bottom` in absolute
screen coordinates, not image-viewer pixels. Source images, frozen predictions,
hashes, capture commands and per-trial CSV/JSON scores were retained outside Git.

A separate temporary judging utility generated PNG/SVG copies and a gallery
**after predictions were frozen and scored**: magenta **+** marks the actual
center; a yellow/black **dot** marks the prediction. All ten trials are included,
original captures remain unchanged, and truth-bearing copies were never observer
inputs. This is secondary visual verification, not a product flag or a utility
shipped in this repository.

These are **saved-image coordinate scores, not native click-delivery results**:
zero native trial clicks and no click-count criterion. Coordinate accuracy does
not guarantee event delivery or establish that delivery issues are fixed. This
small unpaired, full-grid-only pilot is not a statistical model ranking, a crop
comparison or a general accuracy guarantee. Variants describe dispatch selection;
runtime variants and image preprocessing were not instrumented. The grid is
generic annotation, not automated CAPTCHA solving.

## Platform support

| Platform               | Status  |
| ---------------------- | ------- |
| macOS (Apple Silicon)  | Preview |
| macOS (Intel)          | Preview |
| Electron apps (CDP)    | Preview |
| Windows (UIAutomation) | Preview |
| Linux (AT-SPI2)        | Preview |

## Star history

<a href="https://star-history.com/#elithecho/agcu&Date">
  <img src="https://api.star-history.com/svg?repos=elithecho/agcu&type=Date" alt="Star history" width="620" />
</a>

## License

MIT
