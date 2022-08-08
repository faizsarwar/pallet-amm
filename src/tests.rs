// use crate::{Error ,mock::*};
// use frame_support::{assert_ok, assert_noop};
// use super::*;


// // #[test]
// // fn init_works() {
// // 	new_test_ext().execute_with(|| {
// //         let caller=Origin::signed(1);
// //         stk_amm::init(Origin::signed(1),0).unwrap();
// // 		assert_eq!( stk_amm::getMyHoldings( ensure_signed(caller).unwrap()) , (0, 0, 0) );
// //         assert_eq!( stk_amm::getPoolDetails() ,(0, 0, 0,0 ));
// // 	});
// // }


// // #[test]
// // fn faucet_works() {
// // 	new_test_ext().execute_with(|| {
// //         let caller=Origin::signed(1);
// //         stk_amm::init(Origin::signed(1),0).unwrap();
// //         stk_amm::faucet(caller.clone(), 100, 200).unwrap();
// // 		assert_eq!( stk_amm::getMyHoldings( ensure_signed(caller).unwrap()) , (100, 200, 0) );
// // 	});
// // }


// // this test is remaining 

// // #[test]
// // fn zero_liquidity_test() {
// // 	new_test_ext().execute_with(|| {
// //         let caller=Origin::signed(1);
// //         let res = stk_amm::getEquivalentToken1Estimate(5);
// //         // stk_amm::faucet(caller.clone(), 100, 200);
// //         // assert_err!(res, Err(Error::<Test, _>::ZeroLiquidity));
// // 	});
// // }


// #[test]
// fn provide_works() {
// 	new_test_ext().execute_with(|| {
//         let caller=Origin::signed(1);
//         // stk_amm::init(Origin::signed(1),0).unwrap();
//         // stk_amm::faucet(Origin::signed(1), 100, 200).unwrap();

//         // CREATING ASSETS FOR USER WITH ID 1
//         assert_ok!(Assets::force_create(Origin::root(), 1, 1, true, 1));  
//         assert_ok!(Assets::force_create(Origin::root(), 2, 1, true, 1));  

//         // MINTING SOME ASSETS FOR USER 1
//         assert_ok!(Assets::mint(Origin::signed(1), 1, 1, 100));   
//         assert_ok!(Assets::mint(Origin::signed(1), 2, 1, 100));   


//         stk_amm::CreatePool( Origin::signed(1) ,1 ,2 , 0 ).unwrap();
//         stk_amm::provide(Origin::signed(1), 10, 20, 1, 100, 200).unwrap(); 
//         let share = stk_amm::shares(1);
// 		assert_eq!( share ,100);
//         assert_eq!( stk_amm::getPoolDetails(), (10, 20, share, 0));
//         assert_eq!( stk_amm::getMyHoldings(ensure_signed(caller.clone()).unwrap()), (90, 180, share));
// 	});
// }



// #[test]
// fn withdraw_works() {
// 	new_test_ext().execute_with(|| {
//         // stk_amm::init(Origin::signed(1),0).unwrap();
//         // stk_amm::faucet(Origin::signed(1), 100, 200).unwrap();

//         // CREATING ASSETS FOR USER WITH ID 1
//         assert_ok!(Assets::force_create(Origin::root(), 1, 1, true, 1)); 
//         assert_ok!(Assets::force_create(Origin::root(), 2, 1, true, 1)); 

//         // MINTING SOME ASSETS FOR USER 1
//         assert_ok!(Assets::mint(Origin::signed(1), 1, 1, 100));         
//         assert_ok!(Assets::mint(Origin::signed(1), 2, 1, 100));         


//         stk_amm::CreatePool( Origin::signed(1) ,1 ,2 , 0).unwrap(); 
//         stk_amm::provide(Origin::signed(1),10,20,1, 100,200).unwrap(); 

//         let share = stk_amm::shares(1);
//         assert_eq!(stk_amm::getWithdrawEstimate(&1, 1, share / 5).unwrap(), (2, 4));
//         stk_amm::withdraw(Origin::signed(1),1, share / 5).unwrap();  
//         assert_eq!(stk_amm::getMyHoldings(1), (92, 184, 4 * share / 5));
//         assert_eq!(stk_amm::getPoolDetails(), (8, 16, 4 * share / 5, 0)); 
// 	});
// }



// #[test]
// fn swap_works() {
// 	new_test_ext().execute_with(|| {
//         // stk_amm::init(Origin::signed(1),0).unwrap();
//         // stk_amm::faucet(Origin::signed(1), 100, 200).unwrap();


//         // CREATING ASSETS FOR USER WITH ID 1
//         assert_ok!(Assets::force_create(Origin::root(), 1, 1, true, 1));
//         assert_ok!(Assets::force_create(Origin::root(), 2, 1, true, 1));

//         // MINTING SOME ASSETS FOR USER 1
//         assert_ok!(Assets::mint(Origin::signed(1), 1, 1, 100));         
//         assert_ok!(Assets::mint(Origin::signed(1), 2, 1, 100));         


//         stk_amm::CreatePool( Origin::signed(1) ,1 ,2 ,0).unwrap();
//         stk_amm::provide(Origin::signed(1),50,100,1 ,100, 200 ).unwrap(); 


//         assert_eq!(stk_amm::getSwapToken1EstimateGivenToken1(&1 ,1, 50).unwrap(), 50);
//         stk_amm::swapToken1GivenToken1(Origin::signed(1),1,50, 50).unwrap();
//         let share = stk_amm::shares(1);
//         assert_eq!(stk_amm::getMyHoldings(1), (0, 150, share));
//         assert_eq!(stk_amm::getPoolDetails(), (100, 50, share, 0));
// 	});
// }


// #[test]
// fn slippage_works() {
// 	new_test_ext().execute_with(|| {
//         // stk_amm::init(Origin::signed(1),0).unwrap();
//         // stk_amm::faucet(Origin::signed(1), 100, 200).unwrap();


//         // CREATING ASSETS FOR USER WITH ID 1
//         assert_ok!(Assets::force_create(Origin::root(), 1, 1, true, 1));  
//         assert_ok!(Assets::force_create(Origin::root(), 2, 1, true, 1));  

//         // MINTING SOME ASSETS FOR USER 1
//         assert_ok!(Assets::mint(Origin::signed(1), 1, 1, 100));         
//         assert_ok!(Assets::mint(Origin::signed(1), 2, 1, 100));         


//         stk_amm::CreatePool( Origin::signed(1) ,1 ,2 , 0).unwrap(); 
//         stk_amm::provide(Origin::signed(1), 50, 100, 1, 100 , 200).unwrap(); 

//         assert_eq!(stk_amm::getSwapToken1EstimateGivenToken1(&1 ,1, 50).unwrap(), 50);
        
// 		// ditribute fee
// 		assert_noop!(stk_amm::swapToken1GivenToken1(Origin::signed(1),1,50, 51),Error::<Test, _>::SlippageExceeded);

//         let share = stk_amm::shares(1);
//         assert_eq!(stk_amm::getMyHoldings(1), (50, 100, share)); 
//         assert_eq!(stk_amm::getPoolDetails(), (50, 100, share, 0));    
// 	});
// }


// #[test]
// fn trading_fees_works() {
// 	new_test_ext().execute_with(|| {
//         // stk_amm::init(Origin::signed(1),100).unwrap();
//         // stk_amm::faucet(Origin::signed(1), 100, 200).unwrap();

//         // CREATING ASSETS FOR USER WITH ID 1
//         assert_ok!(Assets::force_create(Origin::root(), 1, 1, true, 1));  
//         assert_ok!(Assets::force_create(Origin::root(), 2, 1, true, 1));  
        
//         // MINTING SOME ASSETS FOR USER 1
//         assert_ok!(Assets::mint(Origin::signed(1), 1, 1, 100));         
//         assert_ok!(Assets::mint(Origin::signed(1), 2, 1, 100));         
        
        
//         stk_amm::CreatePool( Origin::signed(1) ,1 ,2  ,100).unwrap(); 
//         stk_amm::provide(Origin::signed(1),50,100,1,100 ,200).unwrap(); 

//         let amountToken2 = stk_amm::getSwapToken1EstimateGivenToken1(&1,1,50).unwrap();
//         assert_eq!(amountToken2, 48);
// 	});
// }