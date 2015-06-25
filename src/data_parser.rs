// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

/// Parser for verifying incoming encoded data
pub enum Parser {
    /// Parser Variant
    ImmutableData(::maidsafe_types::ImmutableData),
    /// Parser Variant
    StructuredData(::maidsafe_types::StructuredData),
    /// Parser Variant
    Maid(::maidsafe_types::IdType),
    /// Parser Variant
    Mpid(::maidsafe_types::IdType),
    /// Parser Variant
    AnMaid(::maidsafe_types::RevocationIdType),
    /// Parser Variant
    AnMpid(::maidsafe_types::RevocationIdType),
    /// Parser Variant
    PublicMaid(::maidsafe_types::PublicIdType),
    /// Parser Variant
    PublicMpid(::maidsafe_types::PublicIdType),
    /// Parser Variant
    Unknown(u64),
}

impl ::rustc_serialize::Decodable for Parser {
    fn decode<D: ::rustc_serialize::Decoder>(d: &mut D) -> Result<Parser, D::Error> {
        let tag = try!(d.read_u64());

        match tag {
            ::maidsafe_types::data_tags::IMMUTABLE_DATA_TAG  => Ok(Parser::ImmutableData(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::STRUCTURED_DATA_TAG => Ok(Parser::StructuredData(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::MAID_TAG            => Ok(Parser::Maid(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::MPID_TAG            => Ok(Parser::Mpid(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::AN_MAID_TAG         => Ok(Parser::AnMaid(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::AN_MPID_TAG         => Ok(Parser::AnMpid(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::PUBLIC_MAID_TAG     => Ok(Parser::PublicMaid(try!(::rustc_serialize::Decodable::decode(d)))),
            ::maidsafe_types::data_tags::PUBLIC_MPID_TAG     => Ok(Parser::PublicMpid(try!(::rustc_serialize::Decodable::decode(d)))),
            _ => Ok(Parser::Unknown(tag)),
        }
    }
}
