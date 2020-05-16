use super::Identity;

use safe_nd::{Error, Money, Result, Transfer, TransferIndices};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct History {
    id: Identity,
    balance: Money,
    incoming: Vec<Transfer>,
    outgoing: Vec<Transfer>,
}

impl History {
    pub fn new(id: Identity, first: Transfer) -> Self {
        Self {
            id,
            balance: Money::zero(),
            incoming: vec![first],
            outgoing: Default::default(),
        }
    }

    pub fn balance(&self) -> Money {
        self.balance
    }

    // zero based indexing, first (outgoing) transfer will be nr 0
    // (we could just as well just compare outgoing.len()..)
    pub fn is_sequential(&self, transfer: &Transfer) -> Result<bool> {
        let id = transfer.id;
        return if id.actor != self.id {
            Err(Error::InvalidOperation)
        } else {
            match self.outgoing.last() {
                None => Ok(id.counter == 0), // if not outgoing transfers have been made, transfer counter must be 0
                Some(previous) => Ok(previous.id.counter + 1 == id.counter),
            }
        };
    }

    pub fn new_since(&self, indices: TransferIndices) -> (Vec<Transfer>, Vec<Transfer>) {
        let in_include_index = indices.incoming + 1;
        let out_include_index = indices.outgoing + 1;
        let new_incoming = if self.incoming.len() > in_include_index {
            self.incoming.split_at(in_include_index).1.to_vec()
        } else {
            vec![]
        };
        let new_outgoing = if self.incoming.len() > out_include_index {
            self.incoming.split_at(out_include_index).1.to_vec()
        } else {
            vec![]
        };
        (new_incoming, new_outgoing)
    }

    pub fn append(&mut self, transfer: Transfer) {
        if self.id == transfer.id.actor {
            match self.balance.checked_sub(transfer.amount) {
                Some(amount) => self.balance = amount,
                None => panic!(),
            }
            self.outgoing.push(transfer);
        } else if self.id == transfer.to {
            match self.balance.checked_add(transfer.amount) {
                Some(amount) => self.balance = amount,
                None => panic!(),
            }
            self.incoming.push(transfer);
        } else {
            panic!()
        }
    }
}
