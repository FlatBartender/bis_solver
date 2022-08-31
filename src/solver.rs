use crate::data::*;

pub mod infinite_dummy;
pub mod split;

pub use infinite_dummy::*;
pub use split::*;

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

#[derive(PartialEq, Eq)]
pub enum SolverType {
    Split,
}

#[derive(PartialEq, Eq)]
pub enum EvaluatorType {
    InfiniteDummy,
}
