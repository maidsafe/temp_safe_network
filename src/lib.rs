pub mod scl_mock;
use scl_mock::{MockSCL, XorName};

use threshold_crypto::serde_impl::SerdeSecret;
use threshold_crypto::{PublicKey, SecretKey, PK_SIZE};
use unwrap::unwrap;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting others like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

// Create a Key on the network and return its XOR name
pub fn keys_create(
    safe_app: &mut MockSCL,
    from: Option<BlsKeyPair>,
    preload_amount: Option<String>,
    pk: Option<String>,
) -> (XorName, Option<BlsKeyPair>) {
    let from_key_pair: KeyPair = match from {
        Some(key_pair) => KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk),
        None => panic!("Missing coins' key pair to cover the costs of the operation"), // TODO: fetch default wallet from account if not provided
    };

    let create_key = |pk| match preload_amount {
        Some(amount) => safe_app.create_balance(&from_key_pair.pk, &from_key_pair.sk, &pk, &amount),
        None => safe_app.create_balance(&from_key_pair.pk, &from_key_pair.sk, &pk, "0"),
    };

    if let Some(pk_str) = pk {
        let pk = pk_from_hex(&pk_str);
        (create_key(pk), None)
    } else {
        let key_pair = KeyPair::random();
        let bls_key_pair = key_pair.to_hex_key_pair();
        (create_key(key_pair.pk), Some(bls_key_pair))
    }
}

// Create a Key on the network, allocates testcoins onto it, and return the Key's XOR name
// This is avilable only when testing with mock-network
// #[cfg(feature = "mock-network")]
pub fn keys_create_test_coins(
    safe_app: &mut MockSCL,
    preload_amount: String,
    pk: Option<String>,
) -> (XorName, Option<BlsKeyPair>) {
    if let Some(pk_str) = pk {
        let pk = pk_from_hex(&pk_str);
        let xorname = safe_app.allocate_test_coins(&pk, &preload_amount);
        (xorname, None)
    } else {
        let key_pair = KeyPair::random();
        let xorname = safe_app.allocate_test_coins(&key_pair.pk, &preload_amount);
        (xorname, Some(key_pair.to_hex_key_pair()))
    }
}

// Check Key's balance from the network from a given PublicKey
pub fn keys_balance_from_pk(safe_app: &MockSCL, key_pair: &BlsKeyPair) -> String {
    let pair = KeyPair::from_hex_keys(&key_pair.pk, &key_pair.sk);
    safe_app.get_balance_from_pk(&pair.pk, &pair.sk)
}

// Check Key's balance from the network from a given XOR name
pub fn keys_balance_from_xorname(safe_app: &MockSCL, xorname: &XorName, sk: &str) -> String {
    let secret_key: SecretKey = sk_from_hex(sk);
    safe_app.get_balance_from_xorname(xorname, &secret_key)
}

// Fetch Key's pk from the network from a given XOR name
pub fn fetch_key_pk(safe_app: &MockSCL, xorname: &XorName, sk: &str) -> String {
    let secret_key: SecretKey = sk_from_hex(sk);
    let public_key = safe_app.fetch_key_pk(xorname, &secret_key);
    pk_to_hex(&public_key)
}

// Private helper functions

// Out internal key pair structure to manage BLS keys
struct KeyPair {
    pub pk: PublicKey,
    pub sk: SecretKey,
}

impl KeyPair {
    fn random() -> Self {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        KeyPair { pk, sk }
    }

    fn from_hex_keys(pk_hex_str: &str, sk_hex_str: &str) -> Self {
        let pk = pk_from_hex(pk_hex_str);
        let sk = sk_from_hex(sk_hex_str);
        KeyPair { pk, sk }
    }

    fn to_hex_key_pair(&self) -> BlsKeyPair {
        let pk: String = pk_to_hex(&self.pk);

        let sk_serialised = bincode::serialize(&SerdeSecret(&self.sk))
            .expect("Failed to serialise the generated secret key");
        let sk: String = sk_serialised.iter().map(|b| format!("{:02x}", b)).collect();

        BlsKeyPair { pk, sk }
    }
}

fn parse_hex(hex_str: &str) -> Vec<u8> {
    let mut hex_bytes = hex_str
        .as_bytes()
        .iter()
        .filter_map(|b| match b {
            b'0'...b'9' => Some(b - b'0'),
            b'a'...b'f' => Some(b - b'a' + 10),
            b'A'...b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .fuse();

    let mut bytes = Vec::new();
    while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
        bytes.push(h << 4 | l)
    }
    bytes
}

fn pk_to_hex(pk: &PublicKey) -> String {
    let pk_as_bytes: [u8; PK_SIZE] = pk.to_bytes();
    pk_as_bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn pk_from_hex(hex_str: &str) -> PublicKey {
    let pk_bytes = parse_hex(&hex_str);
    let mut pk_bytes_array: [u8; PK_SIZE] = [0; PK_SIZE];
    pk_bytes_array.copy_from_slice(&pk_bytes[..PK_SIZE]);
    unwrap!(PublicKey::from_bytes(pk_bytes_array))
}

fn sk_from_hex(hex_str: &str) -> SecretKey {
    let sk_bytes = parse_hex(&hex_str);
    bincode::deserialize(&sk_bytes).expect("Failed to deserialize provided secret key")
}

// Unit Tests

#[test]
fn test_keys_create_test_coins() {
    let mut safe_app = MockSCL::new();
    let (xorname, key_pair) = keys_create_test_coins(&mut safe_app, "12.23".to_string(), None);
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_test_coins_pk() {
    let mut safe_app = MockSCL::new();
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorname, key_pair) = keys_create_test_coins(&mut safe_app, "1.1".to_string(), Some(pk));
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_create() {
    let mut safe_app = MockSCL::new();
    let (_, from_key_pair) = keys_create_test_coins(&mut safe_app, "23.23".to_string(), None);

    let (xorname, key_pair) = keys_create(&mut safe_app, from_key_pair, None, None);
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe_app = MockSCL::new();
    let (_, from_key_pair) = keys_create_test_coins(&mut safe_app, "543.2312".to_string(), None);

    let preload_amount = "1.8";
    let (xorname, key_pair) = keys_create(
        &mut safe_app,
        from_key_pair,
        Some(preload_amount.to_string()),
        None,
    );
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(kp) => {
            let balance = keys_balance_from_pk(
                &safe_app,
                &BlsKeyPair {
                    pk: kp.pk,
                    sk: kp.sk,
                },
            );
            assert_eq!(balance, preload_amount);
        }
    };
}

#[test]
fn test_keys_create_pk() {
    let mut safe_app = MockSCL::new();
    let (_, from_key_pair) = keys_create_test_coins(&mut safe_app, "1.1".to_string(), None);
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorname, key_pair) = keys_create(&mut safe_app, from_key_pair, None, Some(pk));
    assert_eq!(xorname.len(), 64);
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}

#[test]
fn test_keys_test_coins_balance_pk() {
    let mut safe_app = MockSCL::new();
    let preload_amount = "1.1542";
    let (_, key_pair) = keys_create_test_coins(&mut safe_app, preload_amount.to_string(), None);
    let current_balance = keys_balance_from_pk(&safe_app, &unwrap!(key_pair));
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_test_coins_balance_xorname() {
    let mut safe_app = MockSCL::new();
    let preload_amount = "0.243";
    let (xorname, key_pair) =
        keys_create_test_coins(&mut safe_app, preload_amount.to_string(), None);
    let current_balance = keys_balance_from_xorname(&safe_app, &xorname, &unwrap!(key_pair).sk);
    assert_eq!(preload_amount, current_balance);
}

#[test]
fn test_keys_balance_pk() {
    let mut safe_app = MockSCL::new();
    let preload_amount = "1743.234";
    let (_, from_key_pair) =
        keys_create_test_coins(&mut safe_app, preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "1740";
    let (_, to_key_pair) = keys_create(
        &mut safe_app,
        Some(from_key_pair_unwrapped.clone()),
        Some(amount.to_string()),
        None,
    );

    let from_current_balance = keys_balance_from_pk(&safe_app, &from_key_pair_unwrapped);
    assert_eq!("3.234" /*== 1743.234 - 1740*/, from_current_balance);

    let to_current_balance = keys_balance_from_pk(&safe_app, &unwrap!(to_key_pair));
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_keys_balance_xorname() {
    let mut safe_app = MockSCL::new();
    let preload_amount = "435.34";
    let (from_xorname, from_key_pair) =
        keys_create_test_coins(&mut safe_app, preload_amount.to_string(), None);
    let from_key_pair_unwrapped = unwrap!(from_key_pair);

    let amount = "35.3";
    let (to_xorname, to_key_pair) = keys_create(
        &mut safe_app,
        Some(from_key_pair_unwrapped.clone()),
        Some(amount.to_string()),
        None,
    );

    let from_current_balance =
        keys_balance_from_xorname(&safe_app, &from_xorname, &from_key_pair_unwrapped.sk);
    assert_eq!("400.04" /*== 435.34 - 35.3*/, from_current_balance);

    let to_current_balance =
        keys_balance_from_xorname(&safe_app, &to_xorname, &unwrap!(to_key_pair).sk);
    assert_eq!(amount, to_current_balance);
}

#[test]
fn test_fetch_key_pk_test_coins() {
    let mut safe_app = MockSCL::new();
    let (xorname, key_pair) = keys_create_test_coins(&mut safe_app, "23.22".to_string(), None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = fetch_key_pk(&safe_app, &xorname, &key_pair_unwrapped.sk);
    assert_eq!(pk, key_pair_unwrapped.pk);
}

#[test]
fn test_fetch_key_pk() {
    let mut safe_app = MockSCL::new();
    let (_, from_key_pair) = keys_create_test_coins(&mut safe_app, "0.56".to_string(), None);

    let (xorname, key_pair) = keys_create(&mut safe_app, from_key_pair, None, None);
    let key_pair_unwrapped = unwrap!(key_pair);
    let pk = fetch_key_pk(&safe_app, &xorname, &key_pair_unwrapped.sk);
    assert_eq!(pk, key_pair_unwrapped.pk);
}
