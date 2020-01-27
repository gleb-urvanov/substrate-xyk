/// A runtime module template with necessary imports
​
/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references
​
​
/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
​
use frame_support::{decl_module, decl_storage, StorageValue, StorageMap, dispatch::DispatchResult};
use system::ensure_signed;
​
​
/// The module's configuration trait.
pub trait Trait: balances::Trait {
	// TODO: Add other types and constants required configure this module.
​
	// The overarching event type.
	//type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}
​
// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as XykStorage {
        // Declare storage and getter functions here
        OwnedX: map T::AccountId => u64;
        OwnedY: map T::AccountId => u64;
​
        PoolX: u64;
        PoolY: u64;
​
        FeeSignificand: u32;
        FeeDecimals: u32;
    }
}
​
// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn mint_x(origin, value: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
​
            let sum = <OwnedX<T>>::get(&sender) + value;
​
            <OwnedX<T>>::insert(sender, sum);
​
            Ok(())
        }
​
        fn mint_y(origin, value: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
​
            let sum = <OwnedY<T>>::get(&sender) + value;
​
            <OwnedY<T>>::insert(sender, sum);
​
            Ok(())
        }
​
        fn set_default_values(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            PoolX::put(10000 as u64); 
            PoolY::put(10000 as u64);
            FeeSignificand::put(3 as u32);
            FeeDecimals::put(2 as u32);
​
            Ok(())
        } 
​
        fn sell_x(origin, x_amount: u64) -> DispatchResult { 
            let sender = ensure_signed(origin)?;
           
            if <OwnedX<T>>::get(&sender) < x_amount {
               //nejaky mesage a exit ?
            }
            else {
                let x = PoolX::get() as f64;
                let y = PoolY::get() as f64;
                let dx = x_amount as f64;
                let a = dx/x;
               
                let base = 10 as f64;
                // nefunguje neviem preco
                // let p = (FeeSignificand::get() as f64) / ( f64::powi(10.0,FeeDecimals) );
                let p = (FeeSignificand::get() as f64) / (100 as f64);
                let g = 1.0 - p;
                let dy = (a * g * y) / (1.0 + (a * g));
                let x1 = x + dx;
                let y1 = y - dy;
​
                let balanceX = <OwnedX<T>>::get(&sender) - x_amount;
                <OwnedX<T>>::insert(&sender, balanceX);
                let balanceY = <OwnedY<T>>::get(&sender) + (dy as u64);
                <OwnedY<T>>::insert(&sender, balanceY);
                PoolX::put(x1 as u64);
                PoolY::put(y1 as u64);
​
            }
​
            Ok(())
        }
​
        fn sell_y(origin, y_amount: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
​
            if <OwnedX<T>>::get(&sender) < y_amount {
                //nejaky mesage a exit ?
            }
            else {
                let x = PoolX::get() as f64;
                let y = PoolY::get() as f64;
                let dy = y_amount as f64;
                let b = dy/y;
                let y1 = (1.0 + b) * y;
                let base = 10 as f64;
                // nefunguje neviem preco
                // let p = (FeeSignificand::get() as f64) / ( f64::powi(10.0,FeeDecimals) );
                let p = (FeeSignificand::get() as f64) / (100 as f64);
                let g = 1.0 - p;
                let dx = (b * g * x) / (1.0 + (b * g));
                let x1 = y - dy;
​
                let balanceX = <OwnedX<T>>::get(&sender) + (dx as u64);
                <OwnedX<T>>::insert(&sender, balanceX);
                let balanceY = <OwnedY<T>>::get(&sender) - y_amount;
                <OwnedY<T>>::insert(&sender, balanceY);
                PoolX::put(x1 as u64);
                PoolY::put(y1 as u64);
​
            }
​
            Ok(())
        }
​
        fn buy_y(origin, y_amount: u64) -> DispatchResult { 
            let sender = ensure_signed(origin)?;                    
            let x = PoolX::get() as f64;
            let y = PoolY::get() as f64;
            let dy = y_amount as f64;                           
            let base = 10 as f64;
            // nefunguje neviem preco
            // let p = (FeeSignificand::get() as f64) / ( f64::powi(10.0,FeeDecimals) );
            let p = (FeeSignificand::get() as f64) / (100 as f64);
            let g = 1.0 - p;
            let dx = (dy * x) / ((g * x) - (g * dy));
            let x1 = x + dx;
            let y1 = y - dy;
​
            if <OwnedX<T>>::get(&sender) < (dx as u64) {
                //nejaky mesage a exit ?
            }
            else {
                let balanceX = <OwnedX<T>>::get(&sender) - (dx as u64);
                <OwnedX<T>>::insert(&sender, balanceX);
                let balanceY = <OwnedY<T>>::get(&sender) + y_amount;
                <OwnedY<T>>::insert(&sender, balanceY);
                PoolX::put(x1 as u64);
                PoolY::put(y1 as u64);
​
            }
​
            Ok(())
        }
​
        fn buy_x(origin, x_amount: u64) -> DispatchResult { 
            let sender = ensure_signed(origin)?;      
            let x = PoolX::get() as f64;
            let y = PoolY::get() as f64;
            let dx = x_amount as f64;            
            let base = 10 as f64;
            // nefunguje neviem preco
            // let p = (FeeSignificand::get() as f64) / ( f64::powi(10.0,FeeDecimals) );
            let p = (FeeSignificand::get() as f64) / (100 as f64);
            let g = 1.0 - p;
            let dy = (dx * y) / ((g * y) - (g * dx));
            let x1 = x + dx;
            let y1 = y - dy;
​
            if <OwnedY<T>>::get(&sender) < (dy as u64) {
                //nejaky mesage a exit ?
            }
            else {
                let balanceX = <OwnedX<T>>::get(&sender) + x_amount;
                <OwnedX<T>>::insert(&sender, balanceX);
                let balanceY = <OwnedY<T>>::get(&sender) - (dy as u64);
                <OwnedY<T>>::insert(&sender, balanceY);
                PoolX::put(x1 as u64);
                PoolY::put(y1 as u64);
​
            }
​
            Ok(())
        }
        
	}
}