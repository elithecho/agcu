---
name: agcu
description: Operate desktop apps using the agcu CLI via accessibility APIs or Electron CDP. Use for opening apps, inspecting UI, clicking, typing, pressing keys, scrolling, dragging, reading state, managing windows, and computer-use smoke tests on macOS, Linux, or Windows. Trigger on requests such as "compute in Calculator", "read this window", "use agcu", or "control this desktop app". Follow the host's tool and safety policies.
license: MIT
metadata:
  author: elithecho
  version: '0.1.0'
  homepage: https://github.com/elithecho/agcu
---

# agcu — Agent Computer Use Skill

You have access to `agcu`, a CLI tool that controls desktop applications. You can click buttons, type text, read screens, scroll, drag files, move windows — all from the terminal.

## Install and check

```bash
npm install -g agcu@0.1.0
agcu --version
agcu check-permissions
agcu --help
```

The published package includes native binaries; Rust is not required. For a local source build, use `cargo install --path cli` from the repository root instead. Do not silently change approval settings or grant permissions.

## First-run setup (user action)

The first time you run an `agcu` command, Claude Code prompts the user to approve it. The user has three options, in order of preference:

1. **Easiest** — run `agcu setup` in a terminal. Interactive wizard that writes `Bash(agcu *)` to the right settings file. One-time. If the user seems frustrated by repeated approval prompts, suggest this.

   If you (the agent) are running it yourself and can't answer prompts, use the non-interactive flags:
   - `agcu setup --yes` → unsupervised + global (only if the user explicitly requests this approval policy)
   - `agcu setup --mode <unsupervised|supervised> --scope <global|project>` for specific combos

2. **Quickest** — pick _"Yes, and don't ask again for: `agcu *`"_ in the approval prompt. Covers only that specific subcommand pattern; you'll get prompted again for other subcommands.
3. **Manual** — add `"Bash(agcu *)"` to `~/.claude/settings.json` under `permissions.allow`.

Do not attempt to modify the user's settings file yourself — direct them to `agcu setup` instead.

## How to think

**Think like a human sitting at the computer.** Before you act, ask yourself: what would I see on screen? What would I click? What would I type?

A human:

1. Looks at the screen (snapshot)
2. Finds what they need (identify refs)
3. Does one action (click, type, key)
4. Checks what changed (re-snapshot)

You must do the same. Never skip steps. Never assume the UI didn't change after an action.

## Core loop

```
snapshot → identify → act → verify
```

```bash
agcu snapshot -a Music -i -c          # what's on screen?
# read the output, find the right @ref
agcu click @e5                         # do one thing
agcu snapshot -a Music -i -c          # what changed?
```

**Every action changes the UI.** Your previous refs are now stale. Always re-snapshot.

## Opening apps

Always wait for the app to be ready before doing anything:

```bash
agcu open Safari --wait
agcu snapshot -a Safari -i -c
```

Never interact with an app you haven't opened and snapshotted first.

## Finding elements

**Step 1**: Snapshot with `-i -c` (interactive + compact):

```bash
agcu snapshot -a Calculator -i -c
```

This shows only clickable/typeable elements with refs like `@e1`, `@e5`, `@e12`.

**Step 2**: Read the output. Find the element you need by its name, role, or id.

**Step 3**: Use the ref. Refs are the fastest and most reliable way to target elements.

If elements are missing, increase depth:

```bash
agcu snapshot -a Safari -i -c -d 8
```

## Clicking

For buttons, links, menu items — use `click`:

```bash
agcu click @e5                         # single click (AXPress, headless)
agcu click @e5 --count 2               # double-click (opens files, plays songs)
```

`click` tries AXPress first. If AXPress is unsupported, or for double/right clicks, it may fall back to mouse simulation and steal focus. A success response alone does not prove the intended UI state changed.

For elements with stable IDs (won't change between snapshots):

```bash
agcu click 'id="play"' -a Music
agcu click 'id~="track-123"' -a Music  # partial id match
```

## Typing

**With a target element** (preferred — uses AXSetValue, headless):

```bash
agcu type "hello world" -s @e3
```

**Into the focused field** (keyboard simulation, needs app focus):

```bash
agcu type "hello world" -a Safari
```

Always prefer `-s @ref` when you have a ref. It's more reliable.

## Key presses

```bash
agcu key Return -a Calculator
agcu key cmd+k -a Slack
agcu key cmd+a -a TextEdit
agcu key Escape -a Safari
```

## Scrolling

```bash
agcu scroll down -a Music              # scroll main content area
agcu scroll down --amount 10 -a Music  # scroll more
agcu scroll-to @e42                    # scroll element into view (headless)
```

Scroll needs the app to be focused. Use `scroll-to` for headless.

## Reading content

```bash
agcu text -a Calculator                # all visible text
agcu get-value @e5                     # one element's value/state
agcu get-value 'id="title"' -a Music   # by selector
```

Use `get-value` on specific elements instead of `text` on large apps.

## Window management

```bash
agcu move-window -a Notes --x 100 --y 100
agcu resize-window -a Notes --width 800 --height 600
agcu windows -a Finder                 # get window positions and sizes
```

These are instant and headless — use AXSetPosition/AXSetSize.

## Drag and drop

Drag needs the app to be focused and two visible, non-overlapping areas.

**Think like a human**: you need to see both the source and destination.

```bash
# Step 1: Set up windows side by side
agcu move-window -a Finder --x 0 --y 25
agcu resize-window -a Finder --width 720 --height 475
# (open a second Finder window for destination)

# Step 2: Snapshot to find the file
agcu snapshot -a Finder -i -c -d 8

# Step 3: Get the file's position
agcu get-value @e32                    # check position

# Step 4: Drag to destination
agcu drag @e32 @e50 -a Finder         # drag by refs
# or by coordinates:
agcu drag --from-x 300 --from-y 55 --to-x 1000 --to-y 200 -a Finder
```

## Selector syntax

### Refs (always prefer these)

```bash
@e1, @e2, @e3                         # from most recent snapshot
```

### DSL

```bash
'role=button name="Submit"'            # role + exact name
'name="Login"'                         # exact name
'id="AllClear"'                        # exact id (most stable)
'id~="track-123"'                      # id contains (case-insensitive)
'name~="Clear"'                        # name contains (case-insensitive)
'button "Submit"'                      # shorthand: role name
'"Login"'                              # shorthand: just name
'role=button index=2'                  # 3rd match (0-based)
'css=".my-button"'                     # CSS selector (Electron apps only)
```

### Chains

```bash
'id=sidebar >> role=button index=0'    # first button inside sidebar
'name="Form" >> button "Submit"'       # submit button inside form
```

## Electron apps (CDP)

Electron apps (Slack, Cursor, VS Code, Postman, Discord) are automatically detected. First use may relaunch them with a remote debugging port. Ask before disrupting an existing session; use `--no-cdp` for native-only inspection when relaunch is not authorized. Do not test against private messages or unsaved work without permission.

CDP actions can avoid native mouse input, but focus, app support, and delivery still need verification:

```bash
agcu snapshot -a Slack -i -c           # full DOM tree via CDP
agcu click @e5                         # JS element.click()
agcu key cmd+k -a Slack                # CDP key dispatch
agcu type "hello" -a Slack             # CDP insertText
agcu scroll down -a Slack              # JS scrollBy()
agcu text -a Slack                     # document.body.innerText
```

**Typing in Electron apps**: `insertText` goes to the focused element. If you need to type into a specific input:

```bash
agcu snapshot -a Slack -i -c           # find the input ref
agcu click @e18                        # click to focus it
agcu key cmd+a -a Slack                # select all
agcu key backspace -a Slack            # clear
agcu type "your text" -a Slack         # now type
```

## Waiting

When UI takes time to load:

```bash
agcu wait-for 'name="Dashboard"'       # poll until element appears
agcu wait-for 'role=button' --timeout 15
sleep 2                                # simple delay after navigation
```

## Batch operations

Chain multiple commands to avoid per-command startup:

```bash
echo '[["click","@e5"],["key","Return","-a","Music"]]' | agcu batch --bail
```

## Real-world patterns

### Search and play a song in Music (verified flow)

```bash
# 1. Open and snapshot
agcu open Music --wait
agcu snapshot -a Music -i -c
# → @e1 is the Search sidebar item

# 2. Click Search
agcu click @e1 -a Music

# 3. Type into the search field — use role=textField, not a ref (the ref
#    for the search field changes as the view switches)
agcu type "Espresso Sabrina Carpenter" -s 'role=textField' -a Music --submit
sleep 2  # let search results populate

# 4. Pick a result. Grep the snapshot for items matching the track name —
#    the `id` embeds a stable catalog id, so grab that.
agcu snapshot -a Music -c | grep -i "espresso" | head -5
# → [@e53] other("axcell") "Espresso" id=Music.shelfItem.TopSearchLockup[id=top-search-section-top-1744253558,...]

# 5. Open the album (double-click). Use the full id string, not the ref —
#    refs can drift between snapshots during long flows.
agcu click 'id="Music.shelfItem.TopSearchLockup[id=top-search-section-top-1744253558,parentId=top-search-section-top]"' -a Music --count 2
sleep 2

# 6. Find the track row and play. The track has a stable id pattern.
agcu snapshot -a Music -c | grep "track-lockup" | head -3
# → [@e52] group "Espresso" id=Music.shelfItem.AlbumTrackLockup[...]

# 7. Select, then try Return to play. If that fails, fall back to the
#    transport play button directly.
agcu click @e52 -a Music
agcu key Return -a Music
sleep 1

# 8. Verify via the transport button's `name` — "pause" means playing.
agcu find 'id="play"' -a Music --compact
# if name="play", playback didn't start. Fallback:
agcu click 'id="play"' -a Music
```

**Key lessons from this flow:**

- Refs (`@e53`) can go stale between snapshots separated by major UI changes. Prefer full `id="..."` for cross-snapshot targeting.
- `type -s 'role=textField' --submit` combines typing, clearing, and pressing Return reliably.
- Double-click on a search result often _opens_ the item, not _plays_ it. Drill into the detail view, then trigger playback.
- Always verify playback via the `name` of the transport button (`id="play"` is the slot; `name` holds state).
- If `Return` doesn't trigger playback, click the transport play button as a fallback. Have a plan B.

### Compute a multi-step calculation in Calculator

Inspect the current IDs first. On the tested macOS Calculator, the clear button changes **both name and ID**: `AllClear` when reset, `Clear` after input. Do not assume this ID is stable. For an isolated smoke test, `agcu key Escape -a Calculator` reset the input; re-snapshot to confirm before batching the observed digit/operator IDs.

```bash
# 1. Open and snapshot — Calculator buttons have stable ids (Seven, Multiply, Equals, etc.)
agcu open Calculator --wait
agcu snapshot -a Calculator -i -c

# 2. Reset separately, then inspect before batching stable IDs.
agcu key Escape -a Calculator
agcu snapshot -a Calculator -i -c
#    Example: 7^8 = 7×7×7×7×7×7×7×7 = 5,764,801
echo '[
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Equals\"","-a","Calculator"]
]' | agcu batch

# 3. Read the result — walk the tree for any staticText with a value.
agcu tree -a Calculator -d 8 --compact | python3 -c "
import json, sys
d = json.load(sys.stdin)
for n in [d]:
    def walk(n):
        if n.get('value'): print(n.get('role'), '=', repr(n['value']))
        for c in n.get('children', []): walk(c)
    walk(n)
"
# → staticText = '7×7×7×7×7×7×7×7'     ← expression
# → staticText = '57,64,801'             ← result (Indian locale; strip commas → 5764801)
```

**Key lessons:**

- Batch only observed, stable digit/operator IDs. Calculator's clear button is state-dependent (`AllClear` / `Clear`); reset and inspect it separately.
- The result isn't on a labeled element; walk the tree for nodes with a `value`.
- Numbers come locale-formatted and may contain U+200E. Normalize grouping and Unicode format characters before parsing.

### Open a DM in Slack and send a message

```bash
agcu key cmd+k -a Slack
sleep 1
agcu snapshot -a Slack -i -c
# find and click the search input
agcu click @e18
agcu key cmd+a -a Slack
agcu key backspace -a Slack
agcu type "Vukasin" -a Slack
sleep 1
agcu key Return -a Slack
sleep 2
agcu type "hey, check this out" -a Slack
agcu key Return -a Slack
```

### Check calendar events

```bash
agcu open Calendar --wait
agcu snapshot -a Calendar -i -c -d 6
# read the visible dates
agcu text -a Calendar
# navigate to next month
agcu click @e3                         # next month button
sleep 1
agcu text -a Calendar
```

### Fill a web form

```bash
agcu open Safari --wait
agcu snapshot -a Safari -i -c
agcu type "https://example.com/form" -s @e34
agcu key Return -a Safari
sleep 3
agcu snapshot -a Safari -i -c -d 8
agcu type "John Doe" -s @e5
agcu type "john@example.com" -s @e6
agcu type "Hello world" -s @e7
agcu click @e8                         # submit button
agcu snapshot -a Safari -i -c          # verify submission
```

### Drag a file between Finder windows

```bash
# set up side-by-side windows
agcu move-window -a Finder --x 0 --y 25
# (ensure two windows open, Downloads left, Desktop right)
agcu snapshot -a Finder -i -c -d 8
# find the file (look for textfield with val="filename")
agcu click @e32                        # select the file
agcu drag --from-x 300 --from-y 55 --to-x 1000 --to-y 200 -a Finder
```

### Browse App Store

```bash
agcu open "App Store" --wait
agcu snapshot -a "App Store" -i -c -d 10
agcu click 'id="AppStore.tabBar.discover"' -a "App Store"
sleep 2
agcu scroll down --amount 5 -a "App Store"
agcu snapshot -a "App Store" -i -c -d 10
agcu text -a "App Store"
```

## Rules

1. **Always snapshot before acting.** You cannot interact with what you cannot see.
2. **Re-snapshot after acting.** Refs may be stale. Exception: a bounded batch of observed stable IDs, with a state check immediately afterward.
3. **Refs for the next step, IDs for the long haul.** Refs (`@e5`) are fastest for the immediate next action after a snapshot. Full `id="..."` selectors survive across snapshots and are the right choice for anything you'll return to after other UI changes.
4. **Use `-i -c` on snapshots.** Interactive + compact reduces noise by 10x.
5. **Prefer `id=` over `name=` when the app provides stable ids.** IDs don't change with state; names often do.
6. **For state detection, read the `name` attribute — not `id`.** Many apps keep `id` as a fixed slot and flip `name` to reflect current state (play/pause, mute/unmute, bookmark/unbookmark).
7. **Wait after navigation.** `wait-for` a known landmark first; fall back to `sleep 2-3` if there's none.
8. **One action, then verify.** Only batch mutations when targets are observed, stable, and safe; verify the resulting state immediately afterward.
9. **Use `type -s @ref`** over `type -a App`. The selector path uses AXSetValue (reliable). The app path uses keyboard simulation (fragile, needs focus).
10. **Use `scroll-to @ref`** when you know the element. It's headless. `scroll down` needs focus.
11. **Never hardcode app-specific logic.** Don't write a helper like `play_song_in_music()`. Use the same primitives (`snapshot`, `click`, `type`, `key`, `find`) for every app. The scene tree tells you what to do — read it, act on it, verify.
12. **Have a fallback for every action.** If a `click` doesn't produce the expected state change, escalate: double-click → scroll-to + click → keyboard shortcut → direct action button (e.g., transport `id="play"` instead of track-row Return). See the Recovery playbook.
13. **Batch when ids are stable and numerous.** Calculator, forms, keypad flows benefit from piping a JSON array into `agcu batch` — one process start instead of N.

## Recovery playbook

When something doesn't work, the principle is: **diagnose the exact failure mode, then apply the matching fallback**. Never retry the same command blindly.

### Element not found (stale ref)

Symptom: `error: element not found matching SelectorChain { ... }` on a `@ref` that came from an earlier snapshot.

Cause: refs resolve to a specific tree path; major UI changes (navigation, scroll, modal open) invalidate them.

Fix: re-snapshot and use the new ref, or switch to a stable `id=`:

```bash
agcu snapshot -a Safari -i -c
# …find it again, or use the full id you saw earlier:
agcu click 'id="the-stable-id"' -a Safari
```

Prefer `id="..."` for any target you'll act on across multiple snapshots. Refs are for single-step actions only.

### Ambiguous selector

Symptom: `error: ambiguous selector: found N elements matching`.

Fix: add `index=` or narrow by role/name:

```bash
agcu click 'role=button index=0' -a Music
agcu click 'role=button name="Save"' -a TextEdit
```

### Click succeeded but nothing happened

Symptom: `{"success": true, "message": "clicked at (X,Y)"}` but the expected state change (new page, playback, menu) did not occur.

Diagnose in order:

1. **Was AXPress used or did it fall back to coordinates?**
   - `pressed "Name" at (X,Y)` = AXPress succeeded (best).
   - `clicked at (X,Y)` = coordinate click; the element might have moved or be obscured.
2. **Is the element's state actually what you think?** Read the `name` attribute of the state-bearing element (e.g., Music's transport button), not just the tree.
3. **Does the action require focus?** Some apps (especially web UI) need the app to be frontmost. `agcu open App --wait` activates it.

Fallback ladder:

```bash
# Ladder 1: AXPress failed — try double-click
agcu click @e5 --count 2

# Ladder 2: Element offscreen — bring it in, then click
agcu scroll-to @e5
agcu click @e5

# Ladder 3: The clicked element isn't the one that triggers the action —
#   drill into the container and find the explicit action button
agcu snapshot -a Music -c | grep -E "play|pause|Next"
agcu click 'id="play"' -a Music

# Ladder 4: Keyboard shortcut fallback
agcu key Return -a Music          # activate selected item
agcu key space -a Music           # play/pause
```

### Type didn't land

Symptom: text didn't appear in the field.

Causes + fixes:

```bash
# Cause: no ref — fell back to keyboard sim which needs app focus
agcu type "hello" -a Safari
# Fix: use selector path (AXSetValue, reliable)
agcu type "hello" -s @e3

# Cause: field wasn't focused when keyboard sim fired
agcu click @e3              # focus it first
agcu type "hello" -s @e3    # then type

# Cause: existing value not cleared
agcu type "new" -s @e3 --append   # wrong: keeps old text
agcu type "new" -s @e3            # right: clears first
```

### Search submitted but results aren't in the tree yet

Symptom: `snapshot` right after `--submit` shows the old page.

Fix: wait, then re-snapshot. Prefer polling over `sleep`:

```bash
agcu type "query" -s 'role=textField' -a Music --submit
agcu wait-for 'name~="Top Results"' --timeout 5
agcu snapshot -a Music -i -c
```

If `wait-for` doesn't know a stable landmark, fall back to `sleep 2`.

### Snapshot is too shallow — can't find the element

Bump depth:

```bash
agcu snapshot -a Safari -i -c -d 8
agcu snapshot -a Safari -i -c -d 12   # for deeply nested apps
```

### Ref points to the wrong thing

Symptom: `@e5` clicks something but it's not what you expected.

Cause: the snapshot re-ran silently (you took two) and refs got renumbered.

Fix: always inspect the ref's resolved element before acting on anything important:

```bash
agcu get-value @e5 -a Safari
# confirm role, name, id are what you expect, THEN click
```

### Electron app not using CDP

agcu auto-detects Electron apps. If CDP isn't working:

```bash
agcu snapshot -a Slack -i -c -v        # verbose shows CDP status
```

First run takes ~5s as the app relaunches with `--remote-debugging-port`. Every subsequent run uses a cached connection (~15ms).

### DSL selector parse error

Symptom: `unknown filter key: 'id_contains'`.

Fix: the syntax is `id~=` (tilde-equals) for partial match, not `id_contains=`:

```bash
agcu click 'id~="track-"' -a Music
agcu click 'name~="Submit"' -a Safari
```

## Screenshots and coordinate fallback

```bash
agcu screenshot -a Calculator --path /tmp/calculator.png
agcu screenshot -a Calculator --grid --path /tmp/calculator-grid.png
```

`--grid` is macOS-only. Labels are absolute screen points, not image pixels or window-relative coordinates. Use them with `agcu click --x X --y Y -a App` only after inspecting the capture. Recapture after a window moves or resizes, and verify delivery afterward. Prefer refs/selectors whenever possible.

Accessibility permission does not imply Screen Recording permission. If capture fails, report the error and ask the user to check Screen Recording permission and window availability; do not claim a screenshot exists. Keep captures app-scoped and outside Git to avoid exposing unrelated private content.

## Output format

Most commands emit JSON, but `snapshot` emits a text outline and `text` emits plain text. `batch` may emit each command's response followed by a final JSON results array: do not parse its entire stdout as one JSON document. Inspect every result's `success`/`error` even with `--bail`; a failed batch was observed exiting zero. These are example JSON responses:

```json
{"success": true, "message": "pressed \"7\" at (453, 354)"}
{"error": true, "type": "element_not_found", "message": "..."}
{"role": "button", "name": "Submit", "value": null, "position": {"x": 450, "y": 320}}
```
