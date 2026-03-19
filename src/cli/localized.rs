use clap::CommandFactory;

use crate::i18n::{get_message, Language, MessageKey};

use super::Args;

pub fn build_localized_command(lang: Language) -> clap::Command {
    Args::command()
        // --- App-level ---
        .about(msg(lang, MessageKey::AppDescription))
        .after_help(msg(lang, MessageKey::AppAfterHelp))
        // Отключаем встроенный --version, добавляем локализованный
        .disable_version_flag(true)
        .arg(
            clap::Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(msg(lang, MessageKey::ArgVersion))
                .help_heading(msg(lang, MessageKey::HeadingOptions)),
        )
        // --- Positional ---
        .mut_arg("paths", |a| a.help(msg(lang, MessageKey::ArgPaths)))
        // --- Listing Options ---
        .mut_arg("all", |a| {
            a.help(msg(lang, MessageKey::ArgAll))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("dirs_only", |a| {
            a.help(msg(lang, MessageKey::ArgDirsOnly))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("follow_symlinks", |a| {
            a.help(msg(lang, MessageKey::ArgFollow))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("full_path", |a| {
            a.help(msg(lang, MessageKey::ArgFullPath))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("one_fs", |a| {
            a.help(msg(lang, MessageKey::ArgOneFs))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("max_depth", |a| {
            a.help(msg(lang, MessageKey::ArgLevel))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("file_limit", |a| {
            a.help(msg(lang, MessageKey::ArgFileLimit))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        .mut_arg("no_report", |a| {
            a.help(msg(lang, MessageKey::ArgNoReport))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        // --- Filtering ---
        .mut_arg("pattern", |a| {
            a.help(msg(lang, MessageKey::ArgPattern))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("exclude", |a| {
            a.help(msg(lang, MessageKey::ArgExclude))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("match_dirs", |a| {
            a.help(msg(lang, MessageKey::ArgMatchDirs))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("ignore_case", |a| {
            a.help(msg(lang, MessageKey::ArgIgnoreCase))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("prune", |a| {
            a.help(msg(lang, MessageKey::ArgPrune))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("ctime_sort", |a| {
            a.help(msg(lang, MessageKey::ArgCtimeSort))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("unsorted", |a| {
            a.help(msg(lang, MessageKey::ArgUnsorted))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("reverse", |a| {
            a.help(msg(lang, MessageKey::ArgReverse))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("dirs_first", |a| {
            a.help(msg(lang, MessageKey::ArgDirsFirst))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("files_first", |a| {
            a.help(msg(lang, MessageKey::ArgFilesFirst))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        .mut_arg("sort", |a| {
            a.help(msg(lang, MessageKey::ArgSort))
                .help_heading(msg(lang, MessageKey::HeadingFiltering))
        })
        // --- Sorting ---
        .mut_arg("version_sort", |a| {
            a.help(msg(lang, MessageKey::ArgVersionSort))
                .help_heading(msg(lang, MessageKey::HeadingSorting))
        })
        .mut_arg("time_sort", |a| {
            a.help(msg(lang, MessageKey::ArgTimeSort))
                .help_heading(msg(lang, MessageKey::HeadingSorting))
        })
        // --- Display ---
        .mut_arg("no_indent", |a| {
            a.help(msg(lang, MessageKey::ArgNoIndent))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        .mut_arg("ansi", |a| {
            a.help(msg(lang, MessageKey::ArgAnsi))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        .mut_arg("cp437", |a| {
            a.help(msg(lang, MessageKey::ArgCp437))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        .mut_arg("no_color", |a| {
            a.help(msg(lang, MessageKey::ArgNoColor))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        .mut_arg("color_always", |a| {
            a.help(msg(lang, MessageKey::ArgColorAlways))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        .mut_arg("color", |a| {
            a.help(msg(lang, MessageKey::ArgColor))
                .help_heading(msg(lang, MessageKey::HeadingDisplay))
        })
        // --- File Information ---
        .mut_arg("size", |a| {
            a.help(msg(lang, MessageKey::ArgSize))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("human_readable", |a| {
            a.help(msg(lang, MessageKey::ArgHuman))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("help", |a| {
            a.help(msg(lang, MessageKey::ArgHelp))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("si_units", |a| {
            a.help(msg(lang, MessageKey::ArgSi))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("date", |a| {
            a.help(msg(lang, MessageKey::ArgDate))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("time_fmt", |a| {
            a.help(msg(lang, MessageKey::ArgTimeFmt))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("permissions", |a| {
            a.help(msg(lang, MessageKey::ArgPerm))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("uid", |a| {
            a.help(msg(lang, MessageKey::ArgUid))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("gid", |a| {
            a.help(msg(lang, MessageKey::ArgGid))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("inodes", |a| {
            a.help(msg(lang, MessageKey::ArgInodes))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("device", |a| {
            a.help(msg(lang, MessageKey::ArgDevice))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("classify", |a| {
            a.help(msg(lang, MessageKey::ArgClassify))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("safe_print", |a| {
            a.help(msg(lang, MessageKey::ArgSafe))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("literal", |a| {
            a.help(msg(lang, MessageKey::ArgLiteral))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        .mut_arg("charset", |a| {
            a.help(msg(lang, MessageKey::ArgCharset))
                .help_heading(msg(lang, MessageKey::HeadingFileInformation))
        })
        // --- Export ---
        .mut_arg("output_file", |a| {
            a.help(msg(lang, MessageKey::ArgOutput))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("html_base", |a| {
            a.help(msg(lang, MessageKey::ArgHtml))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("html_title", |a| {
            a.help(msg(lang, MessageKey::ArgTitle))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("no_links", |a| {
            a.help(msg(lang, MessageKey::ArgNoLinks))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("html_intro", |a| {
            a.help(msg(lang, MessageKey::ArgHtmlIntro))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("html_outro", |a| {
            a.help(msg(lang, MessageKey::ArgHtmlOutro))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("xml", |a| {
            a.help(msg(lang, MessageKey::ArgXml))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("json", |a| {
            a.help(msg(lang, MessageKey::ArgJson))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        .mut_arg("json_pretty", |a| {
            a.help(msg(lang, MessageKey::ArgJsonPretty))
                .help_heading(msg(lang, MessageKey::HeadingExport))
        })
        // --- Performance ---
        .mut_arg("parallel", |a| {
            a.help(msg(lang, MessageKey::ArgParallel))
                .help_heading(msg(lang, MessageKey::HeadingPerformance))
        })
        .mut_arg("streaming", |a| {
            a.help(msg(lang, MessageKey::ArgStreaming))
                .help_heading(msg(lang, MessageKey::HeadingPerformance))
        })
        .mut_arg("threads", |a| {
            a.help(msg(lang, MessageKey::ArgThreads))
                .help_heading(msg(lang, MessageKey::HeadingPerformance))
        })
        .mut_arg("queue_cap", |a| {
            a.help(msg(lang, MessageKey::ArgQueueCap))
                .help_heading(msg(lang, MessageKey::HeadingPerformance))
        })
        .mut_arg("max_entries", |a| {
            a.help(msg(lang, MessageKey::ArgMaxEntries))
                .help_heading(msg(lang, MessageKey::HeadingListingOptions))
        })
        // --- Icons ---
        .mut_arg("icons", |a| {
            a.help(msg(lang, MessageKey::ArgIcons))
                .help_heading(msg(lang, MessageKey::HeadingIcons))
        })
        .mut_arg("no_icons", |a| {
            a.help(msg(lang, MessageKey::ArgNoIcons))
                .help_heading(msg(lang, MessageKey::HeadingIcons))
        })
        .mut_arg("icon_style", |a| {
            a.help(msg(lang, MessageKey::ArgIconStyle))
                .help_heading(msg(lang, MessageKey::HeadingIcons))
        })
        // --- Windows ---
        .mut_arg("show_streams", |a| {
            a.help(msg(lang, MessageKey::ArgShowStreams))
                .help_heading(msg(lang, MessageKey::HeadingWindows))
        })
        .mut_arg("show_junctions", |a| {
            a.help(msg(lang, MessageKey::ArgShowJunctions))
                .help_heading(msg(lang, MessageKey::HeadingWindows))
        })
        .mut_arg("hide_system", |a| {
            a.help(msg(lang, MessageKey::ArgHideSystem))
                .help_heading(msg(lang, MessageKey::HeadingWindows))
        })
        .mut_arg("perm_mode", |a| {
            a.help(msg(lang, MessageKey::ArgPermissions))
                .help_heading(msg(lang, MessageKey::HeadingWindows))
        })
        .mut_arg("long_paths", |a| {
            a.help(msg(lang, MessageKey::ArgLongPaths))
                .help_heading(msg(lang, MessageKey::HeadingWindows))
        })
        // --- Localization ---
        .mut_arg("lang", |a| {
            a.help(msg(lang, MessageKey::ArgLang))
                .help_heading(msg(lang, MessageKey::HeadingLocalization))
        })
}

#[inline]
fn msg(lang: Language, key: MessageKey) -> String {
    get_message(lang, key).to_string()
}
