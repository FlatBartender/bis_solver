use crate::solver::Evaluator;
use crate::utils::Scalable;
use crate::data::Gearset;

pub trait InfiniteDummyStat: crate::data::StatRepo {
    fn cycle_length(&self) -> f64 {
        self.casts_per_cycle() as f64 * self.adjusted_gcd() + 2.5
    }

    fn casts_per_cycle(&self) -> f64 {
        let gcd = self.adjusted_gcd();
        let casts_per_cycle = (30.0 - 2.5) / gcd;
        let early_refresh_casts = casts_per_cycle.floor();
        let late_refresh_casts = casts_per_cycle.ceil();

        let early_refresh_pps = {
            let cycle = gcd * early_refresh_casts + 2.5;
            let dosis_per_second = early_refresh_casts / cycle;
            let ticks_per_second = cycle.min(30.0) / cycle / 3.0;
            dosis_per_second * 330.0 + ticks_per_second * 70.0
        };
        let late_refresh_pps = {
            let cycle = gcd * late_refresh_casts + 2.5;
            let dosis_per_second = late_refresh_casts / cycle;
            let ticks_per_second = cycle.min(30.0) / cycle / 3.0;
            dosis_per_second * 330.0 + ticks_per_second * 70.0
        };
        if early_refresh_pps >= late_refresh_pps {
            // We're on an early refresh cycle
            early_refresh_casts
        } else {
            // We're on a late refresh cycle
            late_refresh_casts
        }
    }

    fn phlegma_per_cycle(&self) -> f64 {
        self.cycle_length() / 45.0
    }

    fn dosis_per_cycle(&self) -> f64 {
        self.casts_per_cycle() - self.phlegma_per_cycle()
    }

    fn eukr_dosis_ticks_per_cycle(&self) -> f64 {
        self.cycle_length().min(30.0) / 3.0
    }

    fn dosis_per_second(&self) -> f64 {
        self.dosis_per_cycle() / self.cycle_length()
    }

    fn phlegma_per_second(&self) -> f64 {
        self.phlegma_per_cycle() / self.cycle_length()
    }

    fn eukr_dosis_ticks_per_second(&self) -> f64 {
        self.eukr_dosis_ticks_per_cycle() / self.cycle_length()
    }

    fn dosis_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 330.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_factor() * self.dh_factor()
    }

    fn phlegma_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 510.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_factor() * self.dh_factor()
    }

    fn eukr_dosis_score(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let sps = self.sps_multiplier();
        let damage = 70.scale(adj_wd).scale(map).scale(det).scale(sps) * 130 / 100 + 1;
        damage as f64 * self.crit_factor() * self.dh_factor()
    }

    fn pps(&self) -> f64 {
        self.dosis_per_second() * 330.0
            + self.phlegma_per_second() * 510.0
            + self.eukr_dosis_ticks_per_second() * 700.0
    }

    fn dps(&self) -> f64 {
        self.dosis_per_second() * self.dosis_score()
        + self.phlegma_per_second() * self.phlegma_score()
        + self.eukr_dosis_ticks_per_second() * self.eukr_dosis_score()
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
