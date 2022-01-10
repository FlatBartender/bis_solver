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
    fn add(&self, other: &Self) -> Self {
        Self {
            weapon_damage: self.weapon_damage + other.weapon_damage,
            mind: self.mind + other.mind,
            vitality: self.vitality + other.vitality,
            piety: self.piety + other.piety,
            direct_hit: self.direct_hit + other.direct_hit,
            critical: self.critical + other.critical,
            determination: self.determination + other.determination,
            spell_speed: self.spell_speed + other.spell_speed,
        }
    }

    fn apply_food(&mut self, food: &Item) {
        self.critical += food.stats.critical.min(self.critical / 10);
        self.direct_hit += food.stats.direct_hit.min(self.direct_hit / 10);
        self.determination += food.stats.determination.min(self.determination / 10);
        self.spell_speed += food.stats.spell_speed.min(self.spell_speed / 10);
        self.vitality += food.stats.vitality.min(self.vitality / 10);
        self.piety += food.stats.piety.min(self.piety / 10);
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
    slot: String,
    name: String,
    stats: Stats,
    meld_slots: u32,
    overmeldable: u32,
}

impl Item {
    fn stat_max(&self) -> u32 {
        self.stats.stat_max()
    }

    fn from_record(record: &csv::StringRecord) -> Result<Item, Box<dyn std::error::Error>> {
        let item = Item {
            slot: record.get(0).unwrap().to_string(),
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

fn calc_sets() -> Result<(), Box<dyn std::error::Error>> {
    let csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .quoting(false)
        .from_reader(ITEMS.as_bytes());

    let records: Vec<_> = csv_reader.into_records()
        .collect::<Result<_, _>>()?;
    let items: Vec<Item> = records.into_iter()
        .map(|record| Item::from_record(&record))
        .collect::<Result<Vec<_>, _>>()?;

    let base: Vec<_> = vec![Item {
        slot: "Base".to_string(),
        name: "Sage base".to_string(),
        stats: Stats {
            weapon_damage: 0,
            mind: 448,
            vitality: 390,
            piety: 390,
            direct_hit: 400,
            critical: 400,
            determination: 390,
            spell_speed: 400,
        },
        meld_slots: 0,
        overmeldable: 0,
    }];
    let arme: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Arme").collect();
    let tête: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Tête").collect();
    let torse: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Torse").collect();
    let mains: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Mains").collect();
    let jambes: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Jambes").collect();
    let pieds: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Pieds").collect();
    let oreille: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Oreille").collect();
    let collier: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Collier").collect();
    let bracelet: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Bracelet").collect();
    let bague: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Bague").collect();
    let nourriture: Vec<_> = items.clone().into_iter().filter(|item| item.slot == "Nourriture").collect();

    let product = vec![
            base.clone().into_iter(),
            arme.clone().into_iter(),
            tête.clone().into_iter(),
            torse.clone().into_iter(),
            mains.clone().into_iter(),
            jambes.clone().into_iter(),
            pieds.clone().into_iter(),
            oreille.clone().into_iter(),
            collier.clone().into_iter(),
            bracelet.clone().into_iter(),
            bague.clone().into_iter(),
            bague.clone().into_iter(),
        ].into_iter()
        .multi_cartesian_product();

    let mut results: Vec<_> = product
        .par_bridge()
        .filter_map(|vec_items| {
        if vec_items[10].name == vec_items[11].name && vec_items[10].overmeldable == 0 {
            return None;
        }
        let dps = vec_items.iter().fold(Stats::default(), |acc, item| acc.add(&item.stats)).dps();
        Some((dps, vec_items))
    }).collect();

    results.sort_by(|(a_dps, _), (b_dps, _)| b_dps.partial_cmp(a_dps).unwrap());
    results.dedup_by(|(a_dps, _), (b_dps, _)| a_dps == b_dps);

    let candidates: Vec<_> = results[..100].iter().cloned().collect();

    println!("CANDIDATE SETS, PRE-MELD, PRE FOOD");
    for candidate in candidates.iter(){
        println!("    DPS: {}", candidate.0);
        candidate.1.iter()
            .for_each(|item| println!("        Item: {}", item.name));
    }

    let mut melds: Vec<_> = candidates.iter()
        .map(|(_, items)| items)
        .flat_map(|items| {
            let mut slots_matérias_x = 0;
            let mut slots_matérias_ix = 0;
            for item in items.iter() {
                slots_matérias_x += item.meld_slots;
                if item.overmeldable == 1 {
                    slots_matérias_x += 1;
                    slots_matérias_ix += 5 - (item.meld_slots + 1);
                }
            }

            let mut meld_x_slots = vec![0, 0, 0, 0];
            let mut meld_ix_slots = vec![0, 0, 0, 0];
            items.iter()
                .for_each(|item| {
                    let mut item_x_slots = vec![0, 0, 0, 0];
                    let mut item_ix_slots = vec![0, 0, 0, 0];
                    if item.overmeldable == 0 {
                        item_x_slots[0] = item.meld_slots.min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).ceil() as u32);
                        item_x_slots[1] = item.meld_slots.min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).ceil() as u32);
                        item_x_slots[2] = item.meld_slots.min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).ceil() as u32);
                        item_x_slots[3] = item.meld_slots.min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).ceil() as u32);
                    } else {
                        item_x_slots[0] = (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 36.0).ceil() as u32);
                        item_x_slots[1] = (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 36.0).ceil() as u32);
                        item_x_slots[2] = (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 36.0).ceil() as u32);
                        item_x_slots[3] = (item.meld_slots + 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 36.0).ceil() as u32);

                        item_ix_slots[0] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.critical as f64) / 12.0).ceil() as u32);
                        item_ix_slots[1] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.determination as f64) / 12.0).ceil() as u32);
                        item_ix_slots[2] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.direct_hit as f64) / 12.0).ceil() as u32);
                        item_ix_slots[3] += (5 - item.meld_slots - 1).min(((item.stat_max() as f64 - item.stats.spell_speed as f64) / 12.0).ceil() as u32);
                    }

                    for index in 0..4 {
                        meld_x_slots[index] += item_x_slots[index];
                        meld_ix_slots[index] += item_ix_slots[index];
                    }
                });

            let matérias_x_répartitions: Vec<_> = vec![
                (0..=slots_matérias_x),
                (0..=slots_matérias_x),
                (0..=slots_matérias_x),
                (0..=slots_matérias_x),
            ].into_iter()
                .multi_cartesian_product()
                .filter(|répartition| {
                    répartition.clone().into_iter().sum::<u32>() == slots_matérias_x
                        && répartition[0] <= meld_x_slots[0]
                        && répartition[1] <= meld_x_slots[1]
                        && répartition[2] <= meld_x_slots[2]
                        && répartition[3] <= meld_x_slots[3]
                })
                .collect();
            let matérias_ix_répartitions: Vec<_> = vec![
                (0..=slots_matérias_ix),
                (0..=slots_matérias_ix),
                (0..=slots_matérias_ix),
                (0..=slots_matérias_ix),
            ].into_iter()
                .multi_cartesian_product()
                .filter(|répartition| {
                    répartition.clone().into_iter().sum::<u32>() == slots_matérias_ix
                        && répartition[0] <= meld_ix_slots[0]
                        && répartition[1] <= meld_ix_slots[1]
                        && répartition[2] <= meld_ix_slots[2]
                        && répartition[3] <= meld_ix_slots[3]
                })
                .collect();

            std::iter::once(items).cartesian_product(nourriture.iter()).cartesian_product(matérias_x_répartitions.into_iter()).cartesian_product(matérias_ix_répartitions.into_iter())
        })
        .map(|(((items, food), meld_x), meld_ix)| (items, food, meld_x, meld_ix))
        .par_bridge()
        .map(|(items, food, meld_x, meld_ix)| {
            let stats = items.iter().fold(Stats::default(), |acc, item| acc.add(&item.stats));
            let mut stats = stats.add(&Stats {
                critical: meld_x[0] * 36 + meld_ix[0] * 12,
                determination: meld_x[1] * 36 + meld_ix[1] * 12,
                direct_hit: meld_x[2] * 36 + meld_ix[2] * 12,
                spell_speed: meld_x[3] * 36 + meld_ix[3] * 12,
                ..Stats::default()
            });
            stats.apply_food(food);
            (stats.dps(), items, food, meld_x, meld_ix)
        })
        .collect();

    melds.sort_by(|(a_dps, _, _, _, _), (b_dps, _, _, _, _)| b_dps.partial_cmp(a_dps).unwrap());

    println!("MELDED SETS");
    for (_, items, food, melds_x, melds_ix) in &melds[0..10] {
        let mut stats = items.iter().fold(Stats::default(), |acc, item| acc.add(&item.stats));
        stats.apply_food(food);
        stats.critical += melds_x[0] * 36 + melds_ix[0] * 12;
        stats.determination += melds_x[1] * 36 + melds_ix[1] * 12;
        stats.direct_hit += melds_x[2] * 36 + melds_ix[2] * 12;
        stats.spell_speed += melds_x[3] * 36 + melds_ix[3] * 12;

        println!("    DPS: {}", stats.dps());
        items.iter()
            .for_each(|item| {
                println!("        Item: {}", item.name);
            });
        println!("        Food: {}", food.name);
        
        println!("        Melds: {} CRT X, {} DET X, {} DH X, {} SPS X, {} CRT IX, {} DET IX, {} DH IX, {} SPS IX",
            melds_x[0], melds_x[1], melds_x[2], melds_x[3],
            melds_ix[0], melds_ix[1], melds_ix[2], melds_ix[3]);
        println!("        GCD: {}, crit rate: {}, crit damage: {}, DH rate: {}",
            stats.gcd().scalar(), stats.crit_rate().scalar(), stats.crit_multiplier().scalar(), stats.dh_rate().scalar());
        println!("{:?}", stats);
    }

    Ok(())
}

fn main() {
    calc_sets().unwrap();
}
