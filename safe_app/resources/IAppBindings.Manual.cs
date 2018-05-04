using System;
using System.Threading.Tasks;

namespace SafeApp {
    public partial interface IAppBindings {
        Task<App> AppUnregistered(byte[] bootstrapConfig, Action oDisconnectNotifierCb);
        Task<App> AppRegistered(String appId, ref AuthGrantedNative authGranted, Action oDisconnectNotifierCb);
    }
}