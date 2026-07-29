use soroban_sdk::{Address, Env, String};
use scoutchain_progress::{ProgressContract, ProgressContractClient};
use scoutchain_registration::{RegistrationContract, RegistrationContractClient};
use scoutchain_scout_access::{ScoutAccessContract, ScoutAccessContractClient};
use scoutchain_verification::{VerificationContract, VerificationContractClient};

pub struct Harness {
    pub env: Env,
    pub admin: Address,
    pub players: Vec<Address>,
    pub scouts: Vec<Address>,
    pub validators: Vec<Address>,
    pub progress: ProgressContractClient<'static>,
    pub registration: RegistrationContractClient<'static>,
    pub scout_access: ScoutAccessContractClient<'static>,
    pub verification: VerificationContractClient<'static>,
}

impl Harness {
    pub fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let progress_id = env.register(ProgressContract, ());
        let progress = ProgressContractClient::new(&env, &progress_id);
        progress.initialize(&admin);

        let reg_id = env.register(RegistrationContract, ());
        let registration = RegistrationContractClient::new(&env, &reg_id);
        registration.initialize(&admin);

        let ver_id = env.register(VerificationContract, ());
        let verification = VerificationContractClient::new(&env, &ver_id);
        verification.initialize(&admin);

        let sa_id = env.register(ScoutAccessContract, ());
        let scout_access = ScoutAccessContractClient::new(&env, &sa_id);

        let players = vec![
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];
        let scouts = vec![
            Address::generate(&env),
            Address::generate(&env),
        ];
        let validators = vec![
            Address::generate(&env),
            Address::generate(&env),
        ];

        for p in &players {
            let _ = registration.register_player(
                p,
                &scoutchain_registration::PlayerVitals {
                    age: 20,
                    position: String::from_str(&env, "Forward"),
                    region: String::from_str(&env, "West Africa"),
                    nationality: String::from_str(&env, "Ghana"),
                },
                &vec![String::from_str(&env, "QmCID1")],
            );
        }

        for s in &scouts {
            let _ = registration.register_scout(s, &String::from_str(&env, "West Africa"));
        }

        for v in &validators {
            let _ = verification.register_validator(v, &String::from_str(&env, "UEFA B License"));
        }

        Self {
            env,
            admin,
            players,
            scouts,
            validators,
            progress,
            registration,
            scout_access,
            verification,
        }
    }
}
