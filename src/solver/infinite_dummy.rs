use crate::solver::Evaluator;
use crate::utils::Scalable;
use crate::data::Gearset;

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
pub struct InfiniteDummyEvaluator {}

impl Evaluator for InfiniteDummyEvaluator {
    fn dps(&self, gearset: &Gearset) -> f64 {
        InfiniteDummyStat::dps(&gearset.stats())
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
