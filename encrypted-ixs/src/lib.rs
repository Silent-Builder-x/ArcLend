use arcis::*;

#[encrypted]
mod lending_engine {
    use arcis::*;

    pub struct Position {
        pub collateral_value: u64, // 加密的抵押品总价
        pub borrowed_amount: u64,  // 加密的已借金额
        pub ltv_threshold: u64,    // 抵押率阈值 (例如 80 代表 80%)
    }

    pub struct HealthStatus {
        pub is_liquidatable: u64,  // 1 = 可清算, 0 = 安全
        pub health_factor: u64,    // 加密的健康因子 (放大 100 倍存储)
    }

    #[instruction]
    pub fn check_liquidation(
        input_ctxt: Enc<Shared, Position>
    ) -> Enc<Shared, HealthStatus> {
        let pos = input_ctxt.to_arcis();
        
        // 核心公式：HealthFactor = (Collateral * LTV) / Debt
        let numerator = pos.collateral_value * pos.ltv_threshold;
        let denominator = pos.borrowed_amount;

        // 防止除以 0
        let safe_denominator = if denominator > 0 { denominator } else { 1u64 };
        let hf = numerator / safe_denominator;

        // 判定逻辑：
        let liquidatable = if hf < 100 { 1u64 } else { 0u64 };

        let result = HealthStatus {
            is_liquidatable: liquidatable,
            health_factor: hf,
        };

        input_ctxt.owner.from_arcis(result)
    }
}