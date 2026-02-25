# gtt — Git Time Tracker

> Estimate hours worked and billable amounts directly from your commit history.

```bash
$ gtt report --client "Startup X" --last-month

Client: Startup X
Period: 01/01/2026 — 01/31/2026

+-----------+----------+--------+---------+------------------------------+
| Date      | Sessions | Hours  | Commits | Repos                        |
+-----------+----------+--------+---------+------------------------------+
| Mon 01/05 |        2 | 3h 15m |       5 | startupx-web                 |
| Tue 01/06 |        1 | 1h 40m |       3 | startupx-api                 |
| Wed 01/07 |        3 | 4h 50m |       8 | startupx-web, startupx-api   |
| Fri 01/09 |        1 | 2h 10m |       4 | startupx-web                 |
| Total     |        7 | 11h 55m|      20 |                              |
+-----------+----------+--------+---------+------------------------------+

Amount: 11.92h × 80/h = 953.33 USD
```

---

## The Problem

Freelancers who bill by the hour constantly lose money because manually reconstructing time worked is painful. Manual timers are easily forgotten. Eyeball estimates often fall short.

**What you already have:** every commit has an exact timestamp. `gtt` analyzes them to detect work sessions and calculate real hours.

**Why alternatives fall short:**

| Tool | Problem |
|---|---|
| `git-hours` | No clients or rates. Abandoned (broken on Node 18+). |
| GTM | Requires editor plugins. Loses data on rebase/squash. |
| WakaTime | $8/month, sends code to external servers. |
| Toggl / Clockify | Require manual timers — the original problem. |

`gtt` is the only tool that goes from **repos → sessions → hours → billable amount** in a single command, with no servers or subscriptions.

---

## Installation

### From binary (recommended)

Download the binary for your platform from the [releases page](https://github.com/naitsric/gtt/releases):

```bash
# Linux / macOS
curl -L https://github.com/naitsric/gtt/releases/latest/download/gtt-linux-x86_64 -o gtt
chmod +x gtt
sudo mv gtt /usr/local/bin/
```

### With Cargo

```bash
cargo install gtt
```

### From source

```bash
git clone https://github.com/naitsric/gtt
cd gtt
cargo build --release
# The binary is placed at target/release/gtt
```

---

## Quick Start

**1. Configure clients and repos:**

```bash
gtt init
```

The wizard asks for your clients, the paths to their repositories, and your hourly rate. It automatically generates `~/.config/gtt/config.toml`.

**2. View today's and this week's hours:**

```bash
gtt status
```

**3. View last month's report:**

```bash
gtt report --client "Startup X" --last-month
```

**4. Verify sessions before billing:**

```bash
gtt verify --client "Startup X" --last-month
```

**5. Export to CSV for your billing system:**

```bash
gtt export --client "Startup X" --last-month --format csv
# Generates: gtt-startup-x-2026-01.csv
```

---

## Configuration

The configuration file lives at `~/.config/gtt/config.toml`:

```toml
[client."Startup X"]
repos = [
    "/home/user/startupx-web",
    "/home/user/startupx-api",
]
hourly_rate = 80
currency = "USD"

[client."Agency Y"]
repos = ["/home/user/agency-landing"]
hourly_rate = 60
currency = "EUR"

[settings]
session_gap_minutes = 120   # inactivity > 2h = new session
first_commit_minutes = 30   # base time for the first commit of a session
exclude_weekends = false     # avoid crossing sessions over the weekend
```

### `[settings]` Options

| Option | Default | Description |
|---|---|---|
| `session_gap_minutes` | `120` | Minutes of inactivity that start a new session |
| `first_commit_minutes` | `30` | Base minutes assigned to the first commit of each session |
| `exclude_weekends` | `false` | Prevents crossing sessions between Friday and Monday |
| `bot_authors` | `["dependabot[bot]", ...]` | Authors excluded from the analysis |

To edit the config directly:

```bash
gtt config show    # View current configuration
gtt config edit    # Open in $EDITOR
```

---

## Commands

### `gtt init`

Interactive setup. Creates or overwrites `~/.config/gtt/config.toml`.

```bash
gtt init
```

---

### `gtt status`

Quick summary of hours for today and this week per client.

```bash
gtt status

# gtt status
#   Today: 02/25/2026    This week: 02/23/2026 — 02/25/2026
#
#   Startup X — Today: 2h 30m  (4 commits)   This week: 8h 15m  (14 commits)
#   Agency Y  — Today: 0m      (0 commits)   This week: 3h 40m  (7 commits)
```

---

### `gtt report`

Hours report per client. Supports multiple formats and date ranges.

```bash
# Last month (most common for billing)
gtt report --client "Startup X" --last-month

# Last week
gtt report --client "Startup X" --last-week

# Custom range
gtt report --client "Startup X" --since 2026-01-01 --until 2026-01-31

# All clients
gtt report --last-month

# CSV Format (prints to stdout)
gtt report --client "Startup X" --last-month --format csv

# JSON Format
gtt report --client "Startup X" --last-month --format json

# Save to a specific file
gtt report --client "Startup X" --last-month --format csv --output january-2026.csv
```

**Available Flags:**

| Flag | Description |
|---|---|
| `--client <name>` | Filter by client. Without flag, reports all. |
| `--last-week` | Last week (Monday to Sunday) |
| `--last-month` | Previous calendar month |
| `--since <YYYY-MM-DD>` | Range start |
| `--until <YYYY-MM-DD>` | Range end |
| `--format <fmt>` | `table` (default), `csv`, `json` |
| `--output <file>` | Save to file instead of stdout |

---

### `gtt verify`

Lists detected sessions with timestamps and included commits. Use it to validate that the analysis matches your perception before billing.

```bash
gtt verify --client "Startup X" --last-month

# Verify sessions: Startup X
# Period: 01/01/2026 — 01/31/2026
#
# ── Monday 01/05/2026 (2 sessions, 3h 15m) ──
#   Session 1:  09:15 → 10:45  (1h 30m, 3 commits)
#     09:15 a3f2e1b feat: add user authentication
#     09:52 b1c4d5e fix: handle invalid tokens
#     10:45 c2d3e4f test: authentication edge cases
#
#   Session 2:  15:30 → 17:00  (1h 45m, 2 commits)
#     15:30 d4e5f6a refactor: extract auth service
#     17:00 e5f6a7b docs: update API documentation
```

**Flags:** Same as `gtt report` (except `--format` and `--output`).

---

### `gtt export`

Alias for `report` with automatic file name generation.

```bash
# Generates gtt-startup-x-2026-01.csv in the current directory
gtt export --client "Startup X" --last-month --format csv

# JSON
gtt export --client "Startup X" --last-month --format json

# Custom name
gtt export --client "Startup X" --last-month --format csv --output january-invoice.csv
```

The generated name follows the pattern `gtt-<client>-<YYYY-MM>.<format>`.

---

### `gtt config`

```bash
gtt config show   # Prints the current config
gtt config edit   # Opens in $EDITOR (or nano if undefined)
```

---

## How the sessions algorithm works

`gtt` analyzes your commit history to infer when you worked:

1. **Sorts** all commits by `author date` (not commit date — making it robust against `git rebase` and `git commit --amend`).

2. **Detects sessions** by comparing consecutive pairs of commits:
   - If the gap is **> `session_gap_minutes`** (default: 2 hours) → new session
   - If the pair **crosses midnight** → new session (even if the gap is smaller)
   - Otherwise → same work block, the gap counts as time worked

3. **Adds base time** to the first commit of each session (`first_commit_minutes`, default: 30 min), to account for the time spent before the first commit.

4. **Excludes bots**: Commits from Dependabot, GitHub Actions, and similar are ignored by default.

```
Commits:  09:00  09:45  10:30        15:00  15:20
          |------|------|             |------|
          45min  45min               20min
          ←  session 1  →            ← session 2 →

Session 1: 30min base + 45 + 45 = 2h 0m
Session 2: 30min base + 20 = 50m
Total:     2h 50m
```

> **Note:** `gtt` produces **estimates**, not exact records. Use `gtt verify` to review the detected sessions before billing. The README for each report suggests reviewing it with the client if there are disputes.

---

## Export Formats

### CSV

Compatible with FreshBooks, Wave, Invoice Ninja, and any spreadsheet.

```csv
date,sessions,hours,minutes,commits,repos,amount,currency
2026-01-05,2,3.2500,195,5,startupx-web,260.00,USD
2026-01-06,1,1.6667,100,3,startupx-api,133.33,USD
```

### JSON

```json
{
  "client": "Startup X",
  "period_start": "2026-01-01",
  "period_end": "2026-01-31",
  "total_minutes": 715,
  "total_hours": 11.92,
  "total_commits": 20,
  "hourly_rate": 80.0,
  "currency": "USD",
  "billable_amount": 953.33,
  "days": [
    {
      "date": "2026-01-05",
      "sessions": 2,
      "total_minutes": 195,
      "total_hours": 3.25,
      "total_commits": 5,
      "repos": ["startupx-web"],
      "amount": 260.0
    }
  ]
}
```

---

## Advanced Use Cases

### Repos shared with other devs

`gtt` automatically filters by the email configured in `git config user.email` for each repository. Only your commits count.

### Multiple repos per client

```toml
[client."Startup X"]
repos = [
    "/home/user/startupx-web",
    "/home/user/startupx-api",
    "/home/user/startupx-mobile",
]
hourly_rate = 80
currency = "USD"
```

Commits from all repos are combined, and sessions are detected across them. A commit to `api` and a commit to `web` 40 minutes apart are part of the same session.

### Billing Workflows

```bash
# 1. Review detected sessions
gtt verify --client "Startup X" --last-month

# 2. If everything looks good, generate the report
gtt report --client "Startup X" --last-month

# 3. Export to import into your billing system
gtt export --client "Startup X" --last-month --format csv
```

---

## FAQ

**What happens if I used `git rebase` or `git commit --amend`?**
`gtt` always uses the `author date` (the original date of the commit), not the `commit date` (which changes with rebase/amend). Your hours are robust against any history rewriting.

**Why are my commits not showing up?**
`gtt` filters by the author email configured in `git config user.email`. Verify that the email in your repo matches the author of the commits: `git log --format="%ae" | head -5`.

**Do bot commits inflate the time?**
No. Commits from Dependabot, GitHub Actions, and any author ending in `[bot]` are automatically excluded. You can add more exclusions in `bot_authors` in the config.

**Is it accurate?**
It's an estimate. The algorithm cannot know how much time you spent thinking before the first commit or reviewing code without committing. That's why the `first_commit_minutes` parameter exists — and why `gtt verify` allows you to review the sessions before billing.

**Does my data leave my machine?**
Never. `gtt` is a binary that runs completely locally. It has no servers, no telemetry, no network required.

---

## Contributing

```bash
git clone https://github.com/naitsric/gtt
cd gtt
cargo test        # run all tests
cargo check       # verify without compiling
cargo build       # dev build
```

The session algorithm tests are in `tests/session_algorithm_test.rs`. If you modify `src/session/analyzer.rs`, add tests for the edge cases you are covering.

---

## License

MIT — use it, modify it, distribute it.
