mod render;
mod screen;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use usage_rs::{Args, Cli, Subcommands, ValueEnum, complete::Shell};

/// Capture terminal panes as PNG, SVG, or selectable, themeable HTML.
#[derive(Cli)]
#[usage(bin = "tui-shot", version = env!("CARGO_PKG_VERSION"), completion)]
struct App {
    #[usage(flatten)]
    capture: Capture,

    #[usage(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommands)]
enum Commands {
    /// Generate a shell completion script.
    Completions(Completions),
}

#[derive(Args)]
struct Completions {
    /// Shell to generate completions for.
    #[usage(value_enum)]
    shell: CompletionShell,
}

#[derive(ValueEnum)]
enum CompletionShell {
    Bash,
    Fish,
    Zsh,
}

#[derive(PartialEq, Eq)]
enum OutputFormat {
    Ansi,
    Html,
    Png,
    Svg,
    Text,
}

impl OutputFormat {
    fn from_path(path: &Path) -> Result<Self, &'static str> {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "ansi" => Ok(Self::Ansi),
            "html" => Ok(Self::Html),
            "png" => Ok(Self::Png),
            "svg" => Ok(Self::Svg),
            "txt" => Ok(Self::Text),
            _ => Err("expected a .png, .svg, .html, .txt, or .ansi output path"),
        }
    }
}

#[derive(Args)]
struct Capture {
    /// Herdr pane ID (defaults to the calling pane).
    #[usage(long)]
    pane: Option<String>,

    /// Read a saved ANSI viewport instead; use - for stdin.
    #[usage(long, value_hint = usage_rs::ValueHint::FilePath)]
    input: Option<PathBuf>,

    /// Output path (.png, .svg, .html, .txt, or .ansi).
    #[usage(short = 'o', long, value_hint = usage_rs::ValueHint::FilePath)]
    output: Option<PathBuf>,

    /// Color palette.
    #[usage(long, value_enum, default = "dark")]
    theme: render::Theme,

    /// Add a title bar.
    #[usage(long)]
    title: Option<String>,

    /// Print embeddable HTML instead of a complete HTML page.
    #[usage(long)]
    fragment: bool,
}

fn main() -> ExitCode {
    match run(App::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("tui-shot: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(app: App) -> Result<(), Box<dyn Error>> {
    if let Some(Commands::Completions(cmd)) = app.command {
        let script = App::completion_script(match cmd.shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Zsh => Shell::Zsh,
        });
        print!("{script}");
        return Ok(());
    }

    let args = app.capture;
    let output = args.output.ok_or("pass --output (see --help)")?;
    let kind = OutputFormat::from_path(&output)?;
    if args.fragment && kind != OutputFormat::Html {
        return Err("--fragment requires .html output".into());
    }
    let data = match (args.input, args.pane) {
        (Some(_), Some(_)) => return Err("--input and --pane cannot be used together".into()),
        (Some(path), None) if path == Path::new("-") => {
            let mut data = Vec::new();
            io::stdin().lock().read_to_end(&mut data)?;
            data
        }
        (Some(path), None) => fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        (None, pane) => capture_pane(pane)?,
    };
    let screen = || screen::parse(&String::from_utf8_lossy(&data));
    let title = screen::clean_title(args.title.as_deref().unwrap_or_default());
    let rendered = match kind {
        OutputFormat::Ansi => data,
        OutputFormat::Html => {
            render::html(&screen(), args.theme, &title, args.fragment).into_bytes()
        }
        OutputFormat::Text => render::text(&screen()).into_bytes(),
        OutputFormat::Svg => render::svg(&screen(), args.theme, &title).into_bytes(),
        OutputFormat::Png => render::png(&render::svg(&screen(), args.theme, &title))?,
    };
    fs::write(&output, rendered).map_err(|e| format!("{}: {e}", output.display()))?;
    Ok(())
}

fn capture_pane(pane: Option<String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let pane = pane
        .or_else(|| env::var("HERDR_PANE_ID").ok())
        .filter(|s| !s.is_empty())
        .ok_or("pass --pane or --input; HERDR_PANE_ID is not set")?;
    if pane.starts_with('-') {
        return Err("expected a pane ID, not an option".into());
    }
    let read = Command::new("herdr")
        .args([
            "pane", "read", &pane, "--source", "visible", "--format", "ansi", "--raw",
        ])
        .output()
        .map_err(|err| format!("could not run herdr: {err}"))?;
    if !read.status.success() {
        return Err(format!(
            "herdr pane read failed ({})\n{}",
            read.status,
            String::from_utf8_lossy(&read.stderr).trim_end()
        )
        .into());
    }
    Ok(read.stdout)
}
