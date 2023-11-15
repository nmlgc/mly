mod dump;
mod loop_find;
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

    /// Finds the longest fully repeated range of MIDI events.
    #[command(help_template = help())]
    LoopFind,
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
    #[command(subcommand)]
    command: CliCommand,
}

fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::new();
    io::stdin().lock().read_to_end(&mut bytes)?;
    let smf = Smf::parse(&bytes)?;

    match args.command {
        CliCommand::Dump => dump::dump(&smf),
        CliCommand::LoopFind => loop_find::find(&smf)?,
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
