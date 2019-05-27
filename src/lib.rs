use threshold_crypto::{PublicKey, SecretKey};

pub mod scl_mock;
use scl_mock::{MockSCL, XorName};

pub struct BlsKeyPair {
    pub pk: PublicKey,
    pub sk: SecretKey,
}

fn parse_hex(hex_asm: &str) -> Vec<u8> {
    let mut hex_bytes = hex_asm
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

// Create a Key on the network and return its XOR name
pub fn keys_create(
    safe_app: &mut MockSCL,
    preload_amount: Option<String>,
    pk: Option<String>,
) -> (XorName, Option<BlsKeyPair>) {
    // TODO: use sk/pk from the account or provided as args
    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();
    // end TODO

    let create_key = |pk| match preload_amount {
        Some(amount) => safe_app.create_balance(&pk_from, &sk_from, &pk, &amount),
        None => safe_app.create_balance(&pk_from, &sk_from, &pk, "0"),
    };

    if let Some(pk_str) = pk {
        let pk_bytes = parse_hex(&pk_str);
        let mut pk_bytes_array: [u8; 48] = [0; 48];
        pk_bytes_array.copy_from_slice(&pk_bytes[..48]);
        let pk = PublicKey::from_bytes(pk_bytes_array).unwrap();
        (create_key(pk), None)
    } else {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        (create_key(pk), Some(BlsKeyPair { pk, sk }))
    }
}

// Create a Key on the network and return its XOR name
pub fn keys_balance(safe_app: &MockSCL, pk: &PublicKey, sk: &SecretKey) -> String {
    safe_app.get_balance(pk, sk)
}

#[test]
fn test_keys_create() {
    let mut safe_app = MockSCL::new();
    let (xorname, key_pair) = keys_create(&mut safe_app, None, None);
    assert_eq!(xorname.len(), 32);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(_) => assert!(true),
    };
}

#[test]
fn test_keys_create_preload() {
    let mut safe_app = MockSCL::new();
    let preload_amount = "1.8";
    let (xorname, key_pair) = keys_create(&mut safe_app, Some(preload_amount.to_string()), None);
    assert_eq!(xorname.len(), 32);
    match key_pair {
        None => panic!("Key pair was not generated as it was expected"),
        Some(kp) => {
            let balance = keys_balance(&safe_app, &kp.pk, &kp.sk);
            assert_eq!(balance, preload_amount);
        }
    };
}

#[test]
fn test_keys_create_pk() {
    let mut safe_app = MockSCL::new();
    let pk = String::from("a252e6741b524ad70cf340f32d219c60a3f1a38aaec0d0dbfd24ea9ae7390e44ebdc93e7575711e65379eb0f4de083a8");
    let (xorname, key_pair) = keys_create(&mut safe_app, None, Some(pk));
    assert_eq!(xorname.len(), 32);
    match key_pair {
        None => assert!(true),
        Some(kp) => panic!("Unexpected key pair generated: {:?} {:?}", kp.pk, kp.sk),
    };
}
