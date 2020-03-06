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

    // The overarching event type.
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

//create_pool working assert (right values in maps and accounts)
#[test]
fn create_pool_working() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500,
			1,
			500,
		);

		assert_eq!(XykStorage::asset_pool((0, 1)), 500); // amount in pool map
		assert_eq!(XykStorage::asset_pool((1, 0)), 500); // amount in pool map
		assert_eq!(XykStorage::liquidity_pool((0, 1)), 2); // liquidity assetId corresponding to newly created pool
		assert_eq!(XykStorage::totalliquidity(2), 1000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2, 2), 1000); // amount of liquidity assets owned by user by creating pool / initial minting (500+500)
		assert_eq!(XykStorage::get_free_balance(0, 2), 500); // amount in user acc after creating pool / initial minting 
		assert_eq!(XykStorage::get_free_balance(1, 2), 500); // amount in user acc after creating pool / initial minting 
		assert_eq!(XykStorage::get_free_balance(0, 1), 500); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1, 1), 500); // amount in vault acc
	});
}

#[test]
fn mint_working() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500,
			1,
			500,
		);
		// minting pool 0 1 with 250 assetId 0
		XykStorage::mint_liquidity(
			Origin::signed(2),
			0,
			1,
			250,
		);

		assert_eq!(XykStorage::totalliquidity(2), 1500); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1500); // amount of liquidity assets owned by user by creating pool and minting
		assert_eq!(XykStorage::get_free_balance(0,2), 250); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(1,2), 249); // amount in user acc after minting 
		assert_eq!(XykStorage::get_free_balance(0,1), 750); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 751); // amount in vault acc
	});
}

#[test]
fn burn_working() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500,
			1,
			500,
		);
		// burning 250 liquidity assetId2 of pool 0 1
		XykStorage::burn_liquidity(
			Origin::signed(2),
			0,
			1,
			500,
		);

		assert_eq!(XykStorage::totalliquidity(2), 500); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 500); // amount of liquidity assets owned by user by creating pool and burning
		assert_eq!(XykStorage::get_free_balance(0,2), 750); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(1,2), 750); // amount in user acc after burning 
		assert_eq!(XykStorage::get_free_balance(0,1), 250); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 250); // amount in vault acc
	});
}

#[test]
fn sell_working() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500000,
			1,
			500000,
		);
		// burning 250 liquidity assetId2 of pool 0 1
		XykStorage::sell_asset(
			Origin::signed(2),
			0,
			1,
			250000,
		);

		assert_eq!(XykStorage::totalliquidity(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 250000); // amount in user acc after selling 
		assert_eq!(XykStorage::get_free_balance(1,2), 666332); // amount in user acc after buying (check rounding should be 666333?)
		assert_eq!(XykStorage::get_free_balance(0,1), 750000); // amount in vault acc
		assert_eq!(XykStorage::get_free_balance(1,1), 333668); // amount in vault acc (check rounding should be 666337?)
	});
}

#[test]
fn buy_working() {
	new_test_ext().execute_with(|| {
		
		// setting vault to accountId 1
		XykStorage::set_vault_id(Origin::signed(1));
		// creating asset with assetId 0 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000000,
		);
		// creating asset with assetId 1 and minting to accountId 2
		XykStorage::create_asset_to(
		 	Origin::signed(2),
			1000000,
		);
		// creating pool by assetId 2
		XykStorage::create_pool(
			Origin::signed(2),
			0,
			500000,
			1,
			500000,
		);
		// burning 250 liquidity assetId2 of pool 0 1
		XykStorage::buy_asset(
			Origin::signed(2),
			0,
			1,
			150000,
		);

		assert_eq!(XykStorage::totalliquidity(2), 1000000); // total liquidity assets
		assert_eq!(XykStorage::get_free_balance(2,2), 1000000); // amount of liquidity assets owned by user by creating pool and initial minting
		assert_eq!(XykStorage::get_free_balance(0,2), 285069); // amount in user acc after selling (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,2), 650000); // amount in user acc after buying (check rounding )
		assert_eq!(XykStorage::get_free_balance(0,1), 714931); // amount in vault acc (check rounding)
		assert_eq!(XykStorage::get_free_balance(1,1), 350000); // amount in vault acc (check rounding)
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

//create_pool working assert (right values in maps and accounts)
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

