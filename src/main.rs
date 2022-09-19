mod solver;
mod utils;
mod ui;
mod data;
mod xivapi;

use ui::*;


impl TryFrom<csv::StringRecord> for data::Item {
    type Error = eyre::Error;

    fn try_from(record: csv::StringRecord) -> eyre::Result<Self> {
        let item = Self {
            slot: record.get(0).unwrap().parse().unwrap(),
            name: record.get(1).unwrap().to_string(),
            stats: data::Stats {
                weapon_damage: record.get(2).unwrap().parse().unwrap_or_default(),
                mind: record.get(3).unwrap().parse().unwrap_or_default(),
                vitality: record.get(4).unwrap().parse().unwrap_or_default(),
                piety: record.get(5).unwrap().parse().unwrap_or_default(),
                direct_hit: record.get(6).unwrap().parse().unwrap_or_default(),
                critical: record.get(7).unwrap().parse().unwrap_or_default(),
                determination: record.get(8).unwrap().parse().unwrap_or_default(),
                spell_speed: record.get(9).unwrap().parse().unwrap_or_default(),
            },
            meld_slots: record.get(10).unwrap().parse().unwrap_or_default(),
            overmeldable: record.get(11).unwrap().parse().unwrap_or_default(),
        };

        Ok(item)
    }
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "BiS Solver",
        native_options,
        Box::new(|cc| Box::new(Ui::new(cc).unwrap())),
    );

    Ok(())
}

