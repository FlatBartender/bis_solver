use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::BitOr;

use eframe::egui;

use crate::data::ITEM_SLOTS;
use crate::solver::*;

impl crate::data::Gearset {
    pub fn table_ui(&self, ui: &mut egui::Ui) {
        use egui_extras::{Size, TableBuilder};
        let text_size = egui::TextStyle::Body.resolve(ui.style()).size;

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(60.0).at_least(60.0))
            .column(Size::remainder().at_least(260.0))
            .column(Size::initial(50.0).at_least(50.0))
            .column(Size::initial(40.0).at_least(40.0))
            .column(Size::initial(40.0).at_least(40.0))
            .column(Size::initial(80.0).at_least(80.0))
            .column(Size::initial(50.0).at_least(50.0))
            .column(Size::initial(40.0).at_least(40.0))
            .column(Size::initial(40.0).at_least(40.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Slot");
            });
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("WD");
            });
            header.col(|ui| {
                ui.heading("Mind");
            });
            header.col(|ui| {
                ui.heading("DH");
            });
            header.col(|ui| {
                ui.heading("Crit");
            });
            header.col(|ui| {
                ui.heading("Det");
            });
            header.col(|ui| {
                ui.heading("SpS");
            });
            header.col(|ui| {
                ui.heading("Pie");
            });
        })
        .body(|mut body| {
            for (slot_index, slot) in ITEM_SLOTS.iter().enumerate() {
                body.row(text_size, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("{}", slot));
                    });
                    self.items[slot_index].row_ui(&mut row);
                });
            }
            body.row(text_size, |mut row| {
                row.col(|ui| {
                    ui.label("Food");
                });
                self.food.row_ui(&mut row);
            });
            body.separator(text_size);
            body.row(text_size, |mut row| {
                row.col(|ui| {
                    ui.label("Materia X");
                });
                self.meld_x.row_ui(&mut row);
            });
            body.row(text_size, |mut row| {
                row.col(|ui| {
                    ui.label("Materia IX");
                });
                self.meld_ix.row_ui(&mut row);
            });
            body.separator(text_size);
            body.row(text_size, |mut row| {
                row.col(|ui| {
                    ui.label("Stats");
                });
                row.col(|_ui| {
                });
                self.stats().row_ui(&mut row);
            });
            body.row(text_size, |mut row| {
                row.col(|ui| {
                    ui.label("Values");
                });
                row.col(|_ui| {
                });
                self.stats().row_ui_in_depth(&mut row);
            });
        });
    }
}

pub trait StatRepoUi {
    fn row_ui(&self, row: &mut egui_extras::TableRow);
    fn row_ui_in_depth(&self, row: &mut egui_extras::TableRow);
}

impl<T: crate::data::StatRepo> StatRepoUi for T {
    fn row_ui(&self, row: &mut egui_extras::TableRow) {
        use std::num::NonZeroU32;

        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.weapon_damage()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.mind()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.direct_hit()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.critical()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.determination()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.spell_speed()) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self.piety()) {
                ui.label(val.to_string());
            }
        });
    }

    fn row_ui_in_depth(&self, row: &mut egui_extras::TableRow) {
        row.col(|_ui| {});
        row.col(|_ui| {});
        row.col(|ui| {
            ui.label(format!("{:.2}%", self.dh_rate().scalar()*100.0));
        });
        row.col(|ui| {
            ui.label(format!("{:.2}%×{:.3}", self.crit_rate().scalar()*100.0, self.crit_multiplier().scalar()));
        });
        row.col(|ui| {
            ui.label(format!("{:.2}%", self.det_multiplier().scalar()*100.0));
        });
        row.col(|ui| {
            ui.label(format!("{:.2}", self.gcd().scalar()));
        });
        row.col(|_ui| {});

    }
}

trait Separable {
    fn separator(&mut self, text_size: f32);
}

impl<'a> Separable for egui_extras::TableBody<'a> {
    fn separator(&mut self, text_size: f32) {
        let widths: Vec<_> = self.widths().to_vec();
        self.row(0.0, |_row| {});
        self.row(text_size/5.0, |mut row| {
            for _ in widths {
                row.col(|ui| {
                    ui.add(egui::Separator::default().horizontal());
                });
            }
        });
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tabs_panel").show(ctx, |ui| {
            self.tabs(ui);
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            self.status_bar(ui);
        });

        egui::SidePanel::left("gearset_selector").show(ctx, |ui| {
            self.gearset_selector(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Configuration => self.solver_tab(ui),
                Tab::Comparator => self.comparator_tab(ui),
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

#[derive(Clone)]
pub struct UiLink {
    status_text: Arc<Mutex<String>>,
    count: Arc<AtomicUsize>,
    gearsets: Arc<Mutex<Vec<crate::data::Gearset>>>,
}

impl UiLink {
    pub fn new() -> Self {
        Self {
            status_text: Arc::default(),
            count: Arc::default(),
            gearsets: Arc::default(),
        }
    }

    pub fn message(&self, message: impl ToString) -> eyre::Result<()> {
        let message = message.to_string();
        *self.status_text.lock().unwrap() = message;
        Ok(())
    }

    pub fn increment(&self) -> eyre::Result<()> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn set_count(&self, count: usize) -> eyre::Result<()> {
        self.count.store(count, Ordering::Relaxed);
        Ok(())
    }

    fn new_gearsets(&self, gearsets: Vec<crate::data::Gearset>) -> eyre::Result<()> {
        *self.gearsets.lock().unwrap() = gearsets;
        Ok(())
    }
}

fn load_items() -> eyre::Result<Vec<crate::data::Item>> {
    const ITEMS: &str = include_str!("items.csv");

    let csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .quoting(false)
        .from_reader(ITEMS.as_bytes());

    let records: Vec<_> = csv_reader.into_records()
        .collect::<Result<_, _>>()?;
    records.into_iter()
        .map(crate::data::Item::try_from)
        .collect::<Result<Vec<_>, _>>()
}

#[derive(PartialEq, Eq)]
enum Tab {
    Configuration,
    Comparator,
}

pub struct TimelineConfig {
    mind_bonus: f64,
    brd: bool,
    dnc: bool,
    smn: bool,
    rdm: bool,
    mnk: bool,
    drg: bool,
    rpr: bool,
    nin: bool,
    sch: bool,
    ast: bool,
    potions: bool,
    downtimes: Vec<Timespan>,
    kill_time: f64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            mind_bonus: Default::default(),
            brd: Default::default(),
            dnc: Default::default(),
            smn: Default::default(),
            rdm: Default::default(),
            mnk: Default::default(),
            drg: Default::default(),
            rpr: Default::default(),
            nin: Default::default(),
            sch: Default::default(),
            ast: Default::default(),
            potions: Default::default(),
            downtimes: Default::default(),
            kill_time: 600.0,
        }
    }
}

pub struct Ui {
    ui_link: UiLink,

    gearsets: Arc<Mutex<Vec<crate::data::Gearset>>>,

    selected_gearset_a: Option<usize>,
    selected_gearset_b: Option<usize>,

    items: Vec<crate::data::Item>,

    solver: std::sync::Arc<dyn crate::solver::Solver + Send + Sync>,
    solver_type: crate::solver::SolverType,
    evaluator_type: crate::solver::EvaluatorType,

    split_config: SplitConfig,
    rolling_config: RollingConfig,
    timeline_config: TimelineConfig,
    config_changed: bool,

    tab: Tab,
}

impl Ui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> eyre::Result<Self> {
        let items = load_items()?;
        let ui_link = UiLink::new();
        let evaluator = crate::solver::InfiniteDummyEvaluator::default();
        Ok(Self {
            ui_link: ui_link.clone(),

            gearsets: ui_link.gearsets.clone(),

            selected_gearset_a: None,
            selected_gearset_b: None,

            items: items.clone(),

            solver: std::sync::Arc::new(
                crate::solver::SplitSolver::new(ui_link, Arc::new(evaluator))
                    .with_items(items)
                    .with_config(SplitConfig::default())
            ),
            solver_type: crate::solver::SolverType::Split,
            evaluator_type: crate::solver::EvaluatorType::InfiniteDummy,

            split_config: SplitConfig::default(),
            rolling_config: RollingConfig::default(),
            timeline_config: TimelineConfig::default(),
            config_changed: false,

            tab: Tab::Configuration,
        })
    }

    fn rebuild_solver(&mut self) {
        self.config_changed = false;
        let evaluator = match self.evaluator_type {
            EvaluatorType::InfiniteDummy => Arc::new(InfiniteDummyEvaluator::default()) as _,
            EvaluatorType::Timeline => {
                // P5S: vec![Timespan::new(255.0, 267.0)]
                let mut timeline = Timeline::new(
                    self.timeline_config.downtimes.clone(),
                    self.timeline_config.kill_time,
                    self.timeline_config.mind_bonus,
                );
                if self.timeline_config.brd { timeline.with_brd(); };
                if self.timeline_config.dnc { timeline.with_dnc(); };
                if self.timeline_config.smn { timeline.with_smn(); };
                if self.timeline_config.rdm { timeline.with_rdm(); };
                if self.timeline_config.mnk { timeline.with_mnk(); };
                if self.timeline_config.drg { timeline.with_drg(); };
                if self.timeline_config.rpr { timeline.with_rpr(); };
                if self.timeline_config.nin { timeline.with_nin(); };
                if self.timeline_config.sch { timeline.with_sch(); };
                if self.timeline_config.ast { timeline.with_ast(); };
                if self.timeline_config.potions { timeline.with_potions(); };
                Arc::new(timeline) as _
            }
        };
        let solver: Arc<dyn Solver + Send+Sync> = match self.solver_type {
            SolverType::Split => Arc::new(
                SplitSolver::new(self.ui_link.clone(), evaluator)
                    .with_items(self.items.clone())
                    .with_config(self.split_config.clone())
            ) as _,
            SolverType::Rolling => Arc::new(
                RollingSolver::new(self.ui_link.clone(), evaluator)
                    .with_items(self.items.clone())
                    .with_config(self.rolling_config.clone())
            ) as _,
        };

        self.solver = solver;
        self.gearsets.lock().unwrap().sort_by(|a, b| {
            self.solver.dps(b).partial_cmp(&self.solver.dps(a)).unwrap()
        });
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Configuration, "Configuration");
            ui.selectable_value(&mut self.tab, Tab::Comparator, "Comparator");
        });
    }

    fn solver_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::Grid::new("config_grid").striped(true).show(ui, |ui| {
                ui.label("Solvers");
                self.config_changed |= self.split_config_ui(ui).changed();
                self.config_changed |= self.rolling_config_ui(ui).changed();
                ui.end_row();

                ui.label("Evaluators");
                self.config_changed |= self.infinite_dummy_ui(ui).changed();
                self.config_changed |= self.timeline_ui(ui).changed();
                ui.end_row();

            });

            ui.separator();

            if ui.button("Rebuild solver").clicked() {
                if self.config_changed {
                    self.rebuild_solver();
                }
            }

            if ui.button("Run solver").clicked() {
                if self.config_changed {
                    self.rebuild_solver();
                }
                std::thread::spawn({
                    let solver = self.solver.clone();
                    let ui_link = self.ui_link.clone();
                    move || {
                        ui_link.new_gearsets(solver.solve().unwrap()).unwrap();
                        ui_link.message("Finished!").unwrap();
                    }
                });
            }
            if self.config_changed {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Solver configuration changed");
            }
        });
    }

    fn comparator_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(index) = self.selected_gearset_a {
            if let Some(gearset) = self.gearsets.lock().unwrap().get(index) {
                ui.push_id("gearset_a", |ui| {
                    gearset.table_ui(ui);
                });
            }
        }
        ui.separator();
        if let Some(index) = self.selected_gearset_b {
            if let Some(gearset) = self.gearsets.lock().unwrap().get(index) {
                ui.push_id("gearset_b", |ui| {
                    gearset.table_ui(ui);
                });
            }
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(&self.ui_link.status_text.lock().unwrap().clone());
            ui.separator();
            ui.label(format!("{} items processed", self.ui_link.count.load(Ordering::Relaxed)));
        });
    }

    fn gearset_selector(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Size, TableBuilder};

        let text_size_body = egui::TextStyle::Button.resolve(ui.style()).size;
        let text_size_header = egui::TextStyle::Heading.resolve(ui.style()).size;

        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::exact(text_size_body))
            .column(Size::exact(text_size_body))
            .column(Size::initial(90.0).at_least(90.0));

        table.header(text_size_header, |mut header| {
            header.col(|ui| {
                ui.heading("A");
            });
            header.col(|ui| {
                ui.heading("B");
            });
            header.col(|ui| {
                ui.heading("DPS");
            });
        })
        .body(|mut body| {
            for (index, gearset) in self.gearsets.lock().unwrap().iter().enumerate() {
                body.row(text_size_body, |mut row| {
                    row.col(|ui| {
                        ui.radio_value(&mut self.selected_gearset_a, Some(index), "");
                    });
                    row.col(|ui| {
                        ui.radio_value(&mut self.selected_gearset_b, Some(index), "");
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", self.solver.dps(gearset)));
                    });
                });
            }
        });
    }
}

pub trait MatUi {
    fn row_ui(&self, row: &mut egui_extras::TableRow);
}

// Goes for MatIX too because under the alias they are the same type
impl MatUi for crate::data::MatX {
    fn row_ui(&self, row: &mut egui_extras::TableRow) {
        use std::num::NonZeroU32;
        use crate::data::MeldType;

        row.col(|_ui| {
        });
        row.col(|_ui| {
        });
        row.col(|_ui| {
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self[MeldType::DirectHit as usize]) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self[MeldType::Critical as usize]) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self[MeldType::Determination as usize]) {
                ui.label(val.to_string());
            }
        });
        row.col(|ui| {
            if let Some(val) = NonZeroU32::new(self[MeldType::SpellSpeed as usize]) {
                ui.label(val.to_string());
            }
        });
        row.col(|_ui| {
        });
    }
}

impl crate::data::Item {
    pub fn row_ui(&self, row: &mut egui_extras::TableRow) {
        row.col(|ui| {
            ui.label(&self.name);
        });
        self.stats.row_ui(row);
    }
}

impl Ui {
    fn split_config_ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            [
                ui.selectable_value(&mut self.solver_type, SolverType::Split, "Split"),
                ui.add(egui::Slider::new(&mut self.split_config.k_stage_1, 1..=1000).text("K stage 1")),
                ui.add(egui::Slider::new(&mut self.split_config.k_stage_2, 1..=1000).text("K stage 2")),
            ].into_iter().reduce(egui::Response::bitor).unwrap()
        }).inner
    }
}

impl Ui {
    fn rolling_config_ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            [
                ui.selectable_value(&mut self.solver_type, SolverType::Rolling, "Rolling"),
                ui.add(egui::Slider::new(&mut self.rolling_config.rolling_k, 1..=100000).text("Rolling K")),
            ].into_iter().reduce(egui::Response::bitor).unwrap()
        }).inner
    }
}

impl Ui {
    fn infinite_dummy_ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {[
            ui.selectable_value(
                &mut self.evaluator_type,
                EvaluatorType::InfiniteDummy,
                "Infinite Dummy"
            ),
        ].into_iter().reduce(egui::Response::bitor).unwrap()}).inner
    }
}

impl Ui {
    fn timeline_ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {[
            ui.selectable_value(
                &mut self.evaluator_type,
                EvaluatorType::Timeline,
                "Timeline"
            ),
            ui.add(egui::Slider::new(&mut self.timeline_config.mind_bonus, 0.0..=0.05)
                .step_by(0.01)
                .text("Mind bonus")
            ),
            ui.checkbox(&mut self.timeline_config.brd, "Bard"),
            ui.checkbox(&mut self.timeline_config.dnc, "Dancer"),
            ui.checkbox(&mut self.timeline_config.smn, "Summoner"),
            ui.checkbox(&mut self.timeline_config.rdm, "Red Mage"),
            ui.checkbox(&mut self.timeline_config.mnk, "Monk"),
            ui.checkbox(&mut self.timeline_config.drg, "Dragoon"),
            ui.checkbox(&mut self.timeline_config.rpr, "Reaper"),
            ui.checkbox(&mut self.timeline_config.nin, "Ninja"),
            ui.checkbox(&mut self.timeline_config.sch, "Scholar"),
            ui.checkbox(&mut self.timeline_config.ast, "Astrologian"),
            ui.checkbox(&mut self.timeline_config.potions, "Use potions"),
            ui.separator(),
            ui.add(egui::Slider::new(&mut self.timeline_config.kill_time, 0.0..=1200.0)
                .text("Kill time")
            ),
        ].into_iter().reduce(egui::Response::bitor).unwrap()}).inner
    }
}
