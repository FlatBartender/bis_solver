use std::sync::Arc;

use crate::data::*;

pub mod infinite_dummy;
pub mod timeline;
pub mod split;
pub mod rolling;

pub use infinite_dummy::*;
pub use timeline::*;
pub use split::*;
pub use rolling::*;

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

pub trait Evaluator {
    fn dps(&self, gearset: &Gearset) -> f64;
}

pub struct EvaluatorWrapper {
    evaluator: Arc<dyn Evaluator>,
    gearset: Gearset,
}

impl PartialEq for EvaluatorWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.gearset == other.gearset
    }
}

impl Eq for EvaluatorWrapper {}

impl PartialOrd for EvaluatorWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluator.dps(&self.gearset).partial_cmp(&other.evaluator.dps(&other.gearset))
    }
}

impl Ord for EvaluatorWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(PartialEq, Eq)]
pub enum SolverType {
    Split,
    Rolling,
}

#[derive(PartialEq, Eq)]
pub enum EvaluatorType {
    InfiniteDummy,
    Timeline,
}
