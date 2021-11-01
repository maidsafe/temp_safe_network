// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Implementation of the "Node" node for the SAFE Network.

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

pub mod client;
mod dbs;

/// Helpers for analysis of testnet logs
pub mod testnet_grep;

pub use dbs::UsedSpace;

pub mod messaging;
pub mod node;
pub mod prefix_map;
pub mod routing;
pub mod types;
pub mod url;

use tracing_core::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, SystemTime},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    registry::LookupSpan,
};

#[cfg(test)]
use std::sync::Once;

#[cfg(test)]
#[ctor::ctor]
fn test_setup() {
    // If you look down the call stack for `color_eyre::install`, the only error can come from
    // `OnceCell::set` if it's called twice. We could ignore the error, but it would be better to
    // ensure we only call it once.
    color_eyre::install().expect("color_eyre::install can only be called once");
}

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
        let target = event.metadata().file().expect("will never be `None`");
        let span_separation_string = "\t ➤ ";
        let time = SystemTime::default();
        write!(writer, " {} ", level)?;

        time.format_time(&mut writer)?;

        writeln!(
            writer,
            " [{}:L{}]:",
            target,
            event.metadata().line().expect("will never be `None`"),
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

#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
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
            .init()
    });
}

#[cfg(test)]
mod tests {
    use crate::routing::log_markers::LogMarker;
    use crate::testnet_grep::search_testnet;
    use eyre::Result;

    // Check that with one split we have 14 elders.
    // This is intended to be run, just after split, in order to confirm splits are functioning correctly
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Testnet network_assert_ tests should be excluded from normal tests runs, they need to be run in sequence to ensure validity of checks"]
    async fn split_network_assert_expected_elder_counts() -> Result<()> {
        let split_count = search_testnet(&LogMarker::SplitSuccess)?.len();
        assert_eq!(split_count, 7);

        let promoted_count = search_testnet(&LogMarker::PromotedToElder)?.len();
        let demoted_count = search_testnet(&LogMarker::DemotedFromElder)?.len();

        let total_elders = promoted_count - demoted_count;
        assert_eq!(total_elders, 14);

        Ok(())
    }
}
