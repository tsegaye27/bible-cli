# Give the CLI a real argument parser and a `--help`

Type: grilling
Status: resolved
Blocked by: —

## Question

Argument handling in `src/main.rs` is three `args.contains(&"...".to_string())` calls.
There is no `--help`, an unrecognised flag is silently ignored and drops the user into the
TUI, and every check allocates a `String` to compare against.

Does this become a `clap` dependency or a hand-rolled parser? `clap` is the ergonomic
default but adds build time and binary weight to a project that currently has eight
dependencies and three flags. What is the exit behaviour for an unknown flag?

## Answer

Hand-rolled, in a new `src/cli.rs`. No new dependency.

`clap` is the right default when a CLI has subcommands, positional arguments, value parsing
or completions to generate. This one has three boolean flags and nothing else, so the
entire parser is a `match` on `&str` — roughly 30 lines against a dependency tree that
would be the largest in the manifest and would grow a 14 MB binary further for no
capability the program uses.

Shape of the decision:

- `parse()` takes `IntoIterator<Item: AsRef<str>>` rather than reading `std::env::args`
  itself, which is what makes the seven unit tests possible.
- **The first recognised flag wins.** `bible-cli --quote --help` prints the quote. The
  flags are independent requests, and honouring them in encounter order is less surprising
  than a hidden precedence table.
- **An unknown flag is a hard error**, not something to ignore. It prints
  `bible-cli: unknown flag '--colour'` plus a `Try 'bible-cli --help'.` pointer to stderr
  and exits **2**, keeping exit 1 for genuine runtime failures. A bare word gets the same
  treatment worded as `unexpected argument`, since the program takes no positionals.
- `HELP` is a `concat!` const that splices in `CARGO_PKG_VERSION`, so it cannot drift from
  the version. A test asserts every flag appears in it.

`main` now returns `ExitCode` and dispatches on the `Command` enum.
