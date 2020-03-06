#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::traits::SaturatedConversion;
use codec::{Decode, Encode, HasCompact, Input, Output, Error as CodecError};
use frame_support::{
    decl_event, decl_module, decl_storage, decl_error, dispatch::DispatchResult, ensure, StorageMap,
};

use generic_asset::{AssetOptions, Owner, PermissionLatest};
use system::ensure_signed;

use super::*;
use crate::mock::*;

use frame_support::{assert_ok, assert_noop};
pub trait Trait: generic_asset::Trait {
    // TODO: Add other types and constants required configure this module.
    // type Hashing = BlakeTwo256;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}
//set_vault working, id set
#[test]
fn set_vault_works() {
	new_test_ext().execute_with(|| {
        assert_ok!(XykStorage::set_vault_id(Origin::signed(1)));
        assert_eq!(XykStorage::vault_id(), 1);
	});
}


#[test]
fn testt() {
	new_test_ext().execute_with(|| {
		
		XykStorage::set_vault_id(Origin::signed(1));
			XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);

		let amount: T::Balance = 500;
		



		XykStorage::create_pool(
			Origin::signed(1),
			accid1,
			500,
			//500.saturated_into::<T::Balance>(),
			101,
			500,
		);
		assert_eq!(XykStorage::vault_id(), 1);
		assert_eq!(XykStorage::asset_pool((100,101)), 500);
		assert_eq!(XykStorage::liquidity_pool((100,101)), 102);
		assert_eq!(XykStorage::totalliquidity(102), 1000);

	//	assert_eq!(XykStorage::get_free_balance(102,2), 1000);
	});
}


//sell working assert (values as vault, values at vallet, values at maps)
//sell not working if not enough asset to sell
//sell not working if not enough liquidity in pool
//sell not working if pool does not exist

//buy working assert (values as vault, values at vallet, values at maps)
//buy not working if not enough asset to sell
//buy not working if not enough liquidity in pool
//buy not working if pool does not exist

//create_pool working assert (if exists in maps, has right hash/accId)
//create_pool not working if no such assets
//create_pool not working if pool already exists
//create_pool not working if pool already exists other way around (create pool X-Y, but pool Y-X exists)
//create_pool not working if no next hash/accId

//mint working assert (values as vault, values at vallet, values at maps)
//mint working if liquidity 0 assert (values as vault, values at vallet, values at maps)
//mint working if mint order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (values as vault, values at vallet, values at maps)
//mint not working if pool does not exist
//mint not enough assets to mint with

//burn working assert (values as vault, values at vallet, values at maps)
//burn working if burn order in different order as pool (mint pool X-Y, but pool Y-X exists), assert (values as vault, values at vallet, values at maps)
//burn not working if pool does not exist
//burn not enough liquidity assets to burn
//burn not working if not enough pool assets? (if maps are correct, previous test should fail)

