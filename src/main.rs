use itertools::Itertools;
use rayon::prelude::*;

struct Unit<const NUMERATOR: u32, const DENOMINATOR: u32>(u32);

impl<const NUMERATOR: u32, const DENOMINATOR: u32> Unit<NUMERATOR, DENOMINATOR> {
    fn scalar(&self) -> f64 {
        self.0 as f64 * NUMERATOR as f64 / DENOMINATOR as f64 
    }
}

trait Scalable {
    fn scale<const NUMERATOR: u32, const DENOMINATOR: u32>(self, unit: Unit<NUMERATOR, DENOMINATOR>) -> Self;
}

impl Scalable for u32 {
    fn scale<const NUMERATOR: u32, const DENOMINATOR: u32>(self, unit: Unit<NUMERATOR, DENOMINATOR>) -> Self {
        self * unit.0 * NUMERATOR / DENOMINATOR
    }
}

impl Scalable for f64 {
    fn scale<const NUMERATOR: u32, const DENOMINATOR: u32>(self, unit: Unit<NUMERATOR, DENOMINATOR>) -> Self {
        self * (unit.0 * NUMERATOR / DENOMINATOR) as f64
    }
}

const DOSIS_POTENCY: u32 = 330;
const PHLEGMA_POTENCY: u32 = 510;
const EUKRASIAN_DOSIS_POTENCY: u32 = 70;

trait StatRepo {
    fn weapon_damage(&self) -> u32;
    fn mind(&self) -> u32;
    fn vitality(&self) -> u32;
    fn piety(&self) -> u32;
    fn direct_hit(&self) -> u32;
    fn critical(&self) -> u32;
    fn determination(&self) -> u32;
    fn spell_speed(&self) -> u32;

    fn stat_max(&self) -> u32 {
        vec![self.piety(), self.direct_hit(), self.critical(), self.determination(), self.spell_speed()].into_iter().max().unwrap()
    }

    fn weapon_delay(&self) -> Unit<1, 100> {
        Unit(280)
    }

    fn gcd(&self) -> Unit<1, 100> {
        Unit(2500 * (1000 - 130 * (self.spell_speed() - 400) / 1900) / 10000)
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

    fn crit_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.crit_rate().0 + self.crit_rate().0 * self.crit_multiplier().0 / 1000)
    }

    fn dh_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.dh_rate().0 + self.dh_rate().0 * 125 / 100)
    }

    fn cycle_length(&self) -> f64 {
        self.cycle_normal_gcd() as f64 * self.gcd().scalar() + 2.5
    }

    fn cycle_normal_gcd(&self) -> f64 {
        ((30.0 - 2.5) / self.gcd().scalar()).round()
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

trait StatRepoBalance: StatRepo {
    fn PhlegmaB(&self) -> f64 {
        (PHLEGMA_POTENCY - DOSIS_POTENCY) as f64 * self.cycle() / 45.0
    }

    fn PhlegmaTime(&self) -> f64 {
        self.gcd().scalar() * (self.cycle() / 45.0 - 4.0)
    }

    fn getP(&self) -> f64 {
        let cycle = self.cycle() + self.PhlegmaTime();

        let mut result = self.PhlegmaB() / self.cycle() * cycle;

        if (2.5 * DOSIS_POTENCY as f64) > (EUKRASIAN_DOSIS_POTENCY as f64 / 3.0 * self.sps_multiplier().scalar() * (2.5 + (27.5/self.gcd().scalar()).floor() * self.gcd().scalar()) * (self.gcd().scalar() - 27.5 % self.gcd().scalar())) {
            result += 6.0*(27.5/self.gcd().scalar()).ceil() * DOSIS_POTENCY as f64;
            result += 6.0*10.0*self.sps_multiplier().scalar() * EUKRASIAN_DOSIS_POTENCY as f64;
        } else {
            result += 6.0*(27.5/self.gcd().scalar()).floor() * DOSIS_POTENCY as f64;
            result += 6.0*9.0*self.sps_multiplier().scalar() * EUKRASIAN_DOSIS_POTENCY as f64;
            result += 6.0*((3.0-(30.0 % self.gcd().scalar()))/3.0)*self.sps_multiplier().scalar()*EUKRASIAN_DOSIS_POTENCY as f64;
        }

        result / cycle
    }

    fn cycle(&self) -> f64 {
        let mut result = 0.0;
        if (2.5 * DOSIS_POTENCY as f64) > (EUKRASIAN_DOSIS_POTENCY as f64 / 3.0 * self.sps_multiplier().scalar() * (2.5 + (27.5/self.gcd().scalar()).floor() * self.gcd().scalar()) * (self.gcd().scalar() - 27.5 % self.gcd().scalar())) {
            result += 6.0*((27.5/self.gcd().scalar()).ceil() * self.gcd().scalar() + 2.5);
        } else {
            result += 6.0*((27.5/self.gcd().scalar()).floor() * self.gcd().scalar() + 2.5);
        }

        result
    }

    fn CritRate(&self) -> f64 {
        (200.0 * (self.critical() as f64 - 400.0) / 1900.0 + 50.0).floor() / 1000.0
    }

    fn CritDamage(&self) -> f64 {
        (200.0 * (self.critical() as f64 - 400.0) / 1900.0 + 400.0).floor() / 1000.0
    }

    fn balance_dps(&self) -> f64 {
        let damage = (((self.getP() * self.adjusted_weapon_damage().0 as f64 * self.magic_attack_power().scalar()).floor() * self.det_multiplier().scalar()).floor() * 130.0 / 10000.0).floor() ;
        damage * (1.0 + self.dh_rate().scalar() / 4.0) * (1.0 + self.CritRate() * self.CritDamage())
    }
}

#[derive(Debug, Clone, Default)]
struct Stats {
    weapon_damage: u32,
    mind: u32,
    vitality: u32,
    piety: u32,
    direct_hit: u32,
    critical: u32,
    determination: u32,
    spell_speed: u32,
}

impl Stats {
    fn add(&mut self, other: &Self) {
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
        self.critical += materias_x[MeldType::CRITICAL as usize] * 36 + materias_ix[MeldType::CRITICAL as usize] * 12;
        self.determination += materias_x[MeldType::DETERMINATION as usize] * 36 + materias_ix[MeldType::DETERMINATION as usize] * 12;
        self.direct_hit += materias_x[MeldType::DIRECTHIT as usize] * 36 + materias_ix[MeldType::DIRECTHIT as usize] * 12;
        self.spell_speed += materias_x[MeldType::SPELLSPEED as usize] * 36 + materias_ix[MeldType::SPELLSPEED as usize] * 12;
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
}

impl StatRepoBalance for Stats {

}


#[derive(Debug, Clone)]
struct Item {
    slot: ItemSlot,
    name: String,
    stats: Stats,
    meld_slots: u32,
    overmeldable: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ItemSlot {
    WEAPON = 0,
    HEAD,
    BODY,
    HANDS,
    LEGS,
    FEET,
    EARRINGS,
    NECKLACE,
    BRACELET,
    LEFTRING,
    RIGHTRING,
    FOOD,
}

#[derive(Debug)]
enum ItemSlotConversionError {
    Invalid(String)
}

impl ItemSlot {
    fn to_string(&self) -> String {
        match self {
            ItemSlot::WEAPON => "Weapon",
            ItemSlot::HEAD => "Head",
            ItemSlot::BODY => "Body",
            ItemSlot::HANDS => "Hands",
            ItemSlot::LEGS => "Legs",
            ItemSlot::FEET => "Feet",
            ItemSlot::EARRINGS => "Earrings",
            ItemSlot::NECKLACE => "Necklace",
            ItemSlot::BRACELET => "Bracelet",
            ItemSlot::LEFTRING => "Left ring",
            ItemSlot::RIGHTRING => "Right ring",
            ItemSlot::FOOD => "Food"
        }.to_string()
    }
}

use std::str::FromStr;
impl FromStr for ItemSlot {
    type Err = ItemSlotConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string().to_lowercase();    
        match s.as_str() {
            "arme" | "weapon" => Ok(Self::WEAPON),
            "tête" | "head" => Ok(Self::HEAD),
            "torse" | "body" => Ok(Self::BODY),
            "mains" | "hands" => Ok(Self::HANDS),
            "jambes" | "legs" => Ok(Self::LEGS),
            "pieds" | "feet" => Ok(Self::FEET),
            "oreille" | "earrings" => Ok(Self::EARRINGS),
            "collier" | "necklace" => Ok(Self::NECKLACE),
            "bracelet" | "bracelet" => Ok(Self::BRACELET),
            "bague gauche" | "left ring" => Ok(Self::LEFTRING),
            "bague droite" | "right ring" => Ok(Self::RIGHTRING),
            "nourriture" | "food" => Ok(Self::FOOD),
            _ => Err(ItemSlotConversionError::Invalid(format!("Invalid value: {}, expected an equip slot", s)))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum MeldType {
    CRITICAL = 0,
    DETERMINATION,
    DIRECTHIT,
    SPELLSPEED,
}

type MatX = [u32; 4];
type MatIX = [u32; 4];

const SAGE_BASE: Stats = Stats {
    weapon_damage: 0,
    mind: 448,
    vitality: 390,
    piety: 390,
    direct_hit: 400,
    critical: 400,
    determination: 390,
    spell_speed: 400,
};

#[derive(Debug, Clone)]
struct Gearset {
    base: Stats,
    items: [Item; 11],
    food: Item,
    meld_x: MatX,
    meld_ix: MatIX,
}

impl Gearset {
    fn from_items(items: Vec<Item>) -> Self {
        let food = Item {
            slot: ItemSlot::FOOD,
            name: format!("NO FOOD"),
            stats: Stats::default(),
            meld_slots: 0,
            overmeldable: 0,
        };

        Self {
            base: SAGE_BASE.clone(),
            items: items.try_into().unwrap(),
            food,
            meld_x: MatX::default(),
            meld_ix: MatIX::default(),
        }
    }

    fn stats(&self) -> Stats {
        let mut stats = self.items.iter().fold(Stats::default(), |mut acc, item| {
            acc.add(&item.stats);
            acc
        });
        stats.add(&self.base);
        stats.apply_food(&self.food);
        stats.apply_materias(&self.meld_x, &self.meld_ix);
        stats
    }

    fn meld_slots(&self) -> (u32, u32) {
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

    fn possible_melds(&self) -> (MatX, MatIX) {
        use MeldType::*;

        let mut slots_x = MatX::default();
        let mut slots_ix = MatIX::default();

        for item in self.items.iter() {
            if item.overmeldable == 0 {
                slots_x[CRITICAL as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).round() as u32);
                slots_x[DETERMINATION as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).round() as u32);
                slots_x[DIRECTHIT as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).round() as u32);
                slots_x[SPELLSPEED as usize] += item.meld_slots.min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).round() as u32);
            } else {
                slots_x[CRITICAL as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).round() as u32);
                slots_x[DETERMINATION as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).round() as u32);
                slots_x[DIRECTHIT as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).round() as u32);
                slots_x[SPELLSPEED as usize] += (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).round() as u32);

                slots_ix[CRITICAL as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 12.0).round() as u32);
                slots_ix[DETERMINATION as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 12.0).round() as u32);
                slots_ix[DIRECTHIT as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 12.0).round() as u32);
                slots_ix[SPELLSPEED as usize] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 12.0).round() as u32);
            }
        }

        (slots_x, slots_ix)
    }

    fn is_valid(&self) -> bool {
        use ItemSlot::*;
        !(self.items[LEFTRING as usize].name == self.items[RIGHTRING as usize].name && self.items[LEFTRING as usize].overmeldable == 0)
    }
}

impl Item {
    fn stat_max(&self) -> u32 {
        self.stats.stat_max()
    }

    fn from_record(record: &csv::StringRecord) -> Result<Item, Box<dyn std::error::Error>> {
        let item = Item {
            slot: record.get(0).unwrap().parse().unwrap(),
            name: record.get(1).unwrap().to_string(),
            stats: Stats {
                weapon_damage: record.get(2).unwrap().parse().unwrap_or_default(),
                mind: record.get(3).unwrap().parse().unwrap_or_default(),
                vitality: record.get(4).unwrap().parse().unwrap_or_default(),
                piety: record.get(5).unwrap().parse().unwrap_or_default(),
                direct_hit: record.get(6).unwrap().parse().unwrap_or_default(),
                critical: record.get(7).unwrap().parse().unwrap_or_default(),
                determination: record.get(8).unwrap().parse().unwrap_or_default(),
                spell_speed: record.get(9).unwrap().parse().unwrap_or_default()
            },
            meld_slots: record.get(10).unwrap().parse().unwrap_or_default(),
            overmeldable: record.get(11).unwrap().parse().unwrap_or_default(),
        };

        Ok(item)
    }
}

const ITEMS: &str = include_str!("items.csv");

fn load_items() -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .quoting(false)
        .from_reader(ITEMS.as_bytes());

    let records: Vec<_> = csv_reader.into_records()
        .collect::<Result<_, _>>()?;
    records.into_iter()
        .map(|record| Item::from_record(&record))
        .collect::<Result<Vec<_>, _>>()
}

fn calc_sets(dps_function: fn (&Stats) -> f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("Ranking gear sets...");

    let items = load_items()?;

    let arme: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::WEAPON).collect();
    let tête: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::HEAD).collect();
    let torse: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::BODY).collect();
    let mains: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::HANDS).collect();
    let jambes: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::LEGS).collect();
    let pieds: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::FEET).collect();
    let oreille: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::EARRINGS).collect();
    let collier: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::NECKLACE).collect();
    let bracelet: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::BRACELET).collect();
    let bague_gauche: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::LEFTRING).collect();
    let bague_droite: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::RIGHTRING).collect();
    let nourriture: Vec<_> = items.clone().into_iter().filter(|item| item.slot == ItemSlot::FOOD).collect();

    let product = vec![
            arme.clone().into_iter(),
            tête.clone().into_iter(),
            torse.clone().into_iter(),
            mains.clone().into_iter(),
            jambes.clone().into_iter(),
            pieds.clone().into_iter(),
            oreille.clone().into_iter(),
            collier.clone().into_iter(),
            bracelet.clone().into_iter(),
            bague_gauche.clone().into_iter(),
            bague_droite.clone().into_iter(),
        ].into_iter()
        .multi_cartesian_product();

    let mut results: Vec<_> = product
        .map(|items| Gearset::from_items(items))
        .filter_map(|gearset| {
            if !gearset.is_valid() { 
                None
            } else {
                Some((dps_function(&gearset.stats()), gearset))
            }
    }).collect();

    results.sort_by(|(a_dps, _), (b_dps, _)| b_dps.partial_cmp(a_dps).unwrap());
    results.dedup_by(|(a_dps, _), (b_dps, _)| a_dps == b_dps);

    let candidates: Vec<_> = results[..10].iter().cloned().collect();
    
    println!("Ranking food/melds...");

    let mut melds: Vec<_> = candidates.into_iter()
        .map(|(_, gearset)| gearset)
        .flat_map(|gearset| {
            let (possible_melds_x, possible_melds_ix) = gearset.possible_melds();
            let (meld_slots_x, meld_slots_ix) = gearset.meld_slots();

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

            std::iter::once(gearset).cartesian_product(nourriture.iter()).cartesian_product(tentative_meld_x.into_iter()).cartesian_product(tentative_meld_ix.into_iter())
        })
        .map(|(((gearset, food), meld_x), meld_ix)| (gearset, food, meld_x, meld_ix))
        .par_bridge()
        .map(|(mut gearset, food, meld_x, meld_ix)| {
            gearset.food = food.clone();
            gearset.meld_x = meld_x.try_into().unwrap();
            gearset.meld_ix = meld_ix.try_into().unwrap();
            gearset
        })
        .map(|gearset| {
            (dps_function(&gearset.stats()), gearset)
        })
        .collect();

    melds.sort_by(|(a_dps, _,), (b_dps, _,)| b_dps.partial_cmp(a_dps).unwrap());

    println!("MELDED SETS");
    for (_, gearset) in &melds[0..10] {
        let stats = gearset.stats();
        println!("    DPS: {}", dps_function(&stats));
        gearset.items.iter()
            .for_each(|item| {
                println!("        Item: {}", item.name);
            });
        println!("        Food: {}", gearset.food.name);
        
        println!("        Melds: {} CRT X, {} DET X, {} DH X, {} SPS X, {} CRT IX, {} DET IX, {} DH IX, {} SPS IX",
            gearset.meld_x[0], gearset.meld_x[1], gearset.meld_x[2], gearset.meld_x[3],
            gearset.meld_ix[0], gearset.meld_ix[1], gearset.meld_ix[2], gearset.meld_ix[3]);
        println!("        GCD: {}, crit rate: {}, crit damage: {}, DH rate: {}",
            stats.gcd().scalar(), stats.crit_rate().scalar(), stats.crit_multiplier().scalar(), stats.dh_rate().scalar());
        println!("{:?}", stats);
    }

    Ok(())
}

fn tui() -> Result<(), Box<dyn std::error::Error>> {
    use cursive::{Cursive, CursiveExt};
    use cursive::view::*;
    use cursive::views::*;
    use cursive::align::*;
    use cursive::theme::*;

    let items = load_items()?;
    let mut siv = Cursive::new();
    cursive::logger::init();

    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('p', Cursive::toggle_debug_console);

    fn update_stats(s: &mut Cursive) {
        let gearset: &mut Gearset = s.user_data().unwrap();
        let stats = gearset.stats();
        s.call_on_name("WD", |view: &mut TextView| view.set_content(format!("{}", stats.weapon_damage)));
        s.call_on_name("MND", |view: &mut TextView| view.set_content(format!("{}", stats.mind)));
        s.call_on_name("DH", |view: &mut TextView| view.set_content(format!("{}", stats.direct_hit)));
        s.call_on_name("Crit", |view: &mut TextView| view.set_content(format!("{}", stats.critical)));
        s.call_on_name("Det", |view: &mut TextView| view.set_content(format!("{}", stats.determination)));
        s.call_on_name("SpS", |view: &mut TextView| view.set_content(format!("{}", stats.spell_speed)));
        s.call_on_name("Pie", |view: &mut TextView| view.set_content(format!("{}", stats.piety)));
        s.call_on_name("GCD", |view: &mut TextView| view.set_content(format!("{:2}", stats.gcd().scalar())));
        s.call_on_name("Crit Rate", |view: &mut TextView| view.set_content(format!("{:.2}", stats.crit_rate().scalar())));
        s.call_on_name("Crit Mult", |view: &mut TextView| view.set_content(format!("{:.3}", stats.crit_multiplier().scalar())));
        s.call_on_name("DH Rate", |view: &mut TextView| view.set_content(format!("{:.3}", stats.dh_rate().scalar())));
        s.call_on_name("PPS", |view: &mut TextView| view.set_content(format!("{:.2}", stats.getP())));
        s.call_on_name("DPS", |view: &mut TextView| view.set_content(format!("{:.2}", stats.balance_dps())));
    }

    fn on_select_item(s: &mut Cursive, item: &Item) {
        let gearset: &mut Gearset = s.user_data().unwrap();
        gearset.items[item.slot.clone() as usize] = item.clone();

        update_stats(s);
    }

    fn on_select_food(s: &mut Cursive, food: &Item) {
        let gearset: &mut Gearset = s.user_data().unwrap();
        gearset.food = food.clone();

        update_stats(s);
    }

    let mut selects: Vec<_> = vec![
        ItemSlot::WEAPON,
        ItemSlot::HEAD,
        ItemSlot::BODY,
        ItemSlot::HANDS,
        ItemSlot::LEGS,
        ItemSlot::FEET,
        ItemSlot::EARRINGS,
        ItemSlot::NECKLACE,
        ItemSlot::BRACELET,
        ItemSlot::LEFTRING,
        ItemSlot::RIGHTRING,
    ].into_iter()
        .map(|slot| {
            let mut select = SelectView::new();
            items.iter().filter(|item| item.slot == slot).for_each(|item| select.add_item(item.name.to_string(), item.clone()));
            select.set_on_select(on_select_item);

            Dialog::around(LinearLayout::horizontal()
                .child(PaddedView::lrtb(0, 2, 0, 0, TextView::new(slot.to_string()).v_align(VAlign::Center)))
                .child(select.with_name("item")))
        }).collect();
    
    let selects_right = selects.split_off(selects.len() / 2 + 1);

    let mut siv = Cursive::new();
    let set: Vec<_> = [ItemSlot::WEAPON, ItemSlot::HEAD, ItemSlot::BODY, ItemSlot::HANDS, ItemSlot::LEGS, ItemSlot::FEET, ItemSlot::EARRINGS, ItemSlot::NECKLACE, ItemSlot::BRACELET, ItemSlot::LEFTRING, ItemSlot::RIGHTRING].into_iter()
        .map(|slot| {
            items.iter().find(|item| item.slot == slot).unwrap().clone()
        })
        .collect();
    let gearset = Gearset::from_items(set);


    let mut left = LinearLayout::vertical();
    selects.into_iter()
        .for_each(|select| left.add_child(select));
    
    let mut meld_menu = LinearLayout::horizontal();
    vec![
        (MeldType::CRITICAL, "CRT X"),
        (MeldType::DETERMINATION, "DET X"), 
        (MeldType::DIRECTHIT, "DH X"),
        (MeldType::SPELLSPEED, "SPS X")
    ].into_iter()
        .for_each(|(meld_type, materia)| {
            meld_menu.add_child(Dialog::around(LinearLayout::vertical()
                .child(TextView::new(materia))
                .child(EditView::new()
                    .on_edit(move |s: &mut Cursive, content: &str, _| {
                        if let Ok(val) = content.parse() {
                            s.call_on_name(materia, |view: &mut EditView| view.set_style(ColorStyle::inherit_parent()));
                            s.with_user_data(|gearset: &mut Gearset| gearset.meld_x[meld_type as usize] = val);
                            update_stats(s);
                        } else {
                            s.call_on_name(materia, |view: &mut EditView| view.set_style(ColorStyle::highlight()));
                        }
                    })
                    .with_name(materia)
                )));
        });

    vec![
        (MeldType::CRITICAL, "CRT IX"),
        (MeldType::DETERMINATION, "DET IX"), 
        (MeldType::DIRECTHIT, "DH IX"),
        (MeldType::SPELLSPEED, "SPS IX"),
    ].into_iter()
        .for_each(|(meld_type, materia)| {
            meld_menu.add_child(Dialog::around(LinearLayout::vertical()
                .child(TextView::new(materia))
                .child(EditView::new()
                    .on_edit(move |s: &mut Cursive, content: &str, _| {
                        if let Ok(val) = content.parse() {
                            s.call_on_name(materia, |view: &mut EditView| view.set_style(ColorStyle::inherit_parent()));
                            s.with_user_data(|gearset: &mut Gearset| gearset.meld_ix[meld_type as usize] = val);
                            update_stats(s);
                        } else {
                            s.call_on_name(materia, |view: &mut EditView| view.set_style(ColorStyle::highlight()));
                        }
                    })
                    .with_name(materia)
                )));
        });

    left.add_child(meld_menu);

    let mut right = LinearLayout::vertical();
    selects_right.into_iter()
        .for_each(|select| right.add_child(select));
    
    let select_food = Dialog::around(LinearLayout::horizontal()
        .child(PaddedView::lrtb(0, 2, 0, 0, TextView::new("Food").v_align(VAlign::Center)))
        .child(
            SelectView::new()
                .with_all(items.clone().into_iter().filter(|item| item.slot == ItemSlot::FOOD).map(|item| (item.name.clone(), item.clone())))
                .on_select(on_select_food)
        ));

    right.add_child(select_food);

    let gear = LinearLayout::horizontal()
        .child(left)
        .child(right);

    let mut stats = LinearLayout::horizontal();
    vec!["WD", "MND", "DH", "Crit", "Det", "SpS", "Pie", "GCD", "Crit Rate", "Crit Mult", "DH Rate", "PPS", "DPS"].into_iter()
        .for_each(|stat| {
           stats.add_child(Dialog::around(LinearLayout::vertical()
                .child(TextView::new(stat))
                .child(TextView::new("0").with_name(stat))))
        });

    let main = LinearLayout::vertical()
        .child(gear)
        .child(stats.with_name("stats"));
    siv.add_layer(main);

    siv.call_on_all_named("item", |select: &mut SelectView| {
        select.set_selection(0);
    });

    let stats = gearset.stats();
    siv.call_on_name("WD", |view: &mut TextView| view.set_content(format!("{}", stats.weapon_damage)));
    siv.call_on_name("MND", |view: &mut TextView| view.set_content(format!("{}", stats.mind)));
    siv.call_on_name("DH", |view: &mut TextView| view.set_content(format!("{}", stats.direct_hit)));
    siv.call_on_name("Crit", |view: &mut TextView| view.set_content(format!("{}", stats.critical)));
    siv.call_on_name("Det", |view: &mut TextView| view.set_content(format!("{}", stats.determination)));
    siv.call_on_name("SpS", |view: &mut TextView| view.set_content(format!("{}", stats.spell_speed)));
    siv.call_on_name("Pie", |view: &mut TextView| view.set_content(format!("{}", stats.piety)));
    siv.call_on_name("GCD", |view: &mut TextView| view.set_content(format!("{:2}", stats.gcd().scalar())));
    siv.call_on_name("Crit Rate", |view: &mut TextView| view.set_content(format!("{:.2}", stats.crit_rate().scalar())));
    siv.call_on_name("Crit Mult", |view: &mut TextView| view.set_content(format!("{:.3}", stats.crit_multiplier().scalar())));
    siv.call_on_name("DH Rate", |view: &mut TextView| view.set_content(format!("{:.3}", stats.dh_rate().scalar())));
    siv.call_on_name("PPS", |view: &mut TextView| view.set_content(format!("{:.2}", stats.getP())));
    siv.call_on_name("DPS", |view: &mut TextView| view.set_content(format!("{:.2}", stats.balance_dps())));

    siv.set_user_data(gearset);

    siv.run();

    Ok(())
}

fn main() {
    //calc_sets(Stats::balance_dps).unwrap();
    //calc_sets(Stats::dps).unwrap();
    tui().unwrap();
}
