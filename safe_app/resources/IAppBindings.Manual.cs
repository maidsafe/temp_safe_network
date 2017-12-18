using System;
using System.Threading.Tasks;

namespace SafeApp {
    public partial interface IAppBindings {
        void AppUnregistered(byte[] bootstrapConfig, Action oDisconnectNotifierCb, Action<FfiResult, IntPtr> oCb);
        void AppRegistered(String appId, ref AuthGranted authGranted, Action oDisconnectNotifierCb, Action<FfiResult, IntPtr> oCb);
        Task<IpcMsg> DecodeIpcMsgAsync(String msg);
    }
}