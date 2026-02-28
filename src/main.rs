mod cli;
mod config;
mod error;
mod filter;
mod format;
mod i18n;
mod sorter;
mod style;
mod walker;

#[cfg(windows)]
mod windows;

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    mod cli_unit_tests;
}

use std::io::{self, Write};
use std::process::ExitCode;

use config::Config;
use error::TreeError;
use format::{HtmlFormatter, JsonFormatter, TextFormatter, TreeOutput, XmlFormatter};
use walker::{TreeIterator, TreeStats};

fn main() -> ExitCode {
    let args = cli::parse_args();

    // Initialize localization first
    i18n::init(args.lang.as_deref());

    #[cfg(windows)]
    if args.color != cli::ColorWhen::Never {
        windows::console::enable_ansi();
    }

    let config = match Config::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rtree: {}", e);
            return ExitCode::from(2);
        }
    };

    let exit_code = run(config);
    ExitCode::from(exit_code)
}

fn run(config: Config) -> u8 {
    let stderr = io::stderr().lock();
    let mut total_stats = TreeStats::default();

    let paths = if config.paths.is_empty() {
        vec![std::path::PathBuf::from(".")]
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

    // Use a scope to ensure stdout lock is released before we return
    let result = run_with_output(config, paths, output, stderr, &mut total_stats);

    // Handle error types for exit codes:
    // 3 = path not found / not a directory
    // 1 = other errors
    // 0 = success
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

fn run_with_output<W: Write>(
    config: Config,
    paths: Vec<std::path::PathBuf>,
    mut output: W,
    mut stderr: io::StderrLock,
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
            config::OutputFormat::Text => run_with_formatter(
                TextFormatter::new(&config),
                &config,
                path,
                &mut output,
                &mut stderr,
                &mut stats,
            ),
            config::OutputFormat::Html => run_with_formatter(
                HtmlFormatter::new(&config),
                &config,
                path,
                &mut output,
                &mut stderr,
                &mut stats,
            ),
            config::OutputFormat::Xml => run_with_formatter(
                XmlFormatter::new(&config),
                &config,
                path,
                &mut output,
                &mut stderr,
                &mut stats,
            ),
            config::OutputFormat::Json => run_with_formatter(
                JsonFormatter::new(&config),
                &config,
                path,
                &mut output,
                &mut stderr,
                &mut stats,
            ),
        };

        #[allow(clippy::question_mark)]
        if result.is_err() {
            return result;
        }

        total_stats.directories += stats.directories;
        total_stats.files += stats.files;
        total_stats.symlinks += stats.symlinks;
        total_stats.errors += stats.errors;
    }

    Ok(())
}

fn run_with_formatter<W: Write, F: TreeOutput>(
    mut formatter: F,
    config: &Config,
    path: &std::path::Path,
    stdout: &mut W,
    stderr: &mut io::StderrLock,
    stats: &mut TreeStats,
) -> Result<(), TreeError> {
    formatter.begin(stdout)?;

    // Determine lazy metadata flags for root entry
    let needs_file_id = config.one_fs || config.show_inodes || config.show_device;
    let needs_attrs = config.show_permissions;

    let root_entry = walker::TreeEntry::from_path(path, 0, true, vec![], needs_file_id, needs_attrs)?;
    formatter.write_entry(stdout, &root_entry, config)?;

    if root_entry.entry_type.is_directory() {
        stats.directories += 1;
    } else {
        stats.files += 1;
    }

    let iterator = TreeIterator::new(path, config)?;

    for entry_result in iterator {
        match entry_result {
            Ok(entry) => {
                formatter.write_entry(stdout, &entry, config)?;

                if entry.entry_type.is_directory() {
                    stats.directories += 1;
                } else {
                    stats.files += 1;
                }

                if entry.entry_type.is_symlink() {
                    stats.symlinks += 1;
                }
            }
            Err(e) => {
                let _ = writeln!(stderr, "rtree: {}", e);
                stats.errors += 1;
            }
        }
    }

    formatter.end(stdout, stats, config)?;

    Ok(())
}
