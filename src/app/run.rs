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
use crate::core::walker::StreamingEngine;
use crate::core::walker::TreeStats;
use crate::error::{diag_error, diag_warn, report_errors, TreeError};
use crate::i18n;
use crate::render::TextRenderer;

/// Main application entry point.
pub fn run(args: Args) -> ExitCode {
    // Initialize localization (idempotent — safe if main.rs already called it)
    i18n::init(args.lang.as_deref());

    // Build configuration from arguments
    let config = match Config::build(args) {
        Ok(c) => c,
        Err(e) => {
            diag_error(&e);
            return ExitCode::from(2);
        }
    };

    run_with_config(config)
}

/// Execute with a pre-built Config. Allows testing without clap parsing.
pub fn run_with_config(config: Config) -> ExitCode {
    // Platform-specific: enable ANSI escapes on Windows console
    if config.color_enabled {
        crate::platform::enable_ansi();
    }

    ExitCode::from(execute(config))
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
                diag_error(format_args!(
                    "failed to create output file '{}': {}",
                    output_path.display(),
                    e
                ));
                return 1;
            }
        }
    } else {
        Box::new(io::stdout().lock())
    };

    let result = process_paths(&config, paths, &mut output, &mut total_stats);

    // Ensure output is flushed before exit
    if let Err(e) = output.flush() {
        diag_error(format_args!("error writing output: {}", e));
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
            diag_error(&err);
            return Err(err);
        }

        if !path.is_dir() {
            let err = TreeError::NotDirectory(path.clone());
            diag_error(&err);
            return Err(err);
        }

        if idx > 0 {
            let _ = writeln!(output);
        }

        let mut stats = TreeStats::default();

        let result = render_tree(config, path, output, &mut stats);

        if let Err(ref err) = result {
            diag_error(err);
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
    if config.streaming {
        let text_render = TextRenderer::new();
        let engine = StreamingEngine::new(config, &text_render);
        match engine.traverse_and_render(path, output, stats) {
            Ok(result) => {
                stats.errors += report_errors(&result.errors);
                if result.truncated {
                    diag_warn(format_args!(
                        "output truncated at {} entries (--max-entries)",
                        config.max_entries.unwrap_or(0)
                    ));
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
            diag_error(&e);
            stats.errors += 1;
            return Err(e);
        }
    };

    stats.errors += report_errors(&result.errors);

    // Dispatch to appropriate render backend
    let dispatch_result = crate::render::dispatch(&result, config, output, stats);

    // Notify user if output was truncated by --max-entries
    if result.truncated {
        diag_warn(format_args!(
            "output truncated at {} entries (--max-entries)",
            config.max_entries.unwrap_or(0)
        ));
    }

    dispatch_result
}
