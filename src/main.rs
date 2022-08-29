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
        (self * unit.0 * NUMERATOR) / DENOMINATOR
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

const DPS_FUNCTION: fn (&Stats) -> f64 = Stats::dps;

trait StatRepo {
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

    fn crit_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.crit_rate().0 + self.crit_rate().0 * self.crit_multiplier().0 / 1000)
    }

    fn dh_scalar(&self) -> Unit<1, 1000> {
        Unit(1000 - self.dh_rate().0 + self.dh_rate().0 * 125 / 100)
    }

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
    gcd_uptime: f64,
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
    fn gcd_uptime(&self) -> f64 {
        self.gcd_uptime
    }
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
    gcd_uptime: 1.0,
};

#[derive(Debug, Clone)]
struct Gearset {
    base: Stats,
    items: [Item; 11],
    food: Item,
    meld_x: MatX,
    meld_ix: MatIX,
    gcd_uptime: f64,
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
            gcd_uptime: 1.0,
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
        stats.gcd_uptime = self.gcd_uptime;
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

    fn from_record(record: &csv::StringRecord) -> eyre::Result<Item> {
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
                spell_speed: record.get(9).unwrap().parse().unwrap_or_default(),
                gcd_uptime: 1.0,
            },
            meld_slots: record.get(10).unwrap().parse().unwrap_or_default(),
            overmeldable: record.get(11).unwrap().parse().unwrap_or_default(),
        };

        Ok(item)
    }
}

const ITEMS: &str = include_str!("items.csv");

fn load_items() -> eyre::Result<Vec<Item>> {
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

fn calc_sets(ui_link: UiLink) -> eyre::Result<()> {
    ui_link.message("Loading items...")?;
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
    
    ui_link.message("Ranking gear...")?;

    let mut results: Vec<_> = product
        .map(|items| Gearset::from_items(items))
        .filter_map(|gearset| {
            if !gearset.is_valid() { 
                None
            } else {
                Some((DPS_FUNCTION(&gearset.stats()), gearset))
            }
    }).collect();

    results.sort_by(|(a_dps, _), (b_dps, _)| b_dps.partial_cmp(a_dps).unwrap());
    results.dedup_by(|(a_dps, _), (b_dps, _)| a_dps == b_dps);

    let candidates: Vec<_> = results[..10].iter().cloned().collect();
    
    ui_link.message("Ranking food/melds...")?;

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
            (DPS_FUNCTION(&gearset.stats()), gearset)
        })
        .collect();

    melds.sort_by(|(a_dps, _,), (b_dps, _,)| b_dps.partial_cmp(a_dps).unwrap());

    melds.into_iter()
        .take(10)
        .for_each(|(_, gearset)| {
            ui_link.gearset(gearset).unwrap();
        });

    ui_link.message("Finished finding top 10 sets!")?;

    //for (_, gearset) in &melds[..10] {
    //    let stats = gearset.stats();
    //    println!("    DPS: {}", stats.dps());
    //    gearset.items.iter()
    //        .for_each(|item| {
    //            println!("        Item: {}", item.name);
    //        });
    //    println!("        Food: {}", gearset.food.name);
    //    
    //    println!("        Melds: {} CRT X, {} DET X, {} DH X, {} SPS X, {} CRT IX, {} DET IX, {} DH IX, {} SPS IX",
    //        gearset.meld_x[0], gearset.meld_x[1], gearset.meld_x[2], gearset.meld_x[3],
    //        gearset.meld_ix[0], gearset.meld_ix[1], gearset.meld_ix[2], gearset.meld_ix[3]);
    //    println!("        GCD: {}, crit rate: {}, crit damage: {}, DH rate: {}",
    //        stats.gcd().scalar(), stats.crit_rate().scalar(), stats.crit_multiplier().scalar(), stats.dh_rate().scalar());
    //    println!("{:?}", stats);
    //}

    Ok(())
}

use eframe::egui;

#[derive(Clone)]
struct UiLink {
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

    pub fn gearset(&self, gearset: Gearset) -> eyre::Result<()> {
        self.status_send.send(UiMessage::NewGearset(gearset))?;
        Ok(())
    }
}

enum UiMessage {
    StatusMessage(String),
    NewGearset(Gearset),
}

struct Ui {
    status_recv: std::sync::mpsc::Receiver<UiMessage>,

    status: String,
    gearsets: Vec<Gearset>,
}

impl Ui {
    pub fn new(_cc: &eframe::CreationContext<'_>, status_recv: std::sync::mpsc::Receiver<UiMessage>) -> Self {
        Self {
            status_recv,

            status: "Startup".to_string(),
            gearsets: Vec::new(),
        }
    }

    pub fn handle_message(&mut self, message: UiMessage) {
        use UiMessage::*;

        match message {
            StatusMessage(message) => self.status = message,
            NewGearset(gearset) => self.gearsets.push(gearset),
        }
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
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_extras::{Size, TableBuilder};

            let text_size = egui::TextStyle::Body.resolve(ui.style()).size;

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(20.0).at_least(20.0))
                .column(Size::initial(120.0).at_least(40.0))
                .resizable(true);

            table.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("#");
                });
                header.col(|ui| {
                    ui.heading("DPS");
                });
            })
            .body(|mut body| {
                for (index, gearset) in self.gearsets.iter().enumerate() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label((index+1).to_string());
                        });
                        row.col(|ui| {
                            ui.label(DPS_FUNCTION(&gearset.stats()).to_string());
                        });
                    });
                }
            });
        });
    }
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    let (status_send, status_recv) = std::sync::mpsc::channel();

    let app_link = UiLink::new(status_send);
    std::thread::spawn({
        let calc_app_link = app_link.clone();
        move || {
            calc_sets(calc_app_link).unwrap();
        }
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "BiS Solver",
        native_options,
        Box::new(|cc| Box::new(Ui::new(cc, status_recv))),
    );

    Ok(())
}
