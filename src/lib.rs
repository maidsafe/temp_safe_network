use threshold_crypto::SecretKey;
use uuid::Uuid;

mod scl_mock;

use self::scl_mock::{Coin, MockSCL, XorName};

// Create a Ket on the network and return its XOR name
pub fn create_key() -> XorName {
    let mut mock = MockSCL::new();
    let my_uuid = Uuid::new_v4();
    println!("UUID {}", my_uuid);

    let sk_from = SecretKey::random();
    let pk_from = sk_from.public_key();

    let sk_to = SecretKey::random();
    let pk_to = sk_to.public_key();
    //let sig = sk_from.sign();

    let key_xor_name = mock.create_balance(
        pk_from,
        pk_to,
        Coin {
            units: 1,
            parts: 30,
        },
    );

    key_xor_name
}

#[test]
fn test_create_key() {
    println!("New Key at: {:?}", create_key());
}
