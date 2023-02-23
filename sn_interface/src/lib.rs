// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    unsafe_code
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    clippy::unicode_not_nfc,
    clippy::unwrap_used,
    clippy::unused_async
)]

//! SAFE network data types.

// Dbcs on the safe network.
pub mod dbcs;
// Standardised messaging interface
pub mod messaging;
// Knowledge of the safe network
pub mod network_knowledge;
// Types on the safe network
pub mod types;
// Statemap states
pub mod statemap;

#[macro_use]
extern crate tracing;

pub use network_knowledge::{elder_count, SectionAuthorityProvider};

/// Number of copies of a chunk
const DEFAULT_DATA_COPY_COUNT: usize = 4;

// const SN_ELDER_COUNT: &str = "SN_ELDER_COUNT";
const SN_DATA_COPY_COUNT: &str = "SN_DATA_COPY_COUNT";

/// Max number of faulty Elders is assumed to be less than 1/3.
/// So it's no more than 2 with 7 Elders.
pub fn max_num_faulty_elders() -> usize {
    elder_count() / 3
}

/// Get the expected chunk copy count for our network.
/// Defaults to `DEFAULT_DATA_COPY_COUNT`, but can be overridden by the env var `SN_DATA_COPY_COUNT`.
pub fn data_copy_count() -> usize {
    // if we have an env var for this, lets override
    match std::env::var(SN_DATA_COPY_COUNT) {
        Ok(count) => match count.parse() {
            Ok(count) => {
                warn!(
                    "data_copy_count countout set from env var SN_DATA_COPY_COUNT: {:?}",
                    SN_DATA_COPY_COUNT
                );
                count
            }
            Err(error) => {
                warn!("There was an error parsing {:?} env var. DEFAULT_DATA_COPY_COUNT will be used: {:?}", SN_DATA_COPY_COUNT, error);
                DEFAULT_DATA_COPY_COUNT
            }
        },
        Err(_) => DEFAULT_DATA_COPY_COUNT,
    }
}

use tracing_core::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, SystemTime},
        FmtContext, FormatEvent, FormatFields,
    },
    registry::LookupSpan,
};

#[derive(Default, Debug)]
/// Tracing log formatter setup for easier span viewing
pub struct LogFormatter;

impl<S, N> FormatEvent<S, N> for LogFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Write level and target
        let level = *event.metadata().level();
        let module = event.metadata().module_path().unwrap_or("<unknown module>");
        let time = SystemTime::default();

        write!(writer, "[")?;
        time.format_time(&mut writer)?;
        write!(writer, " {level} {module}")?;
        ctx.visit_spans(|span| write!(writer, "/{}", span.name()))?;
        write!(writer, "] ")?;

        // Add the log message and any fields associated with the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
#[cfg(feature = "test-utils")]
use std::sync::Once;
#[cfg(feature = "test-utils")]
static INIT: Once = Once::new();

/// Initialise logger for tests, this is run only once, even if called multiple times.
#[cfg(feature = "test-utils")]
pub fn init_logger() {
    INIT.call_once(|| {
        tracing_subscriber::fmt::fmt()
            // NOTE: uncomment this line for pretty printed log output.
            //.pretty()
            .with_ansi(false)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .event_format(LogFormatter::default())
            .try_init()
            .unwrap_or_else(|_| println!("Error initializing logger"));
    });
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    pub use crate::{
        network_knowledge::{
            section_authority_provider::test_utils::*, test_utils::*, test_utils_st::*,
        },
        types::keys::test_utils::*,
    };
}
