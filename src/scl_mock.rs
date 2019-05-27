use safecoin::{Coins, NanoCoins};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use threshold_crypto::{PublicKey, SecretKey};
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
struct CoinBalance {
    owner: PublicKey,
    value: Coins,
}

pub type XorName = [u8; 32];

#[derive(Default)]
pub struct MockSCL {
    coin_balances: HashMap<XorName, CoinBalance>,
    txs: HashMap<XorName, HashMap<Uuid, String>>, // keep track of TX status per tx ID, per xorname
}

fn xorname_from_pk(pk: &PublicKey) -> XorName {
    let pk_as_bytes: [u8; 48] = pk.to_bytes();
    let mut xorname: XorName = [0; 32];
    xorname.copy_from_slice(&pk_as_bytes[..32]);

    xorname
}

impl MockSCL {
    pub fn new() -> MockSCL {
        MockSCL {
            coin_balances: HashMap::new(),
            txs: HashMap::new(),
        }
    }

    pub fn create_balance(
        &mut self,
        _from_pk: &PublicKey,
        _from_sk: &SecretKey,
        new_balance_owner: &PublicKey,
        amount: &str,
    ) -> XorName {
        let xorname = xorname_from_pk(new_balance_owner);
        let coin = Coins::from_str(amount).unwrap();
        self.coin_balances.insert(
            xorname,
            CoinBalance {
                owner: (*new_balance_owner),
                value: coin,
            },
        );

        xorname
    }

    pub fn get_balance(&self, pk: &PublicKey, _sk: &SecretKey) -> String {
        let xorname = xorname_from_pk(pk);
        let coin_balance = &self.coin_balances[&xorname];
        coin_balance
            .value
            .to_string()
            .replace("Coins(", "")
            .replace(")", "")
    }

    #[allow(dead_code)]
    pub fn allocate_test_coins(&mut self, to_pk: &PublicKey, amount: &str) -> XorName {
        let xorname = xorname_from_pk(to_pk);
        let coin = Coins::from_str(&amount).unwrap();
        self.coin_balances.insert(
            xorname,
            CoinBalance {
                owner: (*to_pk),
                value: coin,
            },
        );

        xorname
    }

    #[allow(dead_code)]
    pub fn send(
        &mut self,
        from_pk: &PublicKey,
        from_sk: &SecretKey,
        to_pk: &PublicKey,
        tx_id: &Uuid,
        amount: &str,
    ) {
        let to_xorname = xorname_from_pk(to_pk);
        let from_xorname = xorname_from_pk(from_pk);

        // generate TX in destination section (to_pk)
        let mut txs_for_xorname = match self.txs.get(&to_xorname) {
            Some(txs) => txs.clone(),
            None => HashMap::new(),
        };
        txs_for_xorname.insert(tx_id.clone(), format!("Success({})", amount).to_string());
        self.txs.insert(to_xorname, txs_for_xorname);

        let amount_coin = Coins::from_str(amount).unwrap();

        // reduce balance from sender
        let from_balance = Coins::from_str(&self.get_balance(from_pk, from_sk)).unwrap();
        let from_nano_balance = NanoCoins::try_from(from_balance).unwrap();
        let amount_nano = NanoCoins::try_from(amount_coin).unwrap();
        let from_new_amount = NanoCoins::new(from_nano_balance.num() - amount_nano.num()).unwrap(); // TODO: check it has enough balance
        self.coin_balances.insert(
            from_xorname,
            CoinBalance {
                owner: (*from_pk),
                value: Coins::from(from_new_amount),
            },
        );

        // credit destination
        let to_balance = Coins::from_str(
            &self.get_balance(to_pk, from_sk /*incorrect but doesn't matter for now*/),
        )
        .unwrap();
        let to_nano_balance = NanoCoins::try_from(to_balance).unwrap();
        let to_new_amount = NanoCoins::new(to_nano_balance.num() + amount_nano.num()).unwrap();
        self.coin_balances.insert(
            to_xorname,
            CoinBalance {
                owner: (*to_pk),
                value: Coins::from(to_new_amount),
            },
        );
    }

    #[allow(dead_code)]
    pub fn get_transaction(&self, tx_id: &Uuid, pk: &PublicKey, _sk: &SecretKey) -> String {
        let xorname = xorname_from_pk(pk);
        let txs_for_xorname = &self.txs[&xorname];
        let tx_state = txs_for_xorname.get(tx_id).unwrap();
        tx_state.to_string()
    }

    #[allow(dead_code)]
    pub fn unpublished_append_only_put(&self) {
        // TODO
    }
}

#[test]
fn test_create_balance() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(&pk_from, &sk_from, &pk_to, "1.234567891")
    );
}

#[test]
fn test_check_balance() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    let balance = "1.234567891";
    println!(
        "New CoinBalance at: {:?}",
        mock.create_balance(&pk_from, &sk_from, &pk_to, balance)
    );
    let current_balance = mock.get_balance(&pk_to, &sk_to);
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);
}

#[test]
fn test_allocate_test_coins() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();

    let balance = "2.345678912";
    mock.allocate_test_coins(&pk_to, balance);

    let current_balance = mock.get_balance(&pk_to, &sk_to);
    println!("Current balance: {}", current_balance);
    assert_eq!(balance, &current_balance);
}

#[test]
fn test_send() {
    use self::MockSCL;
    use threshold_crypto::SecretKey;

    let mut mock = MockSCL::new();

    let sk1 = SecretKey::random();
    let pk1 = sk1.public_key();

    let sk2 = SecretKey::random();
    let pk2 = sk2.public_key();

    let balance1 = "2.5";
    let balance2 = "5.7";
    println!(
        "Allocate testcoins in new CoinBalance 1 at: {:?}",
        mock.allocate_test_coins(&pk1, balance1)
    );

    println!(
        "Allocate testcoins in new CoinBalance 2 at: {:?}",
        mock.allocate_test_coins(&pk2, balance2)
    );

    let curr_balance1 = mock.get_balance(&pk1, &sk1);
    let curr_balance2 = mock.get_balance(&pk2, &sk2);
    println!(
        "Current balances before TX: {} and {}",
        curr_balance1, curr_balance2
    );

    assert_eq!(balance1, curr_balance1);
    assert_eq!(balance2, curr_balance2);

    let tx_id = Uuid::new_v4();
    println!("UUID {}", tx_id);

    mock.send(&pk1, &sk1, &pk2, &tx_id, "1.4");
    println!(
        "Current TX state: {}",
        mock.get_transaction(&tx_id, &pk2, &sk2)
    );

    let curr_balance1 = mock.get_balance(&pk1, &sk1);
    let curr_balance2 = mock.get_balance(&pk2, &sk2);
    println!(
        "Current balances after TX: {} and {}",
        curr_balance1, curr_balance2
    );

    assert_eq!(curr_balance1, "1.1");
    assert_eq!(curr_balance2, "7.1");
}
