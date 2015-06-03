// Copyright 2015 MaidSafe.net limited.
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

pub struct ResponseGetter {
    message_id:         ::routing::types::MessageId,
    response_notifier:  ::ResponseNotifier,
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<::client::callback_interface::CallbackInterface>>,
}

impl ResponseGetter {
    pub fn new(msg_id: ::routing::types::MessageId,
               notifier: ::ResponseNotifier,
               cb_interface: ::std::sync::Arc<::std::sync::Mutex<::client::callback_interface::CallbackInterface>>) -> ResponseGetter {
        ResponseGetter {
            message_id: msg_id,
            response_notifier: notifier,
            callback_interface: cb_interface,
        }
    }

    pub fn get(&mut self) -> Result<Vec<u8>, ::routing::error::ResponseError> {
        let &(ref lock, ref condition_var) = &*self.response_notifier;
        let mut mutex_guard: _;

        {
            let mut cb_interface = self.callback_interface.lock().unwrap();
            match cb_interface.get_response(self.message_id) {
                Some(response_result) => return response_result,
                None                  => mutex_guard = lock.lock().unwrap(),
            }
        }

        while *mutex_guard != self.message_id {
            mutex_guard = condition_var.wait(mutex_guard).unwrap();
        }

        let mut cb_interface = self.callback_interface.lock().unwrap();
        cb_interface.get_response(self.message_id).unwrap()
    }
}
