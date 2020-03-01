use super::*;
use crate::mock::*;

use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_set_vault() {
	new_test_ext().execute_with(|| {
		// Just a dummy test for the dummy function `do_something`
		// calling the `do_something` function with a value 42
		//      assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// asserting that the stored value is equal to what we stored
        //      assert_eq!(TemplateModule::something(), Some(42));
        assert_ok!(XykStorage::set_vault_id(Origin::signed(1)));
        assert_eq!(XykStorage::vault_id(), 1);
	});
}