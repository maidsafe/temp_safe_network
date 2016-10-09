// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

/// This is the equivalent `try!` adapted to deal with futures. It is to be read as `future-try`.
/// This will convert errors from `Result` into a `done` future with corresponding error and return.
macro_rules! fry {
    ($res:expr) => {
        match $res {
            Ok(elt) => elt,
            Err(e) => return futures::done(Err(From::from(e))),
        }
    }
}

/// This is the equivalent of `Result::Ok()` adapted to deal with futures. This should be used to
/// construct the return type equivalent of `Result` in futures paradigm.
macro_rules! fok {
    ($elt:expr) => {
        futures::done(Ok($elt))
    }
}
