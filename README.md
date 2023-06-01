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
  - [ ] Migrate from REPL/raw mode to normal mode
  - [ ] Colors
  - [ ] Other escape sequences
- [x] Fully lexed command parsing
  - [x] Basic lexer
  - [x] String unescaping
- [ ] Complex command execution
- [ ] Novel pipe handling (i.e. extending bash/zsh)

### Known Issues

##### Not Fixed

- There is almost no error handling and panics are common. Eventually, `color-eyre` will be used for error management. I'll have to rework a lot of the channel/thread management to support raising errors.
- `&&` and `||` both block until the left command has completed. Using the `sleep` command hangs the entire shell until it is complete.
  - This isn't exactly a flaw because this is how a shell is supposed to behave, but due to the current REPL type nature of this shell, it makes more sense to not block on any command.

##### Fix Attempted (not proven successful yet)

- The history index is not reset when a new command is added, making each suggestion increasingly behind.
- Running the same command multiple times fills the history.

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
