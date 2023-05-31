# vash

An experimental shell written in Rust. Not intended to replace any other shell.

## Progress

### Todo

- [x] Basic command parsing
- [x] Basic command execution
- [x] Basic pipes
- [x] Basic logic
- [ ] Basic stream operators (`>>`, `<<`, etc.)
  - [x] Partial parsing support
  - [ ] Partial execution support
  - [ ] Full parsing support
  - [ ] Full execution support
- [x] Shell builtins
- [ ] Prompt customization
- [ ] Being an actual TTY/shell
- [ ] Fully lexed command parsing
- [ ] Complex command execution
- [ ] Novel pipe handling (i.e. extending bash/zsh)

### Known Issues

##### Not Fixed

- Stdout/Stderr (`state.output`) does not scroll and will break once enough has been printed

##### Fix Attempted (not proven successful yet)

- There is a race condition between reading a stream and waiting for a process to finish. When a process finishes, there may still be data left in stdout/stderr that will be discarded because the process exit future will resolve before the next read call finishes.

##### Fixed

Nothing yet

## Concept

This program is intended to be a stream plumber. My end goal for this is to allow arbitrary connection
between file descriptors, allowing for very complex stream handling. Ideally, this will be useful
for designing complex process models with complex, direct communication between processes.

# vatty

An experimental terminal emulator with a special trick up it's sleve.
Not at all functional as I'm focusing on vash first.
