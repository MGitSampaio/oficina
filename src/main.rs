mod app;
mod db;
mod model;
mod print;
mod theme;
mod ui;
mod util;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Oficina — gestão de assistência técnica\n\n\
             Uso:\n  \
             oficina            abre o sistema\n  \
             oficina --demo     cria dados de exemplo no banco local e encerra\n  \
             oficina --help     mostra esta ajuda"
        );
        return ExitCode::SUCCESS;
    }

    if args.iter().any(|arg| arg == "--demo") {
        match db::Store::open_default().and_then(|store| {
            let path = store.path.clone();
            store.seed_demo().map(|_| path)
        }) {
            Ok(path) => {
                println!("Dados de exemplo disponíveis em {}", path.display());
                return ExitCode::SUCCESS;
            }
            Err(err) => {
                eprintln!("Não foi possível preparar os dados de exemplo: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    if let Err(err) = run_app() {
        eprintln!("Erro ao abrir a interface: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run_app() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1220.0, 790.0])
            .with_min_inner_size([1020.0, 640.0])
            .with_title("Oficina — Gestão de assistência técnica"),
        ..Default::default()
    };

    eframe::run_native(
        "Oficina",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc)))
        }),
    )
}
