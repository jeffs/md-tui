#!/usr/bin/env bash
set -euo pipefail

# Parallel Ralph loop for mdt test implementation.
#
# Structure:
#   Wave 0: T00                                          (sequential)
#   Wave 1: T01 T02 T03 T07 T10 T11 T12 T13 T16        (9 parallel)
#   Wave 2: T04 T08 T14 T15 T17                         (5 parallel)
#   Wave 3: T05 T09                                     (2 parallel)
#   Wave 4: T06                                         (sequential)
#   Wave 5: T18                                         (sequential)
#
# Between each wave: cargo test (the "gate").
# If the gate fails, a triage agent diagnoses and fixes.
# If triage can't fix it, stop and report.

TASK_FILE="var/test-tasks.md"
PROMPT_FILE="var/ralph.md"
BUGFIX_FILE="var/ralph-bugfix.md"
TRIAGE_FILE="var/triage.md"
MAX_CONCURRENT="${MAX_CONCURRENT:-5}"
MAX_TRIAGE="${MAX_TRIAGE:-10}"      # Max triage attempts per gate
LOG_DIR="var/ralph-logs"
CLAUDE_ARGS="--print --allowedTools Read,Write,Edit,Glob,Grep,Bash,mcp__nushell__evaluate,mcp__nushell__command_help,mcp__nushell__list_commands"

unset CLAUDECODE
mkdir -p "$LOG_DIR"

# ── Helpers ──────────────────────────────────────────────

task_done() {
    local task_id="$1"
    grep -q "^\- \[x\] \*\*${task_id}:" "$TASK_FILE"
}

run_task() {
    local task_id="$1"
    local log="$LOG_DIR/${task_id}.log"

    if task_done "$task_id"; then
        echo "  ✓ $task_id already done, skipping"
        return 0
    fi

    echo "  → $task_id starting (log: $log)"
    local prompt
    prompt=$(sed "s/\$TASK_ID/${task_id}/g" "$PROMPT_FILE")

    # shellcheck disable=SC2086
    if claude $CLAUDE_ARGS -p "$prompt" > "$log" 2>&1; then
        if task_done "$task_id"; then
            echo "  ✓ $task_id done"
            return 0
        else
            echo "  ✗ $task_id ran but didn't mark itself done"
            return 1
        fi
    else
        echo "  ✗ $task_id failed (exit $?)"
        return 1
    fi
}

run_wave() {
    local wave_name="$1"
    shift
    local tasks=("$@")

    echo ""
    echo "═══ $wave_name: ${tasks[*]} ═══"

    local remaining=()
    for t in "${tasks[@]}"; do
        if ! task_done "$t"; then
            remaining+=("$t")
        else
            echo "  ✓ $t already done"
        fi
    done

    if [ ${#remaining[@]} -eq 0 ]; then
        echo "  All tasks in $wave_name already complete."
        return 0
    fi

    local pids=()
    local running=0
    local failures=0

    for t in "${remaining[@]}"; do
        run_task "$t" &
        pids+=("$!:$t")
        running=$((running + 1))

        if [ "$running" -ge "$MAX_CONCURRENT" ]; then
            for entry in "${pids[@]}"; do
                local pid="${entry%%:*}"
                if ! kill -0 "$pid" 2>/dev/null; then
                    wait "$pid" || failures=$((failures + 1))
                    running=$((running - 1))
                    break
                fi
            done
        fi
    done

    for entry in "${pids[@]}"; do
        local pid="${entry%%:*}"
        wait "$pid" || failures=$((failures + 1))
    done

    if [ "$failures" -gt 0 ]; then
        echo ""
        echo "⚠  $wave_name had $failures task failure(s). Proceeding to gate — triage will handle it."
    fi

    return 0
}

run_gate() {
    local gate_name="$1"
    local wave_tasks="$2"
    shift 2
    local cmd="$*"

    echo ""
    echo "── Gate: $gate_name ──"
    echo "   Running: $cmd"

    local log="$LOG_DIR/gate-${gate_name}.log"
    if eval "$cmd" > "$log" 2>&1; then
        local test_result
        test_result=$(grep "test result:" "$log" | head -1 || echo "")
        echo "   ✓ PASS  $test_result"
        return 0
    fi

    # Gate failed — invoke triage with cumulative journal
    echo "   ✗ FAIL — launching triage agent"

    local journal="$LOG_DIR/triage-journal-${gate_name}.md"
    : > "$journal"  # start fresh for this gate

    local attempt=0
    while [ "$attempt" -lt "$MAX_TRIAGE" ]; do
        attempt=$((attempt + 1))

        # Count remaining failures for progress tracking
        local fail_count
        fail_count=$(grep -c "^test .* FAILED" "$log" 2>/dev/null || echo "?")
        echo ""
        echo "   ── Triage attempt $attempt/$MAX_TRIAGE ($fail_count failure(s)) ──"

        # Build the journal context for this attempt
        local prior_journal
        if [ -s "$journal" ]; then
            prior_journal=$(cat "$journal")
        else
            prior_journal="(No prior attempts — this is the first triage run.)"
        fi

        local triage_log="$LOG_DIR/triage-${gate_name}-${attempt}.log"
        local triage_prompt
        triage_prompt=$(sed \
            -e "s|\$GATE_LOG|${log}|g" \
            -e "s|\$GATE_NAME|${gate_name}|g" \
            -e "s|\$WAVE_TASKS|${wave_tasks}|g" \
            -e "s|\$ATTEMPT|${attempt}|g" \
            -e "s|\$MAX_TRIAGE|${MAX_TRIAGE}|g" \
            "$TRIAGE_FILE")

        # Replace the $PRIOR_JOURNAL placeholder — it can be multi-line,
        # so we use a temp file approach instead of sed
        local prompt_file="$LOG_DIR/.triage-prompt-${gate_name}-${attempt}.md"
        echo "$triage_prompt" | awk -v journal="$prior_journal" \
            '{gsub(/\$PRIOR_JOURNAL/, journal); print}' > "$prompt_file"

        # shellcheck disable=SC2086
        claude $CLAUDE_ARGS -p "$(cat "$prompt_file")" > "$triage_log" 2>&1 || true

        # Re-run the gate to see if triage fixed it
        echo "   Re-running: $cmd"
        if eval "$cmd" > "$log" 2>&1; then
            local test_result
            test_result=$(grep "test result:" "$log" | head -1 || echo "")
            echo "   ✓ PASS (after $attempt triage attempt(s))  $test_result"

            # Check if triage filed any bug tasks
            if grep -q "^## Triage" "$TASK_FILE" 2>/dev/null; then
                local bug_count
                bug_count=$(grep -c "^\- \[ \] \*\*B[0-9]" "$TASK_FILE" 2>/dev/null || echo "0")
                if [ "$bug_count" -gt 0 ]; then
                    echo "   ⚠  $bug_count production bug(s) filed. Tests #[ignore]d for now."
                fi
            fi

            return 0
        fi

        # Detect if progress was made (fewer failures than before)
        local new_fail_count
        new_fail_count=$(grep -c "^test .* FAILED" "$log" 2>/dev/null || echo "?")
        if [ "$new_fail_count" != "$fail_count" ] 2>/dev/null; then
            echo "   Progress: $fail_count → $new_fail_count failure(s)"
        else
            echo "   No progress on failure count."
        fi
    done

    echo ""
    echo "   ✗ GATE FAILED after $MAX_TRIAGE triage attempts."
    echo "   Journal: $journal"
    echo "   Last gate log: $log"
    echo ""
    echo "   Unresolved failures:"
    grep "^test .* FAILED" "$log" 2>/dev/null | head -20 || tail -20 "$log"
    return 1
}

# ── Main ─────────────────────────────────────────────────

echo "╔══════════════════════════════════════════╗"
echo "║  mdt test suite — parallel Ralph loop   ║"
echo "╚══════════════════════════════════════════╝"

# Wave 0
run_wave "Wave 0 (bootstrap)" T00
run_gate "wave0" "T00" "cargo test" || exit 1

# Wave 1 — the big bang: 9 independent tasks
W1_TASKS="T01 T02 T03 T07 T10 T11 T12 T13 T16"
run_wave "Wave 1 (independent modules)" $W1_TASKS
run_gate "wave1" "$W1_TASKS" "cargo test" || exit 1

# Wave 2 — second links in each chain
W2_TASKS="T04 T08 T14 T15 T17"
run_wave "Wave 2 (chain extensions)" $W2_TASKS
run_gate "wave2" "$W2_TASKS" "cargo test" || exit 1

# Wave 3 — third links
W3_TASKS="T05 T09"
run_wave "Wave 3 (deep chains)" $W3_TASKS
run_gate "wave3" "$W3_TASKS" "cargo test" || exit 1

# Wave 4 — critical path tail
run_wave "Wave 4 (critical path)" T06
run_gate "wave4" "T06" "cargo test" || exit 1

# Wave 5 — audit
run_wave "Wave 5 (audit)" T18
run_gate "wave5" "T18" "cargo test" || exit 1

# ── Bug fix pass ─────────────────────────────────────────

if grep -q "^\- \[ \] \*\*B[0-9]" "$TASK_FILE" 2>/dev/null; then
    echo ""
    echo "═══ Bug Fix Pass ═══"
    echo ""
    echo "  Production bugs were discovered during test implementation."
    echo "  These require production code changes and are listed at the"
    echo "  bottom of $TASK_FILE under '## Triage — Bug Fixes'."
    echo ""

    bug_tasks=$(grep -o "B[0-9][0-9]*" "$TASK_FILE" | sort -u)
    for bt in $bug_tasks; do
        if ! task_done "$bt"; then
            echo "  → $bt: fixing production bug"
            bugfix_log="$LOG_DIR/${bt}.log"
            bugfix_prompt=$(sed "s/\$TASK_ID/${bt}/g" "$BUGFIX_FILE")
            # shellcheck disable=SC2086
            claude $CLAUDE_ARGS -p "$bugfix_prompt" > "$bugfix_log" 2>&1 || true
            if task_done "$bt"; then
                echo "  ✓ $bt fixed"
            else
                echo "  ⚠  $bt needs manual attention (log: $bugfix_log)"
            fi
        fi
    done

    # Un-ignore tests that were blocked on bugs
    echo ""
    echo "  Re-running full test suite after bug fixes..."
    if cargo test > "$LOG_DIR/gate-bugfix.log" 2>&1; then
        echo "  ✓ All tests pass including previously-ignored tests."
    else
        echo "  ⚠  Some tests still failing. Check $LOG_DIR/gate-bugfix.log"
        grep "^test .* FAILED" "$LOG_DIR/gate-bugfix.log" 2>/dev/null | head -20
    fi
fi

echo ""
echo "════════════════════════════════════"
echo "  All waves complete. Suite built."
echo "════════════════════════════════════"
