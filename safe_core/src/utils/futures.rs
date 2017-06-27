// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use futures::{Async, Future, Poll};

/// This is the equivalent `try!` adapted to deal with futures. It is to be
/// read as `future-try`.  This will convert errors from `Result` into a `done`
/// future with corresponding error and return.
#[macro_export]
macro_rules! fry {
    ($res:expr) => {
        match $res {
            Ok(elt) => elt,
            Err(e) => {
                use $crate::utils::FutureExt;
                return ::futures::future::err(From::from(e)).into_box()
            }
        }
    }
}

/// This is the equivalent of `Result::Ok()` adapted to deal with futures. This
/// should be used to construct the return type equivalent of `Result::Ok` in
/// futures paradigm.
#[macro_export]
macro_rules! ok {
    ($elt:expr) => {{
        use $crate::utils::FutureExt;
        ::futures::future::ok($elt).into_box()
    }}
}

/// This is the equivalent of `Result::Ok()` adapted to deal with futures. This
/// should be used to construct the return type equivalent of `Result::Err` in
/// futures paradigm.
#[macro_export]
macro_rules! err {
    ($elt:expr) => {{
        use $crate::utils::FutureExt;
        ::futures::future::err(From::from($elt)).into_box()
    }}
}

/// Additional future combinators.
pub trait FutureExt: Future + Sized {
    /// Box this future. Similar to `boxed` combinator, but does not require
    /// the future to implement `Send`.
    fn into_box(self) -> Box<Future<Item = Self::Item, Error = Self::Error>>;
}

impl<F: Future + 'static> FutureExt for F {
    // TODO: when trait/impl specialization lands, try to implement this so that
    // it's a no-op when called on already boxed futures.
    fn into_box(self) -> Box<Future<Item = Self::Item, Error = Self::Error>> {
        Box::new(self)
    }
}

/// Repeatedly call `init` until `cond` returns false on the result of the returned future.
pub fn repeat_while<I, C, F>(mut init: I, cond: C) -> RepeatWhile<I, C, F>
    where I: FnMut() -> F,
          C: Fn(&Result<F::Item, F::Error>) -> bool,
          F: Future
{
    let future = init();
    RepeatWhile {
        init: init,
        cond: cond,
        future: future,
    }
}

/// The future returned from `repeat_while`.
pub struct RepeatWhile<I, C, F> {
    init: I,
    cond: C,
    future: F,
}

impl<I, C, F> Future for RepeatWhile<I, C, F>
    where I: FnMut() -> F,
          C: Fn(&Result<F::Item, F::Error>) -> bool,
          F: Future
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = match self.future.poll() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(item)) => Ok(item),
            Err(error) => Err(error),
        };

        if (self.cond)(&result) {
            self.future = (self.init)();
            Ok(Async::NotReady)
        } else {
            result.map(Async::Ready)
        }
    }
}
