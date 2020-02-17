/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
//TODO documentation!

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::SaturatedConversion;
use codec::{Encode, Decode};
use frame_support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::DispatchResult, ensure, decl_event};
use system::ensure_signed;
use generic_asset;



pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Pool<AssetId, Balance> {
    first_asset_id: AssetId,
    second_asset_id: AssetId,
    
    first_asset_amount: Balance,
    second_asset_amount: Balance,

}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Token<AssetId> {
    id: AssetId,
    name: u64,
    
}


decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as XykStorage {
        //alicethepool wonderland
        VaultId: T::AccountId;
       
        Pools get(token_ids): map (T::AssetId, T::AssetId) => Pool<T::AssetId, T::Balance>;
        Tokens get(token_by_id): map T::AssetId => Token<T::AssetId>;

       
    }      
}

// The module's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;
       
        fn set_vault_id(origin) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(!<VaultId<T>>::exists(), "vault already initialized");
            <VaultId<T>>::put(sender);

            Ok(())
        }

        fn create_pool(origin, first_asset_id: T::AssetId, first_asset_amount: T::Balance, second_asset_id: T::AssetId, second_asset_amount: T::Balance) -> DispatchResult {
            //TO DO: create new user as pool_token1_token2
           // let pool_id: T::AccountId = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
            let sender = ensure_signed(origin.clone())?;
            
            let vault_address: T::AccountId  = <VaultId<T>>::get();

            ensure!(!<Pools<T>>::exists((first_asset_id,second_asset_id)), "Pools already exists");
            ensure!(!<Pools<T>>::exists((second_asset_id,first_asset_id)), "Pools already exists");

        //    TODO ensure sender has enought token1 token2 

            let new_pool = Pool {
                first_asset_id: first_asset_id.clone(),
                second_asset_id: second_asset_id.clone(),
                
                first_asset_amount: first_asset_amount.clone(), 
                second_asset_amount: second_asset_amount.clone(), 
             
            };

            <generic_asset::Module<T>>::make_transfer_with_event(
                &first_asset_id, &sender, &vault_address, first_asset_amount.clone())?;
            <generic_asset::Module<T>>::make_transfer_with_event(
                &second_asset_id, &sender, &vault_address, second_asset_amount.clone())?;
                
            <Pools<T>>::insert((first_asset_id, second_asset_id), new_pool);

            Ok(())
        }
    
        // you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        fn sell_asset (origin, sold_asset_id: T::AssetId, bought_asset_id: T::AssetId, sold_asset_amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            //  TODO ensure on token amount           
            //  ensure!(<TokenOwners<T>>::get((&sender, first_asset_id)) >= first_asset_amount), "not enought token1 amount");

            ensure!(<Pools<T>>::exists((sold_asset_id,bought_asset_id)), "no such pool");
            let pool = <Pools<T>>::get((sold_asset_id, bought_asset_id));
            let input_reserve = pool.first_asset_amount;
            let output_reserve = pool.second_asset_amount;
            ensure!(input_reserve > sold_asset_amount, "not enought reserve");
            let bought_asset_amount = Self::calculate_input_price(input_reserve, output_reserve, sold_asset_amount);
            ensure!(output_reserve > bought_asset_amount, "not enought reserve"); 
            let mut new_pool = pool.clone();
            new_pool.first_asset_amount = input_reserve + sold_asset_amount;
            new_pool.second_asset_amount = output_reserve - bought_asset_amount;

            //TODO asserts!

            let vault = <VaultId<T>>::get();

            <generic_asset::Module<T>>::make_transfer_with_event(&sold_asset_id, &sender, &vault, sold_asset_amount)?;
            <generic_asset::Module<T>>::make_transfer_with_event(&bought_asset_id, &vault, &sender, bought_asset_amount)?;
            
            <Pools<T>>::insert((sold_asset_id, bought_asset_id), new_pool);
            Ok(())
        }

    }
}


impl<T: Trait> Module<T> {

    pub fn calculate_input_price (input_reserve: T::Balance, output_reserve: T::Balance, input_amount: T::Balance) -> T::Balance {
        // input_amount_with_fee: uint256 = input_amount * 997
        let input_amount_with_fee = input_amount * 997.saturated_into::<T::Balance>();
        // numerator: uint256 = input_amount_with_fee * output_reserve
        let numenator = input_amount_with_fee * output_reserve;
        // denominator: uint256 = (input_reserve * 1000) + input_amount_with_fee
        let denominator = (input_reserve * 1000.saturated_into::<T::Balance>()) + input_amount_with_fee;
        numenator / denominator
    }

    pub fn calculate_output_price (input_reserve: T::Balance, output_reserve: T::Balance, output_amount: T::Balance) -> T::Balance {
        // numerator: uint256 = input_reserve * output_amount * 1000
        let numenator = input_reserve * output_amount * 1000.saturated_into::<T::Balance>();
        // denominator: uint256 = (output_reserve - output_amount) * 997
        let denominator = (output_reserve - output_amount) * 997.saturated_into::<T::Balance>();
        numenator / denominator + 1.saturated_into::<T::Balance>()
    }

    //Read-only function to be used by RPC
    pub fn get_exchange_input_price(input_asset_id: T::AssetId, output_asset_id: T::AssetId, input_amount: T::Balance) -> T::Balance {
        let pool = <Pools<T>>::get((input_asset_id, output_asset_id));
        let output_amount = Self::calculate_input_price(pool.first_asset_amount, pool.second_asset_amount, input_amount);
        output_amount
    }

    //Read-only function to be used by RPC
    pub fn get_exchange_output_price(input_asset_id: T::AssetId, output_asset_id: T::AssetId, output_amount: T::Balance) -> T::Balance {
        let pool = <Pools<T>>::get((input_asset_id, output_asset_id));
        let input_amount = Self::calculate_output_price(pool.first_asset_amount, pool.second_asset_amount, output_amount);
        input_amount
    }

}