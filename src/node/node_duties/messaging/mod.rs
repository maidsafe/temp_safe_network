
pub mod client;
pub mod sender;
pub mod receiver;

use crate::{cmd::MessagingDuty, utils};
pub use client::{ClientInfo, ClientMessaging, ClientMsg};
use log::{error, info};
use routing::{DstLocation, Node as Routing, SrcLocation};
use safe_nd::{Address, MsgEnvelope, XorName};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
