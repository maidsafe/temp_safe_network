// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref MEDIA_TYPE_STR: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        let mut populate = |subtypes: &[&'static str], range| {
            for (i, subtype) in subtypes.iter().enumerate() {
                let code = range + (i as u16);
                m.insert(code, *subtype);
            }
        };
        populate(&APPLICATION_SUBTYPES, 0x5000);
        populate(&AUDIO_SUBTYPES, 0x8000);
        populate(&FONT_SUBTYPES, 0x9000);
        populate(&IMAGE_SUBTYPES, 0xa000);
        populate(&MULTIPART_SUBTYPES, 0xd000);
        populate(&TEXT_SUBTYPES, 0xe000);
        populate(&VIDEO_SUBTYPES, 0xf000);
        m
    };
    pub static ref MEDIA_TYPE_CODES: HashMap<String, u16> = {
        let mut m = HashMap::new();
        let mut populate = |subtypes: &[&'static str], range| {
            for (i, subtype) in subtypes.iter().enumerate() {
                let code = range + (i as u16);
                m.insert((*subtype).to_string(), code);
            }
        };
        populate(&APPLICATION_SUBTYPES, 0x5000);
        populate(&AUDIO_SUBTYPES, 0x8000);
        populate(&FONT_SUBTYPES, 0x9000);
        populate(&IMAGE_SUBTYPES, 0xa000);
        populate(&MULTIPART_SUBTYPES, 0xd000);
        populate(&TEXT_SUBTYPES, 0xe000);
        populate(&VIDEO_SUBTYPES, 0xf000);
        m
    };
}

/* MIME Types:
 *
 * Based on the information at https://www.iana.org/assignments/media-types/media-types.xhtml
 * the following ranges can be reserved for the different mime types/subtypes.
 * In this implementation we are only declaring the mime types listed in the following article
 * since these should be the most relevant for the web, plus a few more useful for the semantic web:
 * https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Complete_list_of_MIME_types
 */
// Range 0x5000 - 0x7fff (14 bits) reserved for 'application/*' (there currently are ~1,300 subtypes)
static APPLICATION_SUBTYPES: [&str; 38] = [
    "application/x-abiword",
    "application/octet-stream",
    "application/vnd.amazon.ebook",
    "application/x-bzip",
    "application/x-bzip2",
    "application/x-csh",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-fontobject",
    "application/epub+zip",
    "application/ecmascript",
    "application/java-archive",
    "application/javascript",
    "application/json",
    "application/vnd.apple.installer+xml",
    "application/vnd.oasis.opendocument.presentation",
    "application/vnd.oasis.opendocument.spreadsheet",
    "application/vnd.oasis.opendocument.text",
    "application/ogg",
    "application/pdf",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "application/x-rar-compressed",
    "application/rtf",
    "application/x-sh",
    "application/x-shockwave-flash",
    "application/x-tar",
    "application/typescript",
    "application/vnd.visio",
    "application/xhtml+xml",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/xml",
    "application/vnd.mozilla.xul+xml",
    "application/zip",
    "application/x-7z-compressed",
    "application/ld+json",
    "application/rdf+xml",
];

// Range 0x8000 - 0x8fff (12 bits) reserved for 'audio/*' (there currently are ~150 subtypes)
static AUDIO_SUBTYPES: [&str; 10] = [
    "audio/aac",
    "audio/midi",
    "audio/x-midi",
    "audio/ogg",
    "audio/wav",
    "audio/webm",
    "audio/3gpp",
    "audio/3gpp2",
    "audio/mp4",
    "audio/mpeg",
];

// Range 0x9000 - 0x9fff (12 bits) reserved for 'font/*' (there currently are ~8 subtypes)
static FONT_SUBTYPES: [&str; 4] = ["font/otf", "font/ttf", "font/woff", "font/woff2"];

// Range 0xa000 - 0xafff (12 bits) reserved for 'image/*' (there currently are ~60 subtypes)
static IMAGE_SUBTYPES: [&str; 8] = [
    "image/bmp",
    "image/gif",
    "image/x-icon",
    "image/jpeg",
    "image/png",
    "image/svg+xml",
    "image/tiff",
    "image/webp",
];

// Range 0xb000 - 0xbfff (12 bits) reserved for 'message/*' (there currently are ~18 subtypes)
// static MESSAGE_SUBTYPES: [&str; 1] = ["message/sip"];

// Range 0xc000 - 0xcfff (12 bits) reserved for 'model/*' (there currently are ~24 subtypes)
// static MODEL_SUBTYPES: [&str; 1] = ["model/"];

// Range 0xd000 - 0xdfff (12 bits) reserved for 'multipart/*' (there currently are ~13 subtypes)
static MULTIPART_SUBTYPES: [&str; 1] = ["multipart/byteranges"];

// Range 0xe000 - 0xefff (12 bits) reserved for 'text/*' (there currently are ~71 subtypes)
static TEXT_SUBTYPES: [&str; 10] = [
    "text/css",
    "text/csv",
    "text/html",
    "text/calendar",
    "text/markdown",
    "text/n3",
    "text/plain",
    "text/turtle",
    "text/x-markdown",
    "text/xml",
];

// Range 0xf000 - 0xffff (12 bits) reserved for 'video/*' (there currently are ~78 subtypes)
static VIDEO_SUBTYPES: [&str; 8] = [
    "video/x-msvideo",
    "video/mpeg",
    "video/ogg",
    "video/webm",
    "video/3gpp",
    "video/3gpp2",
    "video/jpeg",
    "video/mp4",
];
