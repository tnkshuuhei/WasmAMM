#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

use ink_lang as ink;
const PRECISION: u128 = 1_000_000; // Precision of 6 digits

#[ink::contract]
mod amm {
    use ink_storage::collections::HashMap;

    // Part 1. Define Error enum 

    // Part 2. Define storage struct 

    // Part 3. Helper functions 

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[derive(Default)]
    #[ink(storage)]
    pub struct Amm {
        totalShares: Balance, // Stores the total amount of share issued for the pool
        totalToken1: Balance, // Stores the amount of Token1 locked in the pool
        totalToken2: Balance, // Stores the amount of Token2 locked in the pool
        shares: HashMap<AccountId, Balance>, // Stores the share holding of each provider
        token1Balance: HashMap<AccountId, Balance>, // Stores the token1 balance of each user
        token2Balance: HashMap<AccountId, Balance>, // Stores the token2 balance of each user
        fees: Balance,        // Percent of trading fees charged on trade
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Zero Liquidity
        ZeroLiquidity,
        /// Amount cannot be zero!
        ZeroAmount,
        /// Insufficient amount
        InsufficientAmount,
        /// Equivalent value of tokens not provided
        NonEquivalentValue,
        /// Asset value less than threshold for contribution!
        ThresholdNotReached,
        /// Share should be less than totalShare
        InvalidShare,
        /// Insufficient pool balance
        InsufficientLiquidity,
        /// Slippage tolerance exceeded
        SlippageExceeded,
    }
    #[ink(impl)]
    impl Amm {
    // Ensures that the _qty is non-zero and the user has enough balance
    fn validAmountCheck(
        &self,
        _balance: &HashMap<AccountId, Balance>,
        _qty: Balance,
    ) -> Result<(), Error> {
        let caller = self.env().caller();
        let my_balance = *_balance.get(&caller).unwrap_or(&0);

        match _qty {
            0 => Err(Error::ZeroAmount),
            _ if _qty > my_balance => Err(Error::InsufficientAmount),
            _ => Ok(()),
        }
    }

    // Returns the liquidity constant of the pool
    fn getK(&self) -> Balance {
        self.totalToken1 * self.totalToken2
    }

    // Used to restrict withdraw & swap feature till liquidity is added to the pool
    fn activePool(&self) -> Result<(), Error> {
        match self.getK() {
            0 => Err(Error::ZeroLiquidity),
            _ => Ok(()),
        }
    }
        // Part 4. Constructor
        /// Constructs a new AMM instance
        /// @param _fees: valid interval -> [0,1000)
        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            // Sets fees to zero if not in valid range
            Self {
                fees: if _fees >= 1000 { 0 } else { _fees },
                ..Default::default()
            }
        }        
        // Part 5. Faucet
        /// Sends free token(s) to the invoker
        #[ink(message)]
        pub fn faucet(&mut self, _amountToken1: Balance, _amountToken2: Balance) {
            let caller = self.env().caller();
            let token1 = *self.token1Balance.get(&caller).unwrap_or(&0);
            let token2 = *self.token2Balance.get(&caller).unwrap_or(&0);

            self.token1Balance.insert(caller, token1 + _amountToken1);
            self.token2Balance.insert(caller, token2 + _amountToken2);
        }
        // Part 6. Read current state
        /// Returns the balance of the user
        #[ink(message)]
        pub fn getMyHoldings(&self) -> (Balance, Balance, Balance) {
            let caller = self.env().caller();
            let token1 = *self.token1Balance.get(&caller).unwrap_or(&0);
            let token2 = *self.token2Balance.get(&caller).unwrap_or(&0);
            let myShares = *self.shares.get(&caller).unwrap_or(&0);
            (token1, token2, myShares)
        }

        /// Returns the amount of tokens locked in the pool,total shares issued & trading fee param
        #[ink(message)]
        pub fn getPoolDetails(&self) -> (Balance, Balance, Balance, Balance) {
            (
                self.totalToken1,
                self.totalToken2,
                self.totalShares,
                self.fees,
            )
        }
        // Part 7. Provide
        /// Adding new liquidity in the pool
        /// Returns the amount of share issued for locking given assets
        #[ink(message)]
        pub fn provide(
            &mut self,
            _amountToken1: Balance,
            _amountToken2: Balance,
        ) -> Result<Balance, Error> {
            self.validAmountCheck(&self.token1Balance, _amountToken1)?;
            self.validAmountCheck(&self.token2Balance, _amountToken2)?;

            let share;
            if self.totalShares == 0 {
                // Genesis liquidity is issued 100 Shares
                share = 100 * super::PRECISION;
            } else {
                let share1 = self.totalShares * _amountToken1 / self.totalToken1;
                let share2 = self.totalShares * _amountToken2 / self.totalToken2;

                if share1 != share2 {
                    return Err(Error::NonEquivalentValue);
                }
                share = share1;
            }

            if share == 0 {
                return Err(Error::ThresholdNotReached);
            }

            let caller = self.env().caller();
            let token1 = *self.token1Balance.get(&caller).unwrap();
            let token2 = *self.token2Balance.get(&caller).unwrap();
            self.token1Balance.insert(caller, token1 - _amountToken1);
            self.token2Balance.insert(caller, token2 - _amountToken2);

            self.totalToken1 += _amountToken1;
            self.totalToken2 += _amountToken2;
            self.totalShares += share;
            self.shares
                .entry(caller)
                .and_modify(|val| *val += share)
                .or_insert(share);

            Ok(share)
        }
        /// Returns amount of Token1 required when providing liquidity with _amountToken2 quantity of Token2
        #[ink(message)]
        pub fn getEquivalentToken1Estimate(
            &self,
            _amountToken2: Balance,
        ) -> Result<Balance, Error> {
            self.activePool()?;
            Ok(self.totalToken1 * _amountToken2 / self.totalToken2)
        }

        /// Returns amount of Token2 required when providing liquidity with _amountToken1 quantity of Token1
        #[ink(message)]
        pub fn getEquivalentToken2Estimate(
            &self,
            _amountToken1: Balance,
        ) -> Result<Balance, Error> {
            self.activePool()?;
            Ok(self.totalToken2 * _amountToken1 / self.totalToken1)
        }

        // Part 8. Withdraw
        /// Returns the estimate of Token1 & Token2 that will be released on burning given _share
        #[ink(message)]
        pub fn getWithdrawEstimate(&self, _share: Balance) -> Result<(Balance, Balance), Error> {
            self.activePool()?;
            if _share > self.totalShares {
                return Err(Error::InvalidShare);
            }

            let amountToken1 = _share * self.totalToken1 / self.totalShares;
            let amountToken2 = _share * self.totalToken2 / self.totalShares;
            Ok((amountToken1, amountToken2))
        }

        /// Removes liquidity from the pool and releases corresponding Token1 & Token2 to the withdrawer
        #[ink(message)]
        pub fn withdraw(&mut self, _share: Balance) -> Result<(Balance, Balance), Error> {
            let caller = self.env().caller();
            self.validAmountCheck(&self.shares, _share)?;

            let (amountToken1, amountToken2) = self.getWithdrawEstimate(_share)?;
            self.shares.entry(caller).and_modify(|val| *val -= _share);
            self.totalShares -= _share;

            self.totalToken1 -= amountToken1;
            self.totalToken2 -= amountToken2;

            self.token1Balance
                .entry(caller)
                .and_modify(|val| *val += amountToken1);
            self.token2Balance
                .entry(caller)
                .and_modify(|val| *val += amountToken2);

            Ok((amountToken1, amountToken2))
        }

        // Part 9. Swap
        /// Returns the amount of Token2 that the user will get when swapping a given amount of Token1 for Token2
        #[ink(message)]
        pub fn getSwapToken1EstimateGivenToken1(
            &self,
            _amountToken1: Balance,
        ) -> Result<Balance, Error> {
            self.activePool()?;
            let _amountToken1 = (1000 - self.fees) * _amountToken1 / 1000; // Adjusting the fees charged

            let token1After = self.totalToken1 + _amountToken1;
            let token2After = self.getK() / token1After;
            let mut amountToken2 = self.totalToken2 - token2After;

            // To ensure that Token2's pool is not completely depleted leading to inf:0 ratio
            if amountToken2 == self.totalToken2 {
                amountToken2 -= 1;
            }
            Ok(amountToken2)
        }

        /// Returns the amount of Token1 that the user should swap to get _amountToken2 in return
        #[ink(message)]
        pub fn getSwapToken1EstimateGivenToken2(
            &self,
            _amountToken2: Balance,
        ) -> Result<Balance, Error> {
            self.activePool()?;
            if _amountToken2 >= self.totalToken2 {
                return Err(Error::InsufficientLiquidity);
            }

            let token2After = self.totalToken2 - _amountToken2;
            let token1After = self.getK() / token2After;
            let amountToken1 = (token1After - self.totalToken1) * 1000 / (1000 - self.fees);
            Ok(amountToken1)
        }

        /// Swaps given amount of Token1 to Token2 using algorithmic price determination
        /// Swap fails if Token2 amount is less than _minToken2
        #[ink(message)]
        pub fn swapToken1GivenToken1(
            &mut self,
            _amountToken1: Balance,
            _minToken2: Balance,
        ) -> Result<Balance, Error> {
            let caller = self.env().caller();
            self.validAmountCheck(&self.token1Balance, _amountToken1)?;

            let amountToken2 = self.getSwapToken1EstimateGivenToken1(_amountToken1)?;
            if amountToken2 < _minToken2 {
                return Err(Error::SlippageExceeded);
            }
            self.token1Balance
                .entry(caller)
                .and_modify(|val| *val -= _amountToken1);

            self.totalToken1 += _amountToken1;
            self.totalToken2 -= amountToken2;

            self.token2Balance
                .entry(caller)
                .and_modify(|val| *val += amountToken2);
            Ok(amountToken2)
        }

        /// Swaps given amount of Token1 to Token2 using algorithmic price determination
        /// Swap fails if amount of Token1 required to obtain _amountToken2 exceeds _maxToken1
        #[ink(message)]
        pub fn swapToken1GivenToken2(
            &mut self,
            _amountToken2: Balance,
            _maxToken1: Balance,
        ) -> Result<Balance, Error> {
            let caller = self.env().caller();
            let amountToken1 = self.getSwapToken1EstimateGivenToken2(_amountToken2)?;
            if amountToken1 > _maxToken1 {
                return Err(Error::SlippageExceeded);
            }
            self.validAmountCheck(&self.token1Balance, amountToken1)?;

            self.token1Balance
                .entry(caller)
                .and_modify(|val| *val -= amountToken1);

            self.totalToken1 += amountToken1;
            self.totalToken2 -= _amountToken2;

            self.token2Balance
                .entry(caller)
                .and_modify(|val| *val += _amountToken2);
            Ok(amountToken1)
        }
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let amm = Amm::default();
            assert_eq!(amm.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut amm = Amm::new(false);
            assert_eq!(amm.get(), false);
            amm.flip();
            assert_eq!(amm.get(), true);
        }
    }
}
