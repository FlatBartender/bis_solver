mod solver;
mod utils;
mod ui;
mod data;

use ui::*;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    let (status_send, status_recv) = std::sync::mpsc::channel();

    let app_link = UiLink::new(status_send);
    std::thread::spawn({
        let calc_app_link = app_link.clone();
        move || {
            solver::calc_sets(calc_app_link).unwrap();
        }
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "BiS Solver",
        native_options,
        Box::new(|cc| Box::new(Ui::new(cc, status_recv))),
    );

    Ok(())
}

