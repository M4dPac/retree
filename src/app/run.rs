//! Application orchestration layer.
//!
//! Coordinates the execution flow:
//! 1. Initialize localization and platform features
//! 2. Build configuration from CLI arguments
//! 3. Delegate tree construction to core::build_tree()
//! 4. Delegate rendering to render::dispatch()

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli::Args;
use crate::config::Config;
use crate::config::OutputFormat;
use crate::core::walker::StreamingEngine;
use crate::core::walker::TreeStats;
use crate::error::TreeError;
use crate::i18n;
use crate::render::TextRenderer;

/// Main application entry point.
pub fn run(args: Args) -> ExitCode {
    // Initialize localization first
    i18n::init(args.lang.as_deref());

    // Platform-specific initialization
    if args.effective_color() != crate::cli::ColorWhen::Never {
        crate::platform::enable_ansi();
    }

    // Build configuration from arguments
    let config = match Config::build(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rtree: {}", e);
            return ExitCode::from(2);
        }
    };

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
    let mut output: Box<dyn Write> = if let Some(ref output_path) = config.output_file {
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

    let result = process_paths(&config, paths, &mut output, &mut total_stats);

    // Ensure output is flushed before exit
    if let Err(e) = output.flush() {
        eprintln!("rtree: error writing output: {}", e);
        return 1;
    }

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
    output: &mut W,
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

        let result = render_tree(config, path, output, &mut stats);

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

/// Build tree and dispatch to renderer.
fn render_tree<W: Write>(
    config: &Config,
    path: &std::path::Path,
    output: &mut W,
    stats: &mut TreeStats,
) -> Result<(), TreeError> {
    // Streaming mode: text-only, traverse and render in single pass
    if config.streaming && config.output_format == OutputFormat::Text && !config.prune {
        let text_render = TextRenderer::new(config);
        let engine = StreamingEngine::new(config, &text_render);
        match engine.traverse_and_render(path, output, stats) {
            Ok(result) => {
                for err in &result.errors {
                    eprintln!("rtree: {}", err);
                }
                let hard_errors = result
                    .errors
                    .iter()
                    .filter(|e| !matches!(e, TreeError::ReservedName(_)))
                    .count();
                stats.errors += hard_errors as u64;
                if result.truncated {
                    let max = config.max_entries.unwrap_or(0);
                    eprintln!("rtree: output truncated at {} entries (--max-entries)", max);
                }
                return Ok(());
            }
            Err(_) => {
                // Streaming failed — reset stats before falling through
                // to prevent double-counting if streaming partially updated them.
                *stats = TreeStats::default();
            }
        }
    }

    // Build the tree using core domain logic
    let result = match crate::core::build_tree(config, path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("rtree: {}", e);
            stats.errors += 1;
            return Err(e);
        }
    };

    // Report traversal errors/warnings to stderr
    for err in &result.errors {
        eprintln!("rtree: {}", err);
    }
    // ReservedName is an informational warning, not a hard error — don't affect exit code
    let hard_errors = result
        .errors
        .iter()
        .filter(|e| !matches!(e, TreeError::ReservedName(_)))
        .count();
    stats.errors += hard_errors as u64;

    // Dispatch to appropriate render backend
    let dispatch_result = crate::render::dispatch(&result, config, output, stats);

    // Notify user if output was truncated by --max-entries
    if result.truncated {
        let max = config.max_entries.unwrap_or(0);
        eprintln!("rtree: output truncated at {} entries (--max-entries)", max);
    }

    dispatch_result
}
