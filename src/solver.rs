use itertools::Itertools;

use crate::utils::Scalable;
use crate::ui::UiLink;
use crate::data::*;

// SGE base but viera/veena
const SAGE_BASE: Stats = Stats {
    weapon_damage: 0,
    mind: 450,
    vitality: 390,
    piety: 390,
    direct_hit: 400,
    critical: 400,
    determination: 390,
    spell_speed: 400,
};

#[derive(Debug)]
pub enum ItemSlotConversionError {
    Invalid(String)
}

impl std::fmt::Display for ItemSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemSlot::Weapon => write!(f, "Weapon"),
            ItemSlot::Head => write!(f, "Head"),
            ItemSlot::Body => write!(f, "Body"),
            ItemSlot::Hands => write!(f, "Hands"),
            ItemSlot::Legs => write!(f, "Legs"),
            ItemSlot::Feet => write!(f, "Feet"),
            ItemSlot::Earrings => write!(f, "Earrings"),
            ItemSlot::Necklace => write!(f, "Necklace"),
            ItemSlot::Bracelet => write!(f, "Bracelet"),
            ItemSlot::LeftRing => write!(f, "Left ring"),
            ItemSlot::RightRing => write!(f, "Right ring"),
            ItemSlot::Food => write!(f, "Food"),
        }
    }
}

impl std::str::FromStr for ItemSlot {
    type Err = ItemSlotConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string().to_lowercase();    
        match s.as_str() {
            "arme" | "weapon" => Ok(Self::Weapon),
            "tête" | "head" => Ok(Self::Head),
            "torse" | "body" => Ok(Self::Body),
            "mains" | "hands" => Ok(Self::Hands),
            "jambes" | "legs" => Ok(Self::Legs),
            "pieds" | "feet" => Ok(Self::Feet),
            "oreille" | "earrings" => Ok(Self::Earrings),
            "collier" | "necklace" => Ok(Self::Necklace),
            "bracelet" => Ok(Self::Bracelet),
            "bague gauche" | "left ring" => Ok(Self::LeftRing),
            "bague droite" | "right ring" => Ok(Self::RightRing),
            "nourriture" | "food" => Ok(Self::Food),
            "anneau" | "ring" => Ok(Self::LeftRing),
            _ => Err(ItemSlotConversionError::Invalid(format!("Invalid value: {}, expected an equip slot", s)))
        }
    }
}

pub trait Solver {
    fn k_best_sets(&self, k: usize) -> eyre::Result<Vec<Gearset>>;
    fn dps(&self, gearset: &Gearset) -> f64;
}

pub trait EvaluatorFactory {
    type Wrapper: Ord;
    fn wrap(&self, gearset: Gearset) -> Self::Wrapper;
    fn dps(&self, gearset: &Gearset) -> f64;
    fn unwrap(&self, o: Self::Wrapper) -> Gearset;
}

pub struct SplitSolver<T> {
    items: Vec<Item>,
    ui_link: UiLink,
    evaluator_factory: Box<dyn EvaluatorFactory<Wrapper = T> + Send+Sync>,
}

impl<T> SplitSolver<T> {
    pub fn new(items: Vec<Item>, ui_link: UiLink, evaluator_factory: Box<dyn EvaluatorFactory<Wrapper = T> + Send+Sync>) -> Self {
        Self {
            items,
            ui_link,
            evaluator_factory,
        }
    }
}

impl<T: Ord> Solver for SplitSolver<T> {
    fn k_best_sets(&self, k: usize) -> eyre::Result<Vec<Gearset>> {
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
            .map(|gearset| self.evaluator_factory.wrap(gearset))
            .map(std::cmp::Reverse)
            .k_smallest(k)
            .map(|rev| rev.0)
            .map(|eval| self.evaluator_factory.unwrap(eval));

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
            .map(|gearset| self.evaluator_factory.wrap(gearset))
            .map(std::cmp::Reverse)
            .k_smallest(k)
            .map(|rev| rev.0)
            .map(|eval| self.evaluator_factory.unwrap(eval))
            .collect();


        Ok(gearsets)
    }

    fn dps(&self, gearset: &Gearset) -> f64 {
        self.evaluator_factory.dps(gearset)
    }
}

pub trait InfiniteDummyStat: crate::data::StatRepo {
    fn cycle_length(&self) -> f64 {
        self.cycle_normal_gcd() as f64 * self.adjusted_gcd() + 2.5
    }

    fn cycle_normal_gcd(&self) -> f64 {
        ((30.0 - 2.5) / self.adjusted_gcd()).round()
    }

    fn cycle_phlegma(&self) -> f64 {
        self.cycle_length() * 2.0 / 3.0 / 30.0
    }

    fn cycle_dosis(&self) -> f64 {
        self.cycle_normal_gcd() - self.cycle_phlegma()
    }

    fn cycle_eukr_dosis(&self) -> f64 {
        1.0
    }

    fn dosis_aps(&self) -> f64 {
        self.cycle_dosis() / self.cycle_length()
    }

    fn phlegma_aps(&self) -> f64 {
        self.cycle_phlegma() / self.cycle_length()
    }

    fn eukr_dosis_aps(&self) -> f64 {
        self.cycle_eukr_dosis() / self.cycle_length()
    }

    fn dosis_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 330.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar()
    }

    fn phlegma_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 510.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar()
    }

    fn eukr_dosis_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let sps = self.sps_multiplier();
        let ticks_lost_per_cast = (30.0 - self.cycle_length()).abs() / 3.0;
        let expected_tick_number = 10.0 * (1.0 - ticks_lost_per_cast) + 9.0 * ticks_lost_per_cast;
        let damage = 70.scale(adj_wd).scale(map).scale(det).scale(sps) * 130 / 100 + 1;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar() * expected_tick_number
    }

    fn pps(&self) -> f64 {
        self.dosis_aps() * 330.0
            + self.phlegma_aps() * 510.0
            + self.eukr_dosis_aps() * 700.0
    }

    fn dps(&self) -> f64 {
        self.dosis_aps() * self.dosis_score()
        + self.phlegma_aps() * self.phlegma_score()
        + self.eukr_dosis_aps() * self.eukr_dosis_score()
    }
}

impl<T: crate::data::StatRepo> InfiniteDummyStat for T {}

#[derive(PartialEq, Eq)]
pub struct InfiniteDummyGearset(Gearset);

#[derive(Default)]
pub struct InfiniteDummyEvaluatorFactory {}

impl EvaluatorFactory for InfiniteDummyEvaluatorFactory {
    type Wrapper = InfiniteDummyGearset;

    fn wrap(&self, gearset: Gearset) -> Self::Wrapper {
        InfiniteDummyGearset(gearset)
    }

    fn dps(&self, gearset: &Gearset) -> f64 {
        InfiniteDummyStat::dps(&gearset.stats())
    }

    fn unwrap(&self, o: Self::Wrapper) -> Gearset {
        o.0
    }
}

impl std::cmp::PartialOrd for InfiniteDummyGearset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = InfiniteDummyStat::dps(&self.0.stats());
        let b = InfiniteDummyStat::dps(&other.0.stats());

        a.partial_cmp(&b)
    }
}

impl std::cmp::Ord for InfiniteDummyGearset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(PartialEq, Eq)]
pub enum SolverType {
    Split,
}

#[derive(PartialEq, Eq)]
pub enum EvaluatorType {
    InfiniteDummy,
}
