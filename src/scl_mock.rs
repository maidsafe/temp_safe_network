use rand;
use std::collections::HashMap;
use threshold_crypto::PublicKey;
use uuid::Uuid;

#[derive(Debug)]
pub struct Coin {
    pub units: u32,
    pub parts: u32,
}

#[allow(dead_code)]
struct Credit {
    amount: Coin,
    transaction_id: Uuid,
}

#[allow(dead_code)]
struct CoinTransfer {
    destination: PublicKey,
    credit: Credit,
}

#[derive(Debug)]
#[allow(dead_code)]
struct CoinBalance {
    owner: PublicKey,
    value: Coin,
}

pub type XorName = [u8; 32];

pub struct MockSCL {
    coin_balances: HashMap<XorName, CoinBalance>,
}

impl MockSCL {
    pub fn new() -> MockSCL {
        MockSCL {
            coin_balances: HashMap::new(),
        }
    }

    pub fn create_balance(
        &mut self,
        _from: PublicKey,
        new_balance_owner: PublicKey,
        amount: Coin, /*, signature: Signature*/
    ) -> XorName {
        let xorname: XorName = rand::random();
        self.coin_balances.insert(
            xorname,
            CoinBalance {
                owner: new_balance_owner,
                value: amount,
            },
        );

        xorname
    }

    #[allow(dead_code)]
    pub fn allocate_test_coins() {
        // TODO
    }

    #[allow(dead_code)]
    pub fn get_transaction() {
        // TODO
    }

    #[allow(dead_code)]
    pub fn send() {
        // TODO
    }
}

#[test]
fn create_balance() {
    use self::{Coin, MockSCL};
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    //let sig = sk_from.sign();
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(
            pk_from,
            pk_to,
            Coin {
                units: 1,
                parts: 30
            }
        )
    );
}
