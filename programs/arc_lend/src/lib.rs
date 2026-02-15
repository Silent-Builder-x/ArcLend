use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_CHECK: u32 = comp_def_offset("check_health");

declare_id!("jFkuCZrSgfeKWEY2T5G9iNKBdmZqHXPE8NJX7nZycr9");

#[arcium_program]
pub mod arclend {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [New] Open a lending position
    pub fn open_position(ctx: Context<OpenPosition>) -> Result<()> {
        let pos = &mut ctx.accounts.position;
        pos.owner = ctx.accounts.owner.key();
        pos.encrypted_collateral = [0u8; 32]; // Initialize to 0
        pos.encrypted_debt = [0u8; 32];       // Initialize to 0
        pos.liquidation_threshold = 80;       // 80% LTV
        Ok(())
    }

    /// [Simulation] Update position (deposit/borrow)
    /// In a real MPC application, this should be a homomorphic addition operation
    /// For simplicity in the demo, we allow users to directly overwrite the encrypted state
    pub fn update_position(
        ctx: Context<UpdatePosition>,
        new_collateral: [u8; 32],
        new_debt: [u8; 32],
    ) -> Result<()> {
        let pos = &mut ctx.accounts.position;
        pos.encrypted_collateral = new_collateral;
        pos.encrypted_debt = new_debt;
        msg!("Position updated. Values are encrypted.");
        Ok(())
    }

    /// [Core] Trigger liquidation check
    /// Anyone can call this, but the result must be verified through MPC
    pub fn check_health(
        ctx: Context<CheckHealth>,
        computation_offset: u64,
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let pos = &ctx.accounts.position;
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            // Pass in the ciphertext stored on-chain
            .encrypted_u64(pos.encrypted_collateral)
            .encrypted_u64(pos.encrypted_debt)
            .plaintext_u64(pos.liquidation_threshold) // Threshold can be plaintext
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![CheckHealthCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "check_health")]
    pub fn check_health_callback(
        ctx: Context<CheckHealthCallback>,
        output: SignedComputationOutputs<CheckHealthOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(CheckHealthOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // Parse results: { is_liquidatable, health_factor, shortfall }
        let liq_bytes: [u8; 8] = o.ciphertexts[0][0..8].try_into().unwrap();
        let hf_bytes: [u8; 8] = o.ciphertexts[1][0..8].try_into().unwrap();

        let is_liquidatable = u64::from_le_bytes(liq_bytes) == 1;
        let hf = u64::from_le_bytes(hf_bytes);

        if is_liquidatable {
            msg!("🚨 ALERT: Position is UNDERWATER! (HF: {})", hf);
            msg!("Liquidation logic triggered via CPI...");
            // Execute liquidation here: transfer collateral to the liquidator
        } else {
            msg!("✅ Position is HEALTHY. (HF: {})", hf);
        }
        
        emit!(HealthCheckEvent {
            position: ctx.accounts.computation_account.key(), // Simplified association
            is_liquidatable,
            health_factor: hf,
        });
        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(
        init, 
        payer = owner, 
        space = 8 + 32 + 32 + 32 + 8 + 1,
        seeds = [b"pos", owner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, PositionAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePosition<'info> {
    #[account(mut, has_one = owner)]
    pub position: Account<'info, PositionAccount>,
    pub owner: Signer<'info>,
}

#[account]
pub struct PositionAccount {
    pub owner: Pubkey,
    pub encrypted_collateral: [u8; 32],
    pub encrypted_debt: [u8; 32],
    pub liquidation_threshold: u64,
}

#[queue_computation_accounts("check_health", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CheckHealth<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub position: Account<'info, PositionAccount>, // Read the target position
    
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("check_health")]
#[derive(Accounts)]
pub struct CheckHealthCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("check_health", payer)]
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: Lut
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Lut Prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct HealthCheckEvent {
    pub position: Pubkey,
    pub is_liquidatable: bool,
    pub health_factor: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}