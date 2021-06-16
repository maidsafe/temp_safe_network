// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};
use crate::types::{Credit, CreditId, Debit, OwnerType, Token};
use log::debug;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct WalletSnapshot {
    pub balance: Token,
    pub debit_version: u64,
    pub credit_ids: HashSet<CreditId>,
}

impl From<Wallet> for WalletSnapshot {
    fn from(other: Wallet) -> WalletSnapshot {
        WalletSnapshot {
            balance: other.balance,
            debit_version: other.debit_version,
            credit_ids: other.credit_ids,
        }
    }
}

/// The balance and history of transfers for a wallet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallet {
    id: OwnerType,
    balance: Token,
    debit_version: u64,
    credit_ids: HashSet<CreditId>,
}

impl Wallet {
    /// Creates a new wallet.
    pub fn new(id: OwnerType) -> Self {
        Self {
            id,
            balance: Token::zero(),
            debit_version: 0,
            credit_ids: Default::default(),
        }
    }

    /// Creates a wallet from existing state.
    pub fn from(
        id: OwnerType,
        balance: Token,
        debit_version: u64,
        credit_ids: HashSet<CreditId>,
    ) -> Self {
        Self {
            id,
            balance,
            debit_version,
            credit_ids,
        }
    }

    /// Get the id of the wallet.
    pub fn id(&self) -> &OwnerType {
        &self.id
    }

    /// Query for next version.
    pub fn next_debit(&self) -> u64 {
        self.debit_version
    }

    /// Query for balance.
    pub fn balance(&self) -> Token {
        self.balance
    }

    /// Query for already received credit.
    pub fn contains(&self, id: &CreditId) -> bool {
        self.credit_ids.contains(id)
    }

    /// Mutates state.
    pub fn apply_debit(&mut self, debit: Debit) -> Result<()> {
        debug!("Wallet applying debit");
        if self.id.public_key() == debit.id.actor {
            match self.balance.checked_sub(debit.amount) {
                Some(amount) => self.balance = amount,
                None => return Err(Error::SubtractionOverflow(debit.amount, self.balance)),
            }
            self.debit_version += 1;
            Ok(())
        } else {
            Err(Error::DebitDoesNotBelong(self.id().public_key(), debit))
        }
    }

    /// Mutates state.
    pub fn apply_credit(&mut self, credit: Credit) -> Result<()> {
        debug!("Wallet applying credit");
        if self.id.public_key() == credit.recipient() {
            match self.balance.checked_add(credit.amount) {
                Some(amount) => self.balance = amount,
                None => return Err(Error::AdditionOverflow(self.balance, credit.amount)),
            }
            let _ = self.credit_ids.insert(credit.id);
            Ok(())
        } else {
            Err(Error::CreditDoesNotBelong(self.id().public_key(), credit))
        }
    }

    /// Test-helper API to simulate Client Transfers.
    #[cfg(feature = "simulated-payouts")]
    pub fn simulated_credit(&mut self, credit: Credit) -> Result<()> {
        debug!("Wallet simulated credit");

        if self.id.public_key() == credit.recipient() {
            match self.balance.checked_add(credit.amount) {
                Some(amount) => self.balance = amount,
                None => return Err(Error::AdditionOverflow(self.balance, credit.amount)),
            }
        } else {
            return Err(Error::CreditDoesNotBelong(self.id().public_key(), credit));
        }
        Ok(())
    }

    /// Test-helper API to simulate section payments.
    #[cfg(feature = "simulated-payouts")]
    pub fn simulated_debit(&mut self, debit: Debit) -> Result<()> {
        debug!("Wallet simulated debit");

        if self.id.public_key() == debit.id.actor {
            match self.balance.checked_sub(debit.amount) {
                Some(amount) => self.balance = amount,
                None => return Err(Error::SubtractionOverflow(self.balance, debit.amount)),
            }
            self.debit_version += 1;
        } else {
            return Err(Error::DebitDoesNotBelong(self.id().public_key(), debit));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::PublicKey;
    use bls::SecretKey;
    use crdts::Dot;
    use xor_name::XorName;

    #[test]
    fn applies_credits() -> Result<()> {
        // Arrange
        let balance = Token::from_nano(10);
        let first_credit = Credit {
            id: Default::default(),
            recipient: get_random_pk(),
            amount: balance,
            msg: "asdf".to_string(),
        };
        let mut wallet = Wallet::new(OwnerType::Single(first_credit.recipient));
        wallet.apply_credit(first_credit.clone())?;
        let second_credit = Credit {
            id: Default::default(),
            recipient: first_credit.recipient,
            amount: balance,
            msg: "asdf".to_string(),
        };

        // Act
        wallet.apply_credit(second_credit)?;

        // Assert
        assert_eq!(Some(wallet.balance()), balance.checked_add(balance));
        assert_eq!(wallet.next_debit(), 0);
        Ok(())
    }

    #[test]
    fn applies_debits() -> Result<()> {
        // Arrange
        let balance = Token::from_nano(10);
        let first_credit = Credit {
            id: Default::default(),
            recipient: get_random_pk(),
            amount: balance,
            msg: "asdf".to_string(),
        };
        let mut wallet = Wallet::new(OwnerType::Single(first_credit.recipient));
        wallet.apply_credit(first_credit.clone())?;
        let first_debit = Debit {
            id: Dot::new(first_credit.recipient, 0),
            amount: balance,
        };

        // Act
        wallet.apply_debit(first_debit)?;

        // Assert
        assert_eq!(wallet.balance(), Token::zero());
        assert_eq!(wallet.next_debit(), 1);
        Ok(())
    }

    #[allow(unused)]
    fn get_random_xor() -> XorName {
        XorName::random()
    }

    fn get_random_pk() -> PublicKey {
        PublicKey::from(SecretKey::random().public_key())
    }
}
