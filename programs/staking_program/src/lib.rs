use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount, Mint, Transfer }
};

pub mod account;
pub mod error;
pub mod constants;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;

declare_id!("FbaMJWS14yAPH68LwFAHxaBSukgBHnAY9VaEfhFxWerb");

#[program]
pub mod staking_program {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        global_bump: u8,
        pool_wallet_bump: u8
    ) -> ProgramResult {
        ctx.accounts.global_lottery_pool.load_init()?;
        Ok(())
    }

    pub fn initialize_lottery_pool(
        ctx: Context<InitializeLotteryPool>
    ) -> ProgramResult {
        let mut lottery_pool = ctx.accounts.user_lottery_pool.load_init()?;
        lottery_pool.owner = ctx.accounts.owner.key();
        Ok(())
    }

    pub fn initialize_fixed_pool(
        ctx: Context<InitializeFixedPool>
    ) -> ProgramResult {
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_init()?;
        fixed_pool.owner = ctx.accounts.owner.key();
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_lottery_pool, &ctx.accounts.owner))]
    pub fn stake_nft_to_lottery(
        ctx: Context<StakeNftToLottery>, 
        global_bump: u8,
        staked_nft_bump: u8,
    ) -> ProgramResult {
        
        let timestamp = Clock::get()?.unix_timestamp;

        let staked_item = StakedNFT {
            nft_addr: ctx.accounts.nft_mint.key(),
            stake_time: timestamp,
        };
        let mut lottery_pool = ctx.accounts.user_lottery_pool.load_mut()?;
        lottery_pool.add_nft(staked_item);

        let mut global_lottery_pool = ctx.accounts.global_lottery_pool.load_mut()?;
        global_lottery_pool.add_nft(Item {
            owner: ctx.accounts.owner.key(),
            nft_addr: ctx.accounts.nft_mint.key(),
            stake_time: timestamp,
        });

        ctx.accounts.global_authority.lottery_nft_count += 1;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_token_account.to_account_info(),
            to: ctx.accounts.dest_nft_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info()
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(
            transfer_ctx,
            1
        )?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_lottery_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft_from_lottery(
        ctx: Context<WithdrawNftFromLottery>, 
        global_bump: u8,
        staked_nft_bump: u8,
        withdraw_index: u64
    ) -> ProgramResult {
        
        let timestamp = Clock::get()?.unix_timestamp;
    
        let mut lottery_pool = ctx.accounts.user_lottery_pool.load_mut()?;
        lottery_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(), 
            timestamp
        )?;

        let mut global_lottery_pool = ctx.accounts.global_lottery_pool.load_mut()?;
        global_lottery_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(),
            withdraw_index
        )?;

        ctx.accounts.global_authority.lottery_nft_count -= 1;

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.staked_nft_token_account.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            1
        )?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn stake_nft_to_fixed(
        ctx: Context<StakeNftToFixed>, 
        global_bump: u8,
        staked_nft_bump: u8,
    ) -> ProgramResult {
        
        let timestamp = Clock::get()?.unix_timestamp;

        let staked_item = StakedNFT {
            nft_addr: ctx.accounts.nft_mint.key(),
            stake_time: timestamp,
        };
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        fixed_pool.add_nft(staked_item);

        ctx.accounts.global_authority.fixed_nft_count += 1;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_token_account.to_account_info(),
            to: ctx.accounts.dest_nft_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info()
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(
            transfer_ctx,
            1
        )?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft_from_fixed(
        ctx: Context<WithdrawNftFromFixed>, 
        global_bump: u8,
        staked_nft_bump: u8,
        pool_wallet_bump: u8
    ) -> ProgramResult {
        
        let timestamp = Clock::get()?.unix_timestamp;
    
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        let reward: u64 = fixed_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(), 
            timestamp
        )?;

        fixed_pool.pending_reward += reward;

        ctx.accounts.global_authority.fixed_nft_count -= 1;

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.staked_nft_token_account.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info()
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(
            transfer_ctx,
            1
        )?;
/*
        sol_transfer_with_signer(
            ctx.accounts.pool_wallet.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[POOL_WALLET_SEED.as_ref(), &[pool_wallet_bump]]],
            reward
        )?;
*/
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
        staked_nft_bump: u8,
        pool_wallet_bump: u8
    ) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
    
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        let reward: u64 = fixed_pool.claim_reward(
            timestamp
        )?;

        sol_transfer_with_signer(
            ctx.accounts.pool_wallet.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[POOL_WALLET_SEED.as_ref(), &[pool_wallet_bump]]],
            reward
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(global_bump: u8, pool_wallet_bump: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(zero)]
    pub global_lottery_pool: AccountLoader<'info, GlobalLotteryPool>,

    #[account(
        seeds = [POOL_WALLET_SEED.as_ref()],
        bump = pool_wallet_bump,
    )]
    pub pool_wallet: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct InitializeLotteryPool<'info> {
    #[account(zero)]
    pub user_lottery_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}


#[derive(Accounts)]
pub struct InitializeFixedPool<'info> {
    #[account(zero)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct StakeNftToLottery<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_lottery_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub global_lottery_pool: AccountLoader<'info, GlobalLotteryPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump,
        token::mint = nft_mint,
        token::authority = global_authority
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct WithdrawNftFromLottery<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_lottery_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub global_lottery_pool: AccountLoader<'info, GlobalLotteryPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct StakeNftToFixed<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump,
        token::mint = nft_mint,
        token::authority = global_authority
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8, pool_wallet_bump: u8)]
pub struct WithdrawNftFromFixed<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        seeds = [POOL_WALLET_SEED.as_ref()],
        bump = pool_wallet_bump,
    )]
    pub pool_wallet: AccountInfo<'info>,

    #[account(
        mut,
        constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8, pool_wallet_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        seeds = [POOL_WALLET_SEED.as_ref()],
        bump = pool_wallet_bump,
    )]
    pub pool_wallet: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// Access control modifiers
fn user(pool_loader: &AccountLoader<UserPool>, user: &AccountInfo) -> Result<()> {
    let user_pool = pool_loader.load()?;
    require!(user_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}