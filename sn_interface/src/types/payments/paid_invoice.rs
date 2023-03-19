// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    errors::{Error, Result},
    invoice::Invoice,
    payment::Payment,
};

// A payment to an invoice.
pub struct PaidInvoice {
    pub invoice: Invoice,
    pub payment: Payment,
}

impl PaidInvoice {
    pub fn verify(&self) -> Result<()> {
        self.invoice.verify()?;

        let seller_public_key = self.invoice.content.seller_public_key;
        let payment_sum = self.payment.commitment_sum_by_owner(&seller_public_key)?;
        let invoice_amount_commitment = self.invoice.content.amount_commitment;

        if payment_sum == invoice_amount_commitment {
            Ok(())
        } else {
            Err(Error::PaymentDoesNotMatchInvoiceAmount)
        }
    }
}
