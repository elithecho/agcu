use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SetupMode {
    /// Auto-approve all agcu commands (writes Bash(agcu *) to settings)
    Unsupervised,
    /// Keep default Claude Code behaviour — ask before every command
    Supervised,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SetupScope {
    /// ~/.claude/settings.json — applies to every project
    Global,
    /// ./.claude/settings.local.json — applies only to this directory
    Project,
}

#[derive(Parser)]
#[command(
    name = "agcu",
    bin_name = "agcu",
    version,
    about = "Computer use for AI agents",
    long_about = "agcu — Agent Computer Use: control any desktop app from the command line via the accessibility tree.\n\n\
                  Click buttons, type text, read screens — all without stealing focus.\n\
                  Built for AI agents. Works for humans too.\n\n\
                  All output is JSON by default (machine-readable). Use --human for pretty output.",
    after_help = "Quick start:\n  \
                  agcu apps                              # list running apps\n  \
                  agcu snapshot -a Calculator -i -c          # interactive elements with refs\n  \
                  agcu click @e5                          # click by ref\n  \
                  agcu type \"hello\" -s @e3                   # type into element\n  \
                  agcu key cmd+c --app Safari             # press key combo\n\n\
                  Docs: https://github.com/elithecho/agcu"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Human-readable output instead of JSON
    #[arg(long, global = true)]
    pub human: bool,

    /// Compact single-line JSON output
    #[arg(long, global = true)]
    pub compact: bool,

    /// Global timeout in seconds [default: 5]
    #[arg(long, global = true, default_value = "5")]
    pub timeout: f64,

    /// Show debug traces (selector resolution, timing)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Force CDP (Chrome DevTools Protocol) mode
    #[arg(long, global = true)]
    pub cdp: bool,

    /// CDP port override
    #[arg(long, global = true)]
    pub cdp_port: Option<u16>,

    /// Disable CDP auto-detection
    #[arg(long, global = true)]
    pub no_cdp: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Dump the accessibility tree for an app
    ///
    /// Shows all UI elements in a structured tree format.
    /// Use `snapshot` instead for AI agent workflows.
    #[command(alias = "t")]
    Tree {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process instead of an app name (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Max tree depth to traverse
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Find elements matching a selector
    ///
    /// Returns all matching elements as a JSON array.
    ///
    /// Examples:
    ///   agcu find 'role=button' --app Calculator
    ///   agcu find 'name~="Submit"' --app Safari
    ///   agcu find 'id="login-btn"' -d 5
    #[command(alias = "f")]
    Find {
        /// Selector DSL expression or @ref
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Max tree depth to search
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Click an element (background, no focus steal)
    ///
    /// Tries AXPress first (no mouse, no focus steal).
    /// Falls back to coordinate click if AXPress is unsupported.
    ///
    /// Examples:
    ///   agcu click @e5
    ///   agcu click 'name="Login"' --app Safari
    ///   agcu click @e5 --count 2        # double-click
    ///   agcu click @e5 --expect 'name="Dashboard"'
    #[command(alias = "c")]
    Click {
        /// Selector DSL expression or @ref
        selector: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Click at X coordinate
        #[arg(long)]
        x: Option<f64>,
        /// Click at Y coordinate
        #[arg(long)]
        y: Option<f64>,
        /// Mouse button: left (default), right, middle
        #[arg(long)]
        button: Option<String>,
        /// Number of clicks (2 = double-click)
        #[arg(long)]
        count: Option<u32>,
        /// Verify this element appears after clicking
        #[arg(long)]
        expect: Option<String>,
    },

    /// Clear a field and type text into it
    ///
    /// With --selector: uses AXValue (reliable, no focus steal).
    /// Without: simulates keypresses into the focused app.
    ///
    /// Examples:
    ///   agcu type "hello" -s @e3           # type into element (AXValue)
    ///   agcu type "hello" --app Safari     # keyboard simulation
    ///   agcu type "more" -s @e3 --append   # append without clearing
    #[command(name = "type")]
    Type {
        /// Text to type
        text: String,
        /// Target element (selector DSL or @ref)
        #[arg(short, long)]
        selector: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Press Return/Enter after typing
        #[arg(long)]
        submit: bool,
        /// Append text instead of replacing
        #[arg(long)]
        append: bool,
        /// Verify this element appears after typing
        #[arg(long)]
        expect: Option<String>,
    },

    /// Press a key or key combination
    ///
    /// Examples:
    ///   agcu key Return --app Calculator
    ///   agcu key cmd+c --app TextEdit
    ///   agcu key cmd+shift+p --app "VS Code"
    #[command(alias = "k")]
    Key {
        /// Key expression (e.g., Return, cmd+c, ctrl+shift+p)
        key: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Verify this element appears after key press
        #[arg(long)]
        expect: Option<String>,
    },

    /// Scroll an element into view using AXScrollToVisible
    ///
    /// Examples:
    ///   agcu scroll-to @e42
    ///   agcu scroll-to 'name="Submit"' --app Safari
    #[command(name = "scroll-to")]
    ScrollTo {
        /// Element to scroll into view (selector DSL or @ref)
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
    },

    /// Scroll within an app
    ///
    /// Examples:
    ///   agcu scroll down --app Music
    ///   agcu scroll down --amount 10 --app Music
    ///   agcu scroll down --at 'name="List"' --app Music
    #[command(alias = "s")]
    Scroll {
        /// Direction: up, down, left, right
        direction: String,
        /// Number of scroll ticks [default: 3]
        #[arg(long)]
        amount: Option<u32>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Position mouse at this element before scrolling
        #[arg(long, name = "at")]
        at_selector: Option<String>,
        /// Verify this element appears after scrolling
        #[arg(long)]
        expect: Option<String>,
    },

    /// Drag from one element/position to another
    Drag {
        /// Source element (selector DSL or @ref)
        from: Option<String>,
        /// Destination element (selector DSL or @ref)
        to: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Source X coordinate (if not using selector)
        #[arg(long)]
        from_x: Option<f64>,
        /// Source Y coordinate (if not using selector)
        #[arg(long)]
        from_y: Option<f64>,
        /// Destination X coordinate (if not using selector)
        #[arg(long)]
        to_x: Option<f64>,
        /// Destination Y coordinate (if not using selector)
        #[arg(long)]
        to_y: Option<f64>,
    },

    /// Wait for an element to appear
    ///
    /// Polls the accessibility tree at a regular interval.
    /// Returns the element when found, or errors on timeout.
    ///
    /// Examples:
    ///   agcu wait-for 'name="Dashboard"'
    ///   agcu wait-for 'role=button' --interval 500
    #[command(name = "wait-for", alias = "wait")]
    WaitFor {
        /// Element to wait for (selector DSL)
        selector: String,
        /// Poll interval in milliseconds [default: 200]
        #[arg(long, default_value = "200")]
        interval: u64,
    },

    /// Set text only if it differs from the current value
    ///
    /// Idempotent: won't type if the value already matches.
    ///
    /// Examples:
    ///   agcu ensure-text @e3 "hello@example.com"
    #[command(name = "ensure-text")]
    EnsureText {
        /// Target element (selector DSL or @ref)
        selector: String,
        /// Expected text value
        text: String,
    },

    /// Get an element's value, name, role, and position
    ///
    /// Lighter than `snapshot` — reads a single element.
    ///
    /// Examples:
    ///   agcu get-value @e3
    ///   agcu gv 'name="Email"' --app Safari
    #[command(name = "get-value", alias = "gv")]
    GetValue {
        /// Target element (selector DSL or @ref)
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
    },

    /// Launch an application
    ///
    /// Examples:
    ///   agcu open Safari --wait
    ///   agcu open Calculator
    Open {
        /// Application name
        app: String,
        /// Wait for the app window to appear
        #[arg(long)]
        wait: bool,
    },

    /// Execute a YAML workflow file
    ///
    /// Runs multi-step automation from a YAML file.
    /// See examples/ folder for workflow templates.
    ///
    /// Examples:
    ///   agcu run examples/calculator.yaml
    ///   agcu run workflow.yaml --app Calculator
    ///   agcu run workflow.yaml --dry-run
    #[command(alias = "r")]
    Run {
        /// Path to YAML workflow file
        file: String,
        /// Override the default app in the workflow
        #[arg(short, long)]
        app: Option<String>,
        /// Validate without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Interactive TUI tree explorer
    ///
    /// Navigate the accessibility tree in real time.
    /// Keys: j/k navigate, Enter expand, / search, y copy, q quit
    Observe {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Max tree depth [default: 10]
        #[arg(short, long, default_value = "10")]
        depth: u32,
        /// Refresh interval in seconds [default: 2.0]
        #[arg(long, default_value = "2.0")]
        refresh: f64,
    },

    /// Snapshot the accessibility tree with refs
    ///
    /// The primary command for AI agent workflows. Returns a
    /// compact tree with @refs (e.g., @e1, @e2) for each element.
    /// Use refs with other commands: `agcu click @e5`
    ///
    /// Examples:
    ///   agcu snapshot --app Calculator -i -c
    ///   agcu snapshot --app Music -i -c -d 8
    #[command(alias = "snap")]
    Snapshot {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Max tree depth
        #[arg(short, long)]
        depth: Option<u32>,
        /// Show only interactive elements (buttons, inputs, links)
        #[arg(short, long)]
        interactive: bool,
        /// Remove empty structural elements
        #[arg(short, long)]
        compact: bool,
    },

    /// Execute multiple commands from stdin
    ///
    /// Pipe a JSON array of commands to avoid per-command startup cost.
    ///
    /// Examples:
    ///   echo '[["click","@e5"],["click","@e8"]]' | agcu batch
    ///   echo '[["click","@e1"]]' | agcu batch --bail
    Batch {
        /// Stop on first error
        #[arg(long)]
        bail: bool,
    },

    /// Take a screenshot
    ///
    /// Examples:
    ///   agcu screenshot
    ///   agcu screenshot --path /tmp/screen.png
    ///   agcu screenshot --app Calculator
    Screenshot {
        /// Save path (auto-generated if omitted)
        #[arg(long)]
        path: Option<String>,
        /// Capture only this app's window
        #[arg(short, long)]
        app: Option<String>,
        /// Capture only this process's window (macOS)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Overlay absolute click coordinates every 100 screen points (macOS only).
        /// Without --app/--pid, capture only the main display.
        #[arg(long)]
        grid: bool,
    },

    /// List windows for an app
    Windows {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
    },

    /// Move an app's window to a position
    #[command(name = "move-window")]
    MoveWindow {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// X position
        #[arg(long)]
        x: f64,
        /// Y position
        #[arg(long)]
        y: f64,
    },

    /// Resize an app's window
    #[command(name = "resize-window")]
    ResizeWindow {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
        /// Width
        #[arg(long)]
        width: f64,
        /// Height
        #[arg(long)]
        height: f64,
    },

    /// Get the currently focused element
    Focused,

    /// Get all visible text from an app
    ///
    /// Traverses the full tree — use `get-value` for specific elements.
    Text {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Target a running process (macOS, native accessibility)
        #[arg(long, conflicts_with = "app", value_parser = clap::value_parser!(i32).range(1..))]
        pid: Option<i32>,
    },

    /// List running applications
    Apps,

    /// Check if accessibility permissions are granted
    #[command(name = "check-permissions", alias = "check")]
    CheckPermissions,

    /// Configure Claude Code / agent integrations
    ///
    /// Adds `Bash(agcu *)` to Claude Code's permissions allow-list so
    /// agcu runs without per-command approval prompts. Respects the
    /// trust boundary — nothing happens without explicit consent.
    ///
    /// Fully interactive by default. Use --mode and/or --scope to bypass
    /// individual prompts; --yes is shorthand for --mode unsupervised
    /// --scope global (scriptable for CI or AI agents).
    ///
    /// Examples:
    ///   agcu setup                                    # interactive
    ///   agcu setup --yes                              # unsupervised + global
    ///   agcu setup --mode unsupervised --scope project
    ///   agcu setup --mode supervised                  # no-op, explicit
    Setup {
        /// Skip prompts — alias for --mode unsupervised --scope global
        #[arg(short, long)]
        yes: bool,

        /// Pre-select the mode
        #[arg(long, value_enum)]
        mode: Option<SetupMode>,

        /// Pre-select the scope (only relevant when --mode is unsupervised)
        #[arg(long, value_enum)]
        scope: Option<SetupScope>,
    },

    /// Generate shell completions
    ///
    /// Output shell completions to stdout.
    ///
    /// Setup:
    ///   agcu completions bash > /etc/bash_completion.d/agcu
    ///   agcu completions zsh  > "${fpath[1]}/_agcu"
    ///   agcu completions fish > ~/.config/fish/completions/agcu.fish
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_options_reject_invalid_ids_and_app_conflicts() {
        let commands = [
            vec!["tree"],
            vec!["find", "role=button"],
            vec!["click", "role=button"],
            vec!["type", "hello"],
            vec!["key", "Return"],
            vec!["scroll-to", "role=button"],
            vec!["scroll", "down"],
            vec![
                "drag", "--from-x", "1", "--from-y", "2", "--to-x", "3", "--to-y", "4",
            ],
            vec!["get-value", "role=button"],
            vec!["observe"],
            vec!["snapshot"],
            vec!["screenshot", "--grid"],
            vec!["windows"],
            vec!["move-window", "--x", "1", "--y", "2"],
            vec!["resize-window", "--width", "100", "--height", "100"],
            vec!["text"],
        ];
        for command in commands {
            let mut args = vec!["agcu"];
            args.extend(command);
            args.extend(["--pid", "42"]);
            assert!(Cli::try_parse_from(&args).is_ok(), "{args:?}");
            for invalid in ["0", "-1", "2147483648", "not-a-pid"] {
                *args.last_mut().unwrap() = invalid;
                assert!(Cli::try_parse_from(&args).is_err(), "{args:?}");
            }
            *args.last_mut().unwrap() = "42";
            args.extend(["--app", "Calculator"]);
            let error = Cli::try_parse_from(&args)
                .err()
                .expect("--app conflicts with --pid");
            assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
        }
    }

    #[test]
    fn screenshot_options_and_plain_default() {
        let cli = Cli::try_parse_from([
            "agcu",
            "screenshot",
            "-a",
            "App",
            "--path",
            "shot.png",
            "--grid",
        ])
        .unwrap();
        match cli.command {
            Command::Screenshot {
                app, path, grid, ..
            } => {
                assert_eq!(app.as_deref(), Some("App"));
                assert_eq!(path.as_deref(), Some("shot.png"));
                assert!(grid);
            }
            _ => panic!("expected screenshot"),
        }
        for args in [
            vec!["agcu", "screenshot"],
            vec!["agcu", "screenshot", "--grid"],
        ] {
            let cli = Cli::try_parse_from(&args).unwrap();
            match cli.command {
                Command::Screenshot {
                    app, path, grid, ..
                } => {
                    assert_eq!(app, None);
                    assert_eq!(path, None);
                    assert_eq!(grid, args.len() == 3);
                }
                _ => panic!("expected screenshot"),
            }
        }
        assert!(Cli::try_parse_from(["agcu", "screenshot", "Take", "a", "screenshot"]).is_err());
    }
}
