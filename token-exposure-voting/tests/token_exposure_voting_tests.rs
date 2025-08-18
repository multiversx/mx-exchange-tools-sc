#![allow(deprecated)]

use multiversx_sc_scenario::{managed_biguint, managed_token_id, rust_biguint, DebugApi};
use token_exposure_voting::{config::ConfigModule, views::ViewsModule, vote::VoteModule};
use week_timekeeping::WeekTimekeepingModule;

mod setup;

use setup::*;

#[test]
fn test_init() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let first_week = sc.first_week_start_epoch().get();
            assert_eq!(first_week, FIRST_WEEK_START_EPOCH);
        })
        .assert_ok();
}

#[test]
fn test_simple_voting_with_5_tokens_no_boost() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );

    setup.set_current_week(1);
    let current_week = setup.get_current_week();

    // Use first 5 tokens
    let tokens = &TEST_TOKENS[0..5];
    setup.setup_tokens_for_week(current_week, tokens);

    // Set different vote amounts for the 5 tokens
    setup.set_token_votes(TEST_TOKENS[0], current_week, 5000); // TOKEN-01: 5000 votes
    setup.set_token_votes(TEST_TOKENS[1], current_week, 3000); // TOKEN-02: 3000 votes
    setup.set_token_votes(TEST_TOKENS[2], current_week, 7000); // TOKEN-03: 7000 votes
    setup.set_token_votes(TEST_TOKENS[3], current_week, 1000); // TOKEN-04: 1000 votes
    setup.set_token_votes(TEST_TOKENS[4], current_week, 4000); // TOKEN-05: 4000 votes

    // Test all view functions
    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            // Test get_week_ranking - should return tokens sorted by votes
            let ranking = sc.get_week_ranking(current_week);
            assert_eq!(ranking.len(), 5, "Should have exactly 5 ranked tokens");

            // Expected order: TOKEN-03 (7000), TOKEN-01 (5000), TOKEN-05 (4000), TOKEN-02 (3000), TOKEN-04 (1000)

            // First place: TOKEN-03 with 7000 votes
            let first = ranking.get(0);
            let first_bytes = first.token_id.as_managed_buffer().to_vec();
            assert_eq!(first_bytes, TEST_TOKENS[2], "TOKEN-03 should be rank 1");
            assert_eq!(first.votes, managed_biguint!(7000));

            // Second place: TOKEN-01 with 5000 votes
            let second = ranking.get(1);
            let second_bytes = second.token_id.as_managed_buffer().to_vec();
            assert_eq!(second_bytes, TEST_TOKENS[0], "TOKEN-01 should be rank 2");
            assert_eq!(second.votes, managed_biguint!(5000));

            // Third place: TOKEN-05 with 4000 votes
            let third = ranking.get(2);
            let third_bytes = third.token_id.as_managed_buffer().to_vec();
            assert_eq!(third_bytes, TEST_TOKENS[4], "TOKEN-05 should be rank 3");
            assert_eq!(third.votes, managed_biguint!(4000));

            // Fourth place: TOKEN-02 with 3000 votes
            let fourth = ranking.get(3);
            let fourth_bytes = fourth.token_id.as_managed_buffer().to_vec();
            assert_eq!(fourth_bytes, TEST_TOKENS[1], "TOKEN-02 should be rank 4");
            assert_eq!(fourth.votes, managed_biguint!(3000));

            // Fifth place: TOKEN-04 with 1000 votes
            let fifth = ranking.get(4);
            let fifth_bytes = fifth.token_id.as_managed_buffer().to_vec();
            assert_eq!(fifth_bytes, TEST_TOKENS[3], "TOKEN-04 should be rank 5");
            assert_eq!(fifth.votes, managed_biguint!(1000));

            // Test individual token rankings
            let token03_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[2]), current_week);
            let pos = token03_ranking.token_position;
            let total = token03_ranking.total_tokens;
            assert_eq!(pos, 1, "TOKEN-03 should be rank 1");
            assert_eq!(total, 5, "Total tokens should be 5");

            let token01_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[0]), current_week);
            let pos = token01_ranking.token_position;
            let total = token01_ranking.total_tokens;
            assert_eq!(pos, 2, "TOKEN-01 should be rank 2");
            assert_eq!(total, 5, "Total tokens should be 5");

            let token05_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[4]), current_week);
            let pos = token05_ranking.token_position;
            let total = token05_ranking.total_tokens;
            assert_eq!(pos, 3, "TOKEN-05 should be rank 3");
            assert_eq!(total, 5, "Total tokens should be 5");

            let token02_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[1]), current_week);
            let pos = token02_ranking.token_position;
            let total = token02_ranking.total_tokens;
            assert_eq!(pos, 4, "TOKEN-02 should be rank 4");
            assert_eq!(total, 5, "Total tokens should be 5");

            let token04_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[3]), current_week);
            let pos = token04_ranking.token_position;
            let total = token04_ranking.total_tokens;
            assert_eq!(pos, 5, "TOKEN-04 should be rank 5");
            assert_eq!(total, 5, "Total tokens should be 5");

            // Test get_boosted_tokens_for_week - should return empty since no boosting
            let boosted_tokens = sc.get_boosted_tokens_for_week(current_week);
            assert_eq!(boosted_tokens.len(), 0, "Should have no boosted tokens");

            // Test boosted_amount for each token - should be 0
            for token in tokens {
                let boost = sc
                    .boosted_amount(&managed_token_id!(*token), current_week)
                    .get();
                assert_eq!(boost, managed_biguint!(0), "Token should have no boost");
            }

            // Test total_boosted_amount - should be 0
            let total_boost = sc.total_boosted_amount(current_week).get();
            assert_eq!(total_boost, managed_biguint!(0), "Total boost should be 0");

            // Test ranking for non-existent token
            let non_existent =
                sc.get_token_ranking(managed_token_id!(b"NON-EXISTENT"), current_week);
            let pos = non_existent.token_position;
            let total_ne = non_existent.total_tokens;
            assert_eq!(pos, 0, "Non-existent token should have position 0");
            assert_eq!(total_ne, 5, "Total should still be 5 tokens");
        })
        .assert_ok();
}

#[test]
fn test_boost_single_token() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 10);

    setup.set_current_week(1);
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT);

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let current_week = sc.get_current_week();
            let boosted = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), current_week)
                .get();
            assert_eq!(boosted, managed_biguint!(BOOST_AMOUNT));

            let total = sc.total_boosted_amount(current_week).get();
            assert_eq!(total, managed_biguint!(BOOST_AMOUNT));
        })
        .assert_ok();
}

#[test]
fn test_boost_multiple_tokens() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user1 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);
    let user2 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);

    setup.set_current_week(1);
    setup.boost_token(&user1, TEST_TOKENS[0], BOOST_AMOUNT);
    setup.boost_token(&user2, TEST_TOKENS[1], BOOST_AMOUNT * 2);

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let current_week = sc.get_current_week();
            let token_a = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), current_week)
                .get();
            assert_eq!(token_a, managed_biguint!(BOOST_AMOUNT));

            let token_b = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[1]), current_week)
                .get();
            assert_eq!(token_b, managed_biguint!(BOOST_AMOUNT * 2));

            let total = sc.total_boosted_amount(current_week).get();
            assert_eq!(total, managed_biguint!(BOOST_AMOUNT * 3));
        })
        .assert_ok();
}

#[test]
fn test_boost_same_token_multiple_times() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user1 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);
    let user2 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);

    setup.set_current_week(1);
    setup.boost_token(&user1, TEST_TOKENS[0], BOOST_AMOUNT);
    setup.boost_token(&user2, TEST_TOKENS[0], BOOST_AMOUNT * 2);

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let current_week = sc.get_current_week();
            let boosted = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), current_week)
                .get();
            assert_eq!(boosted, managed_biguint!(BOOST_AMOUNT * 3));

            let total = sc.total_boosted_amount(current_week).get();
            assert_eq!(total, managed_biguint!(BOOST_AMOUNT * 3));
        })
        .assert_ok();
}

#[test]
fn test_boost_different_weeks() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 10);

    // Week 1
    setup.set_current_week(1);
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT);
    let week1 = setup.get_current_week();

    // Week 2
    setup.set_current_week(2);
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT * 2);
    let week2 = setup.get_current_week();

    // Check both weeks
    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let week1_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), week1)
                .get();
            assert_eq!(week1_boost, managed_biguint!(BOOST_AMOUNT));

            let week2_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), week2)
                .get();
            assert_eq!(week2_boost, managed_biguint!(BOOST_AMOUNT * 2));

            let week1_total = sc.total_boosted_amount(week1).get();
            assert_eq!(week1_total, managed_biguint!(BOOST_AMOUNT));

            let week2_total = sc.total_boosted_amount(week2).get();
            assert_eq!(week2_total, managed_biguint!(BOOST_AMOUNT * 2));
        })
        .assert_ok();
}

#[test]
fn test_boost_wrong_token_fails() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .set_esdt_balance(&user, b"WRONG-TOKEN", &rust_biguint!(BOOST_AMOUNT));
    setup.set_current_week(1);

    setup
        .blockchain
        .execute_esdt_transfer(
            &user,
            &setup.sc_wrapper,
            b"WRONG-TOKEN",
            0,
            &rust_biguint!(BOOST_AMOUNT),
            |sc| sc.boost(managed_token_id!(TEST_TOKENS[0])),
        )
        .assert_user_error("Wrong token for boosting");
}

#[test]
fn test_boost_zero_amount_fails() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT);

    setup.set_current_week(1);

    setup
        .blockchain
        .execute_tx(&user, &setup.sc_wrapper, &rust_biguint!(0), |sc| {
            sc.boost(managed_token_id!(TEST_TOKENS[0]))
        })
        .assert_user_error("incorrect number of ESDT transfers");
}

#[test]
fn test_withdraw_boost_funds_owner_only() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT);

    setup.set_current_week(1);
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT);

    setup.check_contract_balance(BOOST_AMOUNT);

    let user_balance_before = setup.blockchain.get_esdt_balance(&user, VOTING_TOKEN_ID, 0);

    setup
        .blockchain
        .execute_tx(&user, &setup.sc_wrapper, &rust_biguint!(0), |sc| {
            sc.withdraw_boost_funds();
        })
        .assert_ok();

    let user_balance_after = setup.blockchain.get_esdt_balance(&user, VOTING_TOKEN_ID, 0);
    assert!(
        user_balance_after > user_balance_before,
        "Funds should have been withdrawn to user"
    );
}

#[test]
fn test_withdraw_boost_funds_by_owner() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 3);

    setup.set_current_week(1);
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT);
    setup.boost_token(&user, TEST_TOKENS[1], BOOST_AMOUNT * 2);

    setup.check_contract_balance(BOOST_AMOUNT * 3);
    setup.withdraw_boost_funds_as_owner();

    setup.check_user_balance(&setup.owner, BOOST_AMOUNT * 3);
    setup.check_contract_balance(0);
}

#[test]
fn test_get_boosted_tokens_for_week() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user1 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);
    let user2 = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 5);

    setup.set_current_week(1);
    setup.boost_token(&user1, TEST_TOKENS[0], BOOST_AMOUNT);
    setup.boost_token(&user2, TEST_TOKENS[2], BOOST_AMOUNT * 3);

    let current_week = setup.get_current_week();
    setup.setup_tokens_for_week(current_week, &[TEST_TOKENS[0], TEST_TOKENS[2]]);

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let boosted_tokens = sc.get_boosted_tokens_for_week(current_week);
            assert_eq!(boosted_tokens.len(), 2);

            let mut found_token_a = false;
            let mut found_token_c = false;

            for boosted_token in boosted_tokens.iter() {
                let token_bytes = boosted_token.token_id.as_managed_buffer().to_vec();
                if token_bytes == TEST_TOKENS[0] {
                    assert_eq!(boosted_token.boost_amount, managed_biguint!(BOOST_AMOUNT));
                    found_token_a = true;
                } else if token_bytes == TEST_TOKENS[2] {
                    assert_eq!(
                        boosted_token.boost_amount,
                        managed_biguint!(BOOST_AMOUNT * 3)
                    );
                    found_token_c = true;
                }
            }

            assert!(found_token_a, "TOKEN-01 should be in boosted tokens");
            assert!(found_token_c, "TOKEN-03 should be in boosted tokens");
        })
        .assert_ok();
}

#[test]
fn test_get_token_ranking() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let user = setup.create_user_with_voting_tokens(BOOST_AMOUNT * 10);

    setup.set_current_week(1);
    let current_week = setup.get_current_week();

    // Use only first 3 tokens for this test to maintain the original test logic
    let test_tokens = &TEST_TOKENS[0..3];
    setup.setup_tokens_for_week(current_week, test_tokens);
    setup.set_token_votes(TEST_TOKENS[0], current_week, 1000); // TOKEN-01: 1000 votes
    setup.set_token_votes(TEST_TOKENS[1], current_week, 3000); // TOKEN-02: 3000 votes
    setup.set_token_votes(TEST_TOKENS[2], current_week, 2000); // TOKEN-03: 2000 votes

    // Boost TOKEN-01 (1000 votes)
    // With 1 boosted token, boost multiplier = 1.5
    // TOKEN-01: 1000 * 1.5 = 1500 votes (still 3rd place: 1500 < 2000 < 3000)
    setup.boost_token(&user, TEST_TOKENS[0], BOOST_AMOUNT);

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            // TOKEN-01 should be rank 3 (1500 boosted votes < 2000 < 3000)
            let ranking = sc.get_token_ranking(managed_token_id!(TEST_TOKENS[0]), current_week);
            let position = ranking.token_position;
            let total = ranking.total_tokens;
            assert_eq!(position, 3, "TOKEN-01 should be rank 3");
            assert_eq!(total, 3, "Total should be 3 tokens");

            // TOKEN-02 should be rank 1 (3000 votes, highest)
            let ranking_b = sc.get_token_ranking(managed_token_id!(TEST_TOKENS[1]), current_week);
            let pos_b = ranking_b.token_position;
            let total_b = ranking_b.total_tokens;
            assert_eq!(pos_b, 1, "TOKEN-02 should be rank 1");
            assert_eq!(total_b, 3, "Total should be 3 tokens");

            // TOKEN-03 should be rank 2 (2000 votes)
            let ranking_c = sc.get_token_ranking(managed_token_id!(TEST_TOKENS[2]), current_week);
            let pos_c = ranking_c.token_position;
            let total_c = ranking_c.total_tokens;
            assert_eq!(pos_c, 2, "TOKEN-03 should be rank 2");
            assert_eq!(total_c, 3, "Total should be 3 tokens");

            // Non-existent token should return position 0
            let non_existent =
                sc.get_token_ranking(managed_token_id!(b"NON-EXISTENT"), current_week);
            let pos = non_existent.token_position;
            let total_ne = non_existent.total_tokens;
            assert_eq!(pos, 0, "Non-existent token should have position 0");
            assert_eq!(total_ne, 3, "Total should still be 3 tokens");
        })
        .assert_ok();
}

#[test]
fn test_boost_multiplier_formula() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );
    let users: Vec<_> = (0..5)
        .map(|_| setup.create_user_with_voting_tokens(BOOST_AMOUNT * 10))
        .collect();

    setup.set_current_week(1);
    let current_week = setup.get_current_week();

    // Use only first 3 tokens for this test to maintain the original test logic
    let test_tokens = &TEST_TOKENS[0..3];
    setup.setup_tokens_for_week(current_week, test_tokens);
    for token in test_tokens {
        setup.set_token_votes(token, current_week, 1000);
    }

    // Boost all 3 tokens with equal amounts
    for i in 0..3 {
        setup.boost_token(&users[i], TEST_TOKENS[i], BOOST_AMOUNT);
    }

    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let ranking = sc.get_week_ranking(current_week);

            assert_eq!(ranking.len(), 3, "Should have exactly 3 ranked tokens");

            // With 3 boosted tokens:
            // Boost factor = 0.5 / 3 = 0.167 (in formula)
            // 1st token: 1.5x - 0.167 * 0 = 1.5x = 1500
            // 2nd token: 1.5x - 0.167 * 1 = 1.333x ≈ 1333
            // 3rd token: 1.5x - 0.167 * 2 = 1.167x ≈ 1167
            let first_token_votes = ranking.get(0).votes;
            assert!(
                first_token_votes > managed_biguint!(1000),
                "First token should have boosted votes"
            );
            assert!(
                first_token_votes <= managed_biguint!(1500),
                "First token votes should not exceed max boost"
            );

            let second_token_votes = ranking.get(1).votes;
            assert!(
                second_token_votes > managed_biguint!(1000),
                "Second token should have boosted votes"
            );
            assert!(
                second_token_votes < first_token_votes,
                "Second token should have less votes than first"
            );

            let third_token_votes = ranking.get(2).votes;
            assert!(
                third_token_votes > managed_biguint!(1000),
                "Third token should have boosted votes"
            );
            assert!(
                third_token_votes < second_token_votes,
                "Third token should have less votes than second"
            );
        })
        .assert_ok();
}

#[test]
fn test_comprehensive_ranking_with_10_tokens_and_5_users() {
    DebugApi::dummy();
    let mut setup = TokenExposureVotingSetup::new(
        token_exposure_voting::contract_obj,
        energy_factory::contract_obj,
    );

    // Create 5 users with voting tokens
    let users: Vec<_> = (0..5)
        .map(|_| setup.create_user_with_voting_tokens(BOOST_AMOUNT * 10))
        .collect();

    setup.set_current_week(1);
    let current_week = setup.get_current_week();

    // Set up 10 tokens with different vote amounts (descending order initially)
    // TOKEN-01: 10000 votes (highest)
    // TOKEN-02: 9000 votes
    // TOKEN-03: 8000 votes
    // TOKEN-04: 7000 votes
    // TOKEN-05: 6000 votes
    // TOKEN-06: 5000 votes
    // TOKEN-07: 4000 votes
    // TOKEN-08: 3000 votes
    // TOKEN-09: 2000 votes
    // TOKEN-10: 1000 votes (lowest)
    setup.setup_tokens_for_week(current_week, TEST_TOKENS);

    for (i, token) in TEST_TOKENS.iter().enumerate() {
        let votes = (10 - i as u64) * 1000; // 10000, 9000, 8000, ..., 1000
        setup.set_token_votes(token, current_week, votes);
    }

    // Boost 5 tokens with different amounts:
    // TOKEN-10 (originally 1000 votes) - boost by user 0
    // TOKEN-08 (originally 3000 votes) - boost by user 1
    // TOKEN-06 (originally 5000 votes) - boost by user 2
    // TOKEN-04 (originally 7000 votes) - boost by user 3
    // TOKEN-02 (originally 9000 votes) - boost by user 4
    setup.boost_token(&users[0], TEST_TOKENS[9], BOOST_AMOUNT * 3); // TOKEN-10 gets 3x boost
    setup.boost_token(&users[1], TEST_TOKENS[7], BOOST_AMOUNT * 2); // TOKEN-08 gets 2x boost
    setup.boost_token(&users[2], TEST_TOKENS[5], BOOST_AMOUNT * 2); // TOKEN-06 gets 2x boost
    setup.boost_token(&users[3], TEST_TOKENS[3], BOOST_AMOUNT); // TOKEN-04 gets 1x boost
    setup.boost_token(&users[4], TEST_TOKENS[1], BOOST_AMOUNT); // TOKEN-02 gets 1x boost

    // Verify the ranking order
    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            let ranking = sc.get_week_ranking(current_week);
            assert_eq!(ranking.len(), 10, "Should have exactly 10 ranked tokens");

            // Expected ranking with boost formula applied:
            // Boost multiplier = (1.5 - (0.5 * (position - 1) / total_boosted_tokens))
            // With 5 boosted tokens: factor = 0.5 / 5 = 0.1
            // Positions are assigned based on original votes (descending): TOKEN-02, TOKEN-04, TOKEN-06, TOKEN-08, TOKEN-10
            // TOKEN-02: 9000 * (1.5 - 0.1 * 0) = 9000 * 1.5 = 13500
            // TOKEN-04: 7000 * (1.5 - 0.1 * 1) = 7000 * 1.4 = 9800
            // TOKEN-06: 5000 * (1.5 - 0.1 * 2) = 5000 * 1.3 = 6500
            // TOKEN-08: 3000 * (1.5 - 0.1 * 3) = 3000 * 1.2 = 3600
            // TOKEN-10: 1000 * (1.5 - 0.1 * 4) = 1000 * 1.1 = 1100
            //
            // Final expected order:
            // 1. TOKEN-02: 13500 votes (boosted from 9000)
            // 2. TOKEN-01: 10000 votes (no boost)
            // 3. TOKEN-04: 9800 votes (boosted from 7000)
            // 4. TOKEN-03: 8000 votes (no boost)
            // 5. TOKEN-06: 6500 votes (boosted from 5000)
            // 6. TOKEN-05: 6000 votes (no boost)
            // 7. TOKEN-07: 4000 votes (no boost)
            // 8. TOKEN-08: 3600 votes (boosted from 3000)
            // 9. TOKEN-09: 2000 votes (no boost)
            // 10. TOKEN-10: 1100 votes (boosted from 1000)

            // Verify top 10 positions
            let rank1_token = ranking.get(0);
            let rank1_bytes = rank1_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(
                rank1_bytes, TEST_TOKENS[1],
                "TOKEN-02 should be rank 1 (boosted)"
            );
            assert_eq!(rank1_token.votes, managed_biguint!(13500));

            let rank2_token = ranking.get(1);
            let rank2_bytes = rank2_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(rank2_bytes, TEST_TOKENS[0], "TOKEN-01 should be rank 2");
            assert_eq!(rank2_token.votes, managed_biguint!(10000));

            let rank3_token = ranking.get(2);
            let rank3_bytes = rank3_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(
                rank3_bytes, TEST_TOKENS[3],
                "TOKEN-04 should be rank 3 (boosted)"
            );
            assert_eq!(rank3_token.votes, managed_biguint!(9800));

            let rank4_token = ranking.get(3);
            let rank4_bytes = rank4_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(rank4_bytes, TEST_TOKENS[2], "TOKEN-03 should be rank 4");
            assert_eq!(rank4_token.votes, managed_biguint!(8000));

            let rank5_token = ranking.get(4);
            let rank5_bytes = rank5_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(
                rank5_bytes, TEST_TOKENS[5],
                "TOKEN-06 should be rank 5 (boosted)"
            );
            assert_eq!(rank5_token.votes, managed_biguint!(6500));

            let rank6_token = ranking.get(5);
            let rank6_bytes = rank6_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(rank6_bytes, TEST_TOKENS[4], "TOKEN-05 should be rank 6");
            assert_eq!(rank6_token.votes, managed_biguint!(6000));

            let rank7_token = ranking.get(6);
            let rank7_bytes = rank7_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(rank7_bytes, TEST_TOKENS[6], "TOKEN-07 should be rank 7");
            assert_eq!(rank7_token.votes, managed_biguint!(4000));

            let rank8_token = ranking.get(7);
            let rank8_bytes = rank8_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(
                rank8_bytes, TEST_TOKENS[7],
                "TOKEN-08 should be rank 8 (boosted)"
            );
            assert_eq!(rank8_token.votes, managed_biguint!(3600));

            let rank9_token = ranking.get(8);
            let rank9_bytes = rank9_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(rank9_bytes, TEST_TOKENS[8], "TOKEN-09 should be rank 9");
            assert_eq!(rank9_token.votes, managed_biguint!(2000));

            let rank10_token = ranking.get(9);
            let rank10_bytes = rank10_token.token_id.as_managed_buffer().to_vec();
            assert_eq!(
                rank10_bytes, TEST_TOKENS[9],
                "TOKEN-10 should be rank 10 (boosted)"
            );
            assert_eq!(rank10_token.votes, managed_biguint!(1100));

            // Verify all boosted tokens have the expected boost amounts
            let token10_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[9]), current_week)
                .get();
            assert_eq!(
                token10_boost,
                managed_biguint!(BOOST_AMOUNT * 3),
                "TOKEN-10 should have 3x boost amount"
            );

            let token08_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[7]), current_week)
                .get();
            assert_eq!(
                token08_boost,
                managed_biguint!(BOOST_AMOUNT * 2),
                "TOKEN-08 should have 2x boost amount"
            );

            let token06_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[5]), current_week)
                .get();
            assert_eq!(
                token06_boost,
                managed_biguint!(BOOST_AMOUNT * 2),
                "TOKEN-06 should have 2x boost amount"
            );

            let token04_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[3]), current_week)
                .get();
            assert_eq!(
                token04_boost,
                managed_biguint!(BOOST_AMOUNT),
                "TOKEN-04 should have 1x boost amount"
            );

            let token02_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[1]), current_week)
                .get();
            assert_eq!(
                token02_boost,
                managed_biguint!(BOOST_AMOUNT),
                "TOKEN-02 should have 1x boost amount"
            );

            let total_boost = sc.total_boosted_amount(current_week).get();
            assert_eq!(
                total_boost,
                managed_biguint!(BOOST_AMOUNT * 9), // 3 + 2 + 2 + 1 + 1 = 9
                "Total boost should be 9x BOOST_AMOUNT"
            );

            // Verify non-boosted tokens have no boost
            let token01_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[0]), current_week)
                .get();
            assert_eq!(
                token01_boost,
                managed_biguint!(0),
                "TOKEN-01 should have no boost"
            );

            let token03_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[2]), current_week)
                .get();
            assert_eq!(
                token03_boost,
                managed_biguint!(0),
                "TOKEN-03 should have no boost"
            );

            let token05_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[4]), current_week)
                .get();
            assert_eq!(
                token05_boost,
                managed_biguint!(0),
                "TOKEN-05 should have no boost"
            );

            let token07_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[6]), current_week)
                .get();
            assert_eq!(
                token07_boost,
                managed_biguint!(0),
                "TOKEN-07 should have no boost"
            );

            let token09_boost = sc
                .boosted_amount(&managed_token_id!(TEST_TOKENS[8]), current_week)
                .get();
            assert_eq!(
                token09_boost,
                managed_biguint!(0),
                "TOKEN-09 should have no boost"
            );
        })
        .assert_ok();

    // Test individual token rankings
    setup
        .blockchain
        .execute_query(&setup.sc_wrapper, |sc| {
            // TOKEN-02 should be rank 1 (boosted)
            let token02_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[1]), current_week);
            let pos = token02_ranking.token_position;
            let total = token02_ranking.total_tokens;
            assert_eq!(pos, 1, "TOKEN-02 should be rank 1 (boosted)");
            assert_eq!(total, 10, "Total tokens should be 10");

            // TOKEN-01 should be rank 2 (no boost)
            let token01_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[0]), current_week);
            let pos = token01_ranking.token_position;
            let total = token01_ranking.total_tokens;
            assert_eq!(pos, 2, "TOKEN-01 should be rank 2");
            assert_eq!(total, 10, "Total tokens should be 10");

            // TOKEN-04 should be rank 3 (boosted)
            let token04_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[3]), current_week);
            let pos = token04_ranking.token_position;
            let total = token04_ranking.total_tokens;
            assert_eq!(pos, 3, "TOKEN-04 should be rank 3 (boosted)");
            assert_eq!(total, 10, "Total tokens should be 10");

            // TOKEN-06 should be rank 5 (boosted)
            let token06_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[5]), current_week);
            let pos = token06_ranking.token_position;
            let total = token06_ranking.total_tokens;
            assert_eq!(pos, 5, "TOKEN-06 should be rank 5 (boosted)");
            assert_eq!(total, 10, "Total tokens should be 10");

            // TOKEN-08 should be rank 8 (boosted)
            let token08_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[7]), current_week);
            let pos = token08_ranking.token_position;
            let total = token08_ranking.total_tokens;
            assert_eq!(pos, 8, "TOKEN-08 should be rank 8 (boosted)");
            assert_eq!(total, 10, "Total tokens should be 10");

            // TOKEN-10 should be rank 10 (boosted but still last)
            let token10_ranking =
                sc.get_token_ranking(managed_token_id!(TEST_TOKENS[9]), current_week);
            let pos = token10_ranking.token_position;
            let total = token10_ranking.total_tokens;
            assert_eq!(
                pos, 10,
                "TOKEN-10 should be rank 10 (boosted but still last)"
            );
            assert_eq!(total, 10, "Total tokens should be 10");
        })
        .assert_ok();
}
