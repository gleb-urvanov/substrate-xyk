#![cfg_attr(not(feature = "std"), no_std)]

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
// TODO documentation!

use sp_runtime::traits::{
    SaturatedConversion,
    Hash,
    BlakeTwo256,
    Zero,
    One
};

use codec::{ Encode, Decode };
use frame_support::{
    decl_event,
    decl_module,
    decl_storage,
    dispatch::DispatchResult,
    ensure,
    StorageMap,
    traits::Randomness
};

use generic_asset;
use system::ensure_signed;

pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.
    // type Hashing = BlakeTwo256;

    type Randomness: Randomness<Self::Hash>;
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
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
        // Vault account for interacting with pools
        pub VaultId: T::AccountId;

        /// Gets ID of the pool asset pair (Asset 1, Asset 2) -> PoolId, (Asset 2, Asset 1) -> PoolId
        pub PoolId get(get_pool_id): map hasher(blake2_256) (T::AssetId, T::AssetId) => T::Hash;

        /// Gets balance of asset 1 in pool pair (Asset 1, Asset 2) -> Pool Balance
        pub PoolBalance get(get_pool_balance): map hasher(blake2_256) (T::AssetId, T::AssetId) => T::Balance;

        /// Gets total pool liquidity (PoolId) -> Balance
        pub PoolLiquidity get(get_pool_liquidity): map hasher(blake2_256) T::Hash => T::Balance;

        /// Gets balance of liquidity for given pool and account ID (PoolId, AccountId) -> Balance
        pub ProviderLiquidityBalance get(get_provider_liquidity_balance): double_map hasher(blake2_256)
            T::Hash, hasher(blake2_256) T::AccountId => T::Balance;

        Nonce get(fn nonce): u32;
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

        /// Creates liquidity pool pair for Asset 1 and Asset 2
        fn create_liquidity_pool(origin,
            first_asset_id: T::AssetId,
            second_asset_id: T::AssetId
        ) -> DispatchResult {
            ensure_signed(origin)?;

            ensure!(
                !<PoolBalance<T>>::contains_key((first_asset_id, second_asset_id)),
                "Pools already exists"
            );

            ensure!(
                !<PoolBalance<T>>::contains_key((second_asset_id, first_asset_id)),
                "Sanity check has failed, the chain is in undefined state"
            );

            // TODO generate separate addresses for pool pairs
            let random_hash = Self::generate_random_hash();

            <PoolId<T>>::insert((first_asset_id, second_asset_id), random_hash);
            <PoolId<T>>::insert((second_asset_id, first_asset_id), random_hash);

            Ok(())
        }

		/// Deposit Asset 1 and Asset 2 and trade asset at current ratio to mint liquidity
		/// Returns amount of liquidity minted.
		///
		/// `origin`
		/// `first_asset_id`
		/// `second_asset_id`
		/// `first_asset_amount`
		/// `maximum second_asset_amount`
		pub fn add_liquidity(
            origin,
            first_asset_id: T::AssetId,
			second_asset_id: T::AssetId,
			first_asset_amount: T::Balance,
            second_asset_amount: T::Balance
            // TODO dispatch result ?
		) {
            let from_account = ensure_signed(origin)?;

			ensure!(
				!first_asset_amount.is_zero() && !second_asset_amount.is_zero(),
				"Cannot add zero liquidity"
			);

            // TODO ensure sender has enough assets

			let pool_id = Self::get_pool_id((first_asset_id, second_asset_id));
            let total_liquidity = Self::get_pool_liquidity(&pool_id);
            // TODO generate separate addresses for pool pairs
            let vault_address = <VaultId<T>>::get();

            // Check if pool is empty
			if total_liquidity.is_zero() {
				<generic_asset::Module<T>>::make_transfer(&first_asset_id, &from_account, &vault_address, first_asset_amount)?;
                <generic_asset::Module<T>>::make_transfer(&second_asset_id, &from_account, &vault_address, second_asset_amount)?;

                <PoolBalance<T>>::insert((first_asset_id, second_asset_id), first_asset_amount);
                <PoolBalance<T>>::insert((second_asset_id, first_asset_id), second_asset_amount);

                let initial_liquidity = first_asset_amount * second_asset_amount;
				<ProviderLiquidityBalance<T>>::insert(&pool_id, &from_account, initial_liquidity);
                <PoolLiquidity<T>>::insert(&pool_id, initial_liquidity);

                // TODO Call deposit event

			} else {
                let first_asset_reserve = <PoolBalance<T>>::get((first_asset_id, second_asset_id));
                let second_asset_reserve = <PoolBalance<T>>::get((second_asset_id, first_asset_id));
                let second_asset_required = first_asset_amount * second_asset_reserve / first_asset_reserve + One::one();
                let liquidity_minted = first_asset_amount * total_liquidity / first_asset_reserve;

				ensure!(
					second_asset_required >= second_asset_amount,
					"Maximum amount of second asset exceeded provided limit"
                );

				<generic_asset::Module<T>>::make_transfer(&first_asset_id, &from_account, &vault_address, first_asset_amount)?;
                <generic_asset::Module<T>>::make_transfer(&second_asset_id, &from_account, &vault_address, second_asset_required)?;

                // TODO overflow check
				<ProviderLiquidityBalance<T>>::insert(
                    &pool_id, &from_account,
                    Self::get_provider_liquidity_balance(&pool_id, &from_account) + liquidity_minted
                );

                // TODO overflow check or use asset constrained by max supply
				<PoolLiquidity<T>>::mutate(pool_id, |balance| *balance += liquidity_minted);

                // TODO Emit event
            }
        }

        /// Burn exchange assets to withdraw Asset 1 and Asset 2 at current rate
		///
		/// `origin`
		/// `first_asset_id`
		/// `second_asset_id`
		/// `liquidity_withdrawn`
		/// `minimum first_asset_amount`
		/// `minimum second_asset_amount`
		pub fn remove_liquidity(
			origin,
			first_asset_id: T::AssetId,
            second_asset_id: T::AssetId,
            liquidity_withdrawn: T::Balance,
			first_asset_withdraw: T::Balance,
            second_asset_withdraw: T::Balance
            // TODO dispatch result ?
		) {
            let from_account = ensure_signed(origin)?;

			ensure!(
				liquidity_withdrawn > Zero::zero(),
				"Cannot withdraw zero liquidity"
            );

            // TODO make zero or Null withdraw without limit?
			ensure!(
				first_asset_withdraw > Zero::zero() && second_asset_withdraw > Zero::zero(),
				"Minimum amount cannot be zero"
			);

            let pool_id = Self::get_pool_id((first_asset_id, second_asset_id));
            let total_liquidity = Self::get_pool_liquidity(&pool_id);

			ensure!(
				total_liquidity > Zero::zero(),
				"Pool has no liquidity"
            );
            ensure!(
				total_liquidity >= liquidity_withdrawn,
				"Not enough liquidity in pool"
			);

            let account_liquidity = Self::get_provider_liquidity_balance(pool_id, &from_account);

			ensure!(
				account_liquidity >= liquidity_withdrawn,
				"Not enough liquidity owned"
			);

            // TODO generate separate addresses for pool pairs
            let vault_address = <VaultId<T>>::get();

            let first_asset_reserve = <PoolBalance<T>>::get((first_asset_id, second_asset_id));
            let second_asset_reserve = <PoolBalance<T>>::get((second_asset_id, first_asset_id));

			let first_asset_required = liquidity_withdrawn * first_asset_reserve / total_liquidity;
			let second_asset_required = liquidity_withdrawn * second_asset_reserve / total_liquidity;

            // TODO add asset id or asset ticker
            ensure!(
                first_asset_required >= first_asset_withdraw,
				"Cannot fullfil minimum asset amount requirement"
			);
            ensure!(
                second_asset_required >= second_asset_required,
				"Cannot fullfil minimum asset amount requirement"
			);

            <generic_asset::Module<T>>::make_transfer(&first_asset_id, &vault_address, &from_account, first_asset_required)?;
            <generic_asset::Module<T>>::make_transfer(&second_asset_id, &vault_address, &from_account, second_asset_required)?;

            // TODO underflow check
            <ProviderLiquidityBalance<T>>::insert(
                &pool_id, &from_account,
                Self::get_provider_liquidity_balance(&pool_id, &from_account) - liquidity_withdrawn
            );

            // TODO underflow check or use asset constrained by max supply
            <PoolLiquidity<T>>::mutate(pool_id, |balance| *balance -= liquidity_withdrawn);

			// TODO Deposit event
		}

        /// you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
        fn sell_asset (
            origin,
            sold_asset_id: T::AssetId,
            bought_asset_id: T::AssetId,
            sold_asset_amount: T::Balance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // TODO ensure sender has enough assets

            ensure!(<PoolBalance<T>>::contains_key((sold_asset_id,bought_asset_id)), "no such pool");

            let input_reserve = <PoolBalance<T>>::get((sold_asset_id, bought_asset_id));
            let output_reserve = <PoolBalance<T>>::get((bought_asset_id, sold_asset_id));

            let bought_asset_amount = Self::calculate_sell_price(
                input_reserve, output_reserve,
                sold_asset_amount
            );

            ensure!(output_reserve > bought_asset_amount, "not enough reserve");

            // TODO asserts!

            let vault = <VaultId<T>>::get();
            <generic_asset::Module<T>>::make_transfer_with_event(
                &sold_asset_id,
                &sender,
                &vault,
                sold_asset_amount
            )?;

            <generic_asset::Module<T>>::make_transfer_with_event(
                &bought_asset_id,
                &vault,
                &sender,
                bought_asset_amount
            )?;

            <PoolBalance<T>>::insert(
                (sold_asset_id, bought_asset_id),
                input_reserve + sold_asset_amount
            );

            <PoolBalance<T>>::insert(
                (bought_asset_id, sold_asset_id),
                output_reserve - bought_asset_amount
            );

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

        return (new_random).using_encoded(<T as system::Trait>::Hashing::hash)
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
        let denominator = (input_reserve * 1000.saturated_into::<T::Balance>()) + input_amount_with_fee;
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

    //Read-only function to be used by RPC
    pub fn get_exchange_sell_price(
        input_asset_id: T::AssetId,
        output_asset_id: T::AssetId,
        sell_amount: T::Balance,
    ) -> T::Balance {
        let sell_price = Self::calculate_sell_price(
            <PoolBalance<T>>::get((input_asset_id, output_asset_id)),
            <PoolBalance<T>>::get((output_asset_id, input_asset_id)),
            sell_amount
        );
        sell_price
    }

    //Read-only function to be used by RPC
    pub fn get_exchange_buy_price(
        input_asset_id: T::AssetId,
        output_asset_id: T::AssetId,
        buy_amount: T::Balance,
    ) -> T::Balance {
        let buy_price = Self::calculate_buy_price(
            <PoolBalance<T>>::get((input_asset_id, output_asset_id)),
            <PoolBalance<T>>::get((output_asset_id, input_asset_id)),
            buy_amount
        );
        buy_price
    }
}
