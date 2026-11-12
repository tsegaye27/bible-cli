# Stop re-parsing the book JSON on every keystroke

Type: task
Status: resolved
Blocked by: —

## Question

`App::load_current_book_filtered` calls `data::load_book`, which runs
`serde_json::from_str` over the whole book. It is called from `next_item`,
`previous_item`, the `g`/`G` jumps, and on every character typed or deleted while
searching the Books pane — so holding `j` down re-parses a book per repeat, and each one
also resets the chapter and verse selection.

What is the cheapest change that avoids the redundant parse without introducing a cache
that can go stale?

## Answer

Memoise on the file path. `App` gains a private `loaded_book_path: Option<String>`;
`load_current_book_filtered` resolves the selected book's path first and returns early if
it matches what is already loaded.

This cannot go stale, because the data is embedded in the binary — a given path maps to
exactly one book for the life of the process. That is what makes the one-entry cache the
cheapest correct answer rather than a liability; a cache over mutable files would need
invalidation, this one does not.

It also fixes a bug that was hiding inside the redundant work. The reload path resets
`chapters_state` and `verses_state` to 0, and it ran on *every* keystroke in the Books
pane — so typing a search query that kept the same book selected silently threw away the
user's chapter and verse position. Now the reset happens only when the book actually
changes, which is the only time it means anything.

The function was also restructured to use `let ... else` instead of a nested `if let`
pyramid, so the three "nothing to do" cases read as three lines at the top.

Two tests pin both halves of the behaviour: reselecting the same book keeps the cursors,
selecting a different book resets them.
