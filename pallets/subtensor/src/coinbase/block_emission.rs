use super::*;
use frame_support::traits::Get;
use substrate_fixed::types::I96F32;
use substrate_fixed::transcendental::log2;

impl<T: Config> Pallet<T> {

    /// Calculates the block emission based on the total issuance.
    ///
    /// This function computes the block emission by applying a logarithmic function
    /// to the total issuance of the network. The formula used takes into account
    /// the current total issuance and adjusts the emission rate accordingly to ensure
    /// a smooth issuance curve. The emission rate decreases as the total issuance increases,
    /// following a logarithmic decay.
    ///
    /// # Returns
    /// * 'Result<u64, &'static str>': The calculated block emission rate or error.
    ///
    pub fn get_block_emission() -> Result<u64, &'static str> {
        // Convert the total issuance to a fixed-point number for calculation.
        Self::get_block_emission_for_issuance(Self::get_total_issuance())
    }

    /// Returns the block emission for an issuance value.
    pub fn get_block_emission_for_issuance(issuance: u64) -> Result<u64, &'static str> {
        // Convert issuance to a float for calculations below.
        let total_issuance: I96F32 = I96F32::from_num(issuance);
        // Check to prevent division by zero when the total supply is reached
        // and creating an issuance greater than the total supply.
        if total_issuance >= I96F32::from_num(TotalSupply::<T>::get()) {
            return Ok(0);
        }
        // Calculate the logarithmic residual of the issuance against half the total supply.
        let residual: I96F32 = log2(
            I96F32::from_num(1.0)
                .checked_div(
                    I96F32::from_num(1.0)
                        .checked_sub(
                            total_issuance
                                .checked_div(
                                    I96F32::from_num(2.0)
                                        .saturating_mul(I96F32::from_num(10_500_000_000_000_000.0)),
                                )
                                .ok_or("Logarithm calculation failed")?,
                        )
                        .ok_or("Logarithm calculation failed")?,
                )
                .ok_or("Logarithm calculation failed")?,
        )
        .map_err(|_| "Logarithm calculation failed")?;
        // Floor the residual to smooth out the emission rate.
        let floored_residual: I96F32 = residual.floor();
        // Calculate the final emission rate using the floored residual.
        // Convert floored_residual to an integer
        let floored_residual_int: u64 = floored_residual.to_num::<u64>();
        // Multiply 2.0 by itself floored_residual times to calculate the power of 2.
        let mut multiplier: I96F32 = I96F32::from_num(1.0);
        for _ in 0..floored_residual_int {
            multiplier = multiplier.saturating_mul(I96F32::from_num(2.0));
        }
        let block_emission_percentage: I96F32 = I96F32::from_num(1.0).saturating_div(multiplier);
        // Calculate the actual emission based on the emission rate
        let block_emission: I96F32 = block_emission_percentage
            .saturating_mul(I96F32::from_num(DefaultBlockEmission::<T>::get()));
        // Convert to u64
        let block_emission_u64: u64 = block_emission.to_num::<u64>();
        if BlockEmission::<T>::get() != block_emission_u64 {
            BlockEmission::<T>::put(block_emission_u64);
        }
        Ok(block_emission_u64)
    }
}