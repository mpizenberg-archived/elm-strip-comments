use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

mod parser;

const USAGE: &str = r#"
elm-strip-comments

USAGE:
    elm-strip-comments Module.elm > ModuleNoComment.elm
    elm-strip-comments --replace src/*.elm
"#;

/// Main entry point.
fn main() {
    parse_args()
        .and_then(run)
        .unwrap_or_else(|err| eprintln!("Error: {:?}", err));
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    Ok(Args {
        help: args.contains("--help"),
        version: args.contains("--version"),
        replace: args.contains("--replace"),
        files: args.free()?,
    })
}

#[derive(Debug)]
/// Options passed as arguments.
pub struct Args {
    pub help: bool,
    pub version: bool,
    pub replace: bool,
    pub files: Vec<String>,
}

/// Start actual program with command line arguments successfully parsed.
fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the --help or --version flags are present.
    if args.help {
        println!("{}", USAGE);
        return Ok(());
    } else if args.version {
        println!("{}", std::env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.files.is_empty() {
        return Err("Missing path to an elm file".into());
    }

    // Get file paths of all modules in canonical form (absolute path)
    let files_abs_paths: Vec<PathBuf> = args
        .files
        .iter()
        // join expanded globs for each pattern
        .flat_map(|pattern| {
            glob(pattern)
                .unwrap_or_else(|_| panic!(format!("Failed to read glob pattern {}", pattern)))
        })
        // filter out errors
        .filter_map(|x| x.ok())
        // canonical form of paths
        .map(|path| {
            path.canonicalize()
                .unwrap_or_else(|_| panic!(format!("Error in canonicalize of {:?}", path)))
        })
        // collect into a set of unique values
        .collect();

    match (files_abs_paths.len(), args.replace) {
        (0, _) => Err("No matching file found".into()),
        (1, false) => {
            print!("{}", strip_comments(&files_abs_paths[0])?);
            Ok(())
        }
        (1, true) => {
            let stripped = strip_comments(&files_abs_paths[0])?;
            fs::write(&files_abs_paths[0], stripped).map_err(|e| e.into())
        }
        (_, false) => Err("You must use --replace for multiple elm files".into()),
        (_, true) => {
            for file in files_abs_paths.iter() {
                let stripped = strip_comments(file)?;
                fs::write(file, stripped)?;
            }
            Ok(())
        }
    }
}

fn strip_comments<F: AsRef<Path>>(file: F) -> Result<String, Box<dyn std::error::Error>> {
    let file_str = fs::read_to_string(file)?;
    let (_, stripped) = parser::remove_comments(&file_str).unwrap();
    Ok(stripped)
}
