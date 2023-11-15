# mly

![License](https://img.shields.io/github/license/nmlgc/mly?cacheSeconds=31536000)

Unix-style filter suite for Standard MIDI Files, built on top of the [midly crate](https://crates.io/crates/midly).

## Commands

### `dump`

Dumps all MIDI events to stdout, with one event per line.

For easier navigation, the output also contains the total MIDI pulse count and the 0-based *quarter-note:pulse* beat number in separate columns.

### `loop-find`

Finds the longest fully repeated range of MIDI events.
