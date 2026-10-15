# agcu installation and permissions

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
