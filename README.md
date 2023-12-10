# mly

![License](https://img.shields.io/github/license/nmlgc/mly?cacheSeconds=31536000)

Unix-style filter suite for Standard MIDI Files, built on top of the [midly crate](https://crates.io/crates/midly).

## Commands

### `cut`

Removes MIDI events within a certain range, and writes the new MIDI to stdout.

Despite the beat/pulse parameters, this command is extremely basic, and simply removes the events that are closest to the given time points. It inserts no *Note Off* events for notes that might be playing at the cut point, nor modifies any delta times to re-synchronize multi-track sequences; you might want to flatten the latter using the `smf0` command beforehand.

### `dump`

Dumps all MIDI events to stdout, with one event per line.

For easier navigation, the output also contains the total MIDI pulse count and the 0-based *quarter-note:pulse* beat number in separate columns.

### `loop-find`

Finds the longest fully repeated and unique range of MIDI events.

### `smf0`

Flattens the sequence into a single track and writes the result as SMF Type 0 to stdout.

With the exception of any *End of Track* events before the final one, all events are preserved, even if they don't make sense in a single-channel sequence (such as any *Track Name* meta events after the first). Simultaneous MIDI events are sorted according to the track order of the input sequence.
