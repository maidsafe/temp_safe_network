using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace SafeApp {
  public abstract class IpcMsg {
  }

  public class AuthIpcMsg : IpcMsg {
    public uint ReqId;
    public AuthGranted AuthGranted;

    public AuthIpcMsg(uint reqId, AuthGranted authGranted) {
        ReqId = reqId;
        AuthGranted = authGranted;
    }
  }

  public class UnregisteredIpcMsg : IpcMsg {
    public uint ReqId;
    public List<byte> SerialisedCfg;

    public UnregisteredIpcMsg(uint reqId, IntPtr serialisedCfgPtr, ulong serialisedCfgLen) {
        ReqId = reqId;
        SerialisedCfg = BindingUtils.CopyToByteList(serialisedCfgPtr, (int) serialisedCfgLen);
    }
  }

  public class ContainersIpcMsg : IpcMsg {
    public uint ReqId;

    public ContainersIpcMsg(uint reqId) {
        ReqId = reqId;
    }
  }

  public class ShareMDataIpcMsg : IpcMsg {
    public uint ReqId;

    public ShareMDataIpcMsg(uint reqId) {
        ReqId = reqId;
    }
  }

  public class RevokedIpcMsg : IpcMsg {

  }

  public class IpcMsgException : FfiException {
    public readonly uint ReqId;

    public IpcMsgException(uint reqId, int code, string description)
        : base(code, description)
    {
        ReqId = reqId;
    }
  }
}
