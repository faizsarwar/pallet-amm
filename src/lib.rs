#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
use frame_support::{decl_module,ensure, decl_storage, decl_event, decl_error, dispatch, PalletId, traits::{Get,Currency,ReservableCurrency,ExistenceRequirement::KeepAlive}};
use sp_runtime::traits::{CheckedAdd, CheckedSub, CheckedDiv,AccountIdConversion,StaticLookup};
use codec::Encode;
use sp_std::{prelude::*,convert::TryInto};
use frame_system::ensure_signed;
pub use pallet_assets;
pub use sp_io;
use frame_support::dispatch::EncodeLike;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::StorageHasher;
use frame_support::dispatch::RawOrigin;
mod types;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


const PALLET_NAME: &'static str = "Assets";
const STORAGE_NAME: &'static str = "Asset";
pub type BalanceOf<T> =	 <T as pallet_assets::Config>::Balance;
pub type CurrencyBalanceOf<T, I = ()> =	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;



pub trait Config<I=DefaultInstance>: frame_system::Config +  pallet_assets::Config {
	type Event: From<Event<Self, I>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	type PalletId: Get<PalletId>;
}



impl<T: Config<I>, I: Instance> Module<T, I>  where u32: EncodeLike<<T as pallet_assets::Config>::AssetId> {

	fn get_storage_value(key: <T as pallet_assets::Config>::AssetId) -> Option<AssetDetails<BalanceOf<T>, T::AccountId, DepositBalanceOf<T, I>>> {
		let pallet_hash = sp_io::hashing::twox_128(PALLET_NAME.as_bytes());
		let storage_hash = sp_io::hashing::twox_128(STORAGE_NAME.as_bytes());
		
		let key_hashed = frame_support::Blake2_128Concat::hash(&key.encode());
		let mut final_key = Vec::new();
		final_key.extend_from_slice(&pallet_hash);
		final_key.extend_from_slice(&storage_hash);
		final_key.extend_from_slice(&key_hashed);
		frame_support::storage::unhashed::get::<AssetDetails<BalanceOf<T>, T::AccountId, DepositBalanceOf<T, I>>>(&final_key)
		
	}
	

	pub fn validAmountCheckToken1Balance(
		caller: &T::AccountId,
		_qty: BalanceOf<T>,
		pool_Id: u32,
	) -> Result<(), Error<T,I>> {
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);

		let my_balance=Token1Balance::<T,I>::get(caller, _Asset1);
		
		let my_balance= Self::balance_to_u128(my_balance);
		let _qty=Self::balance_to_u128(_qty);
		match _qty {
			0 => Err(Error::ZeroAmount),
			_ if _qty > my_balance => Err(Error::<T,I>::InsufficientAmount),
			_ => Ok(()),
		}
	}

	
		pub fn validAmountCheckToken2Balance(
			caller: &T::AccountId,
			_qty: BalanceOf<T>,
			pool_Id: u32,
		) -> Result<(), Error<T,I>> {
			
			let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);

			let my_balance=Token2Balance::<T,I>::get(caller,_Asset2);

			let my_balance= Self::balance_to_u128(my_balance);
			let _qty=Self::balance_to_u128(_qty);
			match _qty {
				0 => Err(Error::ZeroAmount),
				_ if _qty > my_balance => Err(Error::<T,I>::InsufficientAmount),
				_ => Ok(()),
			}
		}

		// have to add assetid as parameter
		pub fn getWithdrawEstimate(_caller: &T::AccountId, pool_Id: u32, _share: BalanceOf<T>) -> Result<(BalanceOf<T>, BalanceOf<T>), Error<T,I>> {
			Self::activePool( pool_Id)?;

			ensure!(_share <= TotalShares::<T,I>::get() , Error::<T, I>::InvalidShare);
			let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);

			let amountToken1 = _share * TotalToken1::<T,I>::get(_Asset1) / TotalShares::<T,I>::get();
			let amountToken2 = _share * TotalToken2::<T,I>::get(_Asset2) / TotalShares::<T,I>::get();

			Ok((amountToken1, amountToken2))

		}



	pub fn validAmountCheckShares(
		caller: &T::AccountId,
		_qty: BalanceOf<T>,
	) -> Result<(), Error<T,I>> {
		
		let my_balance=Shares::<T,I>::get(caller);

		let my_balance= Self::balance_to_u128(my_balance);
		let _qty=Self::balance_to_u128(_qty);
		match _qty {
			0 => Err(Error::ZeroAmount),
			_ if _qty > my_balance => Err(Error::<T,I>::InsufficientAmount),
			_ => Ok(()),
		}
	}

	pub fn getK(pool_Id: u32) -> BalanceOf<T>{ 
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
		let result = TotalToken1::<T,I>::get(_Asset1) * TotalToken2::<T,I>::get(_Asset2);
		result
	}

	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn u128_to_balance_option(input: u128) -> BalanceOf<T> {
		input.try_into().ok().unwrap()
	}


	pub fn u128_to_CurrencyBalance_option(input: u128) -> CurrencyBalanceOf<T,I>  {
		input.try_into().ok().unwrap()
	}


	pub fn balance_to_u128(input: BalanceOf<T>) -> u128 {
		TryInto::<u128>::try_into(input).ok().unwrap()
	}

    // Used to restrict withdraw & swap feature till liquidity is added to the pool
	pub fn activePool(pool_Id: u32) -> Result<(), Error<T,I>> { 
		let val=Self::balance_to_u128(Self::getK( pool_Id));

		match val {
			0 => Err(Error::<T,I>::ZeroLiquidity),
			_ => Ok(()),
		}
	}

	// have to add assetid as parameter
	pub fn getEquivalentToken1Estimate(pool_Id: u32, _amountToken2: BalanceOf<T> ) -> Result<BalanceOf<T>, Error<T,I>> {
		Self::activePool(pool_Id)?;
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
		Ok(TotalToken1::<T,I>::get(_Asset1) * _amountToken2 / TotalToken2::<T,I>::get(_Asset2))
	}

	// have to add assetid as parameter
	pub fn getEquivalentToken2Estimate(poolId: u32 , _amountToken1: BalanceOf<T> ) -> Result<BalanceOf<T>, Error<T,I>> {
		Self::activePool(poolId)?;
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(poolId);
		Ok(TotalToken2::<T,I>::get(_Asset2) * _amountToken1 / TotalToken1::<T,I>::get(_Asset1))
	}

	// have to add assetid as parameter
	pub fn getMyHoldings(caller: T::AccountId, pool_Id: u32) -> (BalanceOf<T>,BalanceOf<T>,BalanceOf<T>) {
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
		let token1 = Token1Balance::<T,I>::get(&caller, _Asset1);
		let token2 =  Token2Balance::<T,I>::get(&caller, _Asset2);
		let myShares =  Shares::<T,I>::get(&caller);
		(token1, token2, myShares)
	}

	// have to add assetid as parameter
	pub fn getPoolDetails(  pool_Id:u32) -> (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>,BalanceOf<T>) {
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
		(
			TotalToken1::<T,I>::get(_Asset1),
			TotalToken2::<T,I>::get(_Asset2),
			TotalShares::<T,I>::get(),
			Fees::<T,I>::get(),
		)
	}

	// Returns the amount of Token2 that the user will get when swapping a given amount of Token1 for Token2
	pub fn getSwapToken1EstimateGivenToken1(  pool_Id: u32, _amountToken1: BalanceOf<T>) -> Result<(BalanceOf<T>,BalanceOf<T>), Error<T,I>> {
		Self::activePool(pool_Id)?;
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
	

		// Adjusting the fees charged
		let AmountToken1 = (Self::u128_to_balance_option(100) - Fees::<T,I>::get()) * _amountToken1 / Self::u128_to_balance_option(100); 
		let token1After = TotalToken1::<T,I>::get(_Asset1).checked_add(&AmountToken1).unwrap();  
		let token2After = Self::getK( pool_Id).checked_div(&token1After).unwrap();  
		let mut amountToken2 = TotalToken2::<T,I>::get(_Asset2).checked_sub(&token2After).unwrap(); 
	
		// To ensure that Token2's pool is not completely depleted leading to inf:0 ratio
		if amountToken2 == TotalToken2::<T,I>::get(_Asset2) {
			amountToken2 = amountToken2.checked_sub(&Self::u128_to_balance_option(1)).unwrap();  
		}

		// subtracting acutal amount with calculated amount
		let fee_charged =_amountToken1 - AmountToken1;

		Ok((amountToken2,fee_charged))
	}
	// token1 30 , token2 50


	// Returns the amount of Token1 that the user should swap to get _amountToken2 in return
	pub fn getSwapToken1EstimateGivenToken2(
		pool_Id: u32,
		_amountToken2: BalanceOf<T>,
	) -> Result<(BalanceOf<T>,BalanceOf<T>), Error<T,I>> {
		Self::activePool(pool_Id)?;
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get(pool_Id);
		if _amountToken2 >= TotalToken2::<T,I>::get(_Asset2) {
			return Err(Error::<T,I>::InsufficientLiquidity);
		}

		let token2After = TotalToken2::<T,I>::get(_Asset2) - _amountToken2;
		let token1After = Self::getK( pool_Id) / token2After;
		let hundred= Self::u128_to_balance_option(100);
		let amountToken1 = (token1After - TotalToken1::<T,I>::get(_Asset1)) * hundred / (hundred - Fees::<T,I>::get());

		let fee_charged = (amountToken1 * Fees::<T,I>::get()) / hundred;
		Ok((amountToken1,fee_charged))  
	}
}



decl_storage! {

	trait Store for Module<T: Config<I>, I: Instance=DefaultInstance> as pallet_stk_amm  where u32: EncodeLike<<T as pallet_assets::Config>::AssetId> {
		TotalShares get(fn total_shares): BalanceOf<T>; // Stores the total amount of share issued for the pool
		TotalToken1 get(fn total_token1): map hasher(blake2_128_concat) T::AssetId => BalanceOf<T>; // Stores the amount of Token1 locked in the pool  (of which asset ?)
		TotalToken2 get(fn total_token2): map hasher(blake2_128_concat) T::AssetId => BalanceOf<T>; // Stores the amount of Token2 locked in the pool	(of which asset ?)
		TotalFeeCollected get(fn feeCollected): map hasher(blake2_128_concat) T::AssetId => BalanceOf<T>; 
		Fees get(fn fees): BalanceOf<T>;                // Percent of trading fees charged on trade
        Shares get(fn shares): map hasher(blake2_128_concat) T::AccountId => BalanceOf<T>; // Stores the share holding of each provider
		Token1Balance get(fn token1_balance): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AssetId => BalanceOf<T>;   // Stores the token1 balance of each user  (of which asset ?)
		Token2Balance get(fn token2_balance): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AssetId => BalanceOf<T>;    // Stores the token2 balance of each user  (of which asset ?)
		LiquidityProvider get(fn liquidity_provider): map hasher(blake2_128_concat) u32 => Vec<T::AccountId>;
		PoolPairs get(fn poolpairs): Vec<(T::AssetId, T::AssetId)>;
		PoolId get(fn pool_id): u32;   // it will store value to calculate next id of the pool 
		Pool get(fn pool) : map hasher(blake2_128_concat) u32 => (T::AssetId, T::AssetId, bool, bool); // stores the assetsIDs locked by the user in the pool 
	}


	add_extra_genesis {
		build(|_config| {
			let account_id = <Module<T, I>>::account_id();
			let min = <T as Config<I>>::Currency::minimum_balance();
			if <T as Config<I>>::Currency::free_balance(&account_id) < min {
				let _ = <T as Config<I>>::Currency::make_free_balance_be(
					&account_id,
					min,
				);
			}
		});
	}

}



decl_event!(
	pub enum Event<T, I=DefaultInstance> where <T as frame_system::Config>::AccountId,Balance = BalanceOf<T>, AssetId= <T as pallet_assets::Config>::AssetId {
		LiquidityProviderAdressAdded(AccountId),
		ShareDistributed(Balance),
		SwapedToken1GivenToken1(Balance,Balance),
		SwapedToken1GivenToken2(Balance,Balance),
		AssetsDistributed(AssetId , AssetId),
		PoolCreated(AccountId, AssetId , AssetId),
	}
);



decl_error! {
	pub enum Error for Module<T: Config<I>, I: Instance>  where u32: EncodeLike<<T as pallet_assets::Config>::AssetId> {
	NoneValue,

	StorageOverflow,

    ZeroLiquidity,

    ZeroAmount,

    InsufficientAmount,

    NonEquivalentValue,

    ThresholdNotReached,

    InvalidShare,

    InsufficientLiquidity,

    SlippageExceeded,

	LiquidityAdressExists,

	AssetNotExists,

	PoolIdAlreadyExists,

	PoolPairAlreadyExists,

	BothTokensCannotBeNative
	}
}



decl_module! {

	pub struct Module<T: Config<I>, I: Instance=DefaultInstance> for enum Call where origin: T::Origin , u32: EncodeLike<<T as pallet_assets::Config>::AssetId>  {
		type Error = Error<T,I>;

		fn deposit_event() = default;

	////Our constructor takes _fees as a parameter which determines the percent of fees the user is charged when performing a swap operation. The value of _fees should be between 0 and 1000 (exclusive) so that any swap operation will be charged _fees/1000 percent of the amount deposited. 	
	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn SetFees(origin, _fees: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;
		let val=Self::u128_to_balance_option(1000);
		let zero=Self::u128_to_balance_option(0);
		if _fees >= val {
			Fees::<T,I>::put(zero);
		}
		else{
			Fees::<T,I>::put(_fees);
		}
		Ok(())
	}


	// Distributing fee to lp providers
	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn distributeShareFee(origin, _fees: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;
		let TotalPoolIds = PoolId::<I>::get();

		let _fee= Fees::<T,I>::get();
		let totalShare= TotalShares::<T,I>::get();

		for pool_Id in 0..(TotalPoolIds+1){
			let _lp= LiquidityProvider::<T,I>::get(pool_Id);
			let (_AssetId1 , _AssetId2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);
	
			for lp_Provider in _lp{
				let lp_provider_share = Shares::<T,I>::get(&lp_Provider);
				let DistributionAmount= (lp_provider_share / totalShare) * _fee;  // percentage of share multiplied with fee
	
				// // converting Asset balance type to integer
				let amount1= Self::balance_to_u128(DistributionAmount);
				// // converting integer to currency balance type 
				let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount1);

				if _IsToken1Native & !_IsToken2Native {
					// transferring tokens
					<T as Config<I>>::Currency::transfer( &Self::account_id(), &lp_Provider , Amount_in_currency_balance , KeepAlive)?;
					<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId2 , T::Lookup::unlookup(lp_Provider.clone()), DistributionAmount)?;
				}
				else if _IsToken2Native & !_IsToken1Native {
					<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId1 , T::Lookup::unlookup(lp_Provider.clone()), DistributionAmount)?;
					<T as Config<I>>::Currency::transfer(  &Self::account_id(),&lp_Provider, Amount_in_currency_balance , KeepAlive)?;
				}
				else{
					// both tokens should not be native 
					ensure!( (_IsToken1Native && _IsToken2Native)!=true  , Error::<T, I>::BothTokensCannotBeNative);		
		
					// transfer asset to pools address
					<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId1 , T::Lookup::unlookup(lp_Provider.clone()), DistributionAmount)?;
					<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId2 , T::Lookup::unlookup(lp_Provider.clone()), DistributionAmount)?;
				}

				// updating the storage
				let NewFee= TotalFeeCollected::<T,I>::get(_AssetId1).checked_sub(&DistributionAmount).unwrap();
				TotalFeeCollected::<T,I>::insert(_AssetId1 , NewFee);
				let NewFee= TotalFeeCollected::<T,I>::get(_AssetId2).checked_sub(&DistributionAmount).unwrap();
				TotalFeeCollected::<T,I>::insert(_AssetId1 , NewFee);
			}	
		}

		Ok(())
	}


	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn CreatePool(origin, _amountToken1: BalanceOf<T>, _amountToken2: BalanceOf<T>, Asset1: T::AssetId, Asset2: T::AssetId,_IsToken1Native: bool , _IsToken2Native:bool , _fees: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;

		let mut pairs = PoolPairs::<T,I>::get();
		let assetPair= (Asset1,Asset2);
		// checking if the pair is present or not
		match  pairs.iter().find(|&&x| x == assetPair) {
			Some(_assetPair) => {
				Err(Error::<T,I>::PoolPairAlreadyExists.into())
			},
			None => {
				let ID:u32 = PoolId::<I>::get().checked_add(1).unwrap();

				// making sure user have assets in his account
				let _value1 = Self::get_storage_value(Asset1).unwrap();
				let _value2 = Self::get_storage_value(Asset2).unwrap();
				ensure!(_value1.owner == _who.clone(),Error::<T, I>::AssetNotExists);
				ensure!(_value2.owner == _who.clone(),Error::<T, I>::AssetNotExists);
				
				// making sure new id is unique
				ensure!(PoolId::<I>::get()!=ID, Error::<T, I>::PoolIdAlreadyExists);
		
				PoolId::<I>::put(ID);
				Pool::<T,I>::insert( PoolId::<I>::get(), (Asset1,Asset2,_IsToken1Native , _IsToken2Native));

				pairs.push(assetPair);
				PoolPairs::<T,I>::put(pairs);

				// add in pool pairs storage
				Self::deposit_event(Event::<T,I>::PoolCreated(_who, Asset1,Asset2));
				Ok(())
			}
		}
	}


	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn withdraw(origin, pool_Id: u32 ,_share: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;
		Self::validAmountCheckShares(&_who, _share)?;

		let (amountToken1, amountToken2) = Self::getWithdrawEstimate(&_who,  pool_Id ,_share)?;		
		
		// updating the share of the user
		let newShares= Shares::<T,I>::get(&_who).checked_sub(&_share).unwrap();
		Shares::<T,I>::insert(&_who, newShares);

		// updating the share of the Pool
		let newtotalShare= TotalShares::<T,I>::get().checked_sub(&_share).unwrap();
		TotalShares::<T,I>::put(newtotalShare);

		let (_AssetId1 , _AssetId2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);

		// Updating the amount of Assets locked in the pool
		let newTotalToken1= TotalToken1::<T,I>::get(_AssetId1).checked_sub(&amountToken1).unwrap();
		let newTotalToken2= TotalToken2::<T,I>::get(_AssetId2).checked_sub(&amountToken2).unwrap();

		TotalToken1::<T,I>::insert(_AssetId1, newTotalToken1);
		TotalToken2::<T,I>::insert(_AssetId2, newTotalToken2);

		// asset balance to be provided to the user
		let newTotalToken1balance = Token1Balance::<T,I>::get(&_who,_AssetId1).checked_add(&amountToken1).unwrap();
		let newTotalToken2balance = Token2Balance::<T,I>::get(&_who,_AssetId2).checked_add(&amountToken2).unwrap();

		Token1Balance::<T,I>::insert(&_who , _AssetId1, newTotalToken1balance);
		Token2Balance::<T,I>::insert(&_who , _AssetId2 , newTotalToken2balance);

		if _IsToken1Native & !_IsToken2Native {
			// converting Asset balance type to integer
			let amount1= Self::balance_to_u128(amountToken1);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount1);
			// transferring tokens
			<T as Config<I>>::Currency::transfer( &Self::account_id(), &_who , Amount_in_currency_balance , KeepAlive)?;
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}
		else if _IsToken2Native & !_IsToken1Native {
			// converting Asset balance type to integer
			let amount2= Self::balance_to_u128(amountToken2);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount2);
			// transferring tokens
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId1 , T::Lookup::unlookup(_who.clone()), amountToken1)?;
			<T as Config<I>>::Currency::transfer( &Self::account_id(),&_who, Amount_in_currency_balance , KeepAlive)?;
		}
		else{
			// both tokens should not be native 
			ensure!( (_IsToken1Native && _IsToken2Native)!=true  , Error::<T, I>::BothTokensCannotBeNative);		

			// transfer asset to pools address
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId1 , T::Lookup::unlookup(_who.clone()), amountToken1)?;
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _AssetId2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}

		Self::deposit_event(Event::<T,I>::AssetsDistributed(_AssetId1,_AssetId2));

		Ok(())
	}


	// Swaps given amount of Token1 to Token2 using algorithmic price determination
	// Swap fails if Token2 amount is less than _minToken2
	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn swapToken1GivenToken1( origin, pool_Id: u32, amountToken1: BalanceOf<T>, minToken2: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;
		Self::validAmountCheckToken1Balance(&_who , amountToken1, pool_Id)?;

		// subtracting the fee with the help of amount1	
		let (amountToken2,_fee_charged) = Self::getSwapToken1EstimateGivenToken1(pool_Id, amountToken1 )?;	   
		ensure!(amountToken2 >= minToken2 ,Error::<T,I>::SlippageExceeded);
		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);

		// updating the storage
		let totalToken1balance = Token1Balance::<T,I>::get(&_who, _Asset1).checked_sub(&amountToken1).unwrap();
		Token1Balance::<T,I>::insert(&_who , _Asset1 , totalToken1balance);
		let totalToken2balance = Token2Balance::<T,I>::get(&_who,_Asset2).checked_add(&amountToken2).unwrap();
		Token2Balance::<T,I>::insert(&_who, _Asset2 ,totalToken2balance);		

		// check if assets then transfer asset else currency
		if _IsToken1Native & !_IsToken2Native {
			// converting Asset balance type to integer
			let amount1= Self::balance_to_u128(amountToken1);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount1);
			
			// getting  token1 form user
			<T as Config<I>>::Currency::transfer( &_who, &Self::account_id() , Amount_in_currency_balance , KeepAlive)?;
			// transferring tokens to user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _Asset2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}
		else if _IsToken2Native & !_IsToken1Native {
			// // converting Asset balance type to integer
			let amount2= Self::balance_to_u128(amountToken2);
			// // converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount2);
			// getting  token1 form user  (origin,assetid,target,amount)
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(_who.clone()).into()  , _Asset1 , T::Lookup::unlookup(Self::account_id().clone()), amountToken1)?;
			// // transferring tokens
			<T as Config<I>>::Currency::transfer(  &Self::account_id(), &_who, Amount_in_currency_balance , KeepAlive)?; // source , dest ,value , _
		}
		else{		
			// both tokens should not be native 
			ensure!( (_IsToken1Native && _IsToken2Native)!=true  , Error::<T, I>::BothTokensCannotBeNative);	
			// getting  token1 form user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(_who.clone()).into()  , _Asset1 , T::Lookup::unlookup(Self::account_id().clone()), amountToken2)?;
			// transferring tokens to user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _Asset2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}

		// swapping the two tokens in the pool
		let total_token1 = TotalToken1::<T,I>::get(_Asset1).checked_add(&amountToken1).unwrap(); 
		let total_token2 = TotalToken2::<T,I>::get(_Asset2).checked_sub(&amountToken2).unwrap(); 
		TotalToken1::<T,I>::insert(_Asset1, total_token1 );
		TotalToken2::<T,I>::insert(_Asset2, total_token2 );

		// updating the share of the user
		let FeeCollected= TotalFeeCollected::<T,I>::get(_Asset1).checked_add(&_fee_charged).unwrap();
		TotalFeeCollected::<T,I>::insert(&_Asset1, FeeCollected);

		Self::deposit_event(Event::<T,I>::SwapedToken1GivenToken1(amountToken1, amountToken2));
		Ok(())
	}

	
	// Swaps given amount of Token1 to Token2 using algorithmic price determination
	// Swap fails if amount of Token1 required to obtain _amountToken2 exceeds _maxToken1
	#[weight = 10_000 + T::DbWeight::get().writes(1)]
	pub fn swapToken1GivenToken2(origin, pool_Id:u32 ,_maxToken1: BalanceOf<T>, amountToken2: BalanceOf<T>) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin)?;

		let (amountToken1,_fee_charged) = Self::getSwapToken1EstimateGivenToken2(pool_Id, amountToken2)?;

		ensure!(amountToken1 <= _maxToken1 ,Error::<T,I>::SlippageExceeded);

		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);
		Self::validAmountCheckToken1Balance(&_who , amountToken1 , pool_Id)?;
	
		let totalToken1balance = Token1Balance::<T,I>::get(&_who,_Asset1);
		Token1Balance::<T,I>::insert(&_who, _Asset1 ,totalToken1balance - amountToken1);

		let totalToken2balance = Token2Balance::<T,I>::get(&_who,_Asset2);
		Token2Balance::<T,I>::insert(&_who ,_Asset2,totalToken2balance + amountToken2);

		// check if assets then transfer asset else currency
		if _IsToken1Native & !_IsToken2Native {
			// converting Asset balance type to integer
			let amount1= Self::balance_to_u128(amountToken1);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount1);
			
			// getting  token1 form user
			<T as Config<I>>::Currency::transfer( &_who , &Self::account_id() , Amount_in_currency_balance , KeepAlive)?;
			// transferring tokens to user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _Asset2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}
		else if _IsToken2Native & !_IsToken1Native {
			// // converting Asset balance type to integer
			let amount2= Self::balance_to_u128(amountToken2);
			// // converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount2);
			// getting  token1 form user  (origin,assetid,target,amount)
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(_who.clone()).into()  , _Asset1 , T::Lookup::unlookup(Self::account_id().clone()), amountToken1)?;
			// // transferring tokens
			<T as Config<I>>::Currency::transfer(  &Self::account_id(), &_who, Amount_in_currency_balance , KeepAlive)?; // source , dest ,value , _
		}
		else{		
			// both tokens should not be native 
			ensure!( (_IsToken1Native && _IsToken2Native)!=true  , Error::<T, I>::BothTokensCannotBeNative);	
			// getting  token1 form user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(_who.clone()).into()  , _Asset1 , T::Lookup::unlookup(Self::account_id().clone()), amountToken2)?;
			// transferring tokens to user
			<pallet_assets::Pallet<T>>::transfer( RawOrigin::Signed(Self::account_id().clone()).into()  , _Asset2 , T::Lookup::unlookup(_who.clone()), amountToken2)?;
		}

		// updating total pool balance 
		let total_token1 = TotalToken1::<T,I>::get(_Asset1); 
		let total_token2 = TotalToken2::<T,I>::get(_Asset2); 
		TotalToken1::<T,I>::insert(_Asset1 , total_token1 + amountToken1);
		TotalToken2::<T,I>::insert(_Asset2 , total_token2 - amountToken2);
		
		// updating the share of the user
		let FeeCollected= TotalFeeCollected::<T,I>::get(_Asset1).checked_add(&_fee_charged).unwrap();
		TotalFeeCollected::<T,I>::insert(&_Asset1, FeeCollected);

		Self::deposit_event(Event::<T,I>::SwapedToken1GivenToken2(amountToken1, amountToken2));

		Ok(())
	}


	#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
	pub fn provide(origin, _amountToken1: BalanceOf<T>, _amountToken2: BalanceOf<T>, pool_Id: u32) -> dispatch::DispatchResult {
		let _who = ensure_signed(origin.clone())?;

		Self::validAmountCheckToken1Balance(&_who,_amountToken1, pool_Id).unwrap();
		Self::validAmountCheckToken2Balance(&_who,_amountToken2, pool_Id).unwrap();

		let share:BalanceOf<T>;

		
		let (AssetId1 , AssetId2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);

		if TotalShares::<T,I>::get() == Self::u128_to_balance_option(0) {	
			// default share 
			share = Self::u128_to_balance_option(100);  
		} else {
			//both value should be the same and it will be stored as share of user
			let share1 = TotalShares::<T,I>::get() * _amountToken1 / TotalToken1::<T,I>::get(AssetId1);
			
			let share2 = TotalShares::<T,I>::get() * _amountToken2 / TotalToken2::<T,I>::get(AssetId2);
				
			ensure!(share1 == share2,Error::<T,I>::NonEquivalentValue);
			share = share1;
		}

		ensure!(share != Self::u128_to_balance_option(0) , Error::<T,I>::ThresholdNotReached);

		// asset balance to be provided to the pool
		let newToken1balance=Token1Balance::<T,I>::get(&_who, AssetId1).checked_sub(&_amountToken1).unwrap();  
		let newToken2balance=Token2Balance::<T,I>::get(&_who, AssetId2).checked_sub(&_amountToken2).unwrap();

		Token1Balance::<T,I>::insert(&_who , AssetId1 , newToken1balance);
		Token2Balance::<T,I>::insert(&_who , AssetId2 ,newToken2balance);

		if _IsToken1Native & !_IsToken2Native {
			// converting Asset balance type to integer
			let amount1= Self::balance_to_u128(_amountToken1);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount1);
			// transferring tokens
			<T as Config<I>>::Currency::transfer(&_who, &Self::account_id(), Amount_in_currency_balance , KeepAlive)?;
			<pallet_assets::Pallet<T>>::transfer( origin.clone() , AssetId2 , T::Lookup::unlookup(Self::account_id().clone()),_amountToken2)?;
		}
		else if _IsToken2Native & !_IsToken1Native {
			// converting Asset balance type to integer
			let amount2= Self::balance_to_u128(_amountToken2);
			// converting integer to currency balance type 
			let Amount_in_currency_balance = Self::u128_to_CurrencyBalance_option(amount2);
			// transferring tokens
			<pallet_assets::Pallet<T>>::transfer( origin.clone() , AssetId1 , T::Lookup::unlookup(Self::account_id().clone()), _amountToken1)?;
			<T as Config<I>>::Currency::transfer( &_who, &Self::account_id(), Amount_in_currency_balance , KeepAlive)?;
		}
		else{
		
		// both tokens should not be native same time
		ensure!( (_IsToken1Native && _IsToken2Native)!=true  , Error::<T, I>::BothTokensCannotBeNative);		

		// transfer asset to pools address
		<pallet_assets::Pallet<T>>::transfer( origin.clone() , AssetId1 , T::Lookup::unlookup(Self::account_id().clone()), _amountToken1)?;
		<pallet_assets::Pallet<T>>::transfer( origin.clone() , AssetId2 , T::Lookup::unlookup(Self::account_id().clone()),_amountToken2)?;
		}


		Self::deposit_event(Event::<T,I>::AssetsDistributed(AssetId1,AssetId2));

		let (_Asset1 , _Asset2, _IsToken1Native , _IsToken2Native) = Pool::<T,I>::get( pool_Id);

		// provided amount should be added to pools liquidity
		// Updating the amount of Assets locked in the pool
		let newTotalToken1=TotalToken1::<T,I>::get(_Asset1).checked_add(&_amountToken1).unwrap(); 
		let newTotalToken2=TotalToken2::<T,I>::get(_Asset2).checked_add(&_amountToken2).unwrap(); 
		TotalToken1::<T,I>::insert(_Asset1, newTotalToken1);
		TotalToken2::<T,I>::insert(_Asset2, newTotalToken2);

		// updating the total shares
		let newshares=TotalShares::<T,I>::get().checked_add(&share).unwrap();
		TotalShares::<T,I>::put(newshares);

		// updating the shares of the user
		let newCurrentshares = Shares::<T,I>::get(&_who).checked_add(&share).unwrap();
		Shares::<T,I>::insert(&_who, newCurrentshares );

		Self::deposit_event(Event::<T,I>::ShareDistributed(share));


		let token1 = Token1Balance::<T,I>::get(&_who, _Asset1);
		let token2 = Token2Balance::<T,I>::get(&_who, _Asset2);
		Token1Balance::<T,I>::insert(&_who ,_Asset1,_amountToken1 + token1);
		Token2Balance::<T,I>::insert(&_who ,_Asset2,_amountToken2 + token2);

		// Adding user to LiquidityProviders Vector to distribute share later
		let mut members = LiquidityProvider::<T,I>::get(pool_Id);

		let _x = match members.binary_search(&_who) {
			Ok(_) => {
				// do nothing
			},
			Err(index) => {
				members.insert(index, _who.clone());
				LiquidityProvider::<T,I>::insert(pool_Id, members);
				Self::deposit_event(Event::<T,I>::LiquidityProviderAdressAdded(_who));			
			}
		};

		Ok(())
	}

	}
}