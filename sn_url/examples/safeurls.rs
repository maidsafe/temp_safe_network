// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use rand::rngs::OsRng;
use sn_data_types::Keypair;
use sn_url::{Error, SafeDataType, SafeUrl, XorUrlBase};
use xor_name::XorName;

fn main() -> Result<(), Error> {
    // Let's generate a ranadom key pair
    let mut rng = OsRng;
    let keypair = Keypair::new_ed25519(&mut rng);

    // We get the corresponding Xorname for
    // the random public key we obtained
    let xorname = XorName::from(keypair.public_key());

    // We can encode a SafeKey XOR-URL using the Xorname
    // and specifying Base32z as the base encoding for it
    let xorurl = SafeUrl::encode_safekey(xorname, XorUrlBase::Base32z)?;

    println!("XorUrl: {}", xorurl);

    // We can parse a Safe-URL and obtain a SafeUrl instance
    let safe_url = SafeUrl::from_url(&xorurl)?;

    assert_eq!(safe_url.data_type(), SafeDataType::SafeKey);
    println!("Data type: {}", safe_url.data_type());

    assert_eq!(safe_url.xorname(), xorname);
    println!("Xorname: {}", safe_url.xorname());

    Ok(())
}
