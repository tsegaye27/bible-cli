# Restore the terminal on panic

Type: task
Status: resolved
Blocked by: —

## Question

`main()` in `src/main.rs` enables raw mode and enters the alternate screen, then only
undoes both after `run_app` returns. A panic anywhere inside the draw or event loop
unwinds straight past that teardown, so the user is dropped back into a shell that is
still in raw mode and still on the alternate screen — no echo, no line editing, and
the panic message itself is usually invisible.

How should the TUI lifecycle be structured so that *every* exit path — normal quit,
`Err` from the loop, and panic — leaves the terminal as it was found?
