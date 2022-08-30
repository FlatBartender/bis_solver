use eframe::egui;

use crate::data::ITEM_SLOTS;
use crate::solver::DPS_FUNCTION;

impl crate::data::Gearset {
    pub fn table_ui(&mut self, ui: &mut egui::Ui) {
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
        row.col(|ui| {
            ui.label(format!("{:.2}", self.dps()));
        });
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
        let widths: Vec<_> = self.widths().iter().cloned().collect();
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
        loop {
            match self.status_recv.try_recv() {
                Ok(message) => {
                    self.handle_message(message);
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                _ => todo!(),
            }
        }

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            self.status_bar(ui);
        });

        egui::SidePanel::left("gearset_selector").show(ctx, |ui| {
            self.gearset_selector(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(index) = self.selected_gearset_a {
                ui.push_id("gearset_a", |ui| {
                    self.gearsets[index].table_ui(ui);
                });
            }
            ui.separator();
            if let Some(index) = self.selected_gearset_b {
                ui.push_id("gearset_b", |ui| {
                    self.gearsets[index].table_ui(ui);
                });
            }
        });
    }
}

#[derive(Clone)]
pub struct UiLink {
    status_send: std::sync::mpsc::Sender<UiMessage>,
}

impl UiLink {
    pub fn new(status_send: std::sync::mpsc::Sender<UiMessage>) -> Self {
        Self {
            status_send,
        }
    }

    pub fn message(&self, message: impl ToString) -> eyre::Result<()> {
        self.status_send.send(UiMessage::StatusMessage(message.to_string()))?;
        Ok(())
    }

    pub fn gearset(&self, gearset: crate::data::Gearset) -> eyre::Result<()> {
        self.status_send.send(UiMessage::NewGearset(Box::new(gearset)))?;
        Ok(())
    }
}

pub enum UiMessage {
    StatusMessage(String),
    NewGearset(Box<crate::data::Gearset>),
}

pub struct Ui {
    status_recv: std::sync::mpsc::Receiver<UiMessage>,

    status: String,
    gearsets: Vec<crate::data::Gearset>,

    selected_gearset_a: Option<usize>,
    selected_gearset_b: Option<usize>,
}

impl Ui {
    pub fn new(_cc: &eframe::CreationContext<'_>, status_recv: std::sync::mpsc::Receiver<UiMessage>) -> Self {
        Self {
            status_recv,

            status: "Startup".to_string(),
            gearsets: Vec::new(),
            selected_gearset_a: None,
            selected_gearset_b: None,
        }
    }

    pub fn handle_message(&mut self, message: UiMessage) {
        use UiMessage::*;

        match message {
            StatusMessage(message) => self.status = message,
            NewGearset(gearset) => self.gearsets.push(*gearset),
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.status);
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
            for (index, gearset) in self.gearsets.iter().enumerate() {
                body.row(text_size_body, |mut row| {
                    row.col(|ui| {
                        ui.radio_value(&mut self.selected_gearset_a, Some(index), "");
                    });
                    row.col(|ui| {
                        ui.radio_value(&mut self.selected_gearset_b, Some(index), "");
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2}", DPS_FUNCTION(&gearset.stats())));
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

