#[derive(Debug)]
pub enum Command {
    FileRead(String),
    RunInlineCode(String),
    Noop,
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "Jérôme Mahuet <jerome.mahuet@gmail.com>")]
#[command(version = "0.5.0")]
#[command(name = "git")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    src: String,

    /// Number of times to greet
    #[arg(short, long)]
    run: String,
}

pub fn read_command() -> Command {
    let args = Args::parse();

    let src_path = args.src;
    let run_string = args.run;

    match (Some(src_path), Some(run_string)) {
        (Some(s), _) => Command::FileRead(s),
        (_, Some(s)) => Command::RunInlineCode(s),
        _ => Command::Noop,
    }
}
