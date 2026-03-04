use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    // 1. Определяем язык ДО полного парсинга clap.
    //    Это нужно, чтобы --help вывелся сразу на нужном языке.
    let lang = rtree::cli::detect_language_early();

    // 2. Инициализируем i18n-систему с определённым языком.
    rtree::i18n::init(Some(lang.code()));

    // 3. Если передан --help / -h — выводим локализованный help и выходим.
    if rtree::cli::has_help_flag() {
        let mut cmd = rtree::cli::build_localized_command(lang);
        cmd.print_help().expect("failed to print help");
        println!(); // финальный перевод строки
        return ExitCode::SUCCESS;
    }

    // 4. Обычный парсинг и запуск приложения.
    let args = rtree::cli::Args::parse();
    rtree::app::run(args)
}
