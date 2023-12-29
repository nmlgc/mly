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

### `duration`

Prints the total duration of the sequence.

This command only examines the track with the highest final MIDI pulse value. Multi-track sequences might have their tempo events on a different track, which will cause all realtime values to be omitted. In that case, you will need to flatten the sequence using the `smf0` command beforehand.

### `filter-note`

Removes all note events within the given range, and writes the modified MIDI to stdout.

This only removes Note On events with nonzero velocity. Any playing notes at the start or end of the removal range are left playing.

### `loop-find`

Finds the longest fully repeated and unique range of MIDI events.

This command can detect two kinds of loops:

* A loop in *note space* that represents the earliest possible event range with equivalent per-channel controller and pitch bend state at both ends. This loop is only appropriate for MIDI players, as its bounds can be placed into the middle of notes that are played with a different channel state at the start and end of the loop. Therefore, it assumes an event-based looping implementation that doesn't stop any playing notes when it jumps back, nor replays non-note messages from the beginning of the sequence to the loop start point.

* If `-s/--shift` or the global `-r`/`--samplerate` option is given, the command derives a second loop in *recording space* from the event space loop. This loop is appropriate for loop-cutting a synthesizer recording of the MIDI sequence:
  * It is only placed in the middle of playing notes if they share the same channel state at both ends of the loop.
  * For easier calibration, it is enforced to start on a *Note On* event with non-zero velocity.

### `loop-unfold`

Repeats a range of MIDI events starting at a given point before the end of the sequence.

Useful for reconstructing a full second repetition of a loop that only appears in truncated form in the original sequence. Does not modify any delta times to re-synchronize multi-track sequences; you might want to flatten such sequences using the `smf0` command beforehand.

### `smf0`

Flattens the sequence into a single track and writes the result as SMF Type 0 to stdout.

With the exception of any *End of Track* events before the final one, all events are preserved, even if they don't make sense in a single-channel sequence (such as any *Track Name* meta events after the first). Simultaneous MIDI events are sorted according to the track order of the input sequence.
