# Soroban "Tap-to-Earn" Game: Code & Deployment Guide

This repository contains a simple "Tap-to-Earn" smart contract project built on the Soroban (Stellar) blockchain.

The project consists of two main contracts:
1.  **Token Contract**: A standard Soroban asset contract (SAC) that serves as the "TapCoin" (TAP) reward.
2.  **Game Contract**: The main `tap_game` contract that holds the game logic, manages cooldowns, and distributes the `TAP` tokens.

---

## 1. The `tap_game` Smart Contract Explained

This is the full code for the main game logic, found in `src/lib.rs`.

### The Code (`src/lib.rs`)

```rust
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    token, // Used for interacting with the token contract
    Address, // Represents a user's wallet or contract
    Env,     // The Soroban environment
    Map,     // To store user data
};

// --- Define Storage Keys ---
// Using keys helps keep storage organized and avoid collisions.
#[contracttype]
#[derive(Clone, Copy)]
enum DataKey {
    Token,     // Key to store the address of our "TapCoin"
    Admin,     // Key to store the admin's address
    TapAmount, // Key to store how many tokens per tap
    Cooldown,  // Key to store the cooldown time in seconds
    LastTap,   // Key to store the map of (User Address -> Last Tap Timestamp)
}

// --- The Main Contract ---
#[contract]
pub struct TapGameContract;

// --- Contract Implementation ---
#[contractimpl]
impl TapGameContract {
    
    /// Initializes the game contract.
    /// This can only be run once.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        tap_amount: u128,
        cooldown_sec: u64,
    ) {
        // Check if already initialized by seeing if 'Admin' key exists
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        // Store all the initial values in instance (global) storage
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TapAmount, &tap_amount);
        env.storage().instance().set(&DataKey::Cooldown, &cooldown_sec);
        
        // Create an empty persistent map to store user tap times
        env.storage().persistent().set(&DataKey::LastTap, &Map::<Address, u64>::new(&env));
    }

    /// The main "tap" function.
    /// Anyone can call this, but they are limited by the cooldown.
    pub fn tap(env: Env, user: Address) {
        // --- 1. Authentication ---
        // This is critical! It ensures the 'user' signed this transaction.
        // This prevents one person from "tapping" for someone else.
        user.require_auth();

        // --- 2. Load Cooldown Rules ---
        let cooldown_time: u64 = env.storage().instance().get(&DataKey::Cooldown).unwrap();
        
        // --- 3. Check Cooldown ---
        // Get the map of all user tap times from persistent storage
        let mut last_tap_map: Map<Address, u64> = env.storage().persistent().get(&DataKey::LastTap).unwrap();

        // Get the last tap time for this specific user (defaults to 0 if never tapped)
        let last_tap_time: u64 = last_tap_map.get(user.clone()).unwrap_or(0);

        // Get the current blockchain ledger time (in seconds)
        let current_time = env.ledger().timestamp();

        // Check if the (last tap + cooldown) is in the future
        if last_tap_time + cooldown_time > current_time {
            panic!("Cooldown active. Please wait.");
        }

        // --- 4. Update User's State ---
        // If the check passes, update their last tap time to now
        last_tap_map.set(user.clone(), current_time);
        env.storage().persistent().set(&DataKey::LastTap, &last_tap_map);

        // --- 5. Send the Reward ---
        // Load the reward amount and the token's contract address
        let tap_amount: u128 = env.storage().instance().get(&DataKey::TapAmount).unwrap();
        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        
        // Create a client to call the token contract
        let token_client = token::Client::new(&env, &token_id);

        // Call the token's "transfer" function.
        token_client.transfer(
            &env.current_contract_address(), // From: this game contract
            &user,                           // To: the user who tapped
            &(tap_amount as i128)            // Amount (must be i128 for token)
        );
    }
}
