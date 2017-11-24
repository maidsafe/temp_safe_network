using System;
using System.Runtime.InteropServices;

namespace SafeApp {
    public abstract class IpcMsg {
    }

    public class AuthIpcMsg : IpcMsg {
        public uint ReqId;
        public AuthGrantedNative AuthGranted;

        public AuthIpcMsg(uint reqId, ref AuthGrantedNative authGranted) {
            ReqId = reqId;
            AuthGranted = authGranted;
        }
    }

    public class UnregisteredIpcMsg : IpcMsg {
        public uint ReqId;
        public IntPtr SerialisedCfgPtr;
        public ulong SerialisedCfgLen;

        public UnregisteredIpcMsg(uint reqId, IntPtr serialisedCfgPtr, ulong serialisedCfgLen) {
            ReqId = reqId;
            SerialisedCfgPtr = serialisedCfgPtr;
            SerialisedCfgLen = serialisedCfgLen;
        }
    }

    public class ContainersIpcMsg : IpcMsg {
        public uint ReqId;

        public ContainersIpcMsg(uint reqId) {
            ReqId = reqId;
        }
    }

    public class ShareMdataIpcMsg : IpcMsg {
        public uint ReqId;

        public ShareMdataIpcMsg(uint reqId) {
            ReqId = reqId;
        }
    }

    public class RevokedIpcMsg : IpcMsg {

    }

    public class IpcMsgException : FfiException {
        public readonly uint ReqId;

        public IpcMsgException(uint reqId, int code, String description)
            : base(code, description)
        {
            ReqId = reqId;
        }
    }
}
