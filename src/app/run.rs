//! Application orchestration layer.
//!
//! Coordinates the execution flow:
//! 1. Initialize localization and platform features
//! 2. Build configuration from CLI arguments
//! 3. Delegate tree construction to core::build_tree()
//! 4. Delegate rendering to format layer

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli::Args;
use crate::config::{Config, OutputFormat};
use crate::core::walker::TreeStats;
use crate::error::TreeError;
use crate::format::{HtmlFormatter, JsonFormatter, TextFormatter, TreeOutput, XmlFormatter};
use crate::i18n;

/// Main application entry point.
///
/// Accepts parsed CLI arguments and orchestrates the entire execution flow.
/// Returns an appropriate exit code based on execution result.
///
/// # Exit Codes
/// - `0` - Success
/// - `1` - Runtime error or files with errors
/// - `2` - Configuration error
/// - `3` - Path not found or not a directory
pub fn run(args: Args) -> ExitCode {
    // Initialize localization first
    i18n::init(args.lang.as_deref());

    // Platform-specific initialization
    #[cfg(windows)]
    if args.effective_color() != crate::cli::ColorWhen::Never {
        crate::windows::console::enable_ansi();
    }

    // Build configuration from arguments
    let config = match Config::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rtree: {}", e);
            return ExitCode::from(2);
        }
    };

    // Execute main logic
    let exit_code = execute(config);
    ExitCode::from(exit_code)
}

/// Execute tree traversal and rendering with the given configuration.
fn execute(config: Config) -> u8 {
    let mut total_stats = TreeStats::default();

    let paths = if config.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        config.paths.clone()
    };

    // Determine output: file or stdout
    let output: Box<dyn Write> = if let Some(ref output_path) = config.output_file {
        match std::fs::File::create(output_path) {
            Ok(file) => Box::new(file),
            Err(e) => {
                eprintln!(
                    "rtree: failed to create output file '{}': {}",
                    output_path.display(),
                    e
                );
                return 1;
            }
        }
    } else {
        Box::new(io::stdout().lock())
    };

    let result = process_paths(&config, paths, output, &mut total_stats);

    match result {
        Err(TreeError::NotFound(_)) => 3,
        Err(TreeError::NotDirectory(_)) => 3,
        Err(_) => 1,
        Ok(()) => {
            if total_stats.errors > 0 {
                1
            } else {
                0
            }
        }
    }
}

/// Process all provided paths and write output.
fn process_paths<W: Write>(
    config: &Config,
    paths: Vec<PathBuf>,
    mut output: W,
    total_stats: &mut TreeStats,
) -> Result<(), TreeError> {
    for (idx, path) in paths.iter().enumerate() {
        if !path.exists() {
            let err = TreeError::NotFound(path.clone());
            eprintln!("rtree: {}", err);
            return Err(err);
        }

        if !path.is_dir() {
            let err = TreeError::NotDirectory(path.clone());
            eprintln!("rtree: {}", err);
            return Err(err);
        }

        if idx > 0 {
            let _ = writeln!(output);
        }

        let mut stats = TreeStats::default();

        let result = match config.output_format {
            OutputFormat::Text => render_tree(
                TextFormatter::new(config),
                config,
                path,
                &mut output,
                &mut stats,
            ),
            OutputFormat::Html => render_tree(
                HtmlFormatter::new(config),
                config,
                path,
                &mut output,
                &mut stats,
            ),
            OutputFormat::Xml => render_tree(
                XmlFormatter::new(config),
                config,
                path,
                &mut output,
                &mut stats,
            ),
            OutputFormat::Json => render_tree(
                JsonFormatter::new(config),
                config,
                path,
                &mut output,
                &mut stats,
            ),
        };

        if let Err(ref e) = result {
            eprintln!("rtree: {}", e);
        }

        total_stats.directories += stats.directories;
        total_stats.files += stats.files;
        total_stats.symlinks += stats.symlinks;
        total_stats.errors += stats.errors;

        #[allow(clippy::question_mark)]
        if result.is_err() {
            return result;
        }
    }

    Ok(())
}

/// Render directory tree using the specified formatter.
///
/// Delegates tree construction to `core::build_tree()` and handles
/// only rendering and error reporting.
fn render_tree<W: Write, F: TreeOutput>(
    mut formatter: F,
    config: &Config,
    path: &std::path::Path,
    output: &mut W,
    stats: &mut TreeStats,
) -> Result<(), TreeError> {
    // Build the tree using core domain logic
    let result = match crate::core::build_tree(config, path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("rtree: {}", e);
            stats.errors += 1;
            return Err(e);
        }
    };

    // Report traversal errors to stderr
    for err in &result.errors {
        eprintln!("rtree: {}", err);
    }
    stats.errors += result.errors.len() as u64;

    // Begin rendering
    formatter.begin(output)?;

    // Render root entry
    formatter.write_entry(output, &result.root, config)?;

    if result.root.entry_type.is_directory() {
        stats.directories += 1;
    } else {
        stats.files += 1;
    }

    // Render all child entries
    for entry in &result.entries {
        formatter.write_entry(output, entry, config)?;

        if entry.entry_type.is_directory() {
            stats.directories += 1;
        } else {
            stats.files += 1;
        }

        if entry.entry_type.is_symlink() {
            stats.symlinks += 1;
        }
    }

    // Finalize
    formatter.end(output, stats, config)?;

    Ok(())
}
