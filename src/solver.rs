use itertools::Itertools;

use crate::ui::UiLink;
use crate::data::*;

pub const DPS_FUNCTION: fn (&Stats) -> f64 = Stats::dps;

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

impl crate::data::Item {
    pub fn from_record(record: &csv::StringRecord) -> eyre::Result<Self> {
        let item = Self {
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
            },
            meld_slots: record.get(10).unwrap().parse().unwrap_or_default(),
            overmeldable: record.get(11).unwrap().parse().unwrap_or_default(),
        };

        Ok(item)
    }
}

impl std::cmp::PartialOrd for crate::data::Gearset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = DPS_FUNCTION(&self.stats());
        let b = DPS_FUNCTION(&other.stats());

        a.partial_cmp(&b)
    }
}

impl std::cmp::Ord for crate::data::Gearset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn calc_sets(ui_link: UiLink) -> eyre::Result<()> {
    ui_link.message("Loading items...")?;
    let items = load_items()?;
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
        ui_link.message("ERROR: Not all items were partitioned")?;
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

    ui_link.message("Ranking gear...")?;

    let results = product
        .map(|items| {
            let mut gearset = Gearset::from_items(items);
            gearset.base = SAGE_BASE;
            gearset
        })
        .filter(|gearset| {
            gearset.is_valid()
        })
        .map(std::cmp::Reverse)
        .k_smallest(10)
        .map(|rev| rev.0);

    ui_link.message("Ranking food/melds...")?;

    results.into_iter()
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
        .map(std::cmp::Reverse)
        .k_smallest(10)
        .map(|rev| rev.0)
        .for_each(|gearset| {
            ui_link.gearset(gearset).unwrap();
        });

    ui_link.message("Finished finding top 10 sets!")?;

    Ok(())
}

