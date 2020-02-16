/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
//use crate::parity_scale_codec::{Encode, Decode};
//use sr_primitives::traits::{As, Hash};

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::SaturatedConversion;
use sp_std::convert::TryInto;
use sp_runtime::traits::{Hash};
use codec::{Encode, Decode};
use frame_support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::DispatchResult, ensure, decl_event, traits::Randomness, traits::Currency};
//use frame_support::{decl_module, decl_storage, StorageValue, StorageMap, dispatch::DispatchResult};
use system::ensure_signed;
use randomness_collective_flip;
use generic_asset::{AssetOptions, Owner, PermissionLatest};



pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Pool<AssetId, Balance> {
    token1_id: AssetId,
    token2_id: AssetId,
    
    token1_amount: Balance, //get pool
    token2_amount: Balance,

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
        // Declare storage and getter functions here

        //alicethepool wonderland
        VaultId: T::AccountId;
       
        Pools get(token_ids): map (AssetId, AssetId) => Pool;
        Tokens get(token_by_id): map AssetId => Token;

       
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

        fn create_pool(origin, token1_id: T::AssetId, token1_amount: T::Balance, token2_id: T::AssetId, token2_amount: T::Balance) -> DispatchResult {
            //TO DO: create new user as pool_token1_token2
           // let pool_id: T::AccountId = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
            let sender = ensure_signed(origin.clone())?;
            
            let vault_address: T::AccountId  = <VaultId<T>>::get();

            ensure!(!<Pools>::exists((token1_id,token2_id)), "Pools already exists");
            ensure!(!<Pools>::exists((token2_id,token1_id)), "Pools already exists");

        //    TODO ensure sender has enought token1 token2 
        //    ensure!(<TokenOwners<T>>::get((&sender, token1_id)) >= token1_amount), "not enought token1 amount");
        //    ensure!(<TokenOwners<T>>::get((&sender, token2_id)) >= token2_amount), "not enought token1 amount");

            let new_pool = Pool {
                token1_id: token1_id.clone(),
                token2_id: token2_id.clone(),
                
                token1_amount: token1_amount.clone(), 
                token2_amount: token1_amount.clone(), 
             
            };


            <Pools>::insert((token1_id, token2_id), new_pool);

            <generic_asset::Module<T>>::transfer(&origin, &token1_id, &vault_address, &token1_amount);
            <generic_asset::Module<T>>::transfer(&origin, &token2_id, &vault_address, &token2_amount);

            Ok(())
        }
    
        // you will sell your token1_amount to get some token2_amount
        fn buy_y_for_x (origin, token1_id: T::AssetId, token2_id: T::AssetId, token1_amount: T::Balance, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let output_reserve: T::Balance;
            let input_reserve: T::Balance;
           

        //  TODO ensure on token amount           
        //  ensure!(<TokenOwners<T>>::get((&sender, token1_id)) >= token1_amount), "not enought token1 amount");

            ensure!(<Pools>::exists((token1_id,token2_id)) || <Pools>::exists((token2_id,token1_id)), "no such pool, transfer zatial impossibru https://i.kym-cdn.com/entries/icons/original/000/004/918/imposibru.jpg" );
            // TODO if pools do not exist, find cheapest way in matrix

            // swap token1 for token2
            if  <Pools>::exists((token1_id,token2_id)){
                input_reserve = (<Pools>::get((token1_id, token2_id))).token1_amount; 
                output_reserve = (<Pools>::get((token1_id, token2_id))).token2_amount;
                ensure!(output_reserve > 0, "not enought reserve"); 
                ensure!(input_reserve > 0, "not enought reserve");
                let dy = Self::get_input_price(input_reserve, output_reserve, amount);
                let mut new_pool = <Pools>::get((token1_id, token2_id));
                new_pool.token1_amount = input_reserve + token1_amount;
                new_pool.token2_amount = output_reserve -dy;
                <Pools>::insert((token1_id, token2_id), new_pool);
                
            }
            // swap token2 for token1
            //TODO remove code duplication. Find the smart way to operate with pools!
            // else{
            //     output_reserve = (<Pools>::get((token1_id, token2_id))).token1_amount; 
            //     input_reserve = (<Pools>::get((token1_id, token2_id))).token2_amount;
            //     ensure!(output_reserve > 0, "not enought reserve"); 
            //     ensure!(input_reserve > 0, "not enought reserve");
            //     let dy = Self::get_input_price(input_reserve, output_reserve, amount);
            //     let mut new_pool = <Pools>::get((token2_id, token1_id));
            //     new_pool.token1_amount = input_reserve - dy;
            //     new_pool.token2_amount = output_reserve + token1_amount;
            //     <Pools>::insert((token2_id, token1_id), new_pool);
            // }

            let vault = <VaultId<T>>::get();

            //TODO transfer fn
            // sender - token1_amount
            // sender + dy
            // vault + token1_amount
            // vault - dy
            <generic_asset::Module<T>>::make_transfer_with_event(token1_id, sender, vault, token1_amount);
            <generic_asset::Module<T>>::make_transfer_with_event(token2_id, vault, sender, dy);

            Ok(())
        }

    }
}


impl<T: Trait> Module<T> {

    fn get_input_price (input_reserve: T::Balance, output_reserve: T::Balance, input_amount: T::Balance) -> T::Balance {
        // input_amount_with_fee: uint256 = input_amount * 997
        let input_amount_with_fee: u64 = input_amount * 997;
        // numerator: uint256 = input_amount_with_fee * output_reserve
        let numenator: u64 = input_amount_with_fee * output_reserve;
        // denominator: uint256 = (input_reserve * 1000) + input_amount_with_fee
        let denominator: u64 = (input_reserve * 1000) + input_amount_with_fee;
        numenator / denominator
    }

    fn get_output_price (input_reserve: T::Balance, output_reserve: T::Balance, output_amount: T::Balance) -> T::Balance {
        // numerator: uint256 = input_reserve * output_amount * 1000
        let numenator: u64 = input_reserve * output_amount * 1000;
        // denominator: uint256 = (output_reserve - output_amount) * 997
        let denominator: u64 = (output_reserve - output_amount) * 997;
        numenator / denominator
    }

}