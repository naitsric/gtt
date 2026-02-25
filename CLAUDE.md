# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build           # dev build
cargo build --release # release build (output: target/release/gtt)
cargo test            # run all tests
cargo check           # type-check without compiling
cargo clippy -- -D warnings  # lint (CI enforces zero warnings)
```

To run a single test:
```bash
cargo test test_name                        # match by test name substring
cargo test session::analyzer::tests        # all tests in a module
```

## Architecture

`gtt` is a CLI tool that estimates billable hours from git commit history. The data flow is:

**Config → Git log → Parse commits → Detect sessions → Aggregate by day → Render output**

### Module Structure

- **`src/main.rs`** — CLI entry point (clap), routes to command handlers
- **`src/config/`** — Loads `~/.config/gtt/config.toml` (TOML via serde). `types.rs` defines `Config`, `ClientConfig`, `Settings`.
- **`src/git/`** — Two responsibilities:
  - `log.rs`: runs `git log` via `std::process::Command`, filters by author email and date range
  - `parser.rs`: parses NUL-separated git log output into `Commit` structs (always uses `author_date`, not `commit_date`)
- **`src/session/`** — Core algorithm:
  - `analyzer.rs`: groups commits into `Session` objects (split on midnight crossings or gaps > `session_gap_minutes`), then groups sessions into `DayReport` via `group_by_day()`
  - `types.rs`: `Session`, `DayReport`, `ClientReport` structs with derived stats
- **`src/commands/`** — One file per CLI subcommand: `init`, `status`, `report`, `verify`, `export`, `config_cmd`
- **`src/output/`** — Rendering: `table.rs` (comfy-table), `csv.rs`, `json_fmt.rs`
- **`src/errors.rs`** — `GttError` enum via `thiserror`; all commands return `anyhow::Result`

### Session Algorithm (the core logic)

`src/session/analyzer.rs::analyze()` takes a flat `Vec<Commit>` (from one or more repos, already filtered by author email):

1. Sort commits by `author_date` ASC
2. Iterate pairs: if gap > `session_gap_minutes` (default 120 min) **or** dates differ (midnight crossing) → new session; otherwise add gap to current session duration
3. Each new session starts with `first_commit_minutes` (default 30 min) as base time

Tests for the session algorithm live in `tests/session_algorithm_test.rs` and inline in `src/session/analyzer.rs`. Tests for git log parsing are inline in `src/git/parser.rs`.

### Configuration

Config file: `~/.config/gtt/config.toml`

Key settings: `session_gap_minutes` (default 120), `first_commit_minutes` (default 30), `exclude_weekends` (default false), `bot_authors` (default: dependabot, github-actions, renovate bots).

Git log format uses NUL-separated fields with `END` markers to avoid issues with newlines in commit subjects: `hash\x00author_date\x00email\x00name\x00subject\x00END`.
