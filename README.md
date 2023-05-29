# vash

An experimental shell written in Rust. Not intended to replace any other shell.

## Progress

- [x] Basic command parsing
- [x] Basic command execution
- [x] Basic pipes
- [x] Basic logic
- [ ] Basic stream operators (`>>`, `<<`, etc.)
  - [x] Partial parsing support
  - [ ] Partial execution support
  - [ ] Full parsing support
  - [ ] Full execution support
- [ ] Shell builtins
- [ ] Prompt customization
- [ ] Being an actual TTY/shell
- [ ] Fully lexed command parsing
- [ ] Complex command execution
- [ ] Novel pipe handling (i.e. extending bash/zsh)

## Concept

This program is intended to be a stream plumber. My end goal for this is to allow arbitrary connection
between file descriptors, allowing for very complex stream handling. Ideally, this will be useful
for designing complex process models with complex, direct communication between processes.

# vatty

An experimental terminal emulator with a special trick up it's sleve.
Not at all functional as I'm focusing on vash first.
