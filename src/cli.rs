//! Argument parsing for the handful of flags `bible-cli` accepts.
//!
//! Hand-rolled rather than delegating to `clap`: there are three flags, no
//! subcommands and no positional arguments, so a dependency would cost build
//! time and binary weight for a `match`.

pub const HELP: &str = concat!(
    "bible-cli ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "A terminal Bible reader with a three-pane display and Vim-like navigation.\n",
    "\n",
    "USAGE:\n",
    "    bible-cli [OPTIONS]\n",
    "\n",
    "OPTIONS:\n",
    "    -q, --quote      Print a random verse and exit\n",
    "    -v, --version    Print the version and exit\n",
    "    -h, --help       Print this help and exit\n",
    "\n",
    "KEYS (in the reader):\n",
    "    h/j/k/l, arrows  Move between panes and items\n",
    "    Enter, l         Open the next pane; copy the verse in the verses pane\n",
    "    /                Filter the active pane\n",
    "    f                Search the whole Bible\n",
    "    m                Bookmark the selected verse (again to remove)\n",
    "    b                List bookmarks (d deletes one)\n",
    "    g / G            Jump to the first / last item\n",
    "    q, Esc           Quit\n",
);

/// What the user asked the program to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Run the interactive reader.
    Tui,
    /// Print a random verse.
    Quote,
    /// Print the version string.
    Version,
    /// Print the usage text.
    Help,
}

/// Parse the argument list, excluding argv[0].
///
/// The first of `--help`, `--version`, `--quote` to appear wins, so
/// `bible-cli --quote --help` prints the quote — the flags are independent
/// requests and honouring them in order is less surprising than a fixed
/// precedence.
pub fn parse<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut command = None;

    for arg in args {
        let arg = arg.as_ref();
        let matched = match arg {
            "-h" | "--help" => Command::Help,
            "-v" | "--version" => Command::Version,
            "-q" | "--quote" => Command::Quote,
            other => return Err(unknown_argument(other)),
        };
        command.get_or_insert(matched);
    }

    Ok(command.unwrap_or(Command::Tui))
}

fn unknown_argument(arg: &str) -> String {
    let kind = if arg.starts_with('-') {
        "unknown flag"
    } else {
        "unexpected argument"
    };
    format!("bible-cli: {} '{}'\nTry 'bible-cli --help'.", kind, arg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arguments_runs_the_reader() {
        assert_eq!(parse(Vec::<String>::new()), Ok(Command::Tui));
    }

    #[test]
    fn long_and_short_flags_are_equivalent() {
        assert_eq!(parse(["--help"]), Ok(Command::Help));
        assert_eq!(parse(["-h"]), Ok(Command::Help));
        assert_eq!(parse(["--version"]), Ok(Command::Version));
        assert_eq!(parse(["-v"]), Ok(Command::Version));
        assert_eq!(parse(["--quote"]), Ok(Command::Quote));
        assert_eq!(parse(["-q"]), Ok(Command::Quote));
    }

    #[test]
    fn the_first_recognised_flag_wins() {
        assert_eq!(parse(["--quote", "--help"]), Ok(Command::Quote));
        assert_eq!(parse(["--help", "--quote"]), Ok(Command::Help));
    }

    #[test]
    fn an_unknown_flag_is_an_error_rather_than_being_ignored() {
        let err = parse(["--colour"]).unwrap_err();

        assert!(err.contains("unknown flag '--colour'"), "{}", err);
        assert!(err.contains("--help"), "{}", err);
    }

    #[test]
    fn a_stray_positional_argument_is_an_error() {
        let err = parse(["genesis"]).unwrap_err();

        assert!(err.contains("unexpected argument 'genesis'"), "{}", err);
    }

    #[test]
    fn an_unknown_flag_after_a_valid_one_is_still_an_error() {
        assert!(parse(["--quote", "--nope"]).is_err());
    }

    #[test]
    fn the_help_text_names_every_flag() {
        for flag in ["--quote", "--version", "--help"] {
            assert!(HELP.contains(flag), "help text is missing {}", flag);
        }
    }
}
