# Clear the clippy warnings and the unwrap in the data loader

Type: task
Status: resolved
Blocked by: —

## Question

`cargo clippy --all-targets` reports 7 warnings on a clean tree. Separately,
`src/data.rs:68` does `file.path().to_str().unwrap()` while building `BookMetadata` —
a panic on any non-UTF-8 path, inside a function that already returns `Result`.

Should each warning be fixed at the source or silenced, and does the `unwrap` become an
error or stay a panic?
