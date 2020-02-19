/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
// TODO documentation!
use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::SaturatedConversion;

use frame_support::{
	decl_event,
	decl_module,
	decl_storage,
	dispatch::DispatchResult,
	ensure,
	StorageMap,
	StorageValue,
};

use generic_asset;
use system::ensure_signed;

pub trait Trait: generic_asset::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
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
		// alicethepool wonderland
		VaultId: T::AccountId;

		Pools get(asset_pool): map (T::AssetId, T::AssetId) => T::Balance;

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

		fn create_pool(origin,
			first_asset_id: T::AssetId,
			first_asset_amount: T::Balance,
			second_asset_id: T::AssetId,
			second_asset_amount: T::Balance
		) -> DispatchResult {
			// TO DO: create new vault
			// let pool_id: T::AccountId = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
			let sender = ensure_signed(origin.clone())?;

			let vault_address: T::AccountId  = <VaultId<T>>::get();

			ensure!(
				!<Pools<T>>::exists((first_asset_id,second_asset_id)),
				"Pools already exists"
			);

			ensure!(
				!<Pools<T>>::exists((second_asset_id,first_asset_id)),
				"Sanity check has failed, the chain is in undefined state"
			);

			// TODO ensure sender has enough assets

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

			<Pools<T>>::insert((first_asset_id, second_asset_id), first_asset_amount.clone());
			<Pools<T>>::insert((second_asset_id, first_asset_id), second_asset_amount.clone());

			Ok(())
		}

		// you will sell your sold_asset_amount of sold_asset_id to get some amount of bought_asset_id
		fn sell_asset (
			origin,
			sold_asset_id: T::AssetId,
			bought_asset_id: T::AssetId,
			sold_asset_amount: T::Balance
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// TODO ensure sender has enough assets

			ensure!(<Pools<T>>::exists((sold_asset_id,bought_asset_id)), "no such pool");

			let input_reserve = <Pools<T>>::get((sold_asset_id, bought_asset_id));
			let output_reserve = <Pools<T>>::get((bought_asset_id, sold_asset_id));

			let bought_asset_amount = Self::calculate_input_price(
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

			<Pools<T>>::insert(
				(sold_asset_id, bought_asset_id),
				input_reserve + sold_asset_amount
			);

			<Pools<T>>::insert(
				(bought_asset_id, sold_asset_id),
				output_reserve - bought_asset_amount
			);

			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {
	pub fn calculate_input_price(
		input_reserve: T::Balance,
		output_reserve: T::Balance,
		input_amount: T::Balance,
	) -> T::Balance {
		// input_amount_with_fee: uint256 = input_amount * 997
		let input_amount_with_fee = input_amount * 997.saturated_into::<T::Balance>();
		// numerator: uint256 = input_amount_with_fee * output_reserve
		let numerator = input_amount_with_fee * output_reserve;
		// denominator: uint256 = (input_reserve * 1000) + input_amount_with_fee
		let denominator =
			(input_reserve * 1000.saturated_into::<T::Balance>()) + input_amount_with_fee;
		numerator / denominator
	}

	pub fn calculate_output_price(
		input_reserve: T::Balance,
		output_reserve: T::Balance,
		output_amount: T::Balance,
	) -> T::Balance {
		// numerator: uint256 = input_reserve * output_amount * 1000
		let numerator = input_reserve * output_amount * 1000.saturated_into::<T::Balance>();
		// denominator: uint256 = (output_reserve - output_amount) * 997
		let denominator = (output_reserve - output_amount) * 997.saturated_into::<T::Balance>();
		numerator / denominator + 1.saturated_into::<T::Balance>()
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
