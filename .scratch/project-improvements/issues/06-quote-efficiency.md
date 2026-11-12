# Decide how `--quote` picks its verse

Type: task
Status: resolved
Blocked by: —

## Question

`data::random_quote()` parses all 81 embedded book JSONs, builds a `Vec<Quote>` holding a
cloned copy of every verse in the Bible along with two cloned book names each, and then
throws all but one away.

Separately, `random_index` seeds a `DefaultHasher` from the current nanosecond and the pid
and takes `hash % len`, which is biased when `len` does not divide the hash space evenly.

What is the right selection strategy, and does the bias matter enough to fix?

## Answer

Two-stage selection: pick the book, then pick a verse inside it. **One** book is parsed
instead of 81, and no `Vec<Quote>` of the whole Bible is ever built.

The catch with the naive two-stage version is that picking a book uniformly over-represents
short books — a verse in Obadiah becomes far likelier than one in Psalms. The fix is to
weight the book draw by its JSON byte length, which `include_dir` exposes as
`file.contents().len()` **without parsing anything**. Byte length tracks verse count
closely enough that the result is near-uniform over verses, at zero read cost. Selection is
then a running-subtraction walk over the weights, and a second draw picks a verse by
flattened index across the chosen book's sections.

Wall time for `bible-cli --quote` went from ~40 ms to below the 10 ms resolution of
`/usr/bin/time`.

The bias was worth fixing, because it was free. `random_index` is replaced by an `Rng`
struct: SplitMix64, seeded once from the clock, the pid and a stack address. Seeding once
matters now that two draws are needed — the old code reseeded a `DefaultHasher` from the
nanosecond clock per call, so two calls in the same nanosecond would have returned the same
value. `below(n)` rejects the tail above `(u64::MAX / n) * n` before taking the modulo,
which removes the bias entirely. Still not cryptographic, and does not need to be.

Two tests cover the range invariant and that every value in a small range is reachable.
