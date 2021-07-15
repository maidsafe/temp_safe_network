// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::OutputFmt;
use ansi_term::Style;
use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use num_traits::Float;
use prettytable::{format::FormatBuilder, Table};
use serde::ser::Serialize;
use sn_api::nrs::NrsMap;
use std::{
    collections::BTreeMap,
    io::{stdin, stdout, Read, Write},
};
use xor_name::{XorName, XOR_NAME_LEN};

const UNKNOWN_PUBLIC_NAME: &str = "<unknown>";

// Warn the user about a dry-run being performed
pub fn notice_dry_run() {
    println!("NOTE the operation is being performed in dry-run mode, therefore no changes are committed to the network.");
}

// Converts the XOR name bytes into a hex encoded string
pub fn xorname_to_hex(xorname: &XorName) -> String {
    xorname.0.iter().map(|b| format!("{:02x}", b)).collect()
}

// Converts a hex encoded string to XOR name bytes
pub fn hex_to_xorname(hex_str: &str) -> Result<XorName> {
    if hex_str.len() != XOR_NAME_LEN {
        bail!("Invalid hex encoded XorName string".to_string())
    } else {
        let mut xorname = XorName::default();
        xorname
            .0
            .copy_from_slice(hex_str[0..XOR_NAME_LEN].as_bytes());
        Ok(xorname)
    }
}

// Read the argument string from the STDIN if is not an arg provided
pub fn get_from_arg_or_stdin(arg: Option<String>, message: Option<&str>) -> Result<String> {
    match arg {
        Some(ref t) if t.is_empty() => {
            let val = get_from_stdin(message)?;
            Ok(String::from_utf8(val).map_err(|err| {
                anyhow!(
                    "String read from STDIN contains invalid UTF-8 characters: {}",
                    err
                )
            })?)
        }
        Some(t) => Ok(t),
        None => {
            let val = get_from_stdin(message)?;
            Ok(String::from_utf8(val).map_err(|err| {
                anyhow!(
                    "String read from STDIN contains invalid UTF-8 characters: {}",
                    err
                )
            })?)
        }
    }
}

pub fn read_stdin_response() -> Result<String> {
    let mut user_input = String::new();
    stdin()
        .read_line(&mut user_input)
        .with_context(|| "Error occurred when attempting to get input from stdin".to_string())?;
    if let Some('\n') = user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r') = user_input.chars().next_back() {
        user_input.pop();
    }
    Ok(user_input)
}

// Outputs a message and then reads from stdin
pub fn get_from_stdin(message: Option<&str>) -> Result<Vec<u8>> {
    let the_message = message.unwrap_or("...awaiting data from STDIN stream...");
    println!("{}", &the_message);
    let mut buffer = Vec::new();
    match std::io::stdin().read_to_end(&mut buffer) {
        Ok(size) => {
            debug!("Read ({} bytes) from STDIN", size);
            Ok(buffer)
        }
        Err(_) => bail!("Failed to read from STDIN stream".to_string()),
    }
}

// Prompt the user with the message provided
pub fn prompt_user(prompt_msg: &str, error_msg: &str) -> Result<String> {
    print!("{}", prompt_msg);
    let _ = stdout().flush();
    let buf = read_stdin_response()?;
    if buf.is_empty() {
        Err(anyhow!(error_msg.to_string()))
    } else {
        Ok(buf)
    }
}

#[allow(dead_code)]
// Unwrap secret key string provided, otherwise prompt user to provide it
pub fn get_secret_key(key_xorurl: &str, sk: Option<String>, msg: &str) -> Result<String> {
    let mut sk = sk.unwrap_or_else(|| String::from(""));

    if sk.is_empty() {
        let msg = if key_xorurl.is_empty() {
            format!("Enter secret key corresponding to {}: ", msg)
        } else {
            format!(
                "Enter secret key corresponding to public key at \"{}\": ",
                key_xorurl
            )
        };
        sk = prompt_user(&msg, "Invalid input")?;
    }

    Ok(sk)
}

pub fn gen_processed_files_table(
    processed_files: &BTreeMap<String, (String, String)>,
    show_change_sign: bool,
) -> (Table, u64) {
    let mut table = Table::new();
    let format = FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 1)
        .build();
    table.set_format(format);
    let mut success_count = 0;
    for (file_name, (change, link)) in processed_files.iter() {
        if change != "E" {
            success_count += 1;
        }
        if show_change_sign {
            table.add_row(row![change, file_name, link]);
        } else {
            table.add_row(row![file_name, link]);
        }
    }
    (table, success_count)
}

// converts "-" to "", both of which mean to read from stdin.
pub fn parse_stdin_arg(src: &str) -> String {
    if src.is_empty() || src == "-" {
        "".to_string()
    } else {
        src.to_string()
    }
}

// serialize structured value using any format from OutputFmt
// except OutputFmt::Pretty, which must be handled by caller.
pub fn serialise_output<T: ?Sized>(value: &T, fmt: OutputFmt) -> String
where
    T: Serialize,
{
    match fmt {
        OutputFmt::Yaml => serde_yaml::to_string(&value)
            .unwrap_or_else(|_| "Failed to serialise output to yaml".to_string()),
        OutputFmt::Json => serde_json::to_string_pretty(&value)
            .unwrap_or_else(|_| "Failed to serialise output to json".to_string()),
        OutputFmt::JsonCompact => serde_json::to_string(&value)
            .unwrap_or_else(|_| "Failed to serialise output to compact json".to_string()),
        OutputFmt::Pretty => {
            "OutputFmt::Pretty' not handled by caller, in serialise_output()".to_string()
        }
    }
}

// Pretty print an NRS Map
pub fn print_nrs_map(nrs_map: &NrsMap, public_name: &Option<String>) {
    let mut table = Table::new();
    table.add_row(row![bFg->"NRS name/subname", bFg->"Created", bFg->"Modified", bFg->"Link"]);

    let summary = nrs_map.get_map_summary();
    let pub_name: &str = match public_name {
        Some(name) => name,
        None => UNKNOWN_PUBLIC_NAME,
    };
    summary.iter().for_each(|(name, rdf_info)| {
        table.add_row(row![
            format!("{}{}", name, pub_name),
            rdf_info["created"],
            rdf_info["modified"],
            rdf_info["link"],
        ]);
    });
    table.printstd();
}

// returns singular or plural version of string, based on count.
pub fn pluralize<'a>(singular: &'a str, plural: &'a str, count: u64) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

// if stdout is a TTY, then it returns a string with ansi codes according to
// style.  Otherwise, it returns the original string.
pub fn if_tty(s: &str, style: Style) -> String {
    if isatty::stdout_isatty() {
        style.paint(s).to_string()
    } else {
        s.to_string()
    }
}

pub fn div_or<X: Float>(num: X, den: X, default: X) -> X {
    if (!num.is_normal() && !num.is_zero()) || !den.is_normal() {
        default
    } else {
        num / den
    }
}
