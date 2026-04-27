#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.sequence_number = 100_000;
        li.min_persistent_entry_ttl = 500;
        li.max_entry_ttl = 600_000;
    });
    env
}

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token_id.address()).mint(admin, &10_000);
    token_id.address()
}

fn setup_contract(env: &Env) -> (LinkoraContractClient<'_>, Address) {
    let contract_id = env.register(LinkoraContract, ());
    (LinkoraContractClient::new(env, &contract_id), contract_id)
}

#[test]
fn test_set_and_get_profile() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_profile(&user, &String::from_str(&env, "alice"), &token);
    let profile = client.get_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "alice"));
}

#[test]
fn test_post_and_tip_fee_split() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin);
    client.set_treasury(&treasury);
    client.set_fee(&500);

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Test post"));

    client.tip(&tipper, &post_id, &token, &1000);
    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 50);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 950);
}

#[test]
fn test_tip_zero_fee_sends_all_to_author() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin);
    client.set_treasury(&treasury);
    client.set_fee(&0);

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Zero fee"));

    client.tip(&tipper, &post_id, &token, &1000);
    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 0);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 1000);
}

#[test]
fn test_set_fee_and_treasury_update() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin);
    client.set_treasury(&treasury);
    client.set_fee(&100);
    assert_eq!(client.get_fee_bps(), 100);
    assert_eq!(client.get_treasury().unwrap(), treasury);

    let new_treasury = Address::generate(&env);
    client.set_treasury(&new_treasury);
    client.set_fee(&250);
    assert_eq!(client.get_fee_bps(), 250);
    assert_eq!(client.get_treasury().unwrap(), new_treasury);
}

#[test]
#[should_panic(expected = "fee_bps cannot exceed 10000")]
fn test_invalid_fee_above_max() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.set_fee(&10_001);
}

#[test]
#[should_panic(expected = "post not found: 999")]
fn test_tip_non_existent_post_message() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    client.tip(&tipper, &999, &token, &100);
}

#[test]
#[should_panic(expected = "post does not exist: 999")]
fn test_delete_post_non_existent_message() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let author = Address::generate(&env);
    client.delete_post(&author, &999);
}

#[test]
#[should_panic(expected = "pool not found: Symbol(missing)")]
fn test_pool_withdraw_non_existent_message() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let recipient = Address::generate(&env);
    client.pool_withdraw(&recipient, &symbol_short!("missing"), &1);
}

#[test]
fn test_follow_and_unfollow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.follow(&alice, &bob);
    client.unfollow(&alice, &bob);
    assert_eq!(client.get_following(&alice).len(), 0);
}

#[test]
fn test_block_prevents_follow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let blocker = Address::generate(&env);
    let blocked = Address::generate(&env);
    client.block_user(&blocker, &blocked);
    assert!(client.is_blocked(&blocker, &blocked));
}

#[test]
fn test_pool_deposit_and_withdraw() {
    let env = setup_env();
    env.mock_all_auths();
    let (client, _) = setup_contract(&env);

    let admin = Address::generate(&env);
    let token = setup_token(&env, &admin);
    let pool_id = symbol_short!("pool1");
    let mut admins = Vec::new(&env);
    admins.push_back(admin.clone());

    client.create_pool(&pool_id, &token, &admins);
    client.pool_deposit(&admin, &pool_id, &token, &500);
    client.pool_withdraw(&admin, &pool_id, &200);

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.balance, 300);
}
