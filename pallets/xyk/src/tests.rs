use super::*;
use crate::mock::*;

use frame_support::{assert_ok, assert_noop};

//set_vault working, id set
#[test]
fn set_vault_works() {
	new_test_ext().execute_with(|| {
        assert_ok!(XykStorage::set_vault_id(Origin::signed(1)));
        assert_eq!(XykStorage::vault_id(), 1);
	});
}

//set_vault not working, already initialized
#[test]
fn set_vault_not_work_if_already_initiated() {
	new_test_ext().execute_with(|| {
		XykStorage::set_vault_id(Origin::signed(1));

		assert_noop!(XykStorage::set_vault_id(Origin::signed(1)),
			Error::<Test>::NoIdAvailable
		);
	   

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

