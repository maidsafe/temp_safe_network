// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{Error, Result};
use log::debug;
use uhttp_uri::HttpUri;
use url::Url;

const SAFE_URL_SCHEME: &str = "safe";

// URL labels must be 63 characters or less.
// See https://www.ietf.org/rfc/rfc1035.txt
const MAX_LEN_URL_LABELS: usize = 63;

// Invalid NRS characters
// These are characters that have no visual presence.
// Confusables are worrisome but not invalid.
const INVALID_NRS_CHARS: [char; 30] = [
    // https://en.wikipedia.org/wiki/Whitespace_character#Unicode
    '\u{200B}', // zero width space
    '\u{200C}', // zero width non-joiner
    '\u{200D}', // zero width joiner
    '\u{2060}', // word joiner
    '\u{FEFF}', // zero width non-breaking space
    '\u{180E}', // Mongolian vowel separator
    // https://en.wikipedia.org/wiki/Whitespace_character#Non-space_blanks
    '\u{2800}', // braille pattern blank
    '\u{3164}', // Hangul filler
    '\u{115F}', // Hangul Choseong filler
    '\u{1160}', // Hangul Jungseong filler
    '\u{FFA0}', // halfwidth Hangul filler
    // https://en.wikipedia.org/wiki/Whitespace_character#File_names
    '\u{2422}', // blank symbol
    // https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
    // Other_Default_Ignorable_Code_Point
    '\u{034F}', // combining grapheme joiner
    '\u{17B4}', // Khmer vowel inherent aq
    '\u{17B5}', // Khmer vowel inherent aa
    '\u{2065}', // reserved
    '\u{FFF0}', // reserved
    '\u{FFF1}', // reserved
    '\u{FFF2}', // reserved
    '\u{FFF3}', // reserved
    '\u{FFF4}', // reserved
    '\u{FFF5}', // reserved
    '\u{FFF6}', // reserved
    '\u{FFF7}', // reserved
    // https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
    // Deprecated
    '\u{206A}', // inhibit symmetric swapping
    '\u{206B}', // activate symmetric swapping
    '\u{206C}', // inhibit arabic form shaping
    '\u{206D}', // activate arabic form shaping
    '\u{206E}', // national digit shapes
    '\u{206F}', // nominal digit shapes
];

// A simple struct to represent the basic components parsed
// from a Safe URL without any decoding.
//
// This is kept internal to the crate, at least for now.
#[derive(Debug, Clone)]
pub(crate) struct SafeUrlParts {
    pub scheme: String,
    pub public_name: String, // "a.b.name" in "a.b.name"
    pub top_name: String,    // "name"     in "a.b.name"
    pub sub_names: String,   // "a.b"      in "a.b.name"
    pub sub_names_vec: Vec<String>,
    pub path: String,
    pub query_string: String,
    pub fragment: String,
}

impl SafeUrlParts {
    // parses a URL into its component parts, performing basic validation.
    pub fn parse(url: &str, ignore_labels_size: bool) -> Result<Self> {
        // detect any invalid url chars before parsing
        validate_url_chars(url)?;
        // Note: we use rust-url for parsing because it is most widely used
        // in rust ecosystem, and should be quite solid.  However, for paths,
        // (see below) we use a different parser to avoid normalization.
        // Parsing twice is inefficient, so there is room for improvement
        // later to standardize on a single parser.
        let parsing_url = Url::parse(&url).map_err(|parse_err| {
            let msg = format!("Problem parsing the URL \"{}\": {}", url, parse_err);
            Error::InvalidXorUrl(msg)
        })?;

        // Validate the url scheme is 'safe'
        let scheme = parsing_url.scheme().to_string();
        if scheme != SAFE_URL_SCHEME {
            let msg = format!(
                "invalid scheme: '{}'. expected: '{}'",
                scheme, SAFE_URL_SCHEME
            );
            return Err(Error::InvalidXorUrl(msg));
        }

        // validate name (url host) is not empty
        let public_name = match parsing_url.host_str() {
            Some(h) => h.to_string(),
            None => {
                let msg = format!("Problem parsing the URL \"{}\": {}", url, "missing name");
                return Err(Error::InvalidXorUrl(msg));
            }
        };

        // validate no empty sub names in name.
        if public_name.contains("..") {
            let msg = "name contains empty subname".to_string();
            return Err(Error::InvalidXorUrl(msg));
        }

        // validate overall name length
        // 255 octets or less
        // see https://www.ietf.org/rfc/rfc1035.txt
        if public_name.len() > 255 {
            let msg = format!(
                "Name is {} chars, must be no more than 255",
                public_name.len()
            );
            return Err(Error::InvalidInput(msg));
        }

        // parse top_name and sub_names from name
        let names_vec: Vec<String> = public_name.split('.').map(String::from).collect();

        // validate names length unless it's not required
        if !ignore_labels_size {
            for name in &names_vec {
                if name.len() > MAX_LEN_URL_LABELS {
                    let msg = format!(
                        "Label is {} chars, must be no more than 63: {}",
                        name.len(),
                        name
                    );
                    return Err(Error::InvalidInput(msg));
                }
            }
        }

        // convert into top_name and sub_names
        let top_name = names_vec[names_vec.len() - 1].to_string();
        let sub_names_vec = (&names_vec[0..names_vec.len() - 1]).to_vec();
        let sub_names = sub_names_vec.join(".");

        // get raw path, without any normalization.
        // We use HttpUri for this because rust-url does too
        // much normalization, eg replacing "../" with no option
        // to obtain raw path. Issue filed at:
        //   https://github.com/servo/rust-url/issues/602
        //
        // HttpUri only supports http(s) urls,
        // so we replace first occurrence of safe:// with http://.
        // This could be improved/optimized at a later time.
        let http_url = url.replacen("safe://", "http://", 1);
        let uri = HttpUri::new(&http_url).map_err(|parse_err| {
            let msg = format!("Problem parsing the URL \"{}\": {:?}", url, parse_err);
            Error::InvalidXorUrl(msg)
        })?;
        let path = uri.resource.path.to_string();

        // get query_params
        let query_string = parsing_url.query().unwrap_or("").to_string();
        let fragment = parsing_url.fragment().unwrap_or("").to_string();

        // double-slash is allowed but discouraged in regular URLs.
        // We don't allow them in Safe URLs.
        // See https://stackoverflow.com/questions/20523318/is-a-url-with-in-the-path-section-valid
        if path.contains("//") {
            let msg = "path contains empty component".to_string();
            return Err(Error::InvalidXorUrl(msg));
        }

        debug!(
            "Parsed url: scheme: {}, public_name: {}, top_name: {}, sub_names: {}, sub_names_vec: {:?}, path: {}, query_string: {}, fragment: {:?}",
            scheme,
            public_name,
            top_name,
            sub_names,
            sub_names_vec,
            path,
            query_string,
            fragment,
        );

        Ok(Self {
            scheme,
            public_name,
            sub_names,
            sub_names_vec,
            top_name,
            path,
            query_string,
            fragment,
        })
    }
}

fn validate_url_chars(url: &str) -> Result<()> {
    // validate no whitespace in url
    if url.contains(char::is_whitespace) {
        let msg = "The URL cannot contain whitespace".to_string();
        return Err(Error::InvalidInput(msg));
    }
    // validate no control characters in url
    if url.contains(char::is_control) {
        let msg = "The URL cannot contain control characters".to_string();
        return Err(Error::InvalidInput(msg));
    }
    // validate no other invalid characters in url
    if url.contains(&INVALID_NRS_CHARS[..]) {
        let msg = "The URL cannot contain invalid characters".to_string();
        return Err(Error::InvalidInput(msg));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};

    #[test]
    fn test_safeurl_validate_url_chars_with_whitespace() -> Result<()> {
        let urls = vec![
            // tests for
            // https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
            // White_Space block
            "safe://with space", // U+0020
            "safe://nonbreaking\u{00a0}space",
            "safe://tab\u{0009}char",
            "safe://new\u{000A}line",
            "safe://line\u{000B}tabulation",
            "safe://form\u{000C}feed",
            "safe://carriage\u{000D}return",
            "safe://next\u{0085}line",
            "safe://ogham\u{1680}spacemark",
            "safe://en\u{2000}quad",
            "safe://em\u{2001}quad",
            "safe://en\u{2002}space",
            "safe://en\u{2003}space",
            "safe://threeper\u{2004}emspace",
            "safe://fourper\u{2005}emspace",
            "safe://sixper\u{2006}emspace",
            "safe://figure\u{2007}space",
            "safe://punctuation\u{2008}space",
            "safe://thin\u{2009}space",
            "safe://hair\u{200A}space",
            "safe://line\u{2028}separator",
            "safe://paragraph\u{2029}separator",
            "safe://narrow\u{202F}nobreakspace",
            "safe://medium\u{205F}mathematicalspace",
            "safe://ideographic\u{3000}space",
        ];
        for url in urls {
            match SafeUrlParts::parse(&url, false) {
                Ok(_) => {
                    return Err(anyhow!(
                        "Unexpectedly validated url with whitespace {}",
                        url
                    ));
                }
                Err(Error::InvalidInput(msg)) => {
                    assert_eq!(msg, "The URL cannot contain whitespace".to_string());
                }
                Err(err) => {
                    return Err(anyhow!("Error returned is not the expected one: {}", err));
                }
            };
        }
        Ok(())
    }

    #[test]
    fn test_safeurl_validate_url_chars_with_control_characters() -> Result<()> {
        let urls = vec![
            // tests for
            // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Basic_ASCII_control_codes
            "safe://null\u{0000}character",
            "safe://start\u{0001}heading",
            "safe://start\u{0002}text",
            "safe://end\u{0003}text",
            "safe://end\u{0004}transmission",
            "safe://enquiry\u{0005}character",
            "safe://acknowledge\u{0006}character",
            "safe://bell\u{0007}character",
            "safe://backspace\u{0008}character",
            //U+0009-000D is also whitspace so is tested there
            "safe://shift\u{000E}out",
            "safe://shift\u{000F}in",
            "safe://datalink\u{0010}escape",
            "safe://device\u{0011}controlone",
            "safe://device\u{0012}controltwo",
            "safe://device\u{0013}controlthree",
            "safe://device\u{0014}controlfour",
            "safe://negative\u{0015}acknowledge",
            "safe://synchronous\u{0016}idle",
            "safe://end\u{0017}transmission",
            "safe://cancel\u{0018}character",
            "safe://end\u{0019}ofmedium",
            "safe://substitute\u{001A}character",
            "safe://escape\u{001B}character",
            "safe://file\u{001C}separator",
            "safe://group\u{001D}separator",
            "safe://record\u{001E}separator",
            "safe://unit\u{001F}separator",
            //U+0020 is also whitespace so is tested there
            "safe://delete\u{007F}character",
            // tests for
            // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#C1_controls
            "safe://padding\u{0080}character",
            "safe://highoctet\u{0081}preset",
            "safe://break\u{0082}permitted",
            "safe://no\u{0083}break",
            "safe://index\u{0084}character",
            //U+0085 is also whitespace so is tested there
            "safe://startof\u{0086}selectedarea",
            "safe://endof\u{0087}selectedarea",
            "safe://character\u{0088}tabulationset",
            "safe://character\u{0089}tabulationwithjustification",
            "safe://line\u{008A}tabulationset",
            "safe://partialline\u{008B}forward",
            "safe://partialline\u{008C}backward",
            "safe://reverse\u{008D}feed",
            "safe://single\u{008E}shift2",
            "safe://single\u{008F}shift3",
            "safe://devicecontrol\u{0090}string",
            "safe://private\u{0091}use1",
            "safe://private\u{0092}use2",
            "safe://set\u{0093}transmitstate",
            "safe://cancel\u{0094}character",
            "safe://message\u{0095}waiting",
            "safe://startof\u{0096}protectedarea",
            "safe://endof\u{0097}protectedarea",
            "safe://startof\u{0098}string",
            "safe://singlegraphic\u{0099}characterintroducer",
            "safe://single\u{009A}characterintroducer",
            "safe://controlsequence\u{009B}introducer",
            "safe://string\u{009C}terminator",
            "safe://operatingsystem\u{009D}command",
            "safe://privacy\u{009E}message",
            "safe://application\u{009F}programcommand",
        ];
        for url in urls {
            match SafeUrlParts::parse(&url, false) {
                Ok(_) => {
                    return Err(anyhow!(
                        "Unexpectedly validated url with control character {}",
                        url
                    ));
                }
                Err(Error::InvalidInput(msg)) => {
                    assert_eq!(msg, "The URL cannot contain control characters".to_string());
                }
                Err(err) => {
                    return Err(anyhow!("Error returned is not the expected one: {}", err));
                }
            };
        }
        Ok(())
    }

    #[test]
    fn test_safeurl_validate_url_chars_with_invalid_characters() -> Result<()> {
        let urls = vec![
            // values from
            // INVALID_NRS_CHARS const
            "safe://zerowidth\u{200B}space",
            "safe://zerowidth\u{200C}nonjoiner",
            "safe://zerowidth\u{200D}joiner",
            "safe://word\u{2060}joiner",
            "safe://zerowidth\u{FEFF}nbsp",
            "safe://mongolian\u{180E}vowelseparator",
            "safe://braille\u{2800}patter",
            "safe://hangul\u{3164}filler",
            "safe://hangul\u{115F}choseongfiller",
            "safe://hangul\u{1160}jungseongfiller",
            "safe://halfwidth\u{FFA0}hangulfiller",
            "safe://blank\u{2422}symbol",
            "safe://combining\u{034F}graphemejoiner",
            "safe://khmervowel\u{17B4}inherentaq",
            "safe://khmervowel\u{17B5}inherentaa",
            "safe://reserved\u{2065}reserved",
            "safe://reserved\u{FFF0}reserved",
            "safe://reserved\u{FFF1}reserved",
            "safe://reserved\u{FFF2}reserved",
            "safe://reserved\u{FFF3}reserved",
            "safe://reserved\u{FFF4}reserved",
            "safe://reserved\u{FFF5}reserved",
            "safe://reserved\u{FFF6}reserved",
            "safe://reserved\u{FFF7}reserved",
            "safe://inhibit\u{206A}symmetricswapping",
            "safe://activate\u{206B}symmetricswapping",
            "safe://inhibit\u{206C}arabicformshaping",
            "safe://activate\u{206D}arabicformshaping",
            "safe://national\u{206E}digitshapes",
            "safe://nominal\u{206F}digitshapes",
        ];
        for url in urls {
            match SafeUrlParts::parse(&url, false) {
                Ok(_) => {
                    return Err(anyhow!(
                        "Unexpectedly validated url with invalid character {}",
                        url
                    ));
                }
                Err(Error::InvalidInput(msg)) => {
                    assert_eq!(msg, "The URL cannot contain invalid characters".to_string());
                }
                Err(err) => {
                    return Err(anyhow!("Error returned is not the expected one: {}", err));
                }
            };
        }
        Ok(())
    }
}
