# Cover the navigation and filtering logic with tests

Type: task
Status: resolved
Blocked by: —

## Question

There is no test of any kind in the repo — no `tests/` directory and no `#[cfg(test)]`
module. The logic most likely to break silently lives in `src/app.rs`: index wrapping in
`next_item`/`previous_item`, the three `filtered_*` methods, and `commit_search`, which
maps a filtered index back to an unfiltered one.

`App::new` takes `Vec<BookMetadata>` and does no I/O, but `load_current_book_filtered`
reads through `include_dir`. What is the smallest seam that lets the state machine be
driven from a test without touching the embedded data, and what behaviours are worth
pinning down?

## Answer

No seam change was needed. `App::new` already takes `Vec<BookMetadata>` and `current_book`
is a public field, so a test can hand-build both and drive the state machine without ever
reaching `include_dir`. The fixture is three books of metadata plus one two-chapter book
whose second chapter deliberately splits its verses across two sections — that shape is
what makes the flattening in `get_flattened_verses_raw` worth testing.

39 tests now pass across three `#[cfg(test)]` modules: 26 in `src/app.rs`, 6 in
`src/data.rs`, 7 in `src/cli.rs`. What they pin down:

- Wrap-around at both ends of every list, and that an empty pane is a no-op rather than a
  subtraction overflow (`filtered.len() - 1` on an empty vec was the latent panic).
- Each `filtered_*` method, including that book search is substring, chapter and verse
  number search are prefix, and that a query only filters the pane that is active.
- `commit_search` mapping a filtered index back to the unfiltered list — the subtlest thing
  in the file, and the easiest to break.
- Pane transitions refusing to run off either end, and refusing to leave Books with no book
  loaded.
- The two book-cache behaviours from *Stop re-parsing the book JSON on every keystroke*:
  reselecting the same book keeps the cursors, selecting a different one resets them.

The two cache tests are the only ones that touch real embedded data, since the cache keys
on a path that must resolve. They no-op if fewer than two books are embedded rather than
asserting against content that might change.
