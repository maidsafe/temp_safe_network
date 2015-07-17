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

/// ResponseGetter is a lazy evaluated response getter. It will fetch either from local cache or
/// wait for the CallbackInterface to notify it of the incoming response from the network.
pub struct ResponseGetter {
    response_notifier:  Option<::client::misc::ResponseNotifier>,
    callback_interface: ::std::sync::Arc<::std::sync::Mutex<::client::callback_interface::CallbackInterface>>,
    requested_location: ::routing::NameType,
    requested_type:     ::client::DataRequest,
}

impl ResponseGetter {
    /// Create a new instance of ResponseGetter
    pub fn new(notifier          : Option<::client::misc::ResponseNotifier>,
               callback_interface: ::std::sync::Arc<::std::sync::Mutex<::client::callback_interface::CallbackInterface>>,
               requested_location: ::routing::NameType,
               requested_type    : ::client::DataRequest) -> ResponseGetter {
        ResponseGetter {
            response_notifier : notifier,
            callback_interface: callback_interface,
            requested_location: requested_location,
            requested_type    : requested_type,
        }
    }

    /// Get either from local cache or (if not available there) get it when it comes from the
    /// network as informed by CallbackInterface. This is blocking.
    pub fn get(&mut self) -> Result<::client::Data, ::errors::ClientError> {
        if let Some(ref notifier) = self.response_notifier {
            let (ref lock, ref condition_var) = **notifier;
            let mut mutex_guard: _;

            {
                let mut cb_interface = self.callback_interface.lock().unwrap();
                match cb_interface.get_response(&self.requested_location) {
                    Ok(response) => return Ok(response),
                    Err(_) => {
                        mutex_guard = lock.lock().unwrap();
                        if *mutex_guard == Some(self.requested_location.clone()) {
                            *mutex_guard = None;
                        }
                    },
                }
            }

            let valid_condition = Some(self.requested_location.clone());
            while *mutex_guard != valid_condition {
                mutex_guard = condition_var.wait(mutex_guard).unwrap();
            }

            let mut cb_interface = self.callback_interface.lock().unwrap();
            let response = try!(cb_interface.get_response(&self.requested_location));

            if let ::client::DataRequest::ImmutableData(_) = self.requested_type {
                cb_interface.local_cache_insert(self.requested_location.clone(), response.clone());
            }

            Ok(response)
        } else {
            let mut cb_interface = self.callback_interface.lock().unwrap();
            Ok(try!(cb_interface.local_cache_get(&self.requested_location)))
        }
    }
}
