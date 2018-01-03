using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace SafeApp {
    public partial class AppBindings {
        #region App Creation

        public void AppUnregistered(List<byte> bootstrapConfig, Action oDisconnectNotifierCb, Action<FfiResult, IntPtr, GCHandle> oCb)
        {
            var userData = BindingUtils.ToHandlePtr((oDisconnectNotifierCb, oCb));

            AppUnregisteredNative(bootstrapConfig.ToArray(),
                                  (IntPtr) bootstrapConfig.Count,
                                  userData,
                                  OnAppDisconnectCb,
                                  OnAppCreateCb);
        }

        public void AppRegistered(String appId,
                                  ref AuthGranted authGranted,
                                  Action oDisconnectNotifierCb,
                                  Action<FfiResult, IntPtr, GCHandle> oCb)
        {
            var authGrantedNative = authGranted.ToNative();
            var userData = BindingUtils.ToHandlePtr((oDisconnectNotifierCb, oCb));

            AppRegisteredNative(appId,
                                ref authGrantedNative,
                                userData,
                                OnAppDisconnectCb,
                                OnAppCreateCb);

            authGrantedNative.Free();
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(NoneCb))]
        #endif
        private static void OnAppDisconnectCb(IntPtr userData) {
            var (action, _) =
                BindingUtils.FromHandlePtr<(Action, Action<FfiResult, IntPtr, GCHandle>)>(
                    userData, false
                );

            action();
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(FfiResultAppCb))]
        #endif
        private static void OnAppCreateCb(IntPtr userData, ref FfiResult result, IntPtr app) {
            var (_, action) =
                BindingUtils.FromHandlePtr<(Action, Action<FfiResult, IntPtr, GCHandle>)>(
                    userData, false
                );

            action(result, app, GCHandle.FromIntPtr(userData));
        }

        #endregion

        #region DecodeIpcMsg

        public Task<IpcMsg> DecodeIpcMsgAsync(String msg) {
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
            tcs.SetResult(new AuthIpcMsg(reqId, new AuthGranted(authGranted)));
        }

        #if __IOS__
        [MonoPInvokeCallback(typeof(UintByteListCb))]
        #endif
        private static void OnDecodeIpcMsgUnregisteredCb(IntPtr userData, uint reqId, IntPtr serialisedCfgPtr, IntPtr serialisedCfgLen)
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