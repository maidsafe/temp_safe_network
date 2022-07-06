// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! SAFE network data types.

/// Standardised messaging interface
pub mod messaging;
/// Knowledge of the safe network
pub mod network_knowledge;
/// Types on the safe network
pub mod types;

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

/// Max number of faulty Elders is assumed to be less than 1/3.
/// So it's no more than 2 with 7 Elders.
pub fn max_num_faulty_elders_for_sap(sap: SectionAuthorityProvider) -> usize {
    sap.elder_count() / 3
}

/// The least number of Elders to select, to be "guaranteed" one correctly functioning Elder.
/// This number will be 3 with 7 Elders.
pub fn at_least_one_correct_elder() -> usize {
    max_num_faulty_elders() + 1
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
            .with_thread_names(true)
            .with_ansi(false)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .event_format(LogFormatter::default())
            .try_init()
            .unwrap_or_else(|_| println!("Error initializing logger"));
    });
}
