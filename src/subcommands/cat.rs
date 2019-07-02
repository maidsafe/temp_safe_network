// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_target_location;
use super::OutputFmt;
use prettytable::Table;
use safe_cli::{Safe, SafeData};

pub fn cat_command(
    location: Option<String>,
    _version: Option<String>,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    // TODO: Get specific versions.
    let xorurl = get_target_location(location)?;
    let content = safe.fetch(&xorurl)?;
    match content {
        SafeData::FilesContainer { files_map, .. } => {
            // Render FilesContainer
            if OutputFmt::Pretty == output_fmt {
                let mut table = Table::new();
                println!("Files of FilesContainer at: \"{}\"", xorurl);
                table.add_row(row![bFg->"Name", bFg->"Size", bFg->"Created", bFg->"Link"]);
                files_map.iter().for_each(|(name, file_item)| {
                    table.add_row(row![
                        name,
                        file_item["size"],
                        file_item["created"],
                        file_item["link"],
                    ]);
                });
                table.printstd();
            } else {
                println!("[{}, {:?}]", xorurl, files_map);
            }
        }
        SafeData::ImmutableData { data, .. } => {
            // Render ImmutableData file
            let data_string = match String::from_utf8(data) {
                Ok(string) => string,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            // data always has \n at end?
            println!("{}", data_string);
        }
        other => println!(
            "Content type '{:?}' not supported yet by cat command",
            other
        ),
    }

    Ok(())
}
