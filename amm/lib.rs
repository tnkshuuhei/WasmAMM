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

    impl Amm {
        // Part 4. Constructor

        // Part 5. Faucet

        // Part 6. Read current state

        // Part 7. Provide

        // Part 8. Withdraw

        // Part 9. Swap

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
