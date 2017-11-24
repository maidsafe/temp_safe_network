using System;
using System.Threading.Tasks;

namespace SafeApp {
    public partial class AppBindings : IAppBindings {
        #region App Creation

        public Task<App> AppUnregistered(byte[] bootstrapConfig, Action oDisconnectNotifierCb)
        {
            var tcs = new TaskCompletionSource<App>();
            var userData = BindingUtils.ToHandlePtr((tcs, oDisconnectNotifierCb));

            AppUnregisteredNative(bootstrapConfig,
                                  (ulong) bootstrapConfig.Length,
                                  userData,
                                  OnAppDisconnectCb,
                                  OnAppCreateCb);

            return tcs.Task;
        }

        public Task<App> AppRegistered(String appId,
                                       ref AuthGrantedNative authGranted,
                                       Action oDisconnectNotifierCb)
        {
            var tcs = new TaskCompletionSource<App>();
            var userData = BindingUtils.ToHandlePtr((tcs, oDisconnectNotifierCb));

            AppRegisteredNative(appId,
                                ref authGranted,
                                userData,
                                OnAppDisconnectCb,
                                OnAppCreateCb);

            return tcs.Task;
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(NoneCb))]
        #endif
        private static void OnAppDisconnectCb(IntPtr userData) {
            var (_, action) =
                BindingUtils.FromHandlePtr<(TaskCompletionSource<App>, Action)>(
                    userData, false
                );

            action();
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(FfiResultAppCb))]
        #endif
        private static void OnAppCreateCb(IntPtr userData, ref FfiResult result, App app) {
            var (tcs, _) =
                BindingUtils.FromHandlePtr<(TaskCompletionSource<App>, Action)>(
                    userData, false
                );

            BindingUtils.CompleteTask(tcs, ref result, app);
        }

        #endregion

        #region DecodeIpcMsg

        public Task<IpcMsg> DecodeIpcMsg(String msg) {
            var (task, userData) = BindingUtils.PrepareTask<IpcMsg>();
            DecodeIpcMsgNative(msg,
                               userData,
                               OnDecodeIpcMsgAuthCb,
                               OnDecodeIpcMsgUnregisteredCb,
                               OnDecodeIpcMsgContainersCb,
                               OnDecodeIpcMsgShareMdataCb,
                               OnDecodeIpcMsgRevokedCb,
                               OnDecodeIpcMsgErrCb);

            return task;
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(UintAuthGrantedNativeCb))]
        #endif
        private static void OnDecodeIpcMsgAuthCb(IntPtr userData, uint reqId, ref AuthGrantedNative authGranted)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetResult(new AuthIpcMsg(reqId, ref authGranted));
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(UintByteListCb))]
        #endif
        private static void OnDecodeIpcMsgUnregisteredCb(IntPtr userData, uint reqId, IntPtr serialisedCfgPtr, ulong serialisedCfgLen)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetResult(new UnregisteredIpcMsg(reqId, serialisedCfgPtr, serialisedCfgLen));
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(UintCb))]
        #endif
        private static void OnDecodeIpcMsgContainersCb(IntPtr userData, uint reqId)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetResult(new ContainersIpcMsg(reqId));
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(UintCb))]
        #endif
        private static void OnDecodeIpcMsgShareMdataCb(IntPtr userData, uint reqId)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetResult(new ShareMdataIpcMsg(reqId));
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(NoneCb))]
        #endif
        private static void OnDecodeIpcMsgRevokedCb(IntPtr userData)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetResult(new RevokedIpcMsg());
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(FfiResultUintCb))]
        #endif
        private static void OnDecodeIpcMsgErrCb(IntPtr userData, ref FfiResult result, uint reqId)
        {
            var tcs = BindingUtils.FromHandlePtr<TaskCompletionSource<IpcMsg>>(userData);
            tcs.SetException(new IpcMsgException(reqId, result.ErrorCode, result.Description));
        }

        #endregion
    }
}