use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub lottery_nft_count: u64, // 8
    pub fixed_nft_count: u64   // 8
}

#[zero_copy]
#[derive(Default)]
pub struct Item {
    // 72
    pub owner: Pubkey,      // 32
    pub nft_addr: Pubkey,   // 32
    pub stake_time: i64     // 8
}

#[account(zero_copy)]
pub struct GlobalLotteryPool {
    // 360_016
    pub lottery_items: [Item; NFT_TOTAL_COUNT], // 72 * 5000 = 360_000
    pub item_count: u64                         // 8
}
impl Default for GlobalLotteryPool {
    #[inline]
    fn default() -> GlobalLotteryPool {
        GlobalLotteryPool {
            lottery_items: [
                Item {
                    ..Default::default()
                }; NFT_TOTAL_COUNT
            ],
            item_count: 0
        }
    }
}
impl GlobalLotteryPool {
    pub fn add_nft(&mut self, item: Item) {
        self.lottery_items[self.item_count as usize] = item;
        self.item_count += 1;
    }
    pub fn remove_nft(&mut self, owner: Pubkey, nft_mint: Pubkey, index: u64) -> Result<()> {
        require!(self.item_count > index, StakingError::IndexOverflow);
        require!(self.lottery_items[index as usize].nft_addr.eq(&nft_mint), StakingError::InvalidNFTAddress);
        require!(self.lottery_items[index as usize].owner.eq(&owner), StakingError::InvalidOwner);
        // remove nft
        if index != self.item_count - 1 {
            let last_idx = self.item_count - 1;
            self.lottery_items[index as usize] = self.lottery_items[last_idx as usize];
        }
        self.item_count -= 1;
        Ok(())
    }
}

#[zero_copy]
#[derive(Default, PartialEq)]
pub struct StakedNFT {
    pub nft_addr: Pubkey,      // 32
    pub stake_time: i64,       // 8
}

#[account(zero_copy)]
pub struct UserPool {
    // 2064
    pub owner: Pubkey,                       // 32
    pub item_count: u64,                     // 8
    pub items: [StakedNFT; NFT_STAKE_MAX_COUNT],   // 40 * 50 = 2000
    pub reward_time: i64,                    // 8
    pub pending_reward: u64                  // 8
}
impl Default for UserPool {
    #[inline]
    fn default() -> UserPool {
        UserPool {
            owner: Pubkey::default(),
            item_count: 0,
            items: [
                StakedNFT {
                    ..Default::default()
                }; NFT_STAKE_MAX_COUNT
            ],
            reward_time: 0,
            pending_reward: 0
        }
    }
}

impl UserPool {
    pub fn add_nft(&mut self, item: StakedNFT) {
        self.items[self.item_count as usize] = item;
        self.item_count += 1;
    }
    pub fn remove_nft(&mut self, owner: Pubkey, nft_mint: Pubkey, now: i64) -> Result<u64> {
        require!(self.owner.eq(&owner), StakingError::InvalidOwner);
        let mut withdrawn: u8 = 0;
        let mut reward: u64 = 0;
        for i in 0..self.item_count {
            let index = i as usize;
            if self.items[index].nft_addr.eq(&nft_mint) {
                
                //require!(self.items[index].stake_time + LIMIT_PERIOD <= now, StakingError::InvalidWithdrawTime);
                let mut last_reward_time = self.reward_time;
                if last_reward_time < self.items[index].stake_time {
                    last_reward_time = self.items[index].stake_time;
                }
                reward = (((now - last_reward_time) / DAY) as u64) * REWARD_PER_DAY;
                // remove nft
                if i != self.item_count - 1 {
                    let last_idx = self.item_count - 1;
                    self.items[index] = self.items[last_idx as usize];
                }
                self.item_count -= 1;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);
        Ok(reward)
    }
    pub fn claim_reward(&mut self, now: i64) -> Result<u64> {
        let mut total_reward: u64 = 0;
        for i in 0..self.item_count {
            let index = i as usize;
            //require!(self.items[index].stake_time + LIMIT_PERIOD <= now, StakingError::InvalidWithdrawTime);
            let mut last_reward_time = self.reward_time;
            if last_reward_time < self.items[index].stake_time {
                last_reward_time = self.items[index].stake_time;
            }
            let reward = (((now - last_reward_time) / DAY) as u64) * REWARD_PER_DAY;
            total_reward += reward;
        }
        total_reward += self.pending_reward;
        self.pending_reward = 0;
        self.reward_time = now;
        Ok(total_reward)
    }
}

