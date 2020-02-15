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



pub trait Trait: system::Trait + balances::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Pool {
    id: u64,
    name: u64,
    token1: u64,
    token2: u64,

}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Token {
    id: u64,
    name: u64,
    amount: u64,
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
        
       
        Pools get(pool_by_id): map u64 => Pool;
        Tokens get(token_by_id): map u64 => Token;

        AllPoolsCount get(all_pools_count): u64;
        AllTokensCount get(all_tokens_count): u64;

        TokenNames get(token_id_by_name):  map u64 => u64;
        PoolNames get(pool_id_by_name):  map u64 => u64;
      //  OwnedPoolsArray get(pool_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedTokensArray get(token_of_owner_by_id): map (T::AccountId, u64) => Token;
    }
}

// The module's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        

        fn create_pool(origin, token1_name: u64, token1_amount: u64, token2_name: u64, token2_amount: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            let mut token1_id: u64;
            let mut token2_id: u64;
            let primes = [2,    3,  5,  7,  11,     13,     17,     19,     23,     29, 
            31,     37,     41,     43,     47,     53,     59,     61,     67,     71, 
            73,     79,     83,     89,     97,     101,    103,    107,    109,    113, 
            127,    131,    137,    139,    149,    151,    157,    163,    167,    173, 
            179,    181,    191,    193,    197,    199,    211,    223,    227,    229, 
            233,    239,    241,    251,    257,    263,    269,    271,    277,    281, 
            ];

            if (!<TokenNames>::exists(token1_name)) {
                token1_id = primes[(AllPoolsCount::get() as usize) + 1];
                Self::create_token(token1_name, token1_id)?;
            }
            else {
                token1_id = <TokenNames>::get(token1_name);
            }

            if (!<TokenNames>::exists(token2_name)) {          
                token2_id = primes[(AllPoolsCount::get() as usize) + 1];
                Self::create_token(token2_name, token2_id)?;
            }
            else {
                token2_id = <TokenNames>::get(token2_name);
            }
           
            let name = token1_id * token2_id;
            let id = token1_id * token2_id;

            ensure!(!<Pools>::exists(id), "Pools already exists");



            let new_pool = Pool {
                id: id,
                // id2: u64,
                // token1: u64,
                // token2: u64,
                // token1id: u64,
                // token2id: u64,

                name: name,
                token1: token1_amount,
                token2: token2_amount,
             
            };



            let all_pools_count = Self::all_pools_count();
            let new_all_pools_count = all_pools_count.checked_add(1)
            .ok_or("Overflow adding a new pool to total supply")?;
            AllPoolsCount::put(new_all_pools_count);

            <Pools>::insert(id, new_pool);
            <PoolNames>::insert(name, id);
     
            //mint user tokens for pool and deduce tokens from user acc


            Ok(())
        }
    
    }
}


impl<T: Trait> Module<T> {
    fn create_token (name: u64, id: u64) -> DispatchResult {
        let new_token = Token {
            id: id,
            name: name,
            amount: 0,
        };

        let all_tokens_count = Self::all_tokens_count();
        let new_all_tokens_count = all_tokens_count.checked_add(1)
        .ok_or("Overflow adding a new token to total supply")?;
        AllTokensCount::put(new_all_tokens_count);

        <Tokens>::insert(id, new_token);
        <TokenNames>::insert(name, id);

        Ok(())  
    }
}