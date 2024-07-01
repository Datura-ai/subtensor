use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use itertools::izip;
use pallet_subtensor::*;
use sp_core::U256;
use substrate_fixed::types::I64F64;
use types::SubnetType;
mod mock;

#[macro_use]
mod helpers;

// To run just the tests in this file, use the following command:
// Use the following command to run the tests in this file with verbose logging:
// RUST_LOG=debug cargo test -p pallet-subtensor --test dtao

#[test]
fn test_add_subnet_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);
        let lock_cost = 100_000_000_000; // 100 TAO

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, lock_cost);
        // Check
        // -- that the lock cost is 100 TAO.
        // -- that the balance is 100 TAO.
        // -- that the root pool is empty.
        // -- that the root alpha pool is empty.
        // -- that the root price is 1.0.
        // -- that the root has zero k value.
        assert_eq!(SubtensorModule::get_network_lock_cost(), lock_cost);
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey), lock_cost);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 0),
            0
        ); // 1 subnets * 100 TAO lock cost.
        assert_eq!(SubtensorModule::get_total_stake_for_subnet(0), 0);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(0), 1.0);
        assert_eq!(SubtensorModule::get_tao_reserve(0), 0);
        assert_eq!(SubtensorModule::get_alpha_reserve(0), 0);
        assert_eq!(SubtensorModule::get_pool_k(0), 0);
        assert!(!SubtensorModule::is_subnet_dynamic(0));

        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(0)
        );
        // Register a network with this coldkey + hotkey for a lock cost of 100 TAO.
        step_block(1);
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        // Check:
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network has someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet is 100 TAO.
        // -- that the new alpha/tao price is 1.0.
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 100 ALPHA
        // -- that the k factor is 100 TAO * 100 ALPHA.
        // -- that the new network is dynamic
        assert_eq!(SubtensorModule::get_network_lock_cost(), 199_999_999_000); // 200 TAO.
                                                                               // TODO:(sam)Decide how to deal with ED , as this account can only stake 199
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            ExistentialDeposit::get()
        ); // 0 TAO.
        assert_eq!(SubtensorModule::get_subnet_owner(1), coldkey);
        assert_eq!(SubtensorModule::get_subnetwork_n(1), 1);
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(1, 0).unwrap(),
            hotkey
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 1),
            100_000_000_000
        ); // 1 subnets * 100 TAO lock cost.
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(1),
            100_000_000_000
        );
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 1.0);
        assert_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 100_000_000_000);
        assert_eq!(
            SubtensorModule::get_pool_k(1),
            100_000_000_000 * 100_000_000_000
        );
        assert!(SubtensorModule::is_subnet_dynamic(1));
        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(1)
        );

        // Register a new network
        assert_eq!(
            SubtensorModule::get_network_lock_cost(),
            2 * (lock_cost - ExistentialDeposit::get())
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            2 * (lock_cost - ExistentialDeposit::get()),
        );
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        // Check:
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network as someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet 2 is 400 TAO.
        // -- that the new alpha/tao price is 0.5.
        // -- that the tao reserve is 200 TAO.
        // -- that the alpha reserve is 400 ALPHA
        // -- that the k factor is 200 TAO * 400 ALPHA.
        // -- that the new network is dynamic
        // TODO:(sam)Decide how to deal with ED , as this account can only stake 199
        assert_eq!(
            SubtensorModule::get_network_lock_cost(),
            400_000_000_000 - ExistentialDeposit::get() * 4
        ); // 400 TAO.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            ExistentialDeposit::get()
        ); // 0 TAO.
        assert_eq!(SubtensorModule::get_subnet_owner(2), coldkey);
        assert_eq!(SubtensorModule::get_subnetwork_n(2), 1);
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(2, 0).unwrap(),
            hotkey
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 2),
            400_000_000_000 - ExistentialDeposit::get() * 4
        ); // 2 subnets * 2 TAO lock cost.
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(2),
            400_000_000_000 - ExistentialDeposit::get() * 4
        );
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.5);
        assert_eq!(
            SubtensorModule::get_tao_reserve(2),
            200_000_000_000 - ExistentialDeposit::get() * 2
        );
        assert_eq!(
            SubtensorModule::get_alpha_reserve(2),
            400_000_000_000 - ExistentialDeposit::get() * 4
        );
        assert_eq!(
            SubtensorModule::get_pool_k(2),
            (200_000_000_000 - ExistentialDeposit::get() as u128 * 2u128)
                * (400_000_000_000 - ExistentialDeposit::get() as u128 * 4u128)
        );
        assert!(SubtensorModule::is_subnet_dynamic(2));
        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(2)
        );

        // Let's remove all of our stake from subnet 2.
        // Check:
        // -- that the balance is initially 0
        // -- that the unstake event is ok.
        // -- that the balance is 100 TAO. Given the slippage.
        // -- that the price per alpha has changed to 0.125
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 800 ALPHA
        // -- that the k factor is 100 TAO * 400 ALPHA. (unchanged)
        // TODO:(sam)Decide how to deal with ED , free balance will always be 1
        assert_eq!(Balances::free_balance(coldkey), ExistentialDeposit::get());
        // We set this to zero , otherwise the alpha calculation is off due to the fact that many tempos will be run
        // over the default lock period (3 months)
        SubtensorModule::set_subnet_owner_lock_period(0);
        assert_eq!(
            SubtensorModule::get_pool_k(2),
            (200_000_000_000 - ExistentialDeposit::get() as u128 * 2u128)
                * (400_000_000_000 - ExistentialDeposit::get() as u128 * 4u128)
        );

        run_to_block(3);
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            2,
            400_000_000_000 - ExistentialDeposit::get() * 4
        ));
        // assert_eq!( Balances::free_balance(coldkey), 100_000_000_000);
        // Also use more rigour calculation for slippage via K
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.125);
        assert_eq!(
            round_to_significant_figures(SubtensorModule::get_tao_reserve(2), 3),
            100_000_000_000
        );
        // Yet another ugly approximation
        assert_eq!(
            round_to_significant_figures(SubtensorModule::get_alpha_reserve(2), 2),
            800_000_000_000
        );

        log::info!(
            "Alpha Reserve is {:?}",
            SubtensorModule::get_alpha_reserve(2)
        );
        log::info!("Tao Reserve is {:?}", SubtensorModule::get_tao_reserve(2));

        // Let's run a block step.
        // Alpha pending emission is not zero at start because we already ran to block 3
        // and had emissions
        // Check
        // -- that the pending emission for the 2 subnets is correct
        // -- that the pending alpha emission of the 2 subnets is correct.
        let tao = 1_000_000_000;

        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(1), 0.9901); // diluted because of emissions in run_to_block
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.125);
        step_block(1);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000u64);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 101);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 102);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 802);
        run_to_block(10);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1).div_ceil(tao), 100);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 101);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 108);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 808);
        run_to_block(30);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1).div_ceil(tao), 104);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 105);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 120);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 820);

        for _ in 0..100 {
            step_block(1);
            log::info!(
                "S1: {}, S2: {}",
                SubtensorModule::get_tao_per_alpha_price(1),
                SubtensorModule::get_tao_per_alpha_price(2)
            );
        }
    });
}

#[test]
fn test_stake_unstake() {
    new_test_ext(1).execute_with(|| {
        // init params.
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        // Register subnet.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000); // 100 TAO.
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));
        assert_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 1.0);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000); // 100 TAO.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            1,
            100_000_000_000
        ));
        assert_eq!(SubtensorModule::get_tao_reserve(1), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 50_000_000_000);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 4); // Price is increased from the stake operation.
    })
}

// To run this test, use the following command:
// cargo test -p pallet-subtensor --test dtao test_calculate_tempos
fn round_to_significant_figures(num: u64, significant_figures: u32) -> u64 {
    if num == 0 {
        return 0;
    }
    let digits = (num as f64).log10().floor() as u32 + 1; // Calculate the number of digits in the number
    let scale = 10u64.pow(digits - significant_figures); // Determine the scaling factor

    // Scale down, round, and scale up
    ((num as f64 / scale as f64).round() as u64) * scale
}
#[test]
fn test_calculate_tempos() {
    new_test_ext(1).execute_with(|| {
        let netuids = vec![1, 2, 3];
        let k = I64F64::from_num(10); // Example constant K
        let prices = vec![
            I64F64::from_num(100.0),
            I64F64::from_num(200.0),
            I64F64::from_num(300.0),
        ];

        let expected_tempos = vec![
            (1, 60), // Calculated tempo for netuid 1
            (2, 30), // Calculated tempo for netuid 2
            (3, 20), // Calculated tempo for netuid 3
        ];

        let tempos = SubtensorModule::calculate_tempos(&netuids, k, &prices).unwrap();
        assert_eq!(tempos, expected_tempos, "Tempos calculated incorrectly");

        // Edge case: Empty netuids and prices
        let empty_netuids = vec![];
        let empty_prices = vec![];
        let empty_tempos =
            SubtensorModule::calculate_tempos(&empty_netuids, k, &empty_prices).unwrap();
        assert!(
            empty_tempos.is_empty(),
            "Empty tempos should be an empty vector"
        );

        // Edge case: Zero prices
        let zero_prices = vec![
            I64F64::from_num(0.0),
            I64F64::from_num(0.0),
            I64F64::from_num(0.0),
        ];
        let zero_tempos = SubtensorModule::calculate_tempos(&netuids, k, &zero_prices).unwrap();
        assert_eq!(
            zero_tempos,
            vec![(1, 0), (2, 0), (3, 0)],
            "Zero prices should lead to zero tempos"
        );

        // Edge case: Negative prices
        let negative_prices = vec![
            I64F64::from_num(-100.0),
            I64F64::from_num(-200.0),
            I64F64::from_num(-300.0),
        ];
        let negative_tempos =
            SubtensorModule::calculate_tempos(&netuids, k, &negative_prices).unwrap();
        assert_eq!(
            negative_tempos, expected_tempos,
            "Negative prices should be treated as positive for tempo calculation"
        );

        // Edge case: Very large prices
        let large_prices = vec![
            I64F64::from_num(1e12),
            I64F64::from_num(2e12),
            I64F64::from_num(3e12),
        ];
        let large_tempos = SubtensorModule::calculate_tempos(&netuids, k, &large_prices).unwrap();
        assert_eq!(
            large_tempos, expected_tempos,
            "Large prices should scale similarly in tempo calculation"
        );

        // Edge case: Mismatched vector sizes
        let mismatched_prices = vec![I64F64::from_num(100.0), I64F64::from_num(200.0)]; // Missing price for netuid 3
        assert!(
            SubtensorModule::calculate_tempos(&netuids, k, &mismatched_prices).is_err(),
            "Mismatched vector sizes should result in an error"
        );

        // Edge case: Extremely small non-zero prices
        let small_prices = vec![
            I64F64::from_num(1e-12),
            I64F64::from_num(1e-12),
            I64F64::from_num(1e-12),
        ];
        let small_tempos = SubtensorModule::calculate_tempos(&netuids, k, &small_prices).unwrap();
        assert_eq!(
            small_tempos,
            vec![(1, 30), (2, 30), (3, 30)],
            "Extremely small prices should return same tempos"
        );

        // Edge case: Prices with high precision
        let high_precision_prices = vec![
            I64F64::from_num(100.123456789),
            I64F64::from_num(200.123456789),
            I64F64::from_num(300.123456789),
        ];
        let high_precision_tempos =
            SubtensorModule::calculate_tempos(&netuids, k, &high_precision_prices).unwrap();
        assert_eq!(
            high_precision_tempos,
            vec![(1, 59), (2, 30), (3, 20)],
            "High precision prices should affect tempo calculations"
        );
    });
}

///////////////////////////////////////////////////////////////////////////////
// Price tests
//
// - Price of a single subnet is 1 if TAO is 1 and Alpha is 1
// - Price of a single subnet with numerous unstakes
// - Price of a single subnet with numerous stakes

#[test]
fn test_price_tao_1_alpha_1() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = 100_000_000_000;
        add_dynamic_network(1, 1, 1, 1, lock_amount);

        // Alpha on delegate should be lock_amount
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, 1),
            lock_amount
        );

        let expected_price = I64F64::from_num(1.0);
        let actual_price: I64F64 = SubtensorModule::get_tao_per_alpha_price(1);

        assert_eq!(expected_price, actual_price);
    });
}

#[test]
fn test_price_tao_alpha_unstake() {
    [
        1u64,
        2,
        3,
        4,
        5,
        100,
        200,
        1234,
        1_000_000_000,
        100_000_000_000,
    ]
    .iter()
    .for_each(|&unstake_alpha_amount| {
        new_test_ext(1).execute_with(|| {
            let delegate = U256::from(1);
            SubtensorModule::set_target_stakes_per_interval(20);
            let lock_amount = 100_000_000_000;
            add_dynamic_network(1, 1, 1, 1, lock_amount);

            // Remove subnet creator lock
            SubtensorModule::set_subnet_owner_lock_period(0);

            // Alpha on delegate should be lock_amount
            assert_eq!(
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, 1),
                lock_amount
            );

            let unstaked_tao = SubtensorModule::estimate_dynamic_unstake(1, unstake_alpha_amount);

            // Unstake half of alpha for subnets 1
            assert_ok!(SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(delegate),
                delegate,
                1,
                unstake_alpha_amount
            ));

            let tao_reserve = lock_amount - unstaked_tao;
            let alpha_reserve = lock_amount + unstake_alpha_amount;

            let expected_price = I64F64::from_num(tao_reserve) / I64F64::from_num(alpha_reserve);
            let actual_price: I64F64 = SubtensorModule::get_tao_per_alpha_price(1);

            // assert_approx_eq!(expected_price.to_num::<f64>(), actual_price.to_num::<f64>());

            assert_eq!(expected_price, actual_price);
        });
    });
}

#[test]
fn test_price_tao_alpha_stake() {
    [
        1,
        2,
        3,
        100,
        1000,
        1000000000u64,
        10000000000u64,
        100000000000u64,
    ]
    .iter()
    .for_each(|&stake_tao_amount| {
        new_test_ext(1).execute_with(|| {
            let delegate = U256::from(1);
            SubtensorModule::set_target_stakes_per_interval(20);
            let lock_amount = 100_000_000_000;
            add_dynamic_network(1, 1, 1, 1, lock_amount);
            SubtensorModule::add_balance_to_coldkey_account(
                &delegate,
                stake_tao_amount + ExistentialDeposit::get(),
            );

            // Alpha on delegate should be lock_amount
            assert_eq!(
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, 1),
                lock_amount
            );

            let k = lock_amount as u128 * lock_amount as u128;
            let new_tao_reserve = lock_amount + stake_tao_amount;
            let new_alpha_reserve: I64F64 = I64F64::from_num(k / new_tao_reserve as u128);
            let expected_price =
                I64F64::from_num(new_tao_reserve) / I64F64::from_num(new_alpha_reserve);

            // Unstake half of alpha for subnets 1
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(delegate),
                delegate,
                1,
                stake_tao_amount
            ));

            // Get actual price
            let actual_price: I64F64 = SubtensorModule::get_tao_per_alpha_price(1);
            // assert_approx_eq!(expected_price.to_num::<f64>(), actual_price.to_num::<f64>());
            assert_eq!(expected_price, actual_price);
        });
    });
}

#[test]
fn test_sum_prices_diverges_2_subnets() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = 100_000_000_000;
        add_dynamic_network(1, 1, 1, 1, lock_amount);
        add_dynamic_network(2, 1, 1, 1, lock_amount);

        for block in 1..=1000 {
            SubtensorModule::run_coinbase(block);
        }

        let expected_sum = 1.0;
        let actual_price_1: I64F64 = SubtensorModule::get_tao_per_alpha_price(1);
        let actual_price_2: I64F64 = SubtensorModule::get_tao_per_alpha_price(2);
        let actual_sum = (actual_price_1 + actual_price_2).to_num::<f64>();

        assert_approx_eq!(expected_sum, actual_sum);
    });
}

#[test]
fn test_sum_prices_diverges_3_subnets() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = 100_000_000_000;
        add_dynamic_network(1, 1, 1, 1, lock_amount);
        add_dynamic_network(2, 1, 1, 1, lock_amount);
        add_dynamic_network(3, 1, 1, 1, lock_amount);

        for block in 1..=1000 {
            SubtensorModule::run_coinbase(block);
        }

        let expected_sum = 1.0;
        let actual_price_1: I64F64 = SubtensorModule::get_tao_per_alpha_price(1);
        let actual_price_2: I64F64 = SubtensorModule::get_tao_per_alpha_price(2);
        let actual_price_3: I64F64 = SubtensorModule::get_tao_per_alpha_price(3);
        let actual_sum = (actual_price_1 + actual_price_2 + actual_price_3).to_num::<f64>();

        assert_approx_eq!(expected_sum, actual_sum);
    });
}

////////////////////////////////
// Dissolve tests
//

#[test]
fn test_dissolve_dtao_fail() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = 100_000_000_000;
        add_dynamic_network(1, 1, 1, 1, lock_amount);

        assert_eq!(
            SubtensorModule::dissolve_network(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                1,
            ),
            Err(Error::<Test>::NotAllowedToDissolve.into())
        );
    });
}

////////////////////////////////
// Block emission tests:
// Check that TotalSubnetTAO + DynamicAlphaReserve have properly increased
//

#[test]
fn test_block_emission_adds_up_1_subnet() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = 100_000_000_000;
        add_dynamic_network(1, 1, 1, 1, lock_amount);

        let block_emission = SubtensorModule::get_block_emission().unwrap_or(0);

        let total_subnet_tao_before = pallet_subtensor::TotalSubnetTAO::<Test>::get(1);
        let dynamic_alpha_reserve_before = pallet_subtensor::DynamicAlphaReserve::<Test>::get(1);

        SubtensorModule::run_coinbase(1);

        let total_subnet_tao_after = pallet_subtensor::TotalSubnetTAO::<Test>::get(1);
        let dynamic_alpha_reserve_after = pallet_subtensor::DynamicAlphaReserve::<Test>::get(1);

        assert_eq!(
            total_subnet_tao_before + dynamic_alpha_reserve_before + block_emission,
            total_subnet_tao_after + dynamic_alpha_reserve_after
        );
    });
}

#[test]
fn test_block_emission_adds_up_many_subnets() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(1000);

        let subnet_count = 20;

        let hotkey = U256::from(1);
        let coldkey = U256::from(1);
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        for netuid in 1u16..=subnet_count {
            let lock_amount = 100_000_000_000 * netuid as u64;
            add_dynamic_network(netuid, 1, 1, 1, lock_amount);

            // Get amount of alpha in the network
            let alpha = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid);

            // Remove stake to make prices lower so that they add up to lower than 1.0
            assert_ok!(SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                alpha * 19 / 20
            ));
        }

        let block_emission = SubtensorModule::get_block_emission().unwrap_or(0);

        let all_total_subnet_tao_before: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();
        let all_dynamic_alpha_reserve_before: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .sum();

        SubtensorModule::run_coinbase(1);

        let all_total_subnet_tao_after: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();
        let all_dynamic_alpha_reserve_after: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .sum();

        // Approximate equality of TAO emissions
        assert_eq!(
            (all_total_subnet_tao_before + all_dynamic_alpha_reserve_before + block_emission)
                / 10_000_000_000,
            (all_total_subnet_tao_after + all_dynamic_alpha_reserve_after) / 10_000_000_000
        );
        // Alpha emissions should be zero
        assert_eq!(
            all_dynamic_alpha_reserve_before,
            all_dynamic_alpha_reserve_after
        );
    });
}

/// This test is only applicable when prices add up to lower than 1, so TAO is emitted
///
#[test]
fn test_tao_subnet_emissions_are_proportional() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let subnet_count = 10;
        let hotkey = U256::from(1);
        let coldkey = U256::from(1);
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        for netuid in 1u16..=subnet_count {
            let lock_amount = 100_000_000_000 * netuid as u64;
            add_dynamic_network(netuid, 1, 1, 1, lock_amount);

            // Get amount of alpha in the network
            let alpha = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid);

            // Remove stake to make prices lower so that they add up to lower than threshold
            assert_ok!(SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                alpha * 19 / 20
            ));
        }

        // Check that we archieved price threshold requirement
        let price_sum = (1u16..=subnet_count)
            .map(|netuid| SubtensorModule::get_tao_per_alpha_price(netuid).to_num::<f64>())
            .sum::<f64>();
        assert!(price_sum < get_price_threshold());

        let block_emission = SubtensorModule::get_block_emission().unwrap_or(0);

        let total_subnet_tao_before: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .collect();
        let dynamic_alpha_reserve_before: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .collect();
        let total_total_subnet_tao_before: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();

        SubtensorModule::run_coinbase(1);

        let total_subnet_tao_after: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .collect();
        let dynamic_alpha_reserve_after: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .collect();

        // Ensure subnet TAO emissions are proportional to the their total TAO
        izip!(
            &dynamic_alpha_reserve_before,
            &total_subnet_tao_before,
            &dynamic_alpha_reserve_after,
            &total_subnet_tao_after,
        )
        .map(|(alpha_bef, tao_bef, alpha_af, tao_af)| {
            (tao_bef, tao_af - tao_bef, alpha_af - alpha_bef)
        })
        .for_each(|(tao_bef, emission, alpha_emission)| {
            let expected_emission =
                block_emission as f64 * (*tao_bef) as f64 / total_total_subnet_tao_before as f64;

            // In this test we don't expect any alpha emission, only TAO
            assert!(((emission as f64 - expected_emission).abs() / expected_emission) < 0.00001);
            assert!(alpha_emission == 0);
        });

        // Also ensure emissions add up to block emission
        let actual_block_emission: u64 = izip!(
            &total_subnet_tao_after,
            &dynamic_alpha_reserve_after,
            &total_subnet_tao_before,
            &dynamic_alpha_reserve_before,
        )
        .map(|(alpha_bef, tao_bef, alpha_af, tao_af)| alpha_bef + tao_bef - alpha_af - tao_af)
        .sum();
        assert_approx_eq!(
            block_emission as f64 / 1_000_000.,
            actual_block_emission as f64 / 1_000_000.
        );

        // Ensure total subnet tao increased by block emission
        let total_total_subnet_tao_after: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();
        assert_approx_eq!(
            (total_total_subnet_tao_after - total_total_subnet_tao_before) as f64 / 1_000.,
            block_emission as f64 / 1_000.
        );
    });
}

/// Tests that alpha emissions for every dynamic subnet is numerically equal to 
/// total block emission if sum of prices is higher than 1
#[test]
fn test_alpha_emission() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let subnet_count = 10;

        for netuid in 1u16..=subnet_count {
            let lock_amount = 100_000_000_000 * netuid as u64;
            add_dynamic_network(netuid, 1, 1, 1, lock_amount);
        }

        // Check that we archieved price threshold requirement
        let price_sum = (1u16..=subnet_count)
            .map(|netuid| SubtensorModule::get_tao_per_alpha_price(netuid).to_num::<f64>())
            .sum::<f64>();
        assert!(price_sum > get_price_threshold());

        let block_emission = SubtensorModule::get_block_emission().unwrap_or(0);

        let total_subnet_tao_before: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .collect();
        let dynamic_alpha_reserve_before: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .collect();
        let total_total_subnet_tao_before: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();

        SubtensorModule::run_coinbase(1);

        let total_subnet_tao_after: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .collect();
        let dynamic_alpha_reserve_after: Vec<u64> = (1u16..=subnet_count)
            .map(pallet_subtensor::DynamicAlphaReserve::<Test>::get)
            .collect();

        // Ensure subnet alpha emissions are all equal to block emission
        izip!(
            &dynamic_alpha_reserve_before,
            &total_subnet_tao_before,
            &dynamic_alpha_reserve_after,
            &total_subnet_tao_after,
        )
        .map(|(alpha_bef, tao_bef, alpha_af, tao_af)| {
            (tao_af - tao_bef, alpha_af - alpha_bef)
        })
        .for_each(|(emission, alpha_emission)| {
            let expected_alpha_emission = block_emission as f64;
            assert!(((alpha_emission as f64 - expected_alpha_emission).abs() / expected_alpha_emission) < 0.00001);
            assert!(emission == 0);
        });

        // Ensure total subnet tao didn't change
        let total_total_subnet_tao_after: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();
        assert!(total_total_subnet_tao_after == total_total_subnet_tao_before);
    });
}

/// Prices need to not converge to the same value, but should remain somewhat proportional to stakes
#[test]
fn test_prices_converge_proportionally() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let subnet_count = 10;
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        for netuid in 1u16..=subnet_count {
            let lock_amount = 100_000_000_000 * netuid as u64;
            add_dynamic_network(netuid, u16::MAX, 1, 1, lock_amount);     
        }

        let mut prev_sq_err = f64::MAX;
        let sq_err = || {
            let total_subnet_tao: u64 = (1u16..=subnet_count)
            .map(pallet_subtensor::TotalSubnetTAO::<Test>::get)
            .sum();

            let mut err = 0.;
            for netuid in 1u16..=subnet_count {
                let tao = pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid);
                let expected_price =
                    tao as f64 / total_subnet_tao as f64;
                let actual_price = SubtensorModule::get_tao_per_alpha_price(netuid);

                let diff = expected_price - actual_price.to_num::<f64>();
                err += diff * diff;
            }

            err
        };

        for block in 1u64..20000 {
            SubtensorModule::run_coinbase(block);

            // If this passes, the prices are likely to converge,
            // nonetheless if it doesn't this is the indication of something
            // being wrong.
            if block % 100 == 0 || block < 10 {
                let err = sq_err();
                assert!(err < prev_sq_err);
                prev_sq_err = err;
            }
        }
    });
}

/// Verify that total subnet TAO is always equal to dynamic TAO reserve, 
/// no matter if prices add up to <1 or >1, or epochs pass.
#[test]
fn test_total_tao_equals_dynamic_tao_reserve() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let subnet_count = 10;
        let tempo = 360;
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        for netuid in 1u16..=subnet_count {
            let lock_amount = 100_000_000_000 * netuid as u64;
            add_dynamic_network(netuid, tempo, 1, 1, lock_amount);     
        }

        let mut emissions_non_zero = false;
        let mut emissions_drained = false;

        // Prices greater or lower than threshold
        let mut prices_greater_than_one = false;
        let mut prices_lower_than_one = false;

        for block in 1u64..5000 {
            SubtensorModule::run_coinbase(block);

            for netuid in 1u16..=subnet_count {
                assert_eq!(
                    pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid),
                    pallet_subtensor::DynamicTAOReserve::<Test>::get(netuid)
                );
            }

            // Check if this test encountered a moment when emissions were drained (epoch)
            if !emissions_drained {
                if !emissions_non_zero {
                    emissions_non_zero = pallet_subtensor::PendingEmission::<Test>::iter().all(|(_, e)| e != 0);
                } else {
                    emissions_drained = pallet_subtensor::PendingEmission::<Test>::iter().any(|(_, e)| e == 0);
                }
            }

            // Check if this test encountered both prices > 1 and prices < 1
            if (1u16..=subnet_count)
                .map(|netuid| SubtensorModule::get_tao_per_alpha_price(netuid).to_num::<f64>())
                .sum::<f64>() > get_price_threshold() {
                prices_greater_than_one = true;
            } else {
                prices_lower_than_one = true;
            }
        }

        assert!(emissions_drained);
        assert!(prices_lower_than_one);
        assert!(prices_greater_than_one);
    });
}

/// Test that if sum of prices is a small step greater than threshold, we get alpha emission
///
#[test]
fn test_alpha_emission_edgecase_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let netuid_stao = 1;
        let netuid_dtao = 2;
        let coldkey = U256::from(1);
        let hotkey = U256::from(1);
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        // Create one stao and one dtao subnet
        let lock_amount = 100_000_000_000;
        create_staked_stao_network(netuid_stao, lock_amount, 0);
        add_dynamic_network(netuid_dtao, 1, 1, 1, lock_amount);

        // At this point both dtao price and price threshold are 0.5
        // If we add 1 rao stake, price will be slightly higher
        let alpha_to_add = 1;

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            netuid_dtao,
            alpha_to_add
        ));

        // Check that we archieved price threshold requirement
        let price_sum = SubtensorModule::get_tao_per_alpha_price(netuid_dtao).to_num::<f64>();
        assert!(price_sum > get_price_threshold());

        let block_emission = SubtensorModule::get_block_emission().unwrap_or(0);

        let total_subnet_tao_before = pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid_dtao);
        let dynamic_alpha_reserve_before = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid_dtao);

        SubtensorModule::run_coinbase(1);

        let total_subnet_tao_after = pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid_dtao);
        let dynamic_alpha_reserve_after = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid_dtao);

        // Ensure subnet alpha emissions are all equal to block emission
        let expected_alpha_emission = block_emission as f64;
        let alpha_emission = dynamic_alpha_reserve_after - dynamic_alpha_reserve_before;
        assert!(((alpha_emission as f64 - expected_alpha_emission).abs() / expected_alpha_emission) < 0.00001);

        // Ensure total subnet tao didn't change
        assert!(total_subnet_tao_after == total_subnet_tao_before);
    });
}

/// Test that if sum of prices is a small step lower than threshold, we get tao emission
///
#[test]
fn test_tao_emission_edgecase_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);

        let netuid_stao = 1;
        let netuid_dtao = 2;
        let coldkey = U256::from(1);
        let hotkey = U256::from(1);
        pallet_subtensor::SubnetOwnerLockPeriod::<Test>::set(0);

        // Create one stao and one dtao subnet
        let lock_amount = 100_000_000_000;
        create_staked_stao_network(netuid_stao, lock_amount, 0);
        add_dynamic_network(netuid_dtao, 1, 1, 1, lock_amount);

        // At this point both dtao price and price threshold are 0.5
        // If we remove 1 rao stake, price will be slightly lower
        let alpha_to_add = 1;

        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            netuid_dtao,
            alpha_to_add
        ));

        // Check that we archieved price threshold requirement
        let price_sum = SubtensorModule::get_tao_per_alpha_price(netuid_dtao).to_num::<f64>();
        assert!(price_sum < get_price_threshold());

        let total_subnet_tao_before = pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid_dtao);
        let dynamic_alpha_reserve_before = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid_dtao);

        SubtensorModule::run_coinbase(1);

        let total_subnet_tao_after = pallet_subtensor::TotalSubnetTAO::<Test>::get(netuid_dtao);
        let dynamic_alpha_reserve_after = pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid_dtao);

        // Ensure subnet alpha emissions are zero
        assert_eq!(dynamic_alpha_reserve_after, dynamic_alpha_reserve_before);

        // Ensure total subnet tao is not zero
        assert!(total_subnet_tao_after > total_subnet_tao_before);
    });
}

///////////////////////////////////////////////////////////////////
// Lock cost tests
//
// - Back to back lock price in the same block doubles
// - Lock price is the same as previous in 14 * 7200 blocks
// - Lock price is get_network_min_lock() in 28 * 7200 blocks
// - No panics or errors in 28 * 7200 + 1 blocks, lock price remains get_network_min_lock()
// - Cases when remaining balance after lock is ED+1, ED, ED-1,
//   - test what can_remove_balance_from_coldkey_account returns
//   - test that we don't register network and kill account
//
// get_network_lock_cost()

#[test]
fn test_lock_cost_doubles_in_same_block() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount1 = SubtensorModule::get_network_lock_cost();
        add_dynamic_network(1, 1, 1, 1, lock_amount1);
        let lock_amount2 = SubtensorModule::get_network_lock_cost();

        assert_eq!(lock_amount1 * 2, lock_amount2);
    });
}

#[test]
fn test_lock_cost_remains_same_after_lock_reduction_interval() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount1 = SubtensorModule::get_network_lock_cost();
        add_dynamic_network(1, 1, 1, 1, lock_amount1);
        step_block(SubtensorModule::get_lock_reduction_interval() as u16);
        let lock_amount2 = SubtensorModule::get_network_lock_cost();

        assert_eq!(lock_amount1, lock_amount2);
    });
}

#[test]
fn test_lock_cost_is_min_after_2_lock_reduction_intervals() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount1 = SubtensorModule::get_network_lock_cost();
        let min_lock_cost = SubtensorModule::get_network_min_lock();
        add_dynamic_network(1, 1, 1, 1, lock_amount1);
        step_block(2 * SubtensorModule::get_lock_reduction_interval() as u16);
        let lock_amount2 = SubtensorModule::get_network_lock_cost();

        assert_eq!(lock_amount2, min_lock_cost);
    });
}

#[test]
fn test_lock_cost_is_min_after_2_lock_reduction_intervals_2_subnets() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount1 = SubtensorModule::get_network_lock_cost();
        let min_lock_cost = SubtensorModule::get_network_min_lock();
        add_dynamic_network(1, 1, 1, 1, lock_amount1);
        let lock_amount2 = SubtensorModule::get_network_lock_cost();
        add_dynamic_network(2, 1, 1, 1, lock_amount2);
        step_block(2 * SubtensorModule::get_lock_reduction_interval() as u16);
        let lock_amount3 = SubtensorModule::get_network_lock_cost();

        assert_eq!(lock_amount3, min_lock_cost);
    });
}

#[test]
fn test_registration_after_2_lock_reduction_intervals_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount1 = SubtensorModule::get_network_lock_cost();
        add_dynamic_network(1, 1, 1, 1, lock_amount1);
        step_block(2 * SubtensorModule::get_lock_reduction_interval() as u16 + 1);
        add_dynamic_network(2, 1, 1, 1, lock_amount1);
    });
}

#[test]
fn test_registration_balance_minimal_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = SubtensorModule::get_network_lock_cost();
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, lock_amount);
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        let account = System::account(coldkey);
        assert_eq!(account.data.free, ExistentialDeposit::get());
    });
}

#[test]
fn test_registration_balance_minimal_plus_ed_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = SubtensorModule::get_network_lock_cost();
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            lock_amount + ExistentialDeposit::get(),
        );
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        let account = System::account(coldkey);
        assert_eq!(account.data.free, ExistentialDeposit::get());
    });
}

#[test]
fn test_registration_balance_minimal_plus_ed_plus_1_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = SubtensorModule::get_network_lock_cost();
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            lock_amount + ExistentialDeposit::get() + 1,
        );
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        let account = System::account(coldkey);
        assert_eq!(account.data.free, ExistentialDeposit::get() + 1);
    });
}

#[test]
fn test_registration_balance_minimal_plus_ed_minus_1_ok() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_target_stakes_per_interval(20);
        let lock_amount = SubtensorModule::get_network_lock_cost();
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            lock_amount + ExistentialDeposit::get() - 1,
        );
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        let account = System::account(coldkey);
        assert_eq!(account.data.free, ExistentialDeposit::get());
    });
}

#[ignore]
#[test]
fn test_stake_unstake_total_issuance() {
    new_test_ext(1).execute_with(|| {
        // init params.
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);
        let coldkey2 = U256::from(2);
        let lock_amount = 100_000_000_000_u64;
        let stake = 100_000_000_000_u64;
        let ed = ExistentialDeposit::get();

        // Register subnet and become a delegate.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, lock_amount);
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));
        assert_eq!(SubtensorModule::get_tao_reserve(1), lock_amount);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), lock_amount);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 1.0);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, stake);

        // Total issuance in balances pallet should be equal to stake + ED now
        assert_eq!(PalletBalances::total_issuance(), stake + ed);

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey,
            1,
            stake
        ));

        assert_eq!(SubtensorModule::get_tao_reserve(1), lock_amount + stake);
        let expected_alpha =
            lock_amount as f64 * stake as f64 / (lock_amount as f64 + stake as f64);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), expected_alpha as u64);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 4); // Price is increased from the stake operation.

        // Total issuance goes down to 2 * ED because we staked everything
        assert_eq!(PalletBalances::total_issuance(), 2 * ed);

        // Unstake everything
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey,
            1,
            expected_alpha as u64
        ));

        // Total issuance goes up to stake + ED because we unstaked everything and got the balance back
        assert_eq!(PalletBalances::total_issuance(), stake + ed);
    })
}