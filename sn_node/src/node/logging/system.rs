// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sysinfo::{DiskUsage, ProcessExt};

/// Struct containing a process' information.
#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct Process {
    pub(super) memory: u64,
    pub(super) virtual_memory: u64,
    pub(super) cpu_usage: f32,
    pub(super) disk_usage: DiskUsage,
}

impl Process {
    pub(super) fn map(process: &sysinfo::Process, processors: usize) -> Process {
        let usage = process.disk_usage();
        Process {
            memory: process.memory(),
            virtual_memory: process.virtual_memory(),
            cpu_usage: process.cpu_usage() / processors as f32,
            disk_usage: DiskUsage {
                total_written_bytes: usage.total_written_bytes,
                written_bytes: usage.written_bytes,
                total_read_bytes: usage.total_read_bytes,
                read_bytes: usage.read_bytes,
            },
        }
    }
}
