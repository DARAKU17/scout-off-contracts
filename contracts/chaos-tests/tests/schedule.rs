use soroban_sdk::{Address, String};
use scoutchain_chaos_tests::fixtures::Harness;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    ApproveMilestone { player_idx: usize, validator_idx: usize },
    RegisterPlayer,
    RegisterScout,
    ContactPlayer { scout_idx: usize, player_idx: usize },
    LogTrialOffer { scout_idx: usize, player_idx: usize },
}

pub struct ScheduleGenerator {
    seed: u64,
}

impl ScheduleGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate(&mut self, max_ops: u32) -> Vec<Operation> {
        let mut ops = Vec::new();
        for _ in 0..max_ops {
            self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let op_type = (self.seed % 5) as usize;
            match op_type {
                0 => ops.push(Operation::ApproveMilestone {
                    player_idx: (self.seed % 3) as usize,
                    validator_idx: (self.seed % 2) as usize,
                }),
                1 => ops.push(Operation::RegisterPlayer),
                2 => ops.push(Operation::RegisterScout),
                3 => ops.push(Operation::ContactPlayer {
                    scout_idx: (self.seed % 2) as usize,
                    player_idx: (self.seed % 3) as usize,
                }),
                4 => ops.push(Operation::LogTrialOffer {
                    scout_idx: (self.seed % 2) as usize,
                    player_idx: (self.seed % 3) as usize,
                }),
                _ => unreachable!(),
            }
        }
        ops
    }
}

impl Harness {
    pub fn apply(&mut self, op: &Operation) -> Result<(), String> {
        match op {
            Operation::ApproveMilestone { player_idx, validator_idx } => {
                let player = &self.players[*player_idx];
                let validator = &self.validators[*validator_idx];
                let result = self.verification.try_approve_milestone(
                    validator,
                    player,
                    &String::from_str(&self.env, "chaos-test"),
                    &String::from_str(&self.env, "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB"),
                );
                result.map_err(|e| format!("approve_milestone failed: {:?}", e))
            }
            Operation::ContactPlayer { scout_idx, player_idx } => {
                let scout = &self.scouts[*scout_idx];
                let player = &self.players[*player_idx];
                let result = self.scout_access.try_pay_to_contact(scout, *player_idx as u64);
                result.map_err(|e| format!("pay_to_contact failed: {:?}", e))
            }
            Operation::LogTrialOffer { scout_idx, player_idx } => {
                let scout = &self.scouts[*scout_idx];
                let player = &self.players[*player_idx];
                let result = self.scout_access.try_log_trial_offer(
                    scout,
                    player,
                    &String::from_str(&self.env, "chaos-trial"),
                );
                result.map_err(|e| format!("log_trial_offer failed: {:?}", e))
            }
            Operation::RegisterPlayer => {
                let wallet = Address::generate(&self.env);
                let result = self.registration.try_register_player(
                    &wallet,
                    &scoutchain_registration::PlayerVitals {
                        age: 20,
                        position: String::from_str(&self.env, "Midfielder"),
                        region: String::from_str(&self.env, "East Africa"),
                        nationality: String::from_str(&self.env, "Kenya"),
                    },
                    &vec![String::from_str(&self.env, "QmCID2")],
                );
                if result.is_ok() {
                    self.players.push(wallet);
                }
                Ok(())
            }
            Operation::RegisterScout => {
                let wallet = Address::generate(&self.env);
                let result = self.registration.try_register_scout(
                    &wallet,
                    &String::from_str(&self.env, "North Africa"),
                );
                if result.is_ok() {
                    self.scouts.push(wallet);
                }
                Ok(())
            }
        }
    }
}
