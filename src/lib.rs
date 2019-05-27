use threshold_crypto::{PublicKey, SecretKey};

pub mod scl_mock;
use scl_mock::{MockSCL, XorName};

pub struct BlsKeyPair {
    pub pk: PublicKey,
    pub sk: SecretKey,
}

// Create a KeY on the network and return its XOR name
pub fn keys_create(safe_app: &mut MockSCL) -> (XorName, BlsKeyPair) {
    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();

    let xorname = safe_app.create_balance(&pk_from, &sk_from, &pk_to, "0");

    (
        xorname,
        BlsKeyPair {
            pk: pk_to,
            sk: sk_to,
        },
    )
}

#[test]
fn test_keys_create() {
    let mut safe_app = MockSCL::new();
    let (xorname, _) = keys_create(&mut safe_app);
    println!("New Key at: {:?}", xorname);
}
