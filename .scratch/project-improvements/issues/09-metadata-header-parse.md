# Build the book list without materialising every verse

Type: task
Status: resolved
Blocked by: —

Surfaced while resolving *Make books searchable by English name*.

## Question

`data::load_books_metadata` runs at startup and deserializes every one of the 81 embedded
files into a full `Book` — every chapter, section and verse — in order to read three header
fields off each, then drops all of it. How much is that costing, and is a header-only
deserialize worth doing?

## Answer

Done, but it is a smaller win than it looks, and the measurement is the point.

Deserializing into a private `BookHeader` (`book_number`, `book_name_am`, `book_name_en`,
`book_short_name_en`) instead of `Book` — serde ignores unknown fields, so the chapter array
is walked and discarded rather than built.

Measured over 10 iterations in a release build:

| | per call |
|---|---|
| header-only (`BookHeader`) | 14.6 ms |
| full parse (`Book`) | 16.6 ms |

**About 12%, not the order of magnitude the change looks like it should give.** The cost is
dominated by `serde_json` scanning and UTF-8-validating ~12 MB of JSON; the allocation of
the chapter tree, which is all this change avoids, is the smaller half. Anyone reading the
diff would over-estimate it, which is why the number is recorded here and in the doc comment
on `BookHeader`.

Kept anyway, on two grounds that do not depend on the timing: it is free — the struct is
seven lines and the call site is unchanged — and it keeps 81 fully-materialised books out of
memory at startup, which is a peak-RSS difference rather than a wall-clock one.

The real remaining cost is the ~15 ms scan itself, which no amount of struct-shaping will
remove; getting rid of it means not reading all 81 files at startup. That is a different and
much larger change — see **Not yet specified** on the map.
