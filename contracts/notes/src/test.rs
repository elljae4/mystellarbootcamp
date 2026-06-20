#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _,
        vec, Address, Env,
    };
    use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};

    const CONTRIBUTION: i128 = 100_0000000i128; // 100 USDC per round per member

    fn setup_three_member_circle() -> (Env, Address, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_sac = StellarAssetClient::new(&env, &token_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let carol = Address::generate(&env);

        // Mint 1000 USDC to each member
        token_sac.mint(&alice, &1_000_0000000i128);
        token_sac.mint(&bob, &1_000_0000000i128);
        token_sac.mint(&carol, &1_000_0000000i128);

        let contract_id = env.register_contract(None, TipidCircle);
        let c = TipidCircleClient::new(&env, &contract_id);

        let members = vec![&env, alice.clone(), bob.clone(), carol.clone()];
        c.init(&token_id, &alice, &members, &CONTRIBUTION);

        (env, contract_id, token_id, alice, bob, carol)
    }

    // -----------------------------------------------------------------------
    // Test 1 — Happy path: all three members contribute → Alice receives pot
    // -----------------------------------------------------------------------
    #[test]
    fn test_round_one_payout_to_alice() {
        let (env, contract_id, token_id, alice, bob, carol) = setup_three_member_circle();
        let c = TipidCircleClient::new(&env, &contract_id);
        let token = TokenClient::new(&env, &token_id);

        let alice_balance_before = token.balance(&alice);

        // All three contribute
        c.contribute(&alice);
        c.contribute(&bob);
        c.contribute(&carol); // triggers payout to Alice (index 0)

        // Alice received 300 USDC (3 × 100 USDC pot)
        let expected_pot = CONTRIBUTION * 3;
        assert_eq!(token.balance(&alice), alice_balance_before - CONTRIBUTION + expected_pot);

        // Contract holds nothing
        assert_eq!(token.balance(&contract_id), 0i128);
    }

    // -----------------------------------------------------------------------
    // Test 2 — Edge case: non-member cannot contribute
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "caller is not a circle member")]
    fn test_non_member_cannot_contribute() {
        let (env, contract_id, _token_id, _alice, _bob, _carol) =
            setup_three_member_circle();
        let c = TipidCircleClient::new(&env, &contract_id);

        let outsider = Address::generate(&env);
        c.contribute(&outsider);
    }

    // -----------------------------------------------------------------------
    // Test 3 — State verification: current_round advances after payout
    // -----------------------------------------------------------------------
    #[test]
    fn test_round_advances_after_payout() {
        let (env, contract_id, _token_id, alice, bob, carol) =
            setup_three_member_circle();
        let c = TipidCircleClient::new(&env, &contract_id);

        // Complete round 0
        c.contribute(&alice);
        c.contribute(&bob);
        c.contribute(&carol);

        let circle = c.get_circle();
        assert_eq!(circle.current_round, 1); // advanced to round 1
        assert_eq!(circle.contributions_this_round, 0); // reset
        assert_eq!(circle.status, CircleStatus::Open);
    }

    // -----------------------------------------------------------------------
    // Test 4 — Edge case: partial round should not pay out early
    // -----------------------------------------------------------------------
    #[test]
    fn test_no_payout_after_partial_contributions() {
        let (env, contract_id, token_id, alice, bob, _carol) =
            setup_three_member_circle();
        let c = TipidCircleClient::new(&env, &contract_id);
        let token = TokenClient::new(&env, &token_id);

        // Only two of three contribute
        c.contribute(&alice);
        c.contribute(&bob);

        // Contract should still hold 200 USDC (2 contributions)
        assert_eq!(token.balance(&contract_id), CONTRIBUTION * 2);

        // Round still at 0, not yet paid
        let circle = c.get_circle();
        assert_eq!(circle.current_round, 0);
        assert_eq!(circle.contributions_this_round, 2);
    }

    // -----------------------------------------------------------------------
    // Test 5 — Full three-round lifecycle → status Complete
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_rounds_complete() {
        let (env, contract_id, _token_id, alice, bob, carol) =
            setup_three_member_circle();
        let c = TipidCircleClient::new(&env, &contract_id);

        // Round 0 — Alice gets pot
        c.contribute(&alice);
        c.contribute(&bob);
        c.contribute(&carol);

        // Round 1 — Bob gets pot
        c.contribute(&alice);
        c.contribute(&bob);
        c.contribute(&carol);

        // Round 2 — Carol gets pot → Complete
        c.contribute(&alice);
        c.contribute(&bob);
        c.contribute(&carol);

        let circle = c.get_circle();
        assert_eq!(circle.status, CircleStatus::Complete);
    }
}