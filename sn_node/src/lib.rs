// Copyright 2022 MaidSafe.net limited.
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

mod dbs;

pub use dbs::UsedSpace;

pub mod node;

use tracing_core::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, SystemTime},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    registry::LookupSpan,
};

// pub(crate) use sn_interface::{data_copy_count, at_least_one_correct_elder, max_num_faulty_elders}

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
            .try_init().unwrap_or_else(|_| println!("Error initializing logger"));
    });
}
