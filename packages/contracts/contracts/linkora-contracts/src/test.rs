#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String,
};

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token_id.address()).mint(admin, &10_000);
    token_id.address()
}

fn setup_contract(env: &Env) -> (LinkoraContractClient, Address, Address) {
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let treasury = Address::generate(env);
    client.initialize(&admin, &treasury, &0);
    (client, admin, treasury)
}

#[test]
fn test_set_and_get_profile() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_profile(&user, &String::from_str(&env, "alice"), &token);
    let profile = client.get_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "alice"));
}

#[test]
fn test_tip_fee_split() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let author = Address::generate(&env);
    let tipper = Address::generate(&env);

    // Initialize with 2.5% fee (250 bps)
    client.initialize(&admin, &treasury, &250);

    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Fee test post"));

    // Tip 1000 units
    client.tip(&tipper, &post_id, &token, &1000);

    // Verify balances
    // Fee = 1000 * 250 / 10000 = 25
    // Author gets 1000 - 25 = 975
    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 25);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 975);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.tip_total, 1000);
}

#[test]
fn test_profile_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_profile(&user1, &String::from_str(&env, "alice"), &token);
    assert_eq!(client.get_profile_count(), 1);

    // Update profile should not increment count
    client.set_profile(&user1, &String::from_str(&env, "alice_new"), &token);
    assert_eq!(client.get_profile_count(), 1);

    client.set_profile(&user2, &String::from_str(&env, "bob"), &token);
    assert_eq!(client.get_profile_count(), 2);
}

#[test]
fn test_post_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    client.create_post(&author, &String::from_str(&env, "Post 1"));
    client.create_post(&author, &String::from_str(&env, "Post 2"));

    assert_eq!(client.get_post_count(), 2);
}

#[test]
fn test_follow_and_unfollow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.follow(&alice, &bob);
    assert_eq!(client.get_following(&alice).len(), 1);
    assert_eq!(client.get_followers(&bob).len(), 1);

    client.unfollow(&alice, &bob);
    assert_eq!(client.get_following(&alice).len(), 0);
    assert_eq!(client.get_followers(&bob).len(), 0);
}

#[test]
fn test_block_prevents_follow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let blocker = Address::generate(&env);
    let blocked = Address::generate(&env);
    client.block_user(&blocker, &blocked);
    assert!(client.is_blocked(&blocker, &blocked));
}

#[test]
#[should_panic(expected = "blocked")]
fn test_blocked_follow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Bob blocks Alice
    client.block_user(&bob, &alice);

    // Alice tries to follow Bob
    client.follow(&alice, &bob);
}

#[test]
fn test_like_post() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    let user = Address::generate(&env);
    let post_id = client.create_post(&author, &String::from_str(&env, "Like test"));

    client.like_post(&user, &post_id);
    assert_eq!(client.get_like_count(&post_id), 1);
    assert!(client.has_liked(&user, &post_id));

    // Duplicate like should not increment
    client.like_post(&user, &post_id);
    assert_eq!(client.get_like_count(&post_id), 1);
}

#[test]
fn test_pool_authorization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let pool_admin1 = Address::generate(&env);
    let pool_admin2 = Address::generate(&env);
    let other_user = Address::generate(&env);
    let token = setup_token(&env, &pool_admin1);

    // Give other_user some tokens to deposit
    StellarAssetClient::new(&env, &token).mint(&other_user, &1000);

    let pool_id = symbol_short!("pool1");
    // Create pool with 2-of-2 threshold
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &2,
    );

    // Deposit works for anyone with tokens
    client.pool_deposit(&other_user, &pool_id, &token, &100);

    // Withdrawal by both admins works
    client.pool_withdraw(
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &pool_id,
        &50,
        &other_user,
    );
    assert_eq!(client.get_pool(&pool_id).unwrap().balance, 50);
}

#[test]
#[should_panic(expected = "insufficient signers")]
fn test_pool_withdraw_insufficient_signers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let pool_admin1 = Address::generate(&env);
    let pool_admin2 = Address::generate(&env);
    let other_user = Address::generate(&env);
    let token = setup_token(&env, &pool_admin1);
    StellarAssetClient::new(&env, &token).mint(&other_user, &1000);

    let pool_id = symbol_short!("pool1");
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &2,
    );
    client.pool_deposit(&other_user, &pool_id, &token, &100);

    // Only 1 signer when 2 required
    client.pool_withdraw(&vec![&env, pool_admin1.clone()], &pool_id, &50, &other_user);
}

#[test]
fn test_sequential_posts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);

    // Set first timestamp
    let ts1 = 1000;
    env.ledger().set_timestamp(ts1);

    // Create first post
    let post_id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    assert_eq!(post_id1, 1);

    let post1 = client.get_post(&post_id1).unwrap();
    assert_eq!(post1.timestamp, ts1);
    assert_eq!(post1.id, 1);

    // Advance timestamp
    let ts2 = 2000;
    env.ledger().set_timestamp(ts2);

    // Create second post
    let post_id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    assert_eq!(post_id2, 2);

    let post2 = client.get_post(&post_id2).unwrap();
    assert_eq!(post2.timestamp, ts2);
    assert_eq!(post2.id, 2);
}

#[test]
#[should_panic(expected = "post does not exist: 999")]
fn test_delete_post_non_existent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    client.delete_post(&author, &999);
}

#[test]
fn test_profile_set_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    let username = String::from_str(&env, "alice");

    client.set_profile(&user, &username, &token);

    // Pull all events and find ProfileSetEvent
    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| {
            e.0 == client.address && // emitted by our contract
        e.1 == symbol_short!("ProfileSetEvent").into_val(&env) // event name as topic
        })
        .expect("ProfileSetEvent not found");

    // Decode topics + data: ProfileSetEvent { user, username }
    let (user_topic, username_data): (Address, String) = env.from_val(&event.2);
    assert_eq!(user_topic, user);
    assert_eq!(username_data, username);
}

#[test]
fn test_post_created_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    let content = String::from_str(&env, "gm linkora");

    let post_id = client.create_post(&author, &content);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("PostCreatedEvent").into_val(&env))
        .expect("PostCreatedEvent not found");

    // Decode topics: PostCreatedEvent { id, author }
    let (id_topic, author_topic): (u64, Address) = env.from_val(&event.2);
    assert_eq!(id_topic, post_id);
    assert_eq!(author_topic, author);
    assert_eq!(post_id, 1); // first post
}

#[test]
fn test_profile_update_emits_new_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    let username1 = String::from_str(&env, "alice");
    let username2 = String::from_str(&env, "alice_new");

    client.set_profile(&user, &username1, &token);

    // Clear events to isolate the second call
    env.events().all(); // consume

    client.set_profile(&user, &username2, &token);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("ProfileSetEvent").into_val(&env))
        .expect("ProfileSetEvent not found after update");

    let (user_topic, username_data): (Address, String) = env.from_val(&event.2);
    assert_eq!(user_topic, user);
    assert_eq!(username_data, username2);
}

#[test]
fn test_follow_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.follow(&alice, &bob);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("FollowEvent").into_val(&env))
        .expect("FollowEvent not found");

    // FollowEvent { #[topic] follower, #[topic] followee }
    let (follower, followee): (Address, Address) = env.from_val(&event.2);
    assert_eq!(follower, alice);
    assert_eq!(followee, bob);
}

#[test]
fn test_no_duplicate_follow_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.follow(&alice, &bob);
    let count_after_first = env
        .events()
        .all()
        .iter()
        .filter(|e| e.1 == symbol_short!("FollowEvent").into_val(&env))
        .count();

    // Follow again - should be no-op
    client.follow(&alice, &bob);
    let count_after_second = env
        .events()
        .all()
        .iter()
        .filter(|e| e.1 == symbol_short!("FollowEvent").into_val(&env))
        .count();

    assert_eq!(count_after_first, 1);
    assert_eq!(
        count_after_second, 1,
        "Duplicate FollowEvent emitted on repeat follow"
    );
}

#[test]
fn test_unfollow_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.follow(&alice, &bob);
    env.events().all(); // clear previous events

    client.unfollow(&alice, &bob);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("UnfollowEvent").into_val(&env))
        .expect("UnfollowEvent not found");

    // UnfollowEvent { #[topic] follower, #[topic] followee }
    let (follower, followee): (Address, Address) = env.from_val(&event.2);
    assert_eq!(follower, alice);
    assert_eq!(followee, bob);
}

#[test]
fn test_no_unfollow_event_on_noop() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Never followed bob, so unfollow should be no-op
    client.unfollow(&alice, &bob);

    let unfollow_events = env
        .events()
        .all()
        .iter()
        .filter(|e| e.1 == symbol_short!("UnfollowEvent").into_val(&env))
        .count();

    assert_eq!(
        unfollow_events, 0,
        "UnfollowEvent emitted when no relationship existed"
    );
}

#[test]
fn test_tip_event_emitted_with_gross_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, treasury) = setup_contract(&env);

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "tip me"));

    let gross_amount: i128 = 1000;
    client.tip(&tipper, &post_id, &token, &gross_amount);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("TipEvent").into_val(&env))
        .expect("TipEvent not found");

    // TipEvent { #[topic] tipper, #[topic] post_id, amount }
    let (tipper_topic, post_id_topic, amount_data): (Address, u64, i128) = env.from_val(&event.2);
    assert_eq!(tipper_topic, tipper);
    assert_eq!(post_id_topic, post_id);
    assert_eq!(
        amount_data, gross_amount,
        "TipEvent amount should be gross, not net"
    );
}

#[test]
fn test_tip_event_amount_equals_gross_with_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let author = Address::generate(&env);
    let tipper = Address::generate(&env);

    // 2.5% fee = 250 bps
    client.initialize(&admin, &treasury, &250);

    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "fee test"));
    let gross_amount: i128 = 1000;

    client.tip(&tipper, &post_id, &token, &gross_amount);

    // Verify actual token splits happened correctly
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&treasury), 25); // 2.5% of 1000
    assert_eq!(token_client.balance(&author), 975); // 1000 - 25

    // But event should still show gross amount
    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("TipEvent").into_val(&env))
        .expect("TipEvent not found");

    let (_, _, amount_data): (Address, u64, i128) = env.from_val(&event.2);
    assert_eq!(
        amount_data, 1000,
        "Event amount must be gross tip when fee_bps > 0"
    );
}

#[test]
fn test_tip_event_amount_with_zero_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env); // fee_bps = 0 from setup_contract

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "no fee"));

    let gross_amount: i128 = 500;
    client.tip(&tipper, &post_id, &token, &gross_amount);

    // Verify full amount went to author
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&author), 500);

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| e.0 == client.address && e.1 == symbol_short!("TipEvent").into_val(&env))
        .expect("TipEvent not found");

    let (_, _, amount_data): (Address, u64, i128) = env.from_val(&event.2);
    assert_eq!(
        amount_data, 500,
        "Event amount should equal full transfer when fee_bps = 0"
    );
}

#[test]
#[should_panic(expected = "duplicate admin")]
fn test_create_pool_duplicate_admins_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let token = Address::generate(&env);
    let pool_id = symbol_short!("duppool");

    // Try to create pool with [alice, alice] and threshold = 2
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, alice.clone(), alice.clone()],
        &2,
    );
}

#[test]
fn test_create_pool_unique_admins_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let token = Address::generate(&env);
    let pool_id = symbol_short!("okpool");

    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, alice.clone(), bob.clone()],
        &2,
    );

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.admins.len(), 2);
    assert_eq!(pool.threshold, 2);
    assert!(pool.admins.contains(&alice));
    assert!(pool.admins.contains(&bob));
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_create_pool_threshold_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let token = Address::generate(&env);
    let pool_id = symbol_short!("zeroth");

    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, alice],
        &0, // threshold can't be 0
    );
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_create_pool_threshold_exceeds_admins() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let token = Address::generate(&env);
    let pool_id = symbol_short!("badth");

    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, alice],
        &2, // threshold > admins.len()
    );
}
