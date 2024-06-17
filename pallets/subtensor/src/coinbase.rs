
use super::*;
use frame_support::IterableStorageDoubleMap;
// use sp_runtime::Saturating;
use substrate_fixed::types::{I64F64, I96F32};

impl<T: Config> Pallet<T> {
    /// The `coinbase` function performs a four-part emission distribution process involving
    /// subnets, epochs, hotkeys, and nominators.
    // It is divided into several steps, each handling a specific part of the distribution:

    // Step 1: Compute the block-wise emission for each subnet.
    // This involves calculating how much (TAO) should be emitted into each subnet using the
    // root epoch function.

    // Step 2: Accumulate the subnet block emission.
    // After calculating the block-wise emission, these values are accumulated to keep track
    // of how much each subnet should emit before the next distribution phase. This accumulation
    // is a running total that gets updated each block.

    // Step 3: Distribute the accumulated emissions through epochs.
    // Subnets periodically distribute their accumulated emissions to hotkeys (active validators/miners)
    // in the network on a `tempo` --- the time between epochs. This step runs Yuma consensus to
    // determine how emissions are split among hotkeys based on their contributions and roles.
    // The accumulation of hotkey emissions is done through the `accumulate_hotkey_emission` function.
    // The function splits the rewards for a hotkey amongst itself and its `parents`. The parents are
    // the hotkeys that are delegating their stake to the hotkey.

    // Step 4: Further distribute emissions from hotkeys to nominators.
    // Finally, the emissions received by hotkeys are further distributed to their nominators,
    // who are stakeholders that support the hotkeys.
    pub fn run_coinbase() {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_subnet_netuids();

        // --- 2. Run the root epoch function which computes the block emission for each subnet.
        // coinbase --> root() --> subnet_block_emission
        match Self::root_epoch(current_block) {
            Ok(_) => (),
            Err(e) => {
                log::trace!("Error while running root epoch: {:?}", e);
            }
        }

        // --- 3. Drain the subnet block emission and accumulate it as subnet emission, which increases until the tempo is reached in #4.
        // subnet_blockwise_emission -> subnet_pending_emission
        for netuid in subnets.clone().iter() {
            // --- 3.1 Get the network's block-wise emission amount.
            // This value is newly minted TAO which has not reached staking accounts yet.
            let subnet_blockwise_emission: u64 = EmissionValues::<T>::get(*netuid);

            // --- 3.2 Accumulate the subnet emission on the subnet.
            PendingEmission::<T>::mutate(*netuid, |subnet_emission| {
                *subnet_emission = subnet_emission.saturating_add(subnet_blockwise_emission);
            });
        }

        // --- 4. Drain the accumulated subnet emissions, pass them through the epoch().
        // Before accumulating on the hotkeys the function redistributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        for netuid in subnets.clone().iter() {
            // 4.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch(*netuid, current_block) {
                // 4.2 Drain the subnet emission.
                let subnet_emission: u64 = PendingEmission::<T>::get(*netuid);
                PendingEmission::<T>::insert(*netuid, 0);

                // 4.3 Pass emission through epoch() --> hotkey emission.
                let hotkey_emission: Vec<(T::AccountId, u64, u64)> =
                    Self::epoch(*netuid, subnet_emission);

                // 4.3 Accumulate the tuples on hotkeys.
                for (hotkey, mining_emission, validator_emission) in hotkey_emission {
                    // 4.4 Accumulate the emission on the hotkey and parent hotkeys.
                    Self::accumulate_hotkey_emission(
                        &hotkey,
                        *netuid,
                        mining_emission.saturating_add(validator_emission),
                    );
                }
            }
        }

        // --- 5. Drain the accumulated hotkey emissions through to the nominators.
        // The hotkey takes a proportion of the emission, the remainder is drained through to the nominators.
        // We keep track of the last stake increase event for accounting purposes.
        // hotkeys --> nominators.
        for (index, ( hotkey, hotkey_emission )) in PendingdHotkeyEmission::<T>::iter().enumerate() {

            // Check for zeros.
            // remove zero values.
            if hotkey_emission == 0 { continue; }

            // --- 5.1 Check if we should drain the hotkey emission on this block.
            // Should be true only once every 7200 blocks.
            if Self::should_drain_hotkey( index as u64 , current_block ) {

                // --- 5.2 Drain the hotkey emission and distribute it to nominators.
                Self::drain_hotkey_emission(&hotkey, hotkey_emission, current_block);

                // --- 5.3 Increase total issuance
                TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(hotkey_emission));
            }
        }
    }

    /// Accumulates the mining and validator emissions on a hotkey and distributes the validator emission among its parents.
    ///
    /// This function is responsible for accumulating the mining and validator emissions associated with a hotkey onto a hotkey.
    /// It first calculates the total stake of the hotkey, considering the stakes contributed by its parents and reduced by its children.
    /// It then retrieves the list of parents of the hotkey and distributes the validator emission proportionally based on the stake contributed by each parent.
    /// The remaining validator emission, after distribution to the parents, along with the mining emission, is then added to the hotkey's own accumulated emission.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey for which emissions are being calculated.
    /// * `netuid` - The unique identifier of the network to which the hotkey belongs.
    /// * `mining_emission` - The amount of mining emission allocated to the hotkey.
    /// * `validator_emission` - The amount of validator emission allocated to the hotkey.
    ///
    pub fn accumulate_hotkey_emission(hotkey: &T::AccountId, netuid: u16, emission: u64) {
        // --- 1. First, calculate the hotkey's share of the emission.
        let take_proportion: I64F64 = I64F64::from_num(Delegates::<T>::get(hotkey))
            .saturating_div(I64F64::from_num(u16::MAX));
        let hotkey_take: u64 = take_proportion
            .saturating_mul(I64F64::from_num(emission))
            .to_num::<u64>();

        // --- 2. Compute the remaining emission after the hotkey's share is deducted.
        let emission_minus_take: u64 = emission.saturating_sub(hotkey_take);

        // --- 3. Track the remaining emission for accounting purposes.
        let mut remaining_emission: u64 = emission_minus_take;

        // --- 4. Calculate the total stake of the hotkey, adjusted by the stakes of parents and children.
        // Parents contribute to the stake, while children reduce it.
        // If this value is zero, no distribution to anyone is necessary.
        let total_hotkey_stake: u64 = Self::get_stake_with_children_and_parents(hotkey, netuid);
        if total_hotkey_stake != 0 {

            // --- 5. If the total stake is not zero, iterate over each parent to determine their contribution to the hotkey's stake,
            // and calculate their share of the emission accordingly.
            for (proportion, parent) in ParentKeys::<T>::get(hotkey, netuid) {

                // --- 5.1 Retrieve the parent's stake. This is the raw stake value including nominators.
                let parent_stake: u64 = Self::get_total_stake_for_hotkey(&parent);

                // --- 5.2 Calculate the portion of the hotkey's total stake contributed by this parent.
                // Then, determine the parent's share of the remaining emission.
                let stake_from_parent: I96F32 = I96F32::from_num(parent_stake).saturating_mul(
                    I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX)),
                );
                let proportion_from_parent: I96F32 =
                    stake_from_parent.saturating_div(I96F32::from_num(total_hotkey_stake));
                let parent_emission_take: u64 = proportion_from_parent
                    .saturating_mul(I96F32::from_num(emission_minus_take))
                    .to_num::<u64>();

                // --- 5.5. Accumulate emissions for the parent hotkey.
                PendingdHotkeyEmission::<T>::mutate(parent, |parent_accumulated| {
                    *parent_accumulated = parent_accumulated.saturating_add(parent_emission_take)
                });

                // --- 5.6. Subtract the parent's share from the remaining emission for this hotkey.
                remaining_emission = remaining_emission.saturating_sub(parent_emission_take);
            }
        }

        // --- 6. Add the remaining emission plus the hotkey's initial take to the pending emission for this hotkey.
        PendingdHotkeyEmission::<T>::mutate(hotkey, |hotkey_accumulated| {
            *hotkey_accumulated =
                hotkey_accumulated.saturating_add(remaining_emission.saturating_add(hotkey_take))
        });
    }

    //. --- 4. Drains the accumulated hotkey emission through to the nominators. The hotkey takes a proportion of the emission.
    /// The remainder is drained through to the nominators keeping track of the last stake increase event to ensure that the hotkey does not
    /// gain more emission than it's stake since the last drain.
    /// hotkeys --> nominators.
    ///
    /// 1. It resets the accumulated emissions for the hotkey to zero.
    /// 4. It calculates the total stake for the hotkey and determines the hotkey's own take from the emissions based on its delegation status.
    /// 5. It then calculates the remaining emissions after the hotkey's take and distributes this remaining amount proportionally among the hotkey's nominators.
    /// 6. Each nominator's share of the emissions is added to their stake, but only if their stake was not manually increased since the last emission drain.
    /// 7. Finally, the hotkey's own take and any undistributed emissions are added to the hotkey's total stake.
    ///
    /// This function ensures that emissions are fairly distributed according to stake proportions and delegation agreements, and it updates the necessary records to reflect these changes.
    pub fn drain_hotkey_emission(hotkey: &T::AccountId, emission: u64, block_number: u64) {
        // --- 1.0 Drain the hotkey emission.
        PendingdHotkeyEmission::<T>::insert(hotkey, 0);

        // --- 2 Retrieve the last time this hotkey's emissions were drained.
        let last_hotkey_emission_drain: u64 = LastHotkeyEmissionDrain::<T>::get(hotkey);

        // --- 3 Update the block value to the current block number.
        LastHotkeyEmissionDrain::<T>::insert(hotkey, block_number);

        // --- 4 Retrieve the total stake for the hotkey from all nominations.
        let total_hotkey_stake: u64 = Self::get_total_stake_for_hotkey(hotkey);

        // --- 5 Calculate the emission take for the hotkey.
        let take_proportion: I64F64 = I64F64::from_num(Delegates::<T>::get(hotkey))
            .saturating_div(I64F64::from_num(u16::MAX));
        let hotkey_take: u64 =
            (take_proportion.saturating_mul(I64F64::from_num(emission))).to_num::<u64>();

        // --- 6 Compute the remaining emission after deducting the hotkey's take.
        let emission_minus_take: u64 = emission.saturating_sub(hotkey_take);

        // --- 7 Calculate the remaining emission after the hotkey's take.
        let mut remainder: u64 = emission_minus_take;

        // --- 8 Iterate over each nominator.
        for (nominator, nominator_stake) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                hotkey,
            )
        {
            // --- 9 Check if the stake was manually increased by the user since the last emission drain for this hotkey.
            // If it was, skip this nominator as they will not receive their proportion of the emission.
            if LastAddStakeIncrease::<T>::get(hotkey, nominator.clone())
                > last_hotkey_emission_drain
            {
                continue;
            }

            // --- 10 Calculate this nominator's share of the emission.
            let nominator_emission: I64F64 = I64F64::from_num(emission_minus_take)
                .saturating_mul(I64F64::from_num(nominator_stake))
                .saturating_div(I64F64::from_num(total_hotkey_stake));

            // --- 11 Increase the stake for the nominator.
            Self::increase_stake_on_coldkey_hotkey_account(
                &nominator,
                hotkey,
                nominator_emission.to_num::<u64>(),
            );

            // --- 12 Subtract the nominator's emission from the remainder.
            remainder = remainder.saturating_sub(nominator_emission.to_num::<u64>());
        }

        // --- 13 Finally, add the stake to the hotkey itself, including its take and the remaining emission.
        Self::increase_stake_on_hotkey_account(hotkey, hotkey_take.saturating_add(remainder));
    }

    ///////////////
    /// Helpers ///
    ///////////////

    /// Determines whether the hotkey emission should be drained based on the current block and index.
    ///
    /// # Arguments
    /// * `hotkey_i` - The hotkey identifier.
    /// * `index` - The index of the hotkey in the iterable storage.
    /// * `block` - The current block number.
    ///
    /// # Returns
    /// * `bool` - True if the hotkey emission should be drained, false otherwise.
    pub fn should_drain_hotkey(index: u64, block: u64) -> bool {
        return block % 7200 == index % 7200; // True once per day for each index assuming we run this every block.
    }

    /// Checks if the epoch should run for a given subnet based on the current block.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `bool` - True if the epoch should run, false otherwise.
    pub fn should_run_epoch(netuid: u16, current_block: u64) -> bool {
        return Self::blocks_until_next_epoch(netuid, Self::get_tempo(netuid), current_block) == 0;
    }

    /// Helper function which returns the number of blocks remaining before we will run the epoch on this
    /// network. Networks run their epoch when (block_number + netuid + 1 ) % (tempo + 1) = 0
    /// tempo | netuid | # first epoch block
    ///   1        0               0
    ///   1        1               1
    ///   2        0               1
    ///   2        1               0
    ///   100      0              99
    ///   100      1              98
    /// Special case: tempo = 0, the network never runs.
    ///
    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        if tempo == 0 {
            return u64::MAX;
        }
        (tempo as u64).saturating_sub(
            (block_number.saturating_add((netuid as u64).saturating_add(1)))
                % (tempo as u64).saturating_add(1),
        )
    }
}
