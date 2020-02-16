/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, StorageValue, StorageMap, dispatch::DispatchResult};
use system::ensure_signed;
// /use codec::{Encode, Decode};
//use assets as assets;
use generic_asset::{AssetOptions, Owner, PermissionLatest};
use sp_runtime::traits::{Hash, SaturatedConversion};

/// The module's configuration trait.
pub trait Trait: generic_asset::Trait {
	// TODO: Add other types and constants required configure this module.

	// The overarching event type.
	//type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as XykStorage {
        // Declare storage and getter functions here
        OwnedX: map T::AccountId => u64;
        OwnedY: map T::AccountId => u64;

        PoolX: u64;
        PoolY: u64;

        FeeSignificand: u16;
        FeeDecimals: u16;

        TotalBalance: T::Balance;
    }
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		// Initializing events
		// this is needed only if you are using events in your module
		fn mint_x(origin, value: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let sum = <OwnedX<T>>::get(&sender) + value;

            <OwnedX<T>>::insert(sender, sum);

            Ok(())
        }

        fn mint_y(origin, value: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let sum = <OwnedY<T>>::get(&sender) + value;

            <OwnedY<T>>::insert(sender, sum);

            Ok(())
        }

        fn mint_asset_x(origin, amount: T::Balance, asset_id: T::AssetId) {
            let sender = ensure_signed(origin)?;
            let default_permission = PermissionLatest {
                update: Owner::Address(sender.clone()),
                mint: Owner::Address(sender.clone()),
                burn: Owner::Address(sender.clone()),
            };
            <generic_asset::Module<T>>::create_asset(None, Some(sender.clone()), AssetOptions {
                initial_issuance: amount,
                permissions: default_permission,
            })?;
            //let asset_id = 100;
            <TotalBalance<T>>::put(<generic_asset::Module<T>>::free_balance(&asset_id, &sender));
            //Assets.issue(amount);
        }

        fn transfer_asset_x(origin, asset_id: T::AssetId, amount: T::Balance, to: T::AccountId) {
            let sender = ensure_signed(origin.clone())?;
            <generic_asset::Module<T>>::transfer(origin, asset_id, to, amount);
        }

        fn set_default_values(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            PoolX::put(10000 as u64); 
            PoolY::put(10000 as u64);
            FeeSignificand::put(3 as u16);
            FeeDecimals::put(2 as u16);

            Ok(())
        } 

        fn swap_x_for_y(origin, x_amount: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let x = PoolX::get() as f64;
            let y = PoolY::get() as f64;
            let dx = x_amount as f64;
            let a = dx/x;
            let x1 = (1.0 + a) * x;
            let base = 10 as f64;
            let p = (FeeSignificand::get() as f64) / (100 as f64);
            let g = 1.0 - p;
            let dy = (a * g * y) / (1.0 + (a * g));
            let y1 = y - dy;

            let balanceX = <OwnedX<T>>::get(&sender) - x_amount;
            <OwnedX<T>>::insert(&sender, balanceX);
            let balanceY = <OwnedY<T>>::get(&sender) + (dy as u64);
            <OwnedY<T>>::insert(&sender, balanceY);
            PoolX::put(x1 as u64);
            PoolY::put(y1 as u64);

            Ok(())
        }
	}
}
