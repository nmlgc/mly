mod dump;
mod event;
mod loop_find;
mod smf;
mod state;
mod time;

use std::{
    error::Error,
    io::{self, Read},
};

use clap::{Parser, Subcommand};
use midly::Smf;

struct HelpTemplate {}

const INDENT: &str = "  ";

impl From<HelpTemplate> for clap::builder::StyledStr {
    fn from(_: HelpTemplate) -> Self {
        format!(
            "{{about-with-newline}}
{{usage-heading}}
{INDENT}(.mid data in stdin) | {{usage}}
{INDENT}<FILE.mid {{usage}}{{tab}}(does not work on PowerShell)

{{all-args}}{{after-help}}",
        )
        .into()
    }
}

fn help() -> HelpTemplate {
    HelpTemplate {}
}

#[derive(Subcommand)]
enum CliCommand {
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
    let smf = Smf::parse(&bytes)?;

    match args.command {
        CliCommand::Dump => dump::dump(&smf),
        CliCommand::LoopFind => {
            let opts = loop_find::Options {
                samplerate: args.samplerate,
            };
            loop_find::find(&smf, opts)
        }?,
        CliCommand::Smf0 {} => smf::smf0(&smf)?,
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
