#![cfg_attr(not(feature = "std"), no_std)]

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
// TODO documentation!
use sp_runtime::traits::{BlakeTwo256, Hash, One, SaturatedConversion, Zero};

use codec::{Decode, Encode};
use frame_support::{
    decl_event,
    decl_module,
    decl_storage, 
    decl_error,
    dispatch::DispatchResult, 
    ensure, 
    traits::Randomness,
    StorageMap,
};

use generic_asset::{AssetOptions, Owner, PermissionLatest};
use system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.
    // type Hashing = BlakeTwo256;

    type Randomness: Randomness<Self::Hash>;
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
    /// Error for the generic-asset module.

    pub enum Error for Module<T: Trait> {
        VaultAlreadySet,
        PoolAlreadyExists,
        NotEnoughAssets,
        NoSuchPool,
        NotEnoughReserve,
        ZeroAmount,
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit function, from our runtime functions
        SomethingStored(u32, AccountId),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as XykStorage {
        // alicethepool wonderland
        VaultId get(vault_id): T::AccountId;

        Pools get(asset_pool): map hasher(blake2_256) (T::AssetId, T::AssetId) => T::Balance;

        LiquidityAssets get(liquidity_asset): map hasher(blake2_256) (T::AssetId, T::AssetId) => T::AssetId;

        LiquidityPools get(liquidity_pool): map hasher(blake2_256) T::AssetId => (T::AssetId, T::AssetId);
        Nonce get (fn nonce): u32;
    }
}

// The module's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        fn set_vault_id(origin) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(
                !<VaultId<T>>::exists(),
                Error::<T>::VaultAlreadySet,
            );
            <VaultId<T>>::put(sender);

            Ok(())
        }

        fn create_pool(
            origin,
            first_asset_id: T::AssetId,
            first_asset_amount: T::Balance,
            second_asset_id: T::AssetId,
            second_asset_amount: T::Balance
        ) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let vault_address: T::AccountId  = <VaultId<T>>::get();
            //  TODO ensure assets exists ?
            //  TODO asset1 != asset2
            ensure!(
                !<Pools<T>>::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );
            ensure!(
                !<Pools<T>>::contains_key((second_asset_id,first_asset_id)),
                Error::<T>::PoolAlreadyExists,
            );
            ensure!(
                <generic_asset::Module<T>>::free_balance(&first_asset_id, &sender) >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );
            ensure!(
                <generic_asset::Module<T>>::free_balance(&second_asset_id, &sender) >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );
            <Pools<T>>::insert(
                (first_asset_id, second_asset_id), first_asset_amount.clone()
            );
            <Pools<T>>::insert(
                (second_asset_id, first_asset_id), second_asset_amount.clone()
            );
            let liquidity_asset_id = <generic_asset::Module<T>>::next_asset_id();
            <LiquidityAssets<T>>::insert(
                (first_asset_id, second_asset_id), liquidity_asset_id.clone()
            );
            <LiquidityPools<T>>::insert(
                liquidity_asset_id.clone(), (first_asset_id, second_asset_id) 
            );
            let initial_liquidity = first_asset_amount + second_asset_amount; //for example, doesn't really matter
            Self::create_asset(origin.clone(), initial_liquidity);
           
            <generic_asset::Module<T>>::make_transfer_with_event(
                &first_asset_id,
                &sender,
                &vault_address,
                first_asset_amount.clone()
            )?;
            <generic_asset::Module<T>>::make_transfer_with_event(
                &second_asset_id,
                &sender,
                &vault_address,
                second_asset_amount.clone()
            )?;
            Ok(())
        }

        /// you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        fn sell_asset (
            origin,
            sold_asset_id: T::AssetId,
            bought_asset_id: T::AssetId,
            sold_asset_amount: T::Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // TODO ensure sender has enough assets

            ensure!(
                <Pools<T>>::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
            let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));

            let bought_asset_amount = Self::calculate_sell_price(
                input_reserve,
                output_reserve,
                sold_asset_amount,
            );

            ensure!(
                <generic_asset::Module<T>>::free_balance(&sold_asset_id, &sender) >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            let vault = <VaultId<T>>::get();
            <generic_asset::Module<T>>::make_transfer_with_event(
                &sold_asset_id,
                &sender,
                &vault,
                sold_asset_amount,
            )?;

            <generic_asset::Module<T>>::make_transfer_with_event(
                &bought_asset_id,
                &vault,
                &sender,
                bought_asset_amount,
            )?;

            <Pools<T>>::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );

            <Pools<T>>::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Ok(())
        }

        fn buy_asset (
            origin,
            sold_asset_id: T::AssetId,
            bought_asset_id: T::AssetId,
            bought_asset_amount: T::Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(
                <Pools<T>>::contains_key((sold_asset_id,bought_asset_id)),
                Error::<T>::NoSuchPool,
            );

            let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
            let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));

            ensure!(
                output_reserve > bought_asset_amount,
                Error::<T>::NotEnoughReserve,
            );

            let sold_asset_amount = Self::calculate_buy_price(
                input_reserve,
                output_reserve,
                bought_asset_amount,
            );

            ensure!(
                <generic_asset::Module<T>>::free_balance(&sold_asset_id, &sender) >= sold_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            let vault = <VaultId<T>>::get();
            <generic_asset::Module<T>>::make_transfer_with_event(
                &sold_asset_id,
                &sender,
                &vault,
                sold_asset_amount,
            )?;

            <generic_asset::Module<T>>::make_transfer_with_event(
                &bought_asset_id,
                &vault,
                &sender,
                bought_asset_amount,
            )?;

            <Pools<T>>::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount,
            );

            <Pools<T>>::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount,
            );

            Ok(())
        }

        fn mint_liquidity (
            origin,
            first_asset_id: T::AssetId,
            second_asset_id: T::AssetId,
            first_asset_amount: T::Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let vault = <VaultId<T>>::get();

            //get liquidity_asset_id of selected pool
            let liquidity_asset_id = Self::get_liquidity_asset(
                 first_asset_id,
                 second_asset_id
            );
           
            ensure!(
                (<Pools<T>>::contains_key((first_asset_id, second_asset_id)) || <Pools<T>>::contains_key((second_asset_id, first_asset_id))),
                Error::<T>::NoSuchPool,
            );

            // ensure!(
            //     <TotalLiquidities<T>>::get(liquidity_asset_id.clone()) > 0.saturated_into(),
            //     "pool has no liquidity"
            // );

            let first_asset_reserve = <Pools<T>>::get((first_asset_id, second_asset_id));
            let second_asset_reserve = <Pools<T>>::get((second_asset_id, first_asset_id));
            let second_asset_amount = first_asset_amount * second_asset_reserve / first_asset_reserve + 1.saturated_into::<T::Balance>();
            let total_liquidity_assets = <generic_asset::Module<T>>::total_issuance(liquidity_asset_id);
            let liquidity_assets_minted = first_asset_amount * total_liquidity_assets / first_asset_reserve;

            ensure!(
                !first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
                Error::<T>::ZeroAmount,
            );
            ensure!(
                <generic_asset::Module<T>>::free_balance(&first_asset_id, &sender) >= first_asset_amount,
                Error::<T>::NotEnoughAssets,
            );
            ensure!(
                <generic_asset::Module<T>>::free_balance(&second_asset_id, &sender) >= second_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            <generic_asset::Module<T>>::make_transfer_with_event(
                &first_asset_id,
                &sender,
                &vault,
                first_asset_amount,
            )?;

            <generic_asset::Module<T>>::make_transfer_with_event(
                &second_asset_id,
                &sender,
                &vault,
                second_asset_amount,
            )?;

            <Pools<T>>::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve + first_asset_amount,
            );

            <Pools<T>>::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve + second_asset_amount,
            );

            <generic_asset::Module<T>>::mint_free(
                 &liquidity_asset_id,
                 &vault,
                 &sender,
                 &liquidity_assets_minted,
            )?;

            Ok(())
        }

        fn burn_liquidity (
            origin,
            first_asset_id: T::AssetId,
            second_asset_id: T::AssetId,
            liquidity_asset_amount: T::Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let vault = <VaultId<T>>::get();

            //get liquidity_asset_id of selected pool
            let liquidity_asset_id = Self::get_liquidity_asset(first_asset_id, second_asset_id);

            ensure!(
                <Pools<T>>::contains_key((first_asset_id, second_asset_id)),
                Error::<T>::NoSuchPool,
            );

            ensure!(
                <generic_asset::Module<T>>::free_balance(&liquidity_asset_id, &sender) >= liquidity_asset_amount,
                Error::<T>::NotEnoughAssets,
            );

            let first_asset_reserve = <Pools<T>>::get((first_asset_id, second_asset_id));
            let second_asset_reserve = <Pools<T>>::get((second_asset_id, first_asset_id));
            let first_asset_amount = first_asset_reserve * liquidity_asset_amount / <generic_asset::Module<T>>::total_issuance(liquidity_asset_id);
            let second_asset_amount = second_asset_reserve * liquidity_asset_amount / <generic_asset::Module<T>>::total_issuance(liquidity_asset_id);

            <generic_asset::Module<T>>::make_transfer_with_event(
                &first_asset_id,
                &vault,
                &sender,
                first_asset_amount,
            )?;

            <generic_asset::Module<T>>::make_transfer_with_event(
                &second_asset_id,
                &vault,
                &sender,
                second_asset_amount,
            )?;

            <Pools<T>>::insert(
                (&first_asset_id, &second_asset_id),
                first_asset_reserve - first_asset_amount,
            );

            <Pools<T>>::insert(
                (&second_asset_id, &first_asset_id),
                second_asset_reserve - second_asset_amount,
            );

            //TODO burn_free of liqudity_pool_id asset to sender in an amount of += liquidity_assets_minted
            <generic_asset::Module<T>>::burn_free(
                &liquidity_asset_id,
                &vault,
                &sender,
                &liquidity_asset_amount,
            )?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn generate_random_hash() -> T::Hash {
        let nonce = <Nonce>::get();

        let random_seed = T::Randomness::random_seed();
        let new_random = (random_seed, nonce)
            .using_encoded(|b| BlakeTwo256::hash(b))
            .using_encoded(|mut b| u64::decode(&mut b))
            .expect("Hash must be bigger than 8 bytes; Qed");

        let new_nonce = <Nonce>::get() + 1;
        <Nonce>::put(new_nonce);

        return (new_random).using_encoded(<T as system::Trait>::Hashing::hash);
    }

    pub fn calculate_sell_price(
        input_reserve: T::Balance,
        output_reserve: T::Balance,
        sell_amount: T::Balance,
    ) -> T::Balance {
        // input_amount_with_fee: uint256 = input_amount * 997
        let input_amount_with_fee = sell_amount * 997.saturated_into::<T::Balance>();
        // numerator: uint256 = input_amount_with_fee * output_reserve
        let numerator = input_amount_with_fee * output_reserve;
        // denominator: uint256 = (input_reserve * 1000) + input_amount_with_fee
        let denominator =
            (input_reserve * 1000.saturated_into::<T::Balance>()) + input_amount_with_fee;
        numerator / denominator
    }

    pub fn calculate_buy_price(
        input_reserve: T::Balance,
        output_reserve: T::Balance,
        buy_amount: T::Balance,
    ) -> T::Balance {
        // numerator: uint256 = input_reserve * output_amount * 1000
        let numerator = input_reserve * buy_amount * 1000.saturated_into::<T::Balance>();
        // denominator: uint256 = (output_reserve - output_amount) * 997
        let denominator = (output_reserve - buy_amount) * 997.saturated_into::<T::Balance>();
        numerator / denominator + 1.saturated_into::<T::Balance>()
    }

    pub fn get_liquidity_asset(
        first_asset_id: T::AssetId,
        second_asset_id: T::AssetId,
    ) -> T::AssetId {
        if <LiquidityAssets<T>>::contains_key((first_asset_id, second_asset_id)) {
            return <LiquidityAssets<T>>::get((first_asset_id, second_asset_id));
        } else {
            return <LiquidityAssets<T>>::get((second_asset_id, first_asset_id));
        }
    }

    fn create_asset(origin: T::Origin, amount: T::Balance) -> DispatchResult {
        let vault: T::AccountId = <VaultId<T>>::get();
        let sender = ensure_signed(origin)?;

        let default_permission = generic_asset::PermissionLatest {
            update: Owner::Address(vault.clone()),
            mint: Owner::Address(vault.clone()),
            burn: Owner::Address(vault.clone()),
        };

        <generic_asset::Module<T>>::create_asset(
            None,
            Some(sender.clone()),
            generic_asset::AssetOptions {
                initial_issuance: amount,
                permissions: default_permission,
            },
        )?;

        Ok(())
    }

    fn get_free_balance(assetId: T::AssetId, from: T::AccountId) -> T::Balance {
        return <generic_asset::Module<T>>::free_balance(&assetId, &from);
    }

    // //Read-only function to be used by RPC
    // pub fn get_exchange_input_price(
    //     input_asset_id: T::AssetId,
    //     output_asset_id: T::AssetId,
    //     input_amount: T::Balance,
    // ) -> T::Balance {
    //     let pool = <Pools<T>>::get((input_asset_id, output_asset_id));
    //     let output_amount = Self::calculate_input_price(
    //         pool.first_asset_amount,
    //         pool.second_asset_amount,
    //         input_amount,
    //     );
    //     output_amount
    // }

    // //Read-only function to be used by RPC
    // pub fn get_exchange_output_price(
    //     input_asset_id: T::AssetId,
    //     output_asset_id: T::AssetId,
    //     output_amount: T::Balance,
    // ) -> T::Balance {
    //     let pool = <Pools<T>>::get((input_asset_id, output_asset_id));
    //     let input_amount = Self::calculate_output_price(
    //         pool.first_asset_amount,
    //         pool.second_asset_amount,
    //         output_amount,
    //     );
    //     input_amount
    // }
}
