# Map: Harden bible-cli

Label: `wayfinder:map`

## Destination

bible-cli is crash-safe, has a tested core, and a CI gate that runs on every push —
plus a decided (not necessarily built) list of the UX features worth doing next.
Reaching the end means: no known way to leave the user's terminal broken, the
navigation/search logic covered by tests, `cargo clippy -- -D warnings` green in CI,
and the feature fog resolved into either tickets or an explicit "not now".

## Notes

- **Domain**: Rust TUI (`ratatui` 0.30 + `crossterm` 0.29), ~713 LOC across
  `src/main.rs` (event loop + CLI args), `src/app.rs` (state + filtering),
  `src/ui.rs` (render), `src/data.rs` (embedded JSON via `include_dir`).
- **Data**: 81 Amharic book JSONs, 12 MB, compiled into the binary (14 MB release).
- **Execution is in scope for this map.** The user asked for the recommended
  changes to be made, not only planned — this map overrides the wayfinder
  plan-don't-do default. Tickets that are pure code health resolve by landing
  the change; tickets that are genuine forks in the road still resolve by decision.
- **Commits**: no `Co-Authored-By` trailers on any commit in this effort.
- Verify with `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Decisions so far

- [Restore the terminal on panic](issues/01-panic-terminal-restore.md) — install a panic hook that runs the same teardown as the normal exit path, and route the whole TUI lifecycle through guarded setup/teardown so no exit path leaves raw mode on.
- [Cover the navigation and filtering logic with tests](issues/02-test-core-logic.md) — 26 unit tests in `src/app.rs` + `src/data.rs` against a hand-built fixture; `App::new` takes metadata so no I/O is needed to drive it.
- [Gate every push on build, test, clippy and fmt](issues/03-ci-gate.md) — `.github/workflows/ci.yml`, one Linux job, `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` + `cargo test`, with a shared registry/target cache.
- [Clear the clippy warnings and the unwrap in the data loader](issues/04-clippy-and-unwrap.md) — all 7 warnings fixed at the source (not `#[allow]`), `to_str().unwrap()` replaced by a real `anyhow` error.
- [Decide how `--quote` picks its verse](issues/06-quote-efficiency.md) — two-stage random (pick a book, then a verse inside it) so one book is parsed instead of 81; seed widened and modulo bias removed via rejection sampling.
- [Stop re-parsing the book JSON on every keystroke](issues/07-book-parse-cache.md) — memoise on `file_path` in `App`, reload only when the selected book actually changes.
- [Give the CLI a real argument parser and a `--help`](issues/05-cli-args-and-help.md) — hand-rolled parser in a new `src/cli.rs`, no new dependency; unknown flags are a hard error to stderr with exit code 2.
- [Make books searchable by English name](issues/08-english-book-search.md) — carry `name_en`/`short_name_en` on `BookMetadata` and match them case-insensitively alongside the Amharic name and number.
- [Build the book list without materialising every verse](issues/09-metadata-header-parse.md) — header-only deserialize at startup; measured at only ~12% (14.6 ms vs 16.6 ms), kept for the memory rather than the time. The ~15 ms scan of all 81 files is the real cost and is still there.

## Not yet specified

- **Persistence.** Last-read position, bookmarks, reading history. Needs a decision
  on where state lives (XDG config/data dirs vs a dotfile), what format, and whether
  bookmarks are a first-class pane or a jump list. Blocked on nothing technical —
  blocked on wanting it.
- **Cross-book search.** Verse search today is scoped to the current chapter. A
  whole-Bible search changes the data story: 12 MB of JSON parsed per query is too
  slow, so it likely needs a prebuilt index. Shape unknown.
- **Second translation.** English text alongside Amharic. Doubles the embedded data
  and the 14 MB binary; may force a shift from `include_dir` to downloaded/cached
  data. Interacts with the search question above.
- **Verses-pane affordances.** Scrollbar, "verse 12 of 176" position readout, and
  feedback when Enter copies a verse (today it copies silently). Probably one
  prototype ticket once someone can react to a mock.
- **Config and theming.** The palette is hardcoded in `src/ui.rs`. Whether this
  deserves a config file at all is the open question.
- **The startup scan.** Every launch spends ~15 ms parsing all 81 embedded JSON files
  just to build the book list, and header-only deserialization only trimmed 12% of it
  (see *Build the book list without materialising every verse*). Removing it properly
  means not reading the files at startup at all — a generated table baked in at build
  time, or a different on-disk format. Whether 15 ms is even worth that is itself the
  open question, and it is entangled with the search and second-translation patches
  above, which would all touch the same data layer.

## Out of scope

- Distribution channels beyond the current install scripts — Homebrew, AUR, winget,
  crates.io publishing. Real work, but it is packaging, not the health of the code
  this map is about. Separate effort.
- A GUI or web version. Different product.
- Changing or re-sourcing the Amharic text itself. Content, not code.
