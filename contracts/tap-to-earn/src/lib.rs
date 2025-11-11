#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Map};

#[contracttype]
#[derive(Clone, Copy)]
enum DataKey {
    Token,
    Admin,
    TapAmount,
    Cooldown,
    LastTap,
}

#[contract]
pub struct TapGameContract;

#[contractimpl]
impl TapGameContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        tap_amount: u128,
        cooldown_sec: u64,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::TapAmount, &tap_amount);
        env.storage()
            .instance()
            .set(&DataKey::Cooldown, &cooldown_sec);

        env.storage()
            .persistent()
            .set(&DataKey::LastTap, &Map::<Address, u64>::new(&env));
    }

    pub fn tap(env: Env, user: Address) {
        user.require_auth();

        let cooldown_time: u64 = env.storage().instance().get(&DataKey::Cooldown).unwrap();

        let mut last_tap_map: Map<Address, u64> =
            env.storage().persistent().get(&DataKey::LastTap).unwrap();

        let last_tap_time: u64 = last_tap_map.get(user.clone()).unwrap_or(0);

        let current_time = env.ledger().timestamp();

        if last_tap_time + cooldown_time > current_time {
            panic!("Cooldown active. Please wait.");
        }

        last_tap_map.set(user.clone(), current_time);
        env.storage()
            .persistent()
            .set(&DataKey::LastTap, &last_tap_map);

        let tap_amount: u128 = env.storage().instance().get(&DataKey::TapAmount).unwrap();
        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();

        let token_client = token::Client::new(&env, &token_id);

        token_client.transfer(
            &env.current_contract_address(),
            &user,
            &(tap_amount as i128),
        );
    }
}
