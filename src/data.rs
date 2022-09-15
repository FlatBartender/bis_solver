use crate::utils::Unit;

pub type MatX = [u32; MeldType::Number as usize];
pub type MatIX = [u32; MeldType::Number as usize];

pub trait StatRepo {
    fn weapon_damage(&self) -> u32;
    fn mind(&self) -> u32;
    fn vitality(&self) -> u32;
    fn piety(&self) -> u32;
    fn direct_hit(&self) -> u32;
    fn critical(&self) -> u32;
    fn determination(&self) -> u32;
    fn spell_speed(&self) -> u32;
    fn gcd_uptime(&self) -> f64;

    fn stat_max(&self) -> u32 {
        vec![self.piety(), self.direct_hit(), self.critical(), self.determination(), self.spell_speed()].into_iter().max().unwrap()
    }

    fn weapon_delay(&self) -> Unit<1, 100> {
        Unit(280)
    }

    fn gcd(&self) -> Unit<1, 100> {
        Unit(2500 * (1000 - 130 * (self.spell_speed() - 400) / 1900) / 10000)
    }

    fn gcd15(&self) -> Unit<1, 100> {
        Unit(1500 * (1000 - 130 * (self.spell_speed() - 400) / 1900) / 10000)
    }

    fn adjusted_gcd(&self) -> f64 {
        self.gcd().scalar() / self.gcd_uptime()
    }

    fn crit_multiplier(&self) -> Unit<1, 1000> {
        Unit(1400 + 200 * (self.critical() - 400) / 1900)
    }

    fn crit_rate(&self) -> Unit<1, 1000> {
        Unit(50 + 200 * (self.critical() - 400) / 1900 )
    }

    fn det_multiplier(&self) -> Unit<1, 1000> {
        Unit(1000 + 140 * (self.determination() - 390) / 1900)
    }

    fn dh_rate(&self) -> Unit<1, 1000> {
        Unit(550 * (self.direct_hit() - 400) / 1900)
    }

    fn sps_multiplier(&self) -> Unit<1, 1000> {
        Unit(1000 + 130 * (self.spell_speed() - 400) / 1900)
    }

    fn adjusted_weapon_damage(&self) -> Unit<1, 100> {
        Unit(390 * 115 / 1000 + self.weapon_damage())
    }

    fn physic_attack_power(&self) -> Unit<1, 100> {
        Unit(233)
    }

    fn magic_attack_power(&self) -> Unit<1, 100> {
        Unit(195 * (self.mind() - 390) / 390 + 100)
    }

    fn trait_bonus(&self) -> Unit<1, 100> {
        Unit(130)
    }

    fn crit_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.crit_rate().0 + self.crit_rate().0 * self.crit_multiplier().0 / 1000)
    }

    fn dh_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.dh_rate().0 + self.dh_rate().0 * 125 / 100)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MeldType {
    Critical = 0,
    Determination,
    DirectHit,
    SpellSpeed,

    Number,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct Stats {
    pub weapon_damage: u32,
    pub mind: u32,
    pub vitality: u32,
    pub piety: u32,
    pub direct_hit: u32,
    pub critical: u32,
    pub determination: u32,
    pub spell_speed: u32,
}

impl Stats {
    pub fn add(&mut self, other: &Self) {
        self.weapon_damage += other.weapon_damage;
        self.mind += other.mind;
        self.vitality += other.vitality;
        self.piety += other.piety;
        self.direct_hit += other.direct_hit;
        self.critical += other.critical;
        self.determination += other.determination;
        self.spell_speed += other.spell_speed;
    }

    fn apply_food(&mut self, food: &Item) {
        self.critical += food.stats.critical.min(self.critical / 10);
        self.direct_hit += food.stats.direct_hit.min(self.direct_hit / 10);
        self.determination += food.stats.determination.min(self.determination / 10);
        self.spell_speed += food.stats.spell_speed.min(self.spell_speed / 10);
        self.vitality += food.stats.vitality.min(self.vitality / 10);
        self.piety += food.stats.piety.min(self.piety / 10);
    }

    fn apply_materias(&mut self, materias_x: &MatX, materias_ix: &MatIX) {
        self.critical += materias_x[MeldType::Critical as usize] * 36 + materias_ix[MeldType::Critical as usize] * 12;
        self.determination += materias_x[MeldType::Determination as usize] * 36 + materias_ix[MeldType::Determination as usize] * 12;
        self.direct_hit += materias_x[MeldType::DirectHit as usize] * 36 + materias_ix[MeldType::DirectHit as usize] * 12;
        self.spell_speed += materias_x[MeldType::SpellSpeed as usize] * 36 + materias_ix[MeldType::SpellSpeed as usize] * 12;
    }

}

impl StatRepo for Stats {
    fn weapon_damage(&self) -> u32 {
        self.weapon_damage
    }
    fn mind(&self) -> u32 {
        self.mind
    }
    fn vitality(&self) -> u32 {
        self.vitality
    }
    fn piety(&self) -> u32 {
        self.piety
    }
    fn direct_hit(&self) -> u32 {
        self.direct_hit
    }
    fn critical(&self) -> u32 {
        self.critical
    }
    fn determination(&self) -> u32 {
        self.determination
    }
    fn spell_speed(&self) -> u32 {
        self.spell_speed
    }
    fn gcd_uptime(&self) -> f64 {
        1.0
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Default)]
pub enum ItemSlot {
    #[default]
    Weapon = 0,
    Head,
    Body,
    Hands,
    Legs,
    Feet,
    Earrings,
    Necklace,
    Bracelet,
    LeftRing,
    RightRing,
    Food,
}

pub const ITEM_SLOTS: [ItemSlot; 11] = [
    ItemSlot::Weapon,
    ItemSlot::Head,
    ItemSlot::Body,
    ItemSlot::Hands,
    ItemSlot::Legs,
    ItemSlot::Feet,
    ItemSlot::Earrings,
    ItemSlot::Necklace,
    ItemSlot::Bracelet,
    ItemSlot::LeftRing,
    ItemSlot::RightRing,
];


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Default)]
pub struct Item {
    pub slot: ItemSlot,
    pub name: String,
    pub stats: Stats,
    pub meld_slots: u32,
    pub overmeldable: u32,
}

impl Item {
    pub fn stat_max(&self) -> u32 {
        self.stats.stat_max()
    }
}

#[derive(Debug, Clone, Eq, Default)]
pub struct Gearset {
    pub base: Stats,
    pub items: [Item; 11],
    pub food: Item,
    pub meld_x: MatX,
    pub meld_ix: MatIX,
}

impl Gearset {
    pub fn from_items(items: Vec<crate::data::Item>) -> Self {
        Self {
            items: items.try_into().unwrap(),
            ..Self::default()
        }
    }

    pub fn stats(&self) -> Stats {
        let mut stats = self.items.iter().fold(Stats::default(), |mut acc, item| {
            acc.add(&item.stats);
            acc
        });
        stats.add(&self.base);
        stats.apply_food(&self.food);
        stats.apply_materias(&self.meld_x, &self.meld_ix);
        stats
    }

    pub fn meld_slots(&self) -> (u32, u32) {
        self.items.iter()
            .map(|item| {
                if item.overmeldable == 0 {
                    (item.meld_slots, 0)
                } else {
                    (item.meld_slots + 1, 5 - item.meld_slots - 1)
                }
            })
            .fold((0, 0), |(slots_x, slots_ix), (item_slots_x, item_slots_ix)| (slots_x + item_slots_x, slots_ix + item_slots_ix))
    }

    pub fn possible_melds(&self) -> (MatX, MatIX) {
        use MeldType::*;

        let mut slots_x = MatX::default();
        let mut slots_ix = MatIX::default();

        for item in self.items.iter() {
            if item.overmeldable == 0 {
                slots_x[Critical as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).round() as u32);
                slots_x[Determination as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).round() as u32);
                slots_x[DirectHit as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).round() as u32);
                slots_x[SpellSpeed as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).round() as u32);
            } else {
                slots_x[Critical as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).round() as u32);
                slots_x[Determination as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).round() as u32);
                slots_x[DirectHit as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).round() as u32);
                slots_x[SpellSpeed as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).round() as u32);

                slots_ix[Critical as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 12.0).round() as u32);
                slots_ix[Determination as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 12.0).round() as u32);
                slots_ix[DirectHit as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 12.0).round() as u32);
                slots_ix[SpellSpeed as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 12.0).round() as u32);
            }
        }

        (slots_x, slots_ix)
    }

    pub fn is_valid(&self) -> bool {
        use ItemSlot::*;
        !(self.items[LeftRing as usize].name == self.items[RightRing as usize].name && self.items[LeftRing as usize].overmeldable == 0) || self.items[LeftRing as usize].name.is_empty()
    }
}

impl std::cmp::PartialEq for Gearset {
    fn eq(&self, other: &Self) -> bool {
        self.items[0..9] == other.items[0..9]
            && self.items[9..11].contains(&other.items[9])
            && self.items[9..11].contains(&other.items[10])
            && self.food == other.food
            && self.meld_x == other.meld_x
            && self.meld_ix == other.meld_ix
    }
}

