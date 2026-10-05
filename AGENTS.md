# Agent instructions

## Testing

`cargo test` is not the whole test suite. Before you start a change and
before you finish it, run every suite as described in
[doc/guides/testing.md](doc/guides/testing.md).

## Lints

Do not fix Clippy warnings in code you did not otherwise change. Upstream
does not configure lints, and unrelated cleanups cause conflicts when this
fork is rebased.
