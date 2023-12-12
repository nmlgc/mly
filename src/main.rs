mod dump;
mod event;
mod loop_find;
mod manip;
mod smf;
mod state;
mod time;

use std::{
    error::Error,
    io::{self, Read},
    sync::OnceLock,
};

use clap::{Parser, Subcommand};
use midly::Smf;

use crate::time::{total_pulse_of_range, PulseOrBeat};

struct HelpTemplate {
    with_bp: bool,
}

impl HelpTemplate {
    fn with_bp(mut self) -> Self {
        self.with_bp = true;
        self
    }
}

const INDENT: &str = "  ";

impl From<HelpTemplate> for clap::builder::StyledStr {
    fn from(value: HelpTemplate) -> Self {
        static OPTS: OnceLock<textwrap::Options> = OnceLock::new();
        let opts = OPTS.get_or_init(|| {
            textwrap::Options::with_termwidth()
                .initial_indent(INDENT)
                .subsequent_indent(INDENT)
        });

        let bp = if value.with_bp {
            static BP: OnceLock<String> = OnceLock::new();
            BP.get_or_init(|| {
                textwrap::fill("
`B/P` parameters can be specified as either beats (in quarter-note:pulse format) or total pulse counts. Either side of the colon may be omitted if its value is 0.
With a PPQN value of 480:

• 4:240 (→ 2160)
• 4:    (→ 1920)
•  :240 (→  240)
", opts
            )
            })
        } else {
            ""
        };
        format!(
            "{{about-with-newline}}
{{usage-heading}}
{INDENT}(.mid data in stdin) | {{usage}}
{INDENT}<FILE.mid {{usage}}{{tab}}(does not work on PowerShell)
{bp}
{{all-args}}{{after-help}}",
        )
        .into()
    }
}

fn help() -> HelpTemplate {
    HelpTemplate { with_bp: false }
}

#[derive(Subcommand)]
enum CliCommand {
    /// Removes MIDI events within a certain range, and writes the new MIDI to stdout.
    ///
    /// Despite the beat/pulse parameters, this command is extremely basic, and simply removes the
    /// events that are closest to the given time points. It inserts no *Note Off* events for notes
    /// that might be playing at the cut point, nor modifies any delta times to re-synchronize
    /// multi-track sequences; you might want to flatten the latter using the `smf0` command
    /// beforehand.
    #[command(help_template = help().with_bp())]
    Cut {
        /// Start of the cut range.
        #[arg(value_name = "B/P")]
        start: PulseOrBeat,

        /// End of the cut range. Defaults to the end of the sequence if omitted.
        #[arg(value_name = "B/P")]
        end: Option<PulseOrBeat>,
    },

    /// Dumps all MIDI events to stdout, with one event per line.
    ///
    /// For easier navigation, the output also contains the total MIDI pulse count and the 0-based
    /// *quarter-note:pulse* beat number in separate columns.
    #[command(help_template = help())]
    Dump,

    /// Finds the longest fully repeated and unique range of MIDI events.
    #[command(help_template = help())]
    LoopFind,

    /// Flattens the sequence into a single track and writes the result as SMF Type 0 to stdout.
    ///
    /// With the exception of any *End of Track* events before the final one, all events are
    /// preserved, even if they don't make sense in a single-channel sequence (such as any *Track
    /// Name* meta events after the first). Simultaneous MIDI events are sorted according to the
    /// track order of the input sequence.
    #[command(help_template = help())]
    Smf0,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about,
    infer_subcommands = true,
    subcommand_help_heading = "Commands (partial matches are supported)",
    help_template = help(),
    after_help = &format!(
        // color_print::cstr!() does nothing else. Spelling out the codes is terser, avoids an
        // otherwise useless dependency, and even Windows supports them these days.
        "\x1B[4;1mLatest version and source code:\x1B[0m\n{INDENT}https://github.com/nmlgc/mly"
    )
)]
struct Cli {
    /// Sampling rate used for converting times to PCM samples
    #[arg(short = 'r', long)]
    samplerate: Option<u32>,

    #[command(subcommand)]
    command: CliCommand,
}

fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::new();
    io::stdin().lock().read_to_end(&mut bytes)?;
    let mut smf = Smf::parse(&bytes)?;
    let timing = smf.header.timing;

    match args.command {
        CliCommand::Cut { start, end } => {
            manip::cut(&mut smf, total_pulse_of_range(&start, &end, &timing)?)?
        }
        CliCommand::Dump => dump::dump(&smf),
        CliCommand::LoopFind => {
            let opts = loop_find::Options {
                samplerate: args.samplerate,
            };
            loop_find::find(&smf, opts)
        }?,
        CliCommand::Smf0 => smf::smf0(&smf)?,
    }
    Ok(())
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        let args = std::env::args()
            .skip(1)
            .fold(String::new(), |a, b| a + " " + &b);
        eprintln!(
            "`{}{args}`: error: {e}",
            std::env::current_exe()
                .ok()
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|p| p.to_string_lossy())
                .unwrap_or(env!("CARGO_PKG_NAME").into())
        );
        std::process::exit(1);
    }
}
