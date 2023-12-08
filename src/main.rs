use std::error::Error;

use clap::Parser;

struct HelpTemplate {}

const INDENT: &str = "  ";

impl From<HelpTemplate> for clap::builder::StyledStr {
    fn from(_: HelpTemplate) -> Self {
        format!(
            "{{about-with-newline}}
{{usage-heading}}
{INDENT}{{usage}}

{{all-args}}{{after-help}}",
        )
        .into()
    }
}

fn help() -> HelpTemplate {
    HelpTemplate {}
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
struct Cli {}

fn run(_: Cli) -> Result<(), Box<dyn Error>> {
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
