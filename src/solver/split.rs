use crate::ui::UiLink;
use itertools::Itertools;
use std::sync::Arc;

use crate::data::*;
use crate::solver::{Evaluator, EvaluatorWrapper, Solver, SAGE_BASE};

#[derive(Clone)]
pub struct SplitConfig {
    pub k_stage_1: usize,
    pub k_stage_2: usize,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            k_stage_1: 10,
            k_stage_2: 10,
        }
    }
}

pub struct SplitSolver {
    items: Vec<Item>,
    ui_link: UiLink,
    evaluator: Arc<dyn Evaluator + Send+Sync>,
    config: SplitConfig,
}

impl SplitSolver {
    pub fn new(ui_link: UiLink, evaluator: Arc<dyn Evaluator + Send+Sync>) -> Self {
        Self {
            items: Vec::default(),
            ui_link,
            evaluator,
            config: SplitConfig::default(),
        }
    }

    pub fn with_items(self, items: Vec<Item>) -> Self {
        Self {
            items,
            ..self
        }
    }

    pub fn with_config(self, config: SplitConfig) -> Self {
        Self {
            config,
            ..self
        }
    }
}

impl Solver for SplitSolver {
    fn solve(&self) -> eyre::Result<Vec<Gearset>> {
        self.ui_link.set_count(0)?;
        self.ui_link.message("Loading items...")?;
        let items = self.items.clone();
        let (arme, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Weapon);
        let (tête, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Head);
        let (torse, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Body);
        let (mains, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Hands);
        let (jambes, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Legs);
        let (pieds, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Feet);
        let (oreille, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Earrings);
        let (collier, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Necklace);
        let (bracelet, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Bracelet);
        let (bagues, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::LeftRing);
        let (nourriture, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Food);

        if !items.is_empty() {
            tracing::error!("Not all items were partitioned: {:?}", items);
            self.ui_link.message("ERROR: Not all items were partitioned")?;
            return Err(eyre::eyre!("Not all items were partitioned"));
        }

        let bagues: Vec<_> = bagues.into_iter()
            .combinations(2)
            .collect();

        let product = vec![
            arme.into_iter(),
            tête.into_iter(),
            torse.into_iter(),
            mains.into_iter(),
            jambes.into_iter(),
            pieds.into_iter(),
            oreille.into_iter(),
            collier.into_iter(),
            bracelet.into_iter(),
        ].into_iter()
            .multi_cartesian_product()
            .cartesian_product(bagues)
            .map(|(items, rings)| items.into_iter().chain(rings.into_iter()).collect::<Vec<_>>());

        self.ui_link.message("Ranking gear...")?;

        let results = product
            .map(|items| {
                let mut gearset = Gearset::from_items(items);
                gearset.base = SAGE_BASE;
                gearset
            })
            .filter(|gearset| {
                gearset.is_valid()
            })
            .inspect(|_| self.ui_link.increment().unwrap())
            .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
            .map(std::cmp::Reverse)
            .k_smallest(self.config.k_stage_1)
            .map(|rev| rev.0)
            .map(|EvaluatorWrapper { gearset, .. }| gearset);

        self.ui_link.set_count(0)?;
        self.ui_link.message("Ranking food/melds...")?;

        let gearsets: Vec<_> = results.into_iter()
            .flat_map(|gearset| {
                let (possible_melds_x, possible_melds_ix) = gearset.possible_melds();
                let (meld_slots_x, meld_slots_ix) = gearset.meld_slots();
                tracing::debug!("{:?}", gearset.items);
                tracing::debug!("possible: {:?}, {:?}", possible_melds_x, possible_melds_ix);
                tracing::debug!("slots: {:?}, {:?}", meld_slots_x, meld_slots_ix);

                let tentative_meld_x: Vec<_> = possible_melds_x.into_iter()
                    .map(|materia_count| (0..=materia_count))
                    .multi_cartesian_product()
                    .filter(|meld| meld.iter().sum::<u32>() == meld_slots_x)
                    .collect();
                let tentative_meld_ix: Vec<_> = possible_melds_ix.into_iter()
                    .map(|materia_count| (0..=materia_count))
                    .multi_cartesian_product()
                    .filter(|meld| meld.iter().sum::<u32>() == meld_slots_ix)
                    .collect();

                tracing::debug!("possible melds X: {}, IX: {}", tentative_meld_x.len(), tentative_meld_ix.len());

                std::iter::once(gearset).cartesian_product(nourriture.iter()).cartesian_product(tentative_meld_x.into_iter()).cartesian_product(tentative_meld_ix.into_iter())
            })
            .map(|(((gearset, food), meld_x), meld_ix)| (gearset, food, meld_x, meld_ix))
            .map(|(mut gearset, food, meld_x, meld_ix)| {
                gearset.food = food.clone();
                gearset.meld_x = meld_x.try_into().unwrap();
                gearset.meld_ix = meld_ix.try_into().unwrap();
                gearset
            })
            .inspect(|_| self.ui_link.increment().unwrap())
            .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
            .map(std::cmp::Reverse)
            .k_smallest(self.config.k_stage_2)
            .map(|rev| rev.0)
            .map(|EvaluatorWrapper { gearset, .. }| gearset)
            .collect();


        Ok(gearsets)
    }

    fn dps(&self, gearset: &Gearset) -> f64 {
        self.evaluator.dps(gearset)
    }
}
