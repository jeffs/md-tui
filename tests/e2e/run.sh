#!/usr/bin/env bash
#
# E2E visual regression test harness for mdt using tmux.
#
# Usage:
#   tests/e2e/run.sh                   # run all scenarios
#   UPDATE_SNAPSHOTS=1 tests/e2e/run.sh  # regenerate expected snapshots
#
# Requirements: tmux, cargo (for building)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SNAPSHOTS_DIR="$SCRIPT_DIR/snapshots"
FIXTURES_DIR="$REPO_ROOT/tests/fixtures"
BINARY="$REPO_ROOT/target/debug/mdt"
SESSION_PREFIX="mdt_e2e_$$"
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# ── Pre-flight checks ──────────────────────────────────────────────

if ! command -v tmux &>/dev/null; then
    echo "SKIP: tmux not found — E2E tests require tmux"
    exit 0
fi

echo "Building mdt…"
cargo build --manifest-path="$REPO_ROOT/Cargo.toml" 2>&1
if [[ ! -x "$BINARY" ]]; then
    echo "FAIL: binary not found at $BINARY after build"
    exit 1
fi

mkdir -p "$SNAPSHOTS_DIR"

# ── Helpers ─────────────────────────────────────────────────────────

# Isolated HOME so the user's config.toml does not affect tests.
# The MDT_* env vars below provide all needed configuration.
E2E_HOME="$(mktemp -d)"
trap 'rm -rf "$E2E_HOME"' EXIT

# start_mdt SESSION_NAME [ARGS...]
#   Creates a detached tmux session with 80x24 geometry and runs mdt inside it.
#   Uses a clean HOME to avoid the host's config file.
start_mdt() {
    local session="$1"; shift
    tmux new-session -d -s "$session" -x 80 -y 24 \
        "HOME=$E2E_HOME MDT_FLAVOR=commonmark MDT_WIDTH=80 $BINARY $*; sleep 86400"
    # Wait for mdt to render its first frame
    sleep 2
}

# send_keys SESSION_NAME KEYS...
#   Sends key events to the tmux session.
send_keys() {
    local session="$1"; shift
    for key in "$@"; do
        tmux send-keys -t "$session" "$key"
        sleep 0.15
    done
}

# capture SESSION_NAME
#   Captures the pane content, stripping trailing whitespace from each line
#   and trailing empty lines.
capture() {
    local session="$1"
    # Strip trailing whitespace per line, then remove trailing blank lines.
    # Uses awk instead of GNU sed for macOS/BSD portability.
    tmux capture-pane -p -t "$session" | sed 's/[[:space:]]*$//' | awk '
        { lines[NR] = $0 }
        /./ { last = NR }
        END { for (i = 1; i <= last; i++) print lines[i] }
    '
}

# compare SCENARIO_NAME ACTUAL_OUTPUT
#   Compares captured output against the snapshot file.
#   If UPDATE_SNAPSHOTS=1, writes the captured output as the new expected snapshot.
#   Returns 0 on match, 1 on mismatch/missing snapshot.
compare() {
    local name="$1"
    local actual="$2"
    local snapshot_file="$SNAPSHOTS_DIR/${name}.txt"

    if [[ "${UPDATE_SNAPSHOTS:-0}" == "1" ]]; then
        printf '%s\n' "$actual" > "$snapshot_file"
        echo "  UPDATED snapshot: $snapshot_file"
        return 0
    fi

    if [[ ! -f "$snapshot_file" ]]; then
        echo "  MISSING snapshot: $snapshot_file"
        echo "  Run with UPDATE_SNAPSHOTS=1 to create it."
        return 1
    fi

    local expected
    expected="$(cat "$snapshot_file")"

    if [[ "$actual" == "$expected" ]]; then
        return 0
    else
        echo "  DIFF (expected vs actual):"
        diff --color=auto <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") || true
        return 1
    fi
}

# cleanup SESSION_NAME
#   Kills the tmux session, ignoring errors if already dead.
cleanup() {
    local session="$1"
    tmux kill-session -t "$session" 2>/dev/null || true
}

# run_scenario SCENARIO_NAME BODY_FUNCTION
#   Wraps a test scenario with pass/fail reporting and cleanup.
run_scenario() {
    local name="$1"
    local body_fn="$2"
    local session="${SESSION_PREFIX}_${name}"

    echo "--- $name ---"
    if "$body_fn" "$session" "$name"; then
        echo "  PASS"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "  FAIL"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    cleanup "$session"
}

# start_mdt_stdin SESSION_NAME INPUT_TEXT
#   Pipes input text to mdt via stdin inside a tmux session.
start_mdt_stdin() {
    local session="$1"
    local input="$2"
    # Use printf with %s to avoid interpretation of backslashes/specials.
    # The outer double-quotes around the tmux command handle variable expansion.
    tmux new-session -d -s "$session" -x 80 -y 24 \
        "printf '%s' \"$input\" | HOME=$E2E_HOME MDT_FLAVOR=commonmark MDT_WIDTH=80 $BINARY; sleep 86400"
    sleep 1
}

# ── Test scenarios ──────────────────────────────────────────────────

# basic_render: Open kitchen_sink.md, capture the initial render.
scenario_basic_render() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# scroll_down_up: Open kitchen_sink.md, press j j j k, capture.
scenario_scroll_down_up() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    send_keys "$session" j j j k
    sleep 0.3
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# search_flow: Open kitchen_sink.md, press f, type "bold", Enter, capture.
scenario_search_flow() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    send_keys "$session" f
    sleep 0.2
    tmux send-keys -t "$session" -l "bold"
    sleep 0.2
    send_keys "$session" Enter
    sleep 0.3
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# search_no_results: Open kitchen_sink.md, press f, type "zzzzz", Enter, capture error modal.
scenario_search_no_results() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    send_keys "$session" f
    sleep 0.2
    tmux send-keys -t "$session" -l "zzzzz"
    sleep 0.2
    send_keys "$session" Enter
    sleep 0.3
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# link_select: Open kitchen_sink.md, press s, capture with link highlighted.
scenario_link_select() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    send_keys "$session" s
    sleep 0.3
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# quit_from_view: Open kitchen_sink.md directly, press q, capture file tree or exit.
scenario_quit_from_view() {
    local session="$1" name="$2"
    start_mdt "$session" "$FIXTURES_DIR/kitchen_sink.md"
    send_keys "$session" q
    sleep 0.3
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# help_menu_false: Open kitchen_sink.md with help_menu=false via config, verify
# content fills the space that the help menu would have occupied.
scenario_help_menu_false() {
    local session="$1" name="$2"
    # Write a config with help_menu disabled into the isolated HOME.
    mkdir -p "$E2E_HOME/.config/mdt"
    printf 'help_menu = false\n' > "$E2E_HOME/.config/mdt/config.toml"
    tmux new-session -d -s "$session" -x 80 -y 24 \
        "HOME=$E2E_HOME MDT_FLAVOR=commonmark MDT_WIDTH=80 $BINARY $FIXTURES_DIR/kitchen_sink.md; sleep 86400"
    sleep 2
    local output
    output="$(capture "$session")"
    # Restore clean config for subsequent scenarios.
    rm -f "$E2E_HOME/.config/mdt/config.toml"
    compare "$name" "$output"
}

# stdin_pipe: Pipe "# Hello" to mdt, capture rendered output.
scenario_stdin_pipe() {
    local session="$1" name="$2"
    start_mdt_stdin "$session" "# Hello"
    local output
    output="$(capture "$session")"
    compare "$name" "$output"
}

# ── Run all scenarios ──────────────────────────────────────────────

run_scenario "basic_render"      scenario_basic_render
run_scenario "scroll_down_up"    scenario_scroll_down_up
run_scenario "search_flow"       scenario_search_flow
run_scenario "search_no_results" scenario_search_no_results
run_scenario "link_select"       scenario_link_select
run_scenario "quit_from_view"    scenario_quit_from_view
run_scenario "help_menu_false"   scenario_help_menu_false
run_scenario "stdin_pipe"        scenario_stdin_pipe

# ── Summary ─────────────────────────────────────────────────────────

echo ""
echo "=============================="
echo "E2E Results: $PASS_COUNT passed, $FAIL_COUNT failed, $SKIP_COUNT skipped"
echo "=============================="

if [[ "$FAIL_COUNT" -gt 0 ]]; then
    exit 1
fi
exit 0
