use clap::{CommandFactory, Parser};
use std::process::ExitCode;

fn main() -> ExitCode {
    // 1. Определяем язык ДО полного парсинга clap.
    //    Это нужно, чтобы --help вывелся сразу на нужном языке.
    let lang = retree::cli::detect_language_early();

    // 2. Инициализируем i18n-систему с определённым языком.
    retree::i18n::init(Some(lang.code()));

    // 3. Если передан --help / -h — выводим локализованный help и выходим.
    if retree::cli::has_help_flag() {
        let mut cmd = retree::cli::build_localized_command(lang);
        if cmd.print_help().is_err() {
            return ExitCode::from(1);
        }
        println!(); // финальный перевод строки
        return ExitCode::SUCCESS;
    }

    // 4. Обычный парсинг.
    let args = retree::cli::Args::parse();

    // 4a. Если запрошены completions — вывести и выйти.
    if let Some(shell) = args.completions {
        let mut cmd = retree::cli::Args::command();
        clap_complete::generate(shell, &mut cmd, "rt", &mut std::io::stdout());
        return ExitCode::SUCCESS;
    }

    // 5. Запуск на потоке с гарантированным стеком 8 МиБ.
    //    Windows выделяет main-thread всего 1 МиБ (vs 8 МиБ на Linux/macOS).
    //    Рекурсивный walker тратит ~10–25 КиБ на фрейм (debug-сборка).
    //    Без этого stack overflow возникает уже на ~100 уровнях вложенности.
    const STACK_SIZE: usize = 8 * 1024 * 1024;

    let fallback_args = args.clone();

    let builder = std::thread::Builder::new()
        .name("retree-main".into())
        .stack_size(STACK_SIZE);

    match builder.spawn(move || retree::app::run(args)) {
        Ok(handle) => match handle.join() {
            Ok(code) => code,
            Err(_) => ExitCode::from(1),
        },
        // Thread creation failed — run on current thread (best effort)
        Err(_) => retree::app::run(fallback_args),
    }
}
