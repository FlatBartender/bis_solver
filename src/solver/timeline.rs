use std::collections::HashSet;
use std::sync::Mutex;
use std::collections::HashMap;

use itertools::Itertools;

use crate::data::*;

#[derive(Clone)]
pub struct TimespanSearch<T: Clone> {
    data: Vec<(Timespan, T)>,
    begins: Vec<usize>,
    ends: Vec<usize>,
}

impl<T: Clone> TimespanSearch<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            begins: Vec::new(),
            ends: Vec::new(),
        }
    }

    pub fn push(&mut self, time: Timespan, data: T) {
        self.begins.push(self.data.len());
        self.ends.push(self.data.len());
        self.data.push((time, data));
        self.sort();
    }

    fn sort(&mut self) {
        self.begins.sort_by(|index_a, index_b| {
            self.data[*index_a].0.begin.partial_cmp(&self.data[*index_b].0.begin).unwrap()
        });
        self.ends.sort_by(|index_a, index_b| {
            self.data[*index_a].0.end.partial_cmp(&self.data[*index_b].0.end).unwrap()
        });
    }

    pub fn spans(&self, instant: f64) -> Vec<&(Timespan, T)> {
        let begin_candidates: HashSet<_> = self.begins.iter()
            .filter(|index| self.data[**index].0.begin <= instant)
            .collect();

        let end_candidates: HashSet<_> = self.ends.iter()
            .filter(|index| self.data[**index].0.end > instant)
            .collect();

        begin_candidates.intersection(&end_candidates)
            .map(|index| &self.data[**index])
            .collect()
    }

    // Doesn't take into account overlapping spans, but it's all right, this only really matters
    // for downtime anyway (and downtime doesn't overlap)
    pub fn next_start(&self, instant: f64) -> Option<&(Timespan, T)> {
        // Array is sorted so we can use next() after a filter to get the first next downtime
        self.begins.iter()
            .find(|index| self.data[**index].0.begin >= instant)
            .and_then(|index| self.data.get(*index))
    }
}

impl<T: Clone> From<Vec<(Timespan, T)>> for TimespanSearch<T> {
    fn from(other: Vec<(Timespan, T)>) -> Self {
        let data = other;
        let begins = (0..data.len()).collect();
        let ends = (0..data.len()).collect();
        let mut ret = Self {
            data, begins, ends
        };

        ret.sort();

        ret
    }
}

impl From<Vec<Timespan>> for TimespanSearch<()> {
    fn from(other: Vec<Timespan>) -> Self {
        Self::from(other.into_iter().map(|timespan| (timespan, ())).collect::<Vec<_>>())
    }
}

type BuffedTimelineAction = (f64, SGEAction, SimplifiedBuff);

pub struct Timeline {
    downtime: TimespanSearch<()>,
    buffs: TimespanSearch<Buff>,
    comp_mind: f64,
    end: f64,
    timeline_cache: Mutex<HashMap<usize, Vec<BuffedTimelineAction>>>,
}

// TODO
// Add tweaking for downtimes to take into account players using 1 GCD after downtime, some buffs
// are also used before the end of downtime (potion, something on NIN, and BRD's songs)
struct TimelineIterator {
    downtime: TimespanSearch<()>,
    step: f64,
    current: f64,
    end: f64,
}

impl Iterator for TimelineIterator {
    type Item = f64;

    // Return the current one and compute the next
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            None
        } else {
            let ret = Some(self.current);
            self.current += self.step;
            self.current = self.downtime.spans(self.current).into_iter().map(|(span, _)| span.end).fold(self.current, f64::max);
            ret
        }
    }
}

impl TimelineIterator {
    fn next_with_step(&mut self, step: f64) -> Option<f64> {
        let old_step = self.step;
        self.step = step;
        let ret = self.next();
        self.step = old_step;
        ret
    }

    fn next_span(&mut self, span: &Timespan) -> Option<f64> {
        if !self.downtime.spans(span.end + self.current).is_empty() {
            // End is in a downtime, reschedule
            self.current += span.end;
        }
        self.next()
    }

    fn from_timeline(timeline: &Timeline, start: f64, step: f64) -> Self {
        Self {
            downtime: timeline.downtime.clone(),
            end: timeline.end,
            current: start,
            step,
        }
    }
}

impl Timeline {
    pub fn new(downtime: Vec<Timespan>, end: f64, comp_mind: f64) -> Self {
        Self {
            downtime: downtime.into(),
            end,
            comp_mind,
            buffs: TimespanSearch::new(),
            timeline_cache: Mutex::new(HashMap::new()),
        }
    }

    // BRD:
    //  - Songs: Might change duration based on rotation
    //      - Mage's Ballad -> 1% damage / 45s
    //      - Army's Paeon -> 3% DH / 45s
    //      - Wanderer's Minuet -> 2% crit / 45s
    //      - Sequencing: 43s WM - 34s MB - 43s AP
    //      - opener at ~.5s (first weave slot)
    //  - Battle Voice -> 20% DH / 15s -> on raid buff window
    //      - opener 3rd GCD/2nd weaving slot -> CD starts at ~6.5s
    //      - window 120s
    //  - Radiant Finale -> 2-4-6% damage / 15s -> on raid buff window, 2% on opener, 6% on buff
    //  windows
    //      - opener 3rd GCD/1st weaving slot -> CD starts at ~5.5s
    //      - window 120s
    //
    // Not the exact song delaying strat used by BRDs but it's good enough for now.
    pub fn with_brd(&mut self) -> &mut Self {
        let song_sequence = [
            (Timespan::new(0.0, 43.0), Buff::Critical(0.02)), // WM
            (Timespan::new(0.0, 34.0), Buff::Damage(0.01)), // MB
            (Timespan::new(0.0, 43.0), Buff::DirectHit(0.03)), // AP
        ];

        let mut song_iter = TimelineIterator::from_timeline(self, 0.5, 0.0);
        for (span, song) in song_sequence.iter().cycle() {
            // Compensate scheduling for downtime
            if let Some(offset) = song_iter.next_with_step(span.end) {
                self.buffs.push(span.clone().offset(offset), song.clone());
            } else {
                break;
            }
        }

        // Opener radiant finale
        let radiant_finale = Timespan::new(0.0, 15.0);
        let mut radiant_iter = TimelineIterator::from_timeline(self, 5.5, 120.0);
        if let Some(offset) = radiant_iter.next() {
            self.buffs.push(radiant_finale.clone().offset(offset), Buff::Damage(0.02));
        }
        // Other radiant finale
        while let Some(offset) = radiant_iter.next_span(&radiant_finale) {
            self.buffs.push(radiant_finale.clone().offset(offset), Buff::Damage(0.06));
        }

        let battle_voice = Timespan::new(0.0, 15.0);
        let mut battle_voice_iter = TimelineIterator::from_timeline(self, 6.5, 120.0);
        while let Some(offset) = battle_voice_iter.next_span(&battle_voice) {
            self.buffs.push(battle_voice.clone().offset(offset), Buff::DirectHit(0.2));
        }

        self
    }

    // DNC:
    //  - Technical finish -> 1-2-3-5% damage / 20s, starts quite late after buff window
    //      - tech step at 2nd GCD, then 4 steps * 1s then release -> ~6.5s
    //      - drifted to finish GCD -> ~121s
    pub fn with_dnc(&mut self) -> &mut Self {
        let technical_finish = Timespan::new(0.0, 20.0);
        let mut technical_finish_iter = TimelineIterator::from_timeline(self, 6.5, 121.0);
        while let Some(offset) = technical_finish_iter.next_span(&technical_finish) {
            self.buffs.push(technical_finish.clone().offset(offset), Buff::Damage(0.05));
        }
        self
    }

    // SMN:
    //  - Searing Light -> 3% damage / 30s -> on raid buff window
    //      - 1s prepull + 1st GCD 2nd weave -> ~2.5s
    //      - window 120s
    pub fn with_smn(&mut self) -> &mut Self {
        let searing_light = Timespan::new(0.0, 30.0);
        let mut searing_light_iter = TimelineIterator::from_timeline(self, 2.5, 120.0);
        while let Some(offset) = searing_light_iter.next_span(&searing_light) {
            self.buffs.push(searing_light.clone().offset(offset), Buff::Damage(0.03));
        }
        self
    }

    // RDM:
    //  - Embolden -> 5% damage / 20s -> on raid buff window
    //      - 3rd GCD 1st weave -> ~5.5s
    //      - window 120s
    pub fn with_rdm(&mut self) -> &mut Self {
        let embolden = Timespan::new(0.0, 20.0);
        let mut embolden_iter = TimelineIterator::from_timeline(self, 5.5, 120.0);
        while let Some(offset) = embolden_iter.next_span(&embolden) {
            self.buffs.push(embolden.clone().offset(offset), Buff::Damage(0.05));
        }
        self
    }

    // MNK:
    //  - Brotherhood -> 5% damage / 15s -> on raid buff window
    //      - 4th gcd 1st weave -> 1.94*3+.5 -> 6.5s (GCD at ~1.94)
    //      or
    //      - 3rd gcd 1st weave -> 1.94*2+.5 -> 4.5s
    //      - window 120s
    // This function assumes 4th GCD 1st weave for better alignment
    pub fn with_mnk(&mut self) -> &mut Self {
        let brotherhood = Timespan::new(0.0, 15.0);
        let mut brotherhood_iter = TimelineIterator::from_timeline(self, 6.5, 120.0);
        while let Some(offset) = brotherhood_iter.next_span(&brotherhood) {
            self.buffs.push(brotherhood.clone().offset(offset), Buff::Damage(0.05));
        }
        self
    }

    // DRG:
    //  - Battle Litany -> 10% crit / 15s -> on raid buff window
    //      - 3rd gcd 1st weave -> 5.5s
    //      - window 120s
    pub fn with_drg(&mut self) -> &mut Self {
        let battle_litany = Timespan::new(0.0, 15.0);
        let mut battle_litany_iter = TimelineIterator::from_timeline(self, 5.5, 120.0);
        while let Some(offset) = battle_litany_iter.next_span(&battle_litany) {
            self.buffs.push(battle_litany.clone().offset(offset), Buff::Critical(0.1));
        }
        self
    }

    // RPR:
    //  - Arcane Circle -> 3% damage / 20s -> on raid buffs window
    //      - prepull 1s, 1st GCD 1st weave -> 1.5s
    //      - window 120s
    pub fn with_rpr(&mut self) -> &mut Self {
        let arcane_circle = Timespan::new(0.0, 20.0);
        let mut arcane_circle_iter = TimelineIterator::from_timeline(self, 1.5, 120.0);
        while let Some(offset) = arcane_circle_iter.next_span(&arcane_circle) {
            self.buffs.push(arcane_circle.clone().offset(offset), Buff::Damage(0.03));
        }
        self
    }

    // NIN:
    //  - Mug -> 5% damage / 20s -> on raid buff window
    //      - 1 clip 2nd GCD 1st weave -> ~3s (GCD at ~2.10)
    //      - window 120s
    pub fn with_nin(&mut self) -> &mut Self {
        let mug = Timespan::new(0.0, 20.0);
        let mut mug_iter = TimelineIterator::from_timeline(self, 3.0, 120.0);
        while let Some(offset) = mug_iter.next_span(&mug) {
            self.buffs.push(mug.clone().offset(offset), Buff::Damage(0.05));
        }
        self
    }

    // SCH:
    //  - Chain Stratagem -> 10% critical / 15s -> on raid buff window
    //      - prepull 1s + 3rd GCD (swifted) 1st weave -> 6.5s
    //      or
    //      - prepull 1s + 3rd GCD 2nd weave -> 7.5s
    //      - window 121s (weaved)
    pub fn with_sch(&mut self) -> &mut Self {
        let chain_stratagem = Timespan::new(0.0, 15.0);
        for offset in TimelineIterator::from_timeline(self, 6.5, 121.0) {
            self.buffs.push(chain_stratagem.clone().offset(offset), Buff::Critical(0.1));
        }
        self
    }

    // AST:
    //  - Divination -> 6% damage / 15s -> on raid buff window
    //      - prepull 1s + 3rd GCD 1st weave -> 6.5s
    //      - window 120s (weaved but lightspeed)
    pub fn with_ast(&mut self) -> &mut Self {
        let divination = Timespan::new(0.0, 15.0);
        let mut divination_iter = TimelineIterator::from_timeline(self, 6.5, 120.0);
        while let Some(offset) = divination_iter.next_span(&divination) {
            self.buffs.push(divination.clone().offset(offset), Buff::Damage(0.06));
        }
        self
    }

    // Potions:
    //  - Assume bonus is maxed (grade 7 tincture is 223 MND)
    //  - First in opener at -3.0, shcedule the rest when there are the most buffs
    //
    //  Must be called after applying buffs !
    pub fn with_potions(&mut self) -> &mut Self {
        let potion = Timespan::new(0.0, 30.0);
        let mut snapshots = Vec::new();
        for (span, _) in self.buffs.data.iter() {
            snapshots.push((span.begin, self.buffs.spans(span.begin).len()));
            snapshots.push((span.end, self.buffs.spans(span.end).len()));
        }

        snapshots.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
        snapshots.dedup();
        // Calculate the score for each span
        let scores: Vec<(f64, f64)> = snapshots.into_iter().tuple_windows()
            .map(|((offset_a, buffs_a), (offset_b, buffs_b))| (offset_a, (offset_b - offset_a) * (buffs_b as f64 - buffs_a as f64)))
            .collect();
        // Calculate the actual potion score would it be used there
        let potion_candidates: Vec<_> = scores.iter().enumerate()
            .map(|(index, (offset, _))| (
                *offset,
                scores[index..]
                    .iter()
                    .take_while(|(end_offset, _)| *end_offset < offset + 30.0)
                    .map(|(_, score)| *score)
                    .sum::<f64>()
            ))
        // Calculate the potion score variation. If this goes in the negative, this means it was
        // better to use the potion on the previous position.
            .tuple_windows()
            .map(|((offset_a, score_a), (_, score_b))| (offset_a, score_b - score_a))
        // Find the places where the gradient crosses 0 from the positive (local maximum)
            .tuple_windows()
            .filter_map(|((offset, gradient_a), (_, gradient_b))| if gradient_a >= 0.0 && gradient_b < 0.0 { Some(offset) } else { None })
            .collect();

        let mut potion_clock = -3.0;
        self.buffs.push(potion.clone().offset(potion_clock), Buff::Mind(223));
        potion_clock += 270.0;

        while potion_clock < self.end {
            if let Some(offset) = potion_candidates.iter().find(|offset| **offset >= potion_clock) {
                self.buffs.push(potion.clone().offset(*offset), Buff::Mind(223));
                potion_clock = offset + 270.0;
            } else {
                break;
            }
        }
        self
    }

    // Establish the SGE cast timeline
    // This has quite a massive impact on performance though because it's a per-gearset
    // optimization, not per-GCD
    //
    // Edosis placement:
    // - renew on the 12th or 13th GCD, depending on how much is clipped with the current GCD
    // - Do not use if the next downtime is in less than dosis_potency/edosis_potency*3s, rounded
    //      up (otherwise it's better to dosis)
    // Phlegma placement:
    // - 2 on opener, CD starts at prepull 1s + eukrasia+edosis + 2 GCD -> 1 + 1 + 2 + 1.5 + 2*GCD
    //      so around ~10.5
    // - phlegma used when about to cap
    // - phlegma used immediately if there are more buffs ongoing than next GCD slot and there is a
    //      phlegma available
    // - on downtime, check if phlegma will overcap during downtime or not
    //    - if it overcaps, check if the overcap is "fine" (eg, no use lost based on kill time)
    //    - if it's not fine, use immediately
    // - TODO find a better heuristic ?
    // Dosis placement:
    // - Every free GCD that is not edosis or phlegma
    pub fn sge_timeline(&self, spell_speed: u32) -> Vec<(f64, SGEAction, SimplifiedBuff)> {
        let stats = crate::data::Stats {
            spell_speed,
            ..Stats::default()
        };
        let gcd = stats.gcd().scalar();
        if let Some(timeline) = self.timeline_cache.lock().unwrap().get(&((gcd*100.0) as usize)) {
            return timeline.clone();
        }
        let gcd15 = stats.gcd15().scalar();
        let mut sge_timeline: Vec<(f64, Option<SGEAction>, Vec<Buff>)> = Vec::new();

        // TODO Take into account GCD clip for the rotation
        let casts_per_cycle = (30.0 - 2.5) / gcd;
        let early_refresh_casts = casts_per_cycle.floor();
        let late_refresh_casts = casts_per_cycle.ceil();
        // Early refresh has no DoT cost, but has dosis cost
        let early_refresh_loss = (30.0 - early_refresh_casts * gcd - 2.5) * 330.0 / gcd;
        // Late refresh has no dosis cost but has a DoT cost
        let late_refresh_loss = (30.0 - late_refresh_casts * gcd - 2.5) * 70.0 / 3.0;
        let casts_per_cycle = if early_refresh_loss <= late_refresh_loss {
            // We're on an early refresh cycle
            early_refresh_casts as usize
        } else {
            // We're on a late refresh cycle
            late_refresh_casts as usize
        };
        let cycle_length = casts_per_cycle as f64 * gcd + 2.5;

        // edosis and eukrasis usage
        // TODO take into account the buffs when deciding whether to refresh ?
        let mut edosis_iter = TimelineIterator::from_timeline(self, 1.0, cycle_length);
        while let Some(offset) = edosis_iter.next() {
            if let Some((next_downtime, _)) = self.downtime.next_start(offset) {
                let edosis_ticks_before_downtime = (next_downtime.begin - offset) / 3.0;
                if edosis_ticks_before_downtime * 70.0 < 330.0 {
                    // Reserve time to cast eukrasis and skip this cast
                    // 0.95 instead 1.0 to allow easy removing of actions that happen in downtime
                    //   later. if it was 1.0, edosis cast would be = downtime.end and would be
                    //   removed.
                    edosis_iter.current = next_downtime.end - 0.95;
                    continue;
                }
            }
            sge_timeline.push((offset, Some(SGEAction::Eukrasis), Vec::new()));
            let buffs = self.buffs.spans(offset).into_iter().map(unwrap_tsearch).collect();
            sge_timeline.push((offset+1.0, Some(SGEAction::Edosis), buffs));
        }

        // Now we've added all casts of e/edosis, add all the GCD casts that we'll fill in later
        // with phlegma and dosis
        let mut casts: Vec<(f64, Option<SGEAction>, Vec<Buff>)> = Vec::new();
        for (offset, _, _) in sge_timeline.iter().filter(|(_, action, _)| *action == Some(SGEAction::Edosis)) {
            // edosis cast puts a recast of 1.5s on the GCD
            let offset = offset + 1.5;
            let mut cast_iter = TimelineIterator::from_timeline(self, offset, gcd);
            if let Some(downtime) = self.downtime.next_start(offset) {
                // Make sure we stop casting before the next downtime
                cast_iter.end = downtime.0.begin;
            }
            for offset in cast_iter.take(casts_per_cycle) {
                let buffs = self.buffs.spans(offset).into_iter().map(unwrap_tsearch).collect();
                casts.push((offset, None, buffs));
            }
        }

        // Add those casts
        sge_timeline.extend(casts.into_iter());
        // Add the prepull dosis
        sge_timeline.push((-gcd15, Some(SGEAction::Dosis), Vec::new()));
        // Sort the timeline by event
        sge_timeline.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());

        // opener: prepull dosis -> e -> edosis -> dosis -> dosis -> phlegma -> phlegma
        // prepull dosis, e and edosis are already in
        let mut unfilled_actions = sge_timeline.iter_mut().filter(|(_, action, _)| action.is_none());
        // First two unfilled slot are dosis
        unfilled_actions.next().unwrap().1 = Some(SGEAction::Dosis);
        unfilled_actions.next().unwrap().1 = Some(SGEAction::Dosis);
        // Next two unfilled slot are phlegma
        // Also, first of those 2 is the start of the phlegma clock
        let first_phlegma = unfilled_actions.next().unwrap();
        first_phlegma.1 = Some(SGEAction::Phlegma);
        let mut phlegma_clock = first_phlegma.0;
        // TODO implement phlegma cap planning
        unfilled_actions.next().unwrap().1 = Some(SGEAction::Phlegma);
        std::mem::drop(unfilled_actions);

        // Find when the next GCD where there is a phlegma charge
        // And the first GCD where phlegma caps
        // then in that GCD range, find the GCD with the most buffs
        // Put phlegma here and arrange the clock accordingly
        // If it can't be put here (downtime or something)
        // Put it in the next available GCD
        while phlegma_clock < self.end {
            let phlegma_clock_stacked = phlegma_clock + 45.0;
            let phlegma_clock_cap = phlegma_clock + 45.0*2.0;
            // TODO optimize by not processing the array many times
            let best_candidate = sge_timeline.iter_mut()
                .skip_while(|(instant, _, _)| *instant < phlegma_clock_stacked)
                .take_while(|(instant, _, _)| *instant <= phlegma_clock_cap)
                .max_by(|(_, _, a), (_, _, b)| a.len().cmp(&b.len()));
            if let Some(candidate) = best_candidate {
                candidate.1 = Some(SGEAction::Phlegma);
                phlegma_clock += 45.0;
            } else if let Some(candidate) = sge_timeline.iter_mut().find(|(instant, action, _)| *instant >= phlegma_clock_cap && action.is_none()) {
                candidate.1 = Some(SGEAction::Phlegma);
                phlegma_clock = candidate.0;
            } else {
                // No more candidates for phlegma
                break;
            }
        }

        sge_timeline.iter_mut()
            .filter(|(_, action, _)| action.is_none())
            .for_each(|(_, action, _)| *action = Some(SGEAction::Dosis));

        sge_timeline.iter_mut()
            .filter(|(_, action, _)| *action == Some(SGEAction::Dosis))
            .for_each(|(instant, _, _)| *instant += gcd15);

        sge_timeline.retain(|(instant, _, _)| self.downtime.spans(*instant).is_empty());

        let timeline: Vec<_>= sge_timeline.into_iter()
            .map(|(instant, action, buffs)| (instant, action.unwrap(), buffs.simplify()))
            .collect();

        self.timeline_cache.lock().unwrap().insert((gcd*100.0) as usize, timeline.clone());

        // TODO
        // This needs testing.

        timeline
    }
}

fn unwrap_tsearch<T: Clone>((_, data): &(Timespan, T)) -> T {
    data.clone()
}

#[derive(PartialEq, Eq, Clone)]
pub enum SGEAction {
    Dosis,
    Eukrasis,
    Edosis,
    Phlegma,
}

#[derive(Clone)]
pub enum Buff {
    Damage(f64),
    DirectHit(f64),
    Critical(f64),
    Mind(u32),
}

trait BuffExt {
    fn simplify(self) -> SimplifiedBuff;
}

impl BuffExt for Vec<Buff> {
    fn simplify(self) -> SimplifiedBuff {
        let mut buff_damage = 0.0;
        let mut buff_direct_hit = 0.0;
        let mut buff_critical = 0.0;
        let mut buff_mind = 0;

        self.into_iter()
            .for_each(|buff| {
                match buff {
                    Buff::Damage(damage) => buff_damage += damage,
                    Buff::DirectHit(direct_hit) => buff_direct_hit += direct_hit,
                    Buff::Critical(critical) => buff_critical += critical,
                    Buff::Mind(mind) => buff_mind += mind,
                }
            });

        SimplifiedBuff {
            damage: buff_damage,
            direct_hit: buff_direct_hit,
            critical: buff_critical,
            mind: buff_mind
        }
    }
}

trait StatExt {
    fn edosis_damage_per_tick(&self) -> f64;
    fn dosis_damage(&self) -> f64;
    fn phlegma_damage(&self) -> f64;
}

use crate::utils::Scalable;
impl<T: StatRepo> StatExt for T {
    fn edosis_damage_per_tick(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let sps = self.sps_multiplier();
        let damage = 70.scale(adj_wd).scale(map).scale(det).scale(sps) * 130 / 100 + 1;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar()
    }

    fn dosis_damage(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 330.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar()
    }

    fn phlegma_damage(&self) -> f64 {
        let adj_wd = self.adjusted_weapon_damage();
        let map = self.magic_attack_power();
        let det = self.det_multiplier();
        let damage = 510.scale(map).scale(det).scale(adj_wd) * 130 / 100;
        damage as f64 * self.crit_scalar().scalar() * self.dh_scalar().scalar()
    }
}

// WORKS ONLY IF THE BUFFS HAVE BEEN SIMPLIFIED
// which they should be
pub fn timeline_dps(tl: &Timeline, gearset: &Gearset) -> f64 {
    let mut stats = gearset.stats();
    // Take into account the composition MND buff
    stats.mind += (stats.mind as f64 * tl.comp_mind)  as u32;
    let timeline = tl.sge_timeline(stats.spell_speed);

    let mut edosis_ticks = Vec::new();
    let mut edosis_tick = 1.5;
    while edosis_tick < tl.end {
        edosis_ticks.push(edosis_tick);
        edosis_tick += 3.0;
    }
    let mut damage = 0.0;
    edosis_ticks.retain(|tick| tl.downtime.spans(*tick).is_empty());
    for tick in edosis_ticks {
        if let Some((_, _, buffs)) = timeline.iter()
            .filter(|(_, action, _)| *action == SGEAction::Edosis)
                .filter(|(instant, _, _)| *instant >= tick - 30.0 && *instant <= tick)
                .last()
                {
                    let mut stats = stats.clone();
                    stats.critical += (buffs.critical * stats.critical as f64) as u32;
                    stats.direct_hit += (buffs.direct_hit * stats.direct_hit as f64) as u32;
                    stats.mind += buffs.mind;
                    damage += (stats.edosis_damage_per_tick() * (1.0 + buffs.damage)).trunc();
                }
    }

    for (_, action, buffs) in timeline {
        let mut stats = stats.clone();
        stats.critical += (buffs.critical * stats.critical as f64) as u32;
        stats.direct_hit += (buffs.direct_hit * stats.direct_hit as f64) as u32;
        stats.mind += buffs.mind;
        damage += match action {
            SGEAction::Dosis => (stats.dosis_damage() * (1.0 + buffs.damage)).trunc(),
            SGEAction::Phlegma => (stats.phlegma_damage() * (1.0 + buffs.damage)).trunc(),
            _ => {0.0}
        }
    }

    damage / tl.end
}

impl crate::solver::Evaluator for Timeline {
    fn dps(&self, gearset: &Gearset) -> f64 {
        timeline_dps(self, gearset)
    }
}

#[derive(Clone)]
pub struct SimplifiedBuff {
    damage: f64,
    mind: u32,
    direct_hit: f64,
    critical: f64,
}

// TODO
// Replace this with ranges
#[derive(Clone)]
pub struct Timespan {
    pub begin: f64,
    pub end: f64
}

impl Timespan {
    pub const fn new(begin: f64, end: f64) -> Self {
        Self {
            begin,
            end
        }
    }

    pub fn offset(self, offset: f64) -> Self {
        Self {
            begin: self.begin + offset,
            end: self.end + offset,
        }
    }
}
