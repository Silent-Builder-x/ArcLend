use arcis::*;

#[encrypted]
mod lending_engine {
    use arcis::*;

    pub struct PositionData {
        pub collateral_value: u64, // 加密抵押品价值 (e.g. in USD)
        pub debt_value: u64,       // 加密债务价值
        pub liquidation_threshold: u64, // 清算阈值 (e.g. 80 = 80%)
    }

    pub struct HealthCheckResult {
        pub is_liquidatable: u64, // 1 = Yes, 0 = No
        pub health_factor: u64,   // HF * 100 (e.g. 150 = 1.5)
        pub shortfall: u64,       // 资不抵债的差额 (用于计算坏账)
    }

    #[instruction]
    pub fn check_health(
        position_ctxt: Enc<Shared, PositionData>
    ) -> Enc<Shared, HealthCheckResult> {
        let pos = position_ctxt.to_arcis();
        
        // --- 核心健康度计算 ---
        // MaxBorrow = Collateral * Threshold / 100
        let max_borrow = (pos.collateral_value * pos.liquidation_threshold) / 100;
        
        // HF = MaxBorrow * 100 / Debt (放大100倍保持精度)
        // 如果 Debt 为 0，HF 设为最大值 (e.g. 99999)
        let safe_debt = if pos.debt_value == 0 { 1u64 } else { pos.debt_value };
        let hf = if pos.debt_value == 0 {
            99999u64 
        } else {
            (max_borrow * 100) / safe_debt
        };

        // --- 清算判定 ---
        // 如果 Debt > MaxBorrow，则可清算
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

        // 结果加密返回给请求者 (清算人或协议本身)
        position_ctxt.owner.from_arcis(result)
    }
}