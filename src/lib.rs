// Find all our documentation at https://docs.near.org
use near_sdk::{log, near};
use near_sdk::{store::UnorderedMap};
use near_sdk::{NearToken, AccountId, borsh::BorshSerialize};


// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    field: UnorderedMap<u32, UnorderedMap<u32, PixelInfo>>,
}

// Define the default, which automatically initializes the contract

// Implement the contract structure


impl Default for Contract {
    fn default() -> Self {
        Self {
            field: UnorderedMap::new(b"b"),
        }
    }
}

#[derive(BorshSerialize)]
struct PixelInfo {
    owner: AccountId,
    price: NearToken,
    color: u32
}

#[near]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_GREETING
    const PIXEL_START_PRICE: u32 = 1;
    const PIXEL_PRICE_INCREASE: u32 = 2;

    // set pixel
    // get pixel
    // get field row 
    // withdraw logic

    // store *active* owners
    // price increase 

    #[init]
    #[private] // only callable by the contract's account
    pub fn init() -> Self {
        Self {
            field: UnorderedMap::new(b"b"),
        }
    }
    #[payable]
    pub fn set_pixel(&mut self, color: u32, position_x: u32, position_y: u32) {
        let mut row = self.field.get(position_x).unwrap_or(UnorderedMap::new(position_x));
        let mut pixelInfo = row.get(position_y).unwrap_or(PixelInfo{owner: env::predecessor_account_id(), price: PIXEL_START_PRICE, color: 0});
        if env::attached_deposit() < pixelInfo.price * PIXEL_PRICE_INCREASE {
            panic!("Not enough tokens attached");
        }
        pixelInfo.owner = env::predecessor_account_id();
        pixelInfo.price = pixelInfo.price * PIXEL_PRICE_INCREASE;
        pixelInfo.color = color;

        row.insert(position_y, pixelInfo);
        self.field.insert(position_x, row);
    }
}
