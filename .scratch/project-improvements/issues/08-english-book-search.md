# Make books searchable by English name

Type: task
Status: resolved
Blocked by: —

## Question

`App::filtered_books` matches the query against `name_am` and the book number only. The
book JSONs carry `book_name_en` and `book_short_name_en`, but `BookMetadata` drops both on
the floor when it is built in `data::load_books_metadata`.

A user on a keyboard without an Amharic layout can therefore only find a book by guessing
its number. Should English names be searchable, and if so, matched how — exact, prefix, or
case-insensitive substring?

## Answer

Yes — case-insensitive substring, on both the full and the short English name, alongside
the existing Amharic-name and number matches.

`BookMetadata` gains `name_en` and `short_name_en`, populated in `load_books_metadata` from
fields the JSON already carried. `filtered_books` lowercases the query once per call, then
matches it against both. Substring rather than prefix, to match how the Amharic name is
already searched, and so `kings` finds *2 Kings*. Case-insensitivity is applied only to the
English names; Ethiopic has no case, so lowercasing the Amharic comparison would be
meaningless work on every keystroke.

Typing `psal`, `Psa`, `PSALMS` or `19` all now find the same book.

Reading the English names out of the JSON exposed something adjacent:
`load_books_metadata` deserialized each file into a full `Book` — every chapter, section
and verse of all 81 books — purely to read a few header fields, then dropped the lot. That
is now a small private `BookHeader` struct. See
*Build the book list without materialising every verse*, which measures the effect.
