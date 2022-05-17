// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! The Safe Network Client.
//!
//! In order to connect to The Safe Network you'll need to send messages back and forth to network nodes.
//! The [Client] has everything needed to perform this communication, with APIs to enable
//! working with data.
//!
//! With these APIs you can easily:
//! - Connect to The Safe Network
//! - Read Public data from the network
//! - Write data to the network (assuming you have a SafeCoin balance)
//!
//! ## Basic Usage
//!
//! Setting up a random client for read only access:
//!
//! # Examples
//!
//! TODO: update once data types are crdt compliant
//!
//!

// For quick_error
#![recursion_limit = "256"]
#![doc(
    html_logo_url = "https://github.com/maidsafe/QA/raw/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(deny(warnings)))
)]
// Forbid some very bad patterns. Forbid is stronger than `deny`, preventing us from suppressing the
// lint with `#[allow(...)]` et-all.
#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    unsafe_code
)]
// Turn on some additional warnings to encourage good style.
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    clippy::unicode_not_nfc
)]

#[macro_use]
extern crate tracing;

mod config_handler;
mod connections;
mod errors;

// Export public API.
pub use api::{Client, RegisterWriteAheadLog};
pub use config_handler::{ClientConfig, DEFAULT_ACK_WAIT, DEFAULT_OPERATION_TIMEOUT};
pub use errors::ErrorMsg;
pub use errors::{Error, Result};
pub use qp2p::Config as QuicP2pConfig;

/// Client trait and related constants.
pub mod api;

/// Utility functions.
pub mod utils;

#[cfg(test)]
/// Helpers for analysis of testnet logs
mod testnet_grep;

use tracing_core::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, SystemTime},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
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
        let target = event.metadata().file().unwrap_or("No target file known.");
        let span_separation_string = "\t ➤ ";
        let time = SystemTime::default();
        write!(writer, " {} ", level)?;

        time.format_time(&mut writer)?;

        writeln!(
            writer,
            " [{}:L{}]:",
            target,
            event.metadata().line().unwrap_or(0),
        )?;

        write!(writer, "{}", span_separation_string)?;

        // let mut span_count = 0;
        // Write spans and fields of each span
        ctx.visit_spans(|span| {
            write!(writer, "{} ", span.name())?;

            let ext = span.extensions();

            // `FormattedFields` is a a formatted representation of the span's
            // fields, which is stored in its extensions by the `fmt` layer's
            // `new_span` method. The fields will have been formatted
            // by the same field formatter that's provided to the event
            // formatter in the `FmtContext`.
            let fields = &ext
                .get::<FormattedFields<N>>()
                .expect("will never be `None`");

            if !fields.is_empty() {
                write!(writer, "{{{}}}", fields)?;
            }

            write!(writer, "\n{}", span_separation_string)?;

            Ok(())
        })?;

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

use std::sync::Once;

static INIT: Once = Once::new();

// TODO: dedupe this w/ sn_node?
/// Initialise logger for tests, this is run only once, even if called multiple times.
pub fn init_test_logger() {
    INIT.call_once(|| {
        tracing_subscriber::fmt::fmt()
            // NOTE: uncomment this line for pretty printed log output.
            //.pretty()
            .with_thread_names(true)
            .with_ansi(false)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .event_format(LogFormatter::default())
            .try_init().unwrap_or_else(|_| println!("Error initializing logger"));
    });
}

#[cfg(test)]
mod tests {
    use crate::testnet_grep::search_testnet_results_per_node;
    use eyre::Result;
    use sn_interface::network_knowledge::elder_count;
    use sn_interface::types::log_markers::LogMarker;

    // Check that with one split we have 14 elders.
    // This is intended to be run, just after split, in order to confirm splits are functioning correctly
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Testnet network_assert_ tests should be excluded from normal tests runs, they need to be run in sequence to ensure validity of checks"]
    async fn split_network_assert_health_check() -> Result<()> {
        let promoted_to_elder_nodes =
            search_testnet_results_per_node(LogMarker::PromotedToElder.to_string())?.len();

        let prefix1_prior_elder_nodes = search_testnet_results_per_node(format!(
            r"{}: Prefix\(1\)",
            LogMarker::StillElderAfterSplit
        ))?
        .len();
        let prefix1_new_elder_nodes = search_testnet_results_per_node(format!(
            r"{}: Prefix\(1\)",
            LogMarker::PromotedToElder
        ))?
        .len();
        let prefix0_prior_elder_nodes = search_testnet_results_per_node(format!(
            r"{}: Prefix\(0\)",
            LogMarker::StillElderAfterSplit
        ))?
        .len();
        let prefix0_new_elder_nodes = search_testnet_results_per_node(format!(
            r"{}: Prefix\(0\)",
            LogMarker::PromotedToElder
        ))?
        .len();

        let split_count =
            search_testnet_results_per_node(LogMarker::SplitSuccess.to_string())?.len();

        let desired_elder_count = elder_count();
        println!("Found splits: {:?}", split_count);
        println!(
            "Desired elder_count() per section: {:?}",
            desired_elder_count
        );
        println!("Promoted to elder so far: {:?}", promoted_to_elder_nodes);

        let total_elders = prefix0_prior_elder_nodes
            + prefix0_new_elder_nodes
            + prefix1_new_elder_nodes
            + prefix1_prior_elder_nodes;
        println!("Found elders: {:?}", total_elders);

        println!(
            "Found prefix_0_prior_elders: {:?}",
            prefix0_prior_elder_nodes
        );
        println!("Found prefix_0_new_elders: {:?}", prefix0_new_elder_nodes);

        println!(
            "Found prefix_1_prior_elders: {:?}",
            prefix1_prior_elder_nodes
        );
        println!("Found prefix_1_new_elders: {:?}", prefix1_new_elder_nodes);

        // assert!(prefix0_new_elder_nodes + prefix0_prior_elder_nodes >= desired_elder_count);
        // assert!(prefix1_prior_elder_nodes + prefix1_new_elder_nodes >= desired_elder_count);

        // // we're not discounting demotions at the moment, so just more than 14 is fine
        // assert!(total_elders >= 2 * desired_elder_count);

        assert!(split_count >= desired_elder_count);

        Ok(())
    }
}
