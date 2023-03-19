// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Flow:
///  1. Buyer informs seller s/he wishes to send payment, and attaches a public key.
///  2. Seller creates an invoice and returns to buyer.
///     The invoice consists of the `invoice_content` field, and `seller_signature` - the seller signature over it.
///     An `InvoiceContent` has the following fields:
///       a. `amount_commitment`: an amount commitment.
///       b. `amount_secrets_cipher`: `AmountSecret` (amount, blindfactor) ciphertext encrypted to buyer's pubkey
///       c. `seller_public_key`:  owner's well-known pubkey (should be one-time-use)
///  3. Buyer verifies seller's signature over `invoice_content`.
///  4. Buyer decrypts the `AmountSecret` to obtain the invoice amount.
///  5. Buyer reissues necessary DBC(s) using `seller_public_key` as the recipient's well-known key
///     in the exact amount of the invoice.
///  6. Buyer constructs a `Payment` which consists of one or more DBCs paying to seller.
///  7. Buyer constructs a `PaidInvoice` which consists of the `Invoice` and `Payment`.
///  8. Buyer sends the `PaidInvoice` to the seller. Transaction is complete.
///  9. Seller verifies that:
///       a. the invoice is valid, with seller's own signature
///       b. the sum of payment commitments is equal to the invoice amount commitment.
///  10. Seller is satisfied, delivers goods to buyer.
///
///  11. If buyer ever needs to prove payment, buyer can show any third party the `PaidInvoice`
///      and the 3rd party can verify that payment amount matches invoice amount, but
///      cannot actually see the invoice or payment amount (without obtaining buyer's secret key).
///
///      Note that payment is not actually proven unless/until buyer can prove that
///      seller has access to the `PaidInvoice`. This can be done by publishing it for
///      all to see.
mod errors;
mod invoice;
mod invoice_content;
mod paid_invoice;
mod payment;

pub use self::{
    errors::{Error, Result},
    invoice::Invoice,
    invoice_content::InvoiceContent,
    paid_invoice::PaidInvoice,
    payment::Payment,
};

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use sn_dbc::{AmountSecrets, Token};

    #[tokio::test]
    async fn invoice_can_be_read_buyer() -> Result<()> {
        let buyer_secret = bls::SecretKey::random();
        let seller_secret = bls::SecretKey::random();

        let amount = Token::from_nano(1234);
        let invoice = Invoice::new(amount, &buyer_secret.public_key(), &seller_secret);

        // verify invoice is built correctly
        let invoice_verification = invoice.verify();
        assert_matches!(invoice_verification, Ok(()));

        // verify buyer can read the amount
        let decryption_result =
            AmountSecrets::try_from((&buyer_secret, &invoice.content.amount_secrets_cipher));
        assert_matches!(decryption_result, Ok(amount_secret) => {
            assert!(invoice.content.matches_commitment(&amount_secret));
        });

        Ok(())
    }
}
