# Gate every push on build, test, clippy and fmt

Type: task
Status: resolved
Blocked by: 02

## Question

`.github/workflows/release.yml` is the only workflow, and it fires on `v*` tags. Nothing
runs on a push to `main` or on a pull request, so a change that fails to compile is only
caught when someone cuts a release across six targets.

What should the pre-release gate check, and on which platforms? The release matrix builds
six targets; repeating that on every push is slow and mostly redundant, since the code has
no platform-specific paths beyond what `crossterm` and `arboard` already abstract.

## Answer

One `ubuntu-latest` job in `.github/workflows/ci.yml`, on push to `main`, on every pull
request, and on `workflow_dispatch`. It runs, in order: `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, then
`cargo build --release`.

Not repeating the release matrix. The six targets exist to produce artefacts, and the code
has no `#[cfg(target_os)]` anywhere — the platform differences live inside `crossterm` and
`arboard`. A second Linux job would catch nothing a first one missed, and six would turn a
two-minute gate into a ten-minute one. If a platform-specific bug ever does land, that is
the moment to widen the matrix, not before.

Formatting is checked because `cargo fmt` had never been run: it reformatted all four
source files on its first pass, so without a gate the next contributor's diff would be
mostly whitespace. `concurrency` with `cancel-in-progress` stops stacked pushes from
queueing, and the registry plus `target` are cached on `Cargo.lock`.
