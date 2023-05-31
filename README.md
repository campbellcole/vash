# vash

An experimental shell written in Rust. Not intended to replace any other shell.

## Progress

### Todo

- [x] ~~Basic command parsing~~ (replaced by lexer)
- [x] Basic command execution
- [x] Basic pipes
- [x] Basic logic
- [ ] Basic stream operators (`>>`, `<<`, etc.)
  - [x] Partial parsing support
  - [ ] Partial execution support
  - [ ] Full parsing support
  - [ ] Full execution support
- [x] Shell builtins
- [x] Scrolling
- [ ] Prompt customization
- [ ] Being an actual TTY/shell
  - [ ] Colors
  - [ ] Other escape sequences
- [ ] Fully lexed command parsing
  - [x] Basic lexer
  - [ ] String unescaping (currently only works with double quotes)
- [ ] Complex command execution
- [ ] Novel pipe handling (i.e. extending bash/zsh)

### Known Issues

##### Not Fixed

- The history index is not reset when a new command is added, making each suggestion increasingly behind.

##### Fix Attempted (not proven successful yet)

Nothing yet.

##### Fixed

- Stdout/Stderr (`state.output`) does not scroll and will break once enough has been printed
- There is a race condition between reading a stream and waiting for a process to finish. When a process finishes, there may still be data left in stdout/stderr that will be discarded because the process exit future will resolve before the next read call finishes.

## Concept

This program is intended to be a stream plumber. My end goal for this is to allow arbitrary connection
between file descriptors, allowing for very complex stream handling. Ideally, this will be useful
for designing complex process models with complex, direct communication between processes.

# vatty

An experimental terminal emulator with a special trick up it's sleve.
Not at all functional as I'm focusing on vash first.
