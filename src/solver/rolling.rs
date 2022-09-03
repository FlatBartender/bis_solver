use std::sync::Arc;
use crate::data::*;
use crate::ui::UiLink;
use crate::solver::{Evaluator, EvaluatorWrapper, Solver, SAGE_BASE};

use itertools::Itertools;

#[derive(Clone)]
pub struct RollingConfig {
    pub rolling_k: usize,
}

impl Default for RollingConfig {
    fn default() -> Self {
        Self {
            rolling_k: 128,
        }
    }
}

pub struct RollingSolver {
    items: Vec<Item>,
    ui_link: UiLink,
    evaluator: Arc<dyn Evaluator + Send+Sync>,
    config: RollingConfig,
}

impl RollingSolver {
    pub fn new(ui_link: UiLink, evaluator: Arc<dyn Evaluator + Send+Sync>) -> Self {
        Self {
            items: Vec::default(),
            ui_link,
            evaluator,
            config: RollingConfig::default(),
        }
    }

    pub fn with_items(self, items: Vec<Item>) -> Self {
        Self {
            items,
            ..self
        }
    }

    pub fn with_config(self, config: RollingConfig) -> Self {
        Self {
            config,
            ..self
        }
    }
}
impl Solver for RollingSolver {
    fn solve(&self) -> eyre::Result<Vec<Gearset>> {
        self.ui_link.set_count(0)?;
        self.ui_link.message("Loading items...")?;
        let items = self.items.clone();
        let (weapon, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Weapon);
        let (head, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Head);
        let (torso, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Body);
        let (hands, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Hands);
        let (legs, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Legs);
        let (feet, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Feet);
        let (ear, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Earrings);
        let (neck, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Necklace);
        let (bracelet, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Bracelet);
        let (rings, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::LeftRing);
        let (food, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.slot == ItemSlot::Food);

        if !items.is_empty() {
            tracing::error!("Not all items were partitioned: {:?}", items);
            self.ui_link.message("ERROR: Not all items were partitioned")?;
            return Err(eyre::eyre!("Not all items were partitioned"));
        }

        let (left_ring, right_ring): (Vec<_>, Vec<_>) = rings.into_iter()
            .tuple_combinations()
            .map(|(mut left, mut right)| {
                left.slot = ItemSlot::LeftRing;
                right.slot = ItemSlot::RightRing;
                (left, right)
            })
            .unzip();

        let items = vec![
            weapon.into_iter(),
            head.into_iter(),
            torso.into_iter(),
            hands.into_iter(),
            legs.into_iter(),
            feet.into_iter(),
            ear.into_iter(),
            neck.into_iter(),
            bracelet.into_iter(),
            left_ring.into_iter(),
            right_ring.into_iter(),
        ];

        self.ui_link.message("Ranking gear...")?;

        let base_gearset = Gearset {
            base: SAGE_BASE,
            ..Default::default()
        };
        let mut gearsets = vec![base_gearset];
        for item_list in items {
            gearsets = item_list
                .cartesian_product(gearsets.into_iter())
                .map(|(item, mut gearset)| {
                    gearset.items[item.slot as usize] = item.clone();
                    gearset
                })
                .filter(|gearset| {
                    gearset.is_valid()
                })
                .inspect(|_| self.ui_link.increment().unwrap())
                .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
                .map(std::cmp::Reverse)
                .k_smallest(self.config.rolling_k)
                .dedup()
                .map(|rev| rev.0)
                .map(|EvaluatorWrapper { gearset, .. }| gearset)
                .collect();
        }

        self.ui_link.set_count(0)?;
        self.ui_link.message("Ranking food/melds...")?;

        let gearsets: Vec<_> = gearsets.into_iter()
            .flat_map(|gearset| {
                let (possible_melds_x, _) = gearset.possible_melds();
                let (meld_slots_x, _) = gearset.meld_slots();
                let tentative_meld_x = possible_melds_x.into_iter()
                    .map(|materia_count| (0..=materia_count))
                    .multi_cartesian_product()
                    .filter(move |meld| meld.iter().sum::<u32>() == meld_slots_x);
                std::iter::once(gearset).cartesian_product(tentative_meld_x)
            })
            .map(|(mut gearset, meld_x)| {
                gearset.meld_x = meld_x.try_into().unwrap();
                gearset
            })
            .inspect(|_| self.ui_link.increment().unwrap())
            .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
            .map(std::cmp::Reverse)
            .k_smallest(self.config.rolling_k)
            .map(|rev| rev.0)
            .map(|EvaluatorWrapper { gearset, .. }| gearset)
            .flat_map(|gearset| {
                let (_, possible_melds_ix) = gearset.possible_melds();
                let (_, meld_slots_ix) = gearset.meld_slots();
                let tentative_meld_ix = possible_melds_ix.into_iter()
                    .map(|materia_count| (0..=materia_count))
                    .multi_cartesian_product()
                    .filter(move |meld| meld.iter().sum::<u32>() == meld_slots_ix);
                std::iter::once(gearset).cartesian_product(tentative_meld_ix)
            })
            .map(|(mut gearset, meld_ix)| {
                gearset.meld_ix = meld_ix.try_into().unwrap();
                gearset
            })
            .inspect(|_| self.ui_link.increment().unwrap())
            .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
            .map(std::cmp::Reverse)
            .k_smallest(self.config.rolling_k)
            .map(|rev| rev.0)
            .map(|EvaluatorWrapper { gearset, .. }| gearset)
            .cartesian_product(food.into_iter())
            .map(|(mut gearset, food)| {
                gearset.food = food;
                gearset
            })
            .inspect(|_| self.ui_link.increment().unwrap())
            .map(|gearset| EvaluatorWrapper { evaluator: self.evaluator.clone(), gearset })
            .map(std::cmp::Reverse)
            .k_smallest(self.config.rolling_k)
            .map(|rev| rev.0)
            .map(|EvaluatorWrapper { gearset, .. }| gearset)
            .collect();

        Ok(gearsets)
    }

    fn dps(&self, gearset: &Gearset) -> f64 {
        self.evaluator.dps(gearset)
    }
}
