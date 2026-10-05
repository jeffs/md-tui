# Testing

The project has two test suites. `cargo test` runs only the first; a change
is not tested until both pass.

1. Unit and integration tests:

   ```sh
   cargo test
   ```

2. End-to-end snapshot tests, which drive the real `mdt` binary in tmux and
   compare each screen against `tests/e2e/snapshots/`:

   ```sh
   tests/e2e/run.sh
   ```

   The script ends with `E2E Results: N passed, M failed, K skipped` and
   exits nonzero on any failure.

Run `cargo test` after every edit. The E2E suite takes about 20 seconds;
run it before you start, after any change that affects what the screen
shows, and before you finish.

## E2E pitfalls

- Without tmux, the script prints `SKIP: tmux not found` and exits 0. That
  is not a pass. Install tmux and rerun.
- The script builds with `cargo build`, then runs `target/debug/mdt` under
  the repository root. Do not set `CARGO_TARGET_DIR`: the build would go
  elsewhere and the script would test a stale binary, or fail to find one.
- The script sets `HOME` to `tests/e2e/home`, so your own
  `~/.config/mdt/config.toml` does not affect the captures. Settings that
  affect rendering belong in that fixture config.

## Changing what the screen shows

When a change alters rendering on purpose, regenerate the snapshots:

```sh
UPDATE_SNAPSHOTS=1 tests/e2e/run.sh
jj diff tests/e2e/snapshots
```

Read every snapshot diff and confirm it shows only the intended change.
Commit the new snapshots in the same commit as the change that caused them,
so that every commit passes both suites. When rewriting history, run both
suites at each rewritten commit.

## Where to add tests

See [making-changes.md](making-changes.md).
