#![allow(unused)]

use std::collections::HashMap;

use flagset::{flags, FlagSet};
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    page_next: Option<usize>,
    page_prev: Option<usize>,
    paget_total: usize,
    results: usize,
    results_per_page: usize,
    results_total: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Search<T> {
    pagination: Pagination,
    results: Vec<T>,
}

flags! {
    pub enum ClassJobCategoryFlags: usize {
        ACN,
        ADV,
        ALC,
        ARC,
        ARM,
        AST,
        BLM,
        BLU,
        BRD,
        BSM,
        BTN,
        CNJ,
        CRP,
        CUL,
        DNC,
        DRG,
        DRK,
        FSH,
        GLA,
        GNB,
        GSM,
        LNC,
        LTW,
        MCH,
        MIN,
        MNK,
        MRD,
        NIN,
        PGL,
        PLD,
        RDM,
        ROG,
        RPR,
        SAM,
        SCH,
        SGE,
        SMN,
        THM,
        WAR,
        WHM,
        WVR,
    }

    pub enum EquipSlotCategoryFlags: usize {
        Body,
        Ears,
        Feet,
        FingerL,
        FingerR,
        Gloves,
        Head,
        Legs,
        MainHand,
        Neck,
        OffHand,
        SoulCrystal,
        Waist,
        Wrists,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct ClassJobCategoryPayload {
    acn: u8,
    adv: u8,
    alc: u8,
    arc: u8,
    arm: u8,
    ast: u8,
    blm: u8,
    blu: u8,
    brd: u8,
    bsm: u8,
    btn: u8,
    cnj: u8,
    crp: u8,
    cul: u8,
    dnc: u8,
    drg: u8,
    drk: u8,
    fsh: u8,
    gla: u8,
    gnb: u8,
    gsm: u8,
    lnc: u8,
    ltw: u8,
    mch: u8,
    min: u8,
    mnk: u8,
    mrd: u8,
    nin: u8,
    pgl: u8,
    pld: u8,
    rdm: u8,
    rog: u8,
    rpr: u8,
    sam: u8,
    sch: u8,
    sge: u8,
    smn: u8,
    thm: u8,
    war: u8,
    whm: u8,
    wvr: u8
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EquipSlotCategoryPayload {
    body: u8,
    ears: u8,
    feet: u8,
    finger_l: u8,
    finger_r: u8,
    gloves: u8,
    head: u8,
    legs: u8,
    main_hand: u8,
    neck: u8,
    off_hand: u8,
    soul_crystal: u8,
    waist: u8,
    wrists: u8,
}

pub type ClassJobCategory = FlagSet<ClassJobCategoryFlags>;
pub type EquipSlotCategory = FlagSet<EquipSlotCategoryFlags>;

impl From<ClassJobCategoryPayload> for FlagSet<ClassJobCategoryFlags> {
    fn from(other: ClassJobCategoryPayload) -> Self {
        use ClassJobCategoryFlags::*;

        let mut result = Self::default();

        if other.acn > 0 { result |= ACN };
        if other.adv > 0 { result |= ADV };
        if other.alc > 0 { result |= ALC };
        if other.arc > 0 { result |= ARC };
        if other.arm > 0 { result |= ARM };
        if other.ast > 0 { result |= AST };
        if other.blm > 0 { result |= BLM };
        if other.blu > 0 { result |= BLU };
        if other.brd > 0 { result |= BRD };
        if other.bsm > 0 { result |= BSM };
        if other.btn > 0 { result |= BTN };
        if other.cnj > 0 { result |= CNJ };
        if other.crp > 0 { result |= CRP };
        if other.cul > 0 { result |= CUL };
        if other.dnc > 0 { result |= DNC };
        if other.drg > 0 { result |= DRG };
        if other.drk > 0 { result |= DRK };
        if other.fsh > 0 { result |= FSH };
        if other.gla > 0 { result |= GLA };
        if other.gnb > 0 { result |= GNB };
        if other.gsm > 0 { result |= GSM };
        if other.lnc > 0 { result |= LNC };
        if other.ltw > 0 { result |= LTW };
        if other.mch > 0 { result |= MCH };
        if other.min > 0 { result |= MIN };
        if other.mnk > 0 { result |= MNK };
        if other.mrd > 0 { result |= MRD };
        if other.nin > 0 { result |= NIN };
        if other.pgl > 0 { result |= PGL };
        if other.pld > 0 { result |= PLD };
        if other.rdm > 0 { result |= RDM };
        if other.rog > 0 { result |= ROG };
        if other.rpr > 0 { result |= RPR };
        if other.sam > 0 { result |= SAM };
        if other.sch > 0 { result |= SCH };
        if other.sge > 0 { result |= SGE };
        if other.smn > 0 { result |= SMN };
        if other.thm > 0 { result |= THM };
        if other.war > 0 { result |= WAR };
        if other.whm > 0 { result |= WHM };
        if other.wvr > 0 { result |= WVR };

        result
    }
}

impl From<EquipSlotCategoryPayload> for EquipSlotCategory {
    fn from(other: EquipSlotCategoryPayload) -> Self {
        use EquipSlotCategoryFlags::*;

        let mut result = Self::default();

        if other.body         > 0 { result |= Body        };
        if other.ears         > 0 { result |= Ears        };
        if other.feet         > 0 { result |= Feet        };
        if other.finger_l     > 0 { result |= FingerL     };
        if other.finger_r     > 0 { result |= FingerR     };
        if other.gloves       > 0 { result |= Gloves      };
        if other.head         > 0 { result |= Head        };
        if other.legs         > 0 { result |= Legs        };
        if other.main_hand    > 0 { result |= MainHand    };
        if other.neck         > 0 { result |= Neck        };
        if other.off_hand     > 0 { result |= OffHand     };
        if other.soul_crystal > 0 { result |= SoulCrystal };
        if other.waist        > 0 { result |= Waist       };
        if other.wrists       > 0 { result |= Wrists      };

        result
    }
}

#[derive(Deserialize, Eq, PartialEq, Hash)]
enum Stat {
    CriticalHit,
    Determination,
    Strength,
    Vitality,
    Mind,
    DirectHitRate,
    SpellSpeed,
    SkillSpeed,
    Piety,
    Tenacity,
    Dexterity,
    Intelligence,
}

// TODO Maybe support fetching potions ? Sounds like a hassle for not much benefit

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StatPayload {
    hq: Option<u32>,
    nq: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemPayload {
    stats: HashMap<Stat, StatPayload>,
    class_job_category: ClassJobCategoryPayload,
    equip_slot_category: EquipSlotCategoryPayload,
    damage_mag: u32,
    damage_phys: u32,
    defense_mag: u32,
    defense_phys: u32,
    delay_ms: u32,
}

pub struct Item {
    stats: HashMap<Stat, u32>,
    // TODO
}
