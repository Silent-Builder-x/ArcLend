use arcis::*;

#[encrypted]
mod lending_engine {
    use arcis::*;

    pub struct PositionData {
        pub collateral_value: u64, // Encrypted collateral value (e.g. in USD)
        pub debt_value: u64,       // Encrypted debt value
        pub liquidation_threshold: u64, // Liquidation threshold (e.g. 80 = 80%)
    }

    pub struct HealthCheckResult {
        pub is_liquidatable: u64, // 1 = Yes, 0 = No
        pub health_factor: u64,   // HF * 100 (e.g. 150 = 1.5)
        pub shortfall: u64,       // Shortfall amount (used to calculate bad debt)
    }

    #[instruction]
    pub fn check_health(
        position_ctxt: Enc<Shared, PositionData>
    ) -> Enc<Shared, HealthCheckResult> {
        let pos = position_ctxt.to_arcis();
        
        // --- Core health factor calculation ---
        // MaxBorrow = Collateral * Threshold / 100
        let max_borrow = (pos.collateral_value * pos.liquidation_threshold) / 100;
        
        // HF = MaxBorrow * 100 / Debt (scaled by 100 for precision)
        // If Debt is 0, set HF to the maximum value (e.g. 99999)
        let safe_debt = if pos.debt_value == 0 { 1u64 } else { pos.debt_value };
        let hf = if pos.debt_value == 0 {
            99999u64 
        } else {
            (max_borrow * 100) / safe_debt
        };

        // --- Liquidation determination ---
        // If Debt > MaxBorrow, the position is liquidatable
        let is_unsafe = pos.debt_value > max_borrow;
        
        let liquidatable_flag = if is_unsafe { 1u64 } else { 0u64 };
        
        let shortfall_amount = if is_unsafe {
            pos.debt_value - max_borrow
        } else {
            0u64
        };

        let result = HealthCheckResult {
            is_liquidatable: liquidatable_flag,
            health_factor: hf,
            shortfall: shortfall_amount,
        };

        // Encrypt the result and return it to the requester (liquidator or the protocol itself)
        position_ctxt.owner.from_arcis(result)
    }
}