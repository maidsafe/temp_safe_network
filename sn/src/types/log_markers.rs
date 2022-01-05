// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// this gets us to_string easily enough
use strum_macros::{Display as StrumDisplay, EnumIter, EnumString};

/// Internal log marker, to be used in tests asserts.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, EnumIter, StrumDisplay, EnumString)]
#[allow(missing_docs)]
pub enum LogMarker {
    // stats
    TotalNetworkElders,
    // split
    StillElderAfterSplit,
    SplitSuccess,
    SplitAttempt,
    NewPrefix,
    AeSendUpdateToSiblings,
    AgreementOfOnline,
    // messaging
    ServiceMsgToBeHandled,
    SystemMsgToBeHandled,
    // chunks
    StoringChunk,
    ChunkStoreReceivedAtElder,
    StoredNewChunk,
    QueryResponseReceviedFromAdult,
    ChunkQueryReceviedAtElder,
    ChunkQueryReceviedAtAdult,

    // register
    RegisterWrite,
    RegisterWriteReceivedAtElder,
    RegisterQueryReceviedAtElder,
    RegisterQueryReceviedAtAdult,
    RegisterQueryResponseReceviedFromAdult,

    RegisterQueryReceived,
    // routing commands
    DispatchHandleMsgCmd,
    DispatchSendMsgCmd,
    CommandHandleSpawned,
    CommandHandleStart,
    CommandHandleEnd,
    CommandHandleError,
    // dkg + promotion
    PromotedToElder,
    DemotedFromElder,
    DkgSendFailureObservation,
    DkgBroadcastMsg,
    DkgMessageHandling,
    DkgSessionComplete,
    DkgSessionAlreadyCompleted,
    HandlingDkgSuccessfulOutcome,
    HandlingNewEldersAgreement,
    NewSignedSap,
    NewKeyShareStored,
    TriggeringPromotionAndDemotion,
    // ae
    AeResendAfterRetry,
    AeResendAfterAeRedirect,
    AeSendRedirect,
    AeSendRetryAsOutdated,
    AeSendRetryDstPkFail,
    // Sending messages
    SendOrHandle,
    SendDirect,
    SendDirectToNodes,
    SendResourceProofChallenge,
    SendJoinRequest,
    SendJoinAsRelocatedResponse,
    SendJoinRejected,
    SendJoinRedirected,
    SendJoinRetryNotCorrectKey,
    SendJoinRetryAgeIssue,
    SendJoinRetryNotAdult,
    SendJoinsDisallowed,
    SendDKGUnderway,
    SendNodeApproval,
    // approved to join
    ReceivedJoinApproved,
    // Connections
    ConnectionOpened,
    ConnectionClosed,
    ConnectionReused,
    // relocation
    RelocateStart,
    RelocateEnd,
}
