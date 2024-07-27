use std::u128;

use near_sdk::borsh::{self, BorshDeserialize};
use near_sdk::serde::{self, Serialize};
use near_sdk::store::{LookupMap, LookupSet, UnorderedMap};
use near_sdk::{env, near, BorshStorageKey, NearSchema, Promise};
use near_sdk::{NearToken, AccountId, borsh::BorshSerialize};

const FIELD_WIDTH: u32 = 100;
const FIELD_HEIGHT: u32 = 100;
const PIXEL_START_PRICE: NearToken = NearToken::from_millinear(1);
const PIXEL_PRICE_INCREASE: u32 = 2;
const GAME_PERIOD: u64 = 300_000;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "borsh")]
enum StorageKey {
    FieldRow { row_id: u32 },
    Field,
    AccountCells,
    AccountWithdraw,
}

#[near(contract_state)]
pub struct Contract {
    field: UnorderedMap<u32, UnorderedMap<u32, PixelInfo>>,
    account_cells: LookupMap<AccountId, u32>,
    account_withdraw: LookupSet<AccountId>,
    start_timestamp: u64,
    last_change_block_height: u64,
    reward_dist_balance: u128,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            field: UnorderedMap::new(StorageKey::Field),
            account_cells: LookupMap::new(StorageKey::AccountCells),
            account_withdraw: LookupSet::new(StorageKey::AccountWithdraw),
            start_timestamp: env::block_timestamp_ms(),
            reward_dist_balance: env::account_balance().as_yoctonear(),
            last_change_block_height: env::block_height(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, NearSchema)]
#[borsh(crate = "borsh")]
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
struct PixelInfo {
    owner: AccountId,
    price: NearToken,
    color: u32,
    position_x: u32,
    position_y: u32,
}

#[near]
impl Contract {
    #[init]
    #[private] // only callable by the contract's account
    pub fn init() -> Self {
        Self {
            field: UnorderedMap::new(StorageKey::Field),
            account_cells: LookupMap::new(StorageKey::AccountCells),
            account_withdraw: LookupSet::new(StorageKey::AccountWithdraw),
            start_timestamp: env::block_timestamp_ms(),
            reward_dist_balance: env::account_balance().as_yoctonear(),
            last_change_block_height: env::block_height(),
        }
    }

    pub fn number_of_blocks_unchanged(&self) -> u64 {
        env::block_height() - self.last_change_block_height
    }

    pub fn get_pixel(&self, position_x: u32, position_y: u32) -> Option<PixelInfo> {
        return if let Some(row) = self.field.get(&position_y) {
            row.get(&position_x).map(|p| p.clone())
        } else {
            None
        };
    }

    pub fn get_field_row(&self, position_y: u32) -> Vec<PixelInfo> {
        self.field
            .get(&position_y)
            .unwrap_or(&UnorderedMap::new(b"b"))
            .values()
            .map(|pixel| pixel.clone())
            .collect()
    }

    pub fn is_game_finished(&self) -> bool {
        return if self.game_finish_timestamp() > env::block_timestamp_ms() {
            false
        } else {
            true
        };
    }

    pub fn game_finish_timestamp(&self) -> u64 {
        self.start_timestamp + GAME_PERIOD
    }

    #[payable]
    pub fn set_pixel(&mut self, color: u32, position_x: u32, position_y: u32) {
        if self.is_game_finished() {
            panic!("The game is already finished")
        }

        if FIELD_HEIGHT < position_y || FIELD_WIDTH < position_x {
            panic!("Incorect coordinates");
        }

        let row = if let Some(row) = self.field.get_mut(&position_y) {
            row
        } else {
            let new_row = UnorderedMap::new(StorageKey::FieldRow { row_id: position_y });
            self.field.insert(position_y, new_row);
            self.field.get_mut(&position_y).unwrap()
        };

        let mut d_cell = 1;
        let pixel_info = if let Some(pixel_info) = row.get_mut(&position_x) {
            pixel_info
        } else {
            let new_pixel = PixelInfo {owner:env::predecessor_account_id(),price:PIXEL_START_PRICE,color:0, position_x, position_y };
            row.insert(position_x, new_pixel);
            d_cell = 0;
            row.get_mut(&position_x).unwrap()
        };

        let new_price = pixel_info.price.as_yoctonear() * PIXEL_PRICE_INCREASE as u128; 
        let attached_deposit = env::attached_deposit().as_yoctonear();
        if attached_deposit < new_price {
            panic!("Not enough tokens attached");
        }

        let mut prev_number_of_cells = self.account_cells.get(&pixel_info.owner).unwrap_or(&0).clone();
        prev_number_of_cells -= d_cell;
        self.account_cells.insert(pixel_info.owner.clone(), prev_number_of_cells);

        pixel_info.owner = env::predecessor_account_id();
        pixel_info.price = NearToken::from_yoctonear(new_price);
        pixel_info.color = color;

        let mut number_of_cells = self.account_cells.get(&pixel_info.owner).unwrap_or(&0).clone();
        number_of_cells += 1;
        self.account_cells.insert(pixel_info.owner.clone(), number_of_cells);

        self.reward_dist_balance = env::account_balance().as_yoctonear();
        self.last_change_block_height = env::block_height();
    }

    pub fn withdraw(&mut self) {
        if !self.is_game_finished() {
            panic!("The game is still going on");
        }
        
        let account_id = env::predecessor_account_id();

        if self.account_withdraw.contains(&account_id) {
            panic!("You have already withdrawn the money");
        }

        let number_of_user_cells = self.account_cells.get(&account_id).unwrap_or(&0).clone();
        let contract_balance = self.reward_dist_balance;

        let user_reward = (contract_balance * number_of_user_cells as u128) / (2 * (FIELD_HEIGHT * FIELD_WIDTH)) as u128;

        self.account_withdraw.insert(account_id.clone());

        let near_reward = NearToken::from_yoctonear(user_reward);

        Promise::new(account_id).transfer(near_reward);
    }
}
