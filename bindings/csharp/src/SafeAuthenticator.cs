using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
public class AccountInfo {
    /// Number of used mutations.
    public ulong mutationsDone;
    /// Number of available mutations.
    public ulong mutationsAvailable;
}

[StructLayout(LayoutKind.Sequential)]
public class MDataInfo {
    /// Name of the mutable data.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.XOR_NAME_LEN)]
    public byte[] name;
    /// Type tag of the mutable data.
    public ulong typeTag;
    /// Flag indicating whether the encryption info (`enc_key` and `enc_nonce`).
    /// is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasEncInfo;
    /// Encryption key. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Encryption nonce. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_NONCE_LEN)]
    public byte[] encNonce;
    /// Flag indicating whether the new encryption info is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasNewEncInfo;
    /// New encryption key (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_KEY_LEN)]
    public byte[] newEncKey;
    /// New encryption nonce (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_NONCE_LEN)]
    public byte[] newEncNonce;
}

[StructLayout(LayoutKind.Sequential)]
public class PermissionSet {
    /// How to modify the read permission.
    [MarshalAs(UnmanagedType.Bool)]
    public bool read;
    /// How to modify the insert permission.
    [MarshalAs(UnmanagedType.Bool)]
    public bool insert;
    /// How to modify the update permission.
    [MarshalAs(UnmanagedType.Bool)]
    public bool update;
    /// How to modify the delete permission.
    [MarshalAs(UnmanagedType.Bool)]
    public bool delete;
    /// How to modify the manage permissions permission.
    [MarshalAs(UnmanagedType.Bool)]
    public bool managePermissions;
}

[StructLayout(LayoutKind.Sequential)]
public class AuthReq {
    /// The application identifier for this request
    public AppExchangeInfo app;
    /// `true` if the app wants dedicated container for itself. `false`
    /// otherwise.
    [MarshalAs(UnmanagedType.Bool)]
    public bool appContainer;
    /// Array of `ContainerPermissions`
    public ContainerPermissions containers;
    /// Size of container permissions array
    public ulong containersLen;
    /// Capacity of container permissions array. Internal field
    /// required for the Rust allocator.
    public ulong containersCap;
}

[StructLayout(LayoutKind.Sequential)]
public class ContainersReq {
    /// Exchange info
    public AppExchangeInfo app;
    /// Requested containers
    public ContainerPermissions containers;
    /// Size of requested containers array
    public ulong containersLen;
    /// Capacity of requested containers array. Internal field
    /// required for the Rust allocator.
    public ulong containersCap;
}

[StructLayout(LayoutKind.Sequential)]
public class AppExchangeInfo {
    /// UTF-8 encoded id
    [MarshalAs(UnmanagedType.LPStr)]
    public String id;
    /// Reserved by the frontend
    ///
    /// null if not present
    [MarshalAs(UnmanagedType.LPStr)]
    public String scope;
    /// UTF-8 encoded application friendly-name.
    [MarshalAs(UnmanagedType.LPStr)]
    public String name;
    /// UTF-8 encoded application provider/vendor (e.g. MaidSafe)
    [MarshalAs(UnmanagedType.LPStr)]
    public String vendor;
}

[StructLayout(LayoutKind.Sequential)]
public class ContainerPermissions {
    /// The UTF-8 encoded id
    [MarshalAs(UnmanagedType.LPStr)]
    public String contName;
    /// The requested permission set
    public PermissionSet access;
}

[StructLayout(LayoutKind.Sequential)]
public class ShareMDataReq {
    /// Info about the app requesting shared access
    public AppExchangeInfo app;
    /// List of MD names & type tags and permissions that need to be shared
    public ShareMData mdata;
    /// Length of the mdata array
    public ulong mdataLen;
    /// Capacity of the mdata vec - internal implementation detail
    public ulong mdataCap;
}

[StructLayout(LayoutKind.Sequential)]
public class ShareMData {
    /// The mutable data type.
    public ulong typeTag;
    /// The mutable data name.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.XOR_NAME_LEN)]
    public byte[] name;
    /// The permissions being requested.
    public PermissionSet perms;
}

[StructLayout(LayoutKind.Sequential)]
public class AuthGranted {
    /// The access keys.
    public AppKeys appKeys;
    /// Access container info
    public AccessContInfo accessContainerInfo;
    /// Access container entry
    public AccessContainerEntry accessContainerEntry;
    /// Crust's bootstrap config
    public IntPtr bootstrapConfigPtr;
    /// `bootstrap_config`'s length
    public ulong bootstrapConfigLen;
    /// Used by Rust memory allocator
    public ulong bootstrapConfigCap;
}

[StructLayout(LayoutKind.Sequential)]
public class AppKeys {
    /// Owner signing public key
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SIGN_PUBLIC_KEY_LEN)]
    public byte[] ownerKey;
    /// Data symmetric encryption key
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Asymmetric sign public key.
    ///
    /// This is the identity of the App in the Network.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SIGN_PUBLIC_KEY_LEN)]
    public byte[] signPk;
    /// Asymmetric sign private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SIGN_SECRET_KEY_LEN)]
    public byte[] signSk;
    /// Asymmetric enc public key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.ASYM_PUBLIC_KEY_LEN)]
    public byte[] encPk;
    /// Asymmetric enc private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.ASYM_SECRET_KEY_LEN)]
    public byte[] encSk;
}

[StructLayout(LayoutKind.Sequential)]
public class AccessContInfo {
    /// ID
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.XOR_NAME_LEN)]
    public byte[] id;
    /// Type tag
    public ulong tag;
    /// Nonce
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SYM_NONCE_LEN)]
    public byte[] nonce;
}

[StructLayout(LayoutKind.Sequential)]
public class AccessContainerEntry {
    /// Pointer to the array of `ContainerInfo`.
    public ContainerInfo ptr;
    /// Size of the array.
    public ulong len;
    /// Internal field used by rust memory allocator.
    public ulong cap;
}

[StructLayout(LayoutKind.Sequential)]
public class ContainerInfo {
    /// Container name as UTF-8 encoded null-terminated string.
    [MarshalAs(UnmanagedType.LPStr)]
    public String name;
    /// Container's `MDataInfo`
    public MDataInfo mdataInfo;
    /// App's permissions in the container.
    public PermissionSet permissions;
}

[StructLayout(LayoutKind.Sequential)]
public class AppAccess {
    /// App's or user's public key
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.SIGN_PUBLIC_KEY_LEN)]
    public byte[] signKey;
    /// A list of permissions
    public PermissionSet permissions;
    /// App's user-facing name
    [MarshalAs(UnmanagedType.LPStr)]
    public String name;
    /// App id.
    [MarshalAs(UnmanagedType.LPStr)]
    public String appId;
}

[StructLayout(LayoutKind.Sequential)]
public class MetadataResponse {
    /// Name or purpose of this mutable data.
    [MarshalAs(UnmanagedType.LPStr)]
    public String name;
    /// Description of how this mutable data should or should not be shared.
    [MarshalAs(UnmanagedType.LPStr)]
    public String description;
    /// Xor name of this struct's corresponding MData object.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.XOR_NAME_LEN)]
    public byte[] xorName;
    /// Type tag of this struct's corresponding MData object.
    public ulong typeTag;
}

[StructLayout(LayoutKind.Sequential)]
public class MDataValue {
    /// Content pointer.
    public IntPtr contentPtr;
    /// Content length.
    public ulong contentLen;
    /// Entry version.
    public ulong entryVersion;
}

[StructLayout(LayoutKind.Sequential)]
public class MDataKey {
    /// Key value pointer.
    public IntPtr valPtr;
    /// Key length.
    public ulong valLen;
}

[StructLayout(LayoutKind.Sequential)]
public class File {
    /// File size in bytes.
    public ulong size;
    /// Creation time (seconds part).
    public long createdSec;
    /// Creation time (nanoseconds part).
    public uint createdNsec;
    /// Modification time (seconds part).
    public long modifiedSec;
    /// Modification time (nanoseconds part).
    public uint modifiedNsec;
    /// Pointer to the user metadata.
    public IntPtr userMetadataPtr;
    /// Size of the user metadata.
    public ulong userMetadataLen;
    /// Capacity of the user metadata (internal field).
    public ulong userMetadataCap;
    /// Name of the `ImmutableData` containing the content of this file.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeAuthenticator.XOR_NAME_LEN)]
    public byte[] dataMapName;
}

[StructLayout(LayoutKind.Sequential)]
public class RegisteredApp {
    /// Unique application identifier
    public AppExchangeInfo appInfo;
    /// List of containers that this application has access to
    public ContainerPermissions containers;
    /// Length of the containers array
    public ulong containersLen;
    /// Capacity of the containers array. Internal data required
    /// for the Rust allocator.
    public ulong containersCap;
}

[StructLayout(LayoutKind.Sequential)]
public struct Authenticator {
    private IntPtr value;
}

public static class SafeAuthenticator {
    #if __IOS__
    private const String DLL_NAME = "__Internal";
    #else
    private const String DLL_NAME = "safe_authenticator";
    #endif

    public const ulong MAIDSAFE_TAG = 5483000;
    public const ulong DIR_TAG = 15000;
    public const String SAFE_MOCK_UNLIMITED_MUTATIONS = "SAFE_MOCK_UNLIMITED_MUTATIONS";
    public const String SAFE_MOCK_IN_MEMORY_STORAGE = "SAFE_MOCK_IN_MEMORY_STORAGE";
    public const String SAFE_MOCK_VAULT_PATH = "SAFE_MOCK_VAULT_PATH";

    public static bool IsMockBuild() {
        return IsMockBuildNative();
    }

    [DllImport(DLL_NAME, EntryPoint = "is_mock_build")]
    private static extern bool IsMockBuildNative();

    public static void CreateAcc(String accountLocator, String accountPassword, String invitation, Action oDisconnectNotifierCb, Action<FfiResult, Authenticator> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oDisconnectNotifierCb, oCb)));
        CreateAccNative(accountLocator, accountPassword, invitation, userData, OnNoneAndFfiResultAuthenticatorCb0, OnNoneAndFfiResultAuthenticatorCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "create_acc")]
    private static extern void CreateAccNative([MarshalAs(UnmanagedType.LPStr)] String accountLocator, [MarshalAs(UnmanagedType.LPStr)] String accountPassword, [MarshalAs(UnmanagedType.LPStr)] String invitation, IntPtr userData, NoneAndFfiResultAuthenticatorCb0 oDisconnectNotifierCb, NoneAndFfiResultAuthenticatorCb1 oCb);

    public static void Login(String accountLocator, String accountPassword, Action oDisconnectNotifierCb, Action<FfiResult, Authenticator> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oDisconnectNotifierCb, oCb)));
        LoginNative(accountLocator, accountPassword, userData, OnNoneAndFfiResultAuthenticatorCb0, OnNoneAndFfiResultAuthenticatorCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "login")]
    private static extern void LoginNative([MarshalAs(UnmanagedType.LPStr)] String accountLocator, [MarshalAs(UnmanagedType.LPStr)] String accountPassword, IntPtr userData, NoneAndFfiResultAuthenticatorCb0 oDisconnectNotifierCb, NoneAndFfiResultAuthenticatorCb1 oCb);

    public static void AuthReconnect(Authenticator auth, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthReconnectNative(auth, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_reconnect")]
    private static extern void AuthReconnectNative(Authenticator auth, IntPtr userData, FfiResultCb oCb);

    public static void AuthAccountInfo(Authenticator auth, Action<FfiResult, AccountInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthAccountInfoNative(auth, userData, OnFfiResultAccountInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_account_info")]
    private static extern void AuthAccountInfoNative(Authenticator auth, IntPtr userData, FfiResultAccountInfoCb oCb);

    public static void AuthExeFileStem(Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthExeFileStemNative(userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_exe_file_stem")]
    private static extern void AuthExeFileStemNative(IntPtr userData, FfiResultStringCb oCb);

    public static void AuthSetAdditionalSearchPath(String newPath, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthSetAdditionalSearchPathNative(newPath, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_set_additional_search_path")]
    private static extern void AuthSetAdditionalSearchPathNative([MarshalAs(UnmanagedType.LPStr)] String newPath, IntPtr userData, FfiResultCb oCb);

    public static void AuthFree(Authenticator auth) {
        AuthFreeNative(auth);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_free")]
    private static extern void AuthFreeNative(Authenticator auth);

    public static void AuthRmRevokedApp(Authenticator auth, String appId, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthRmRevokedAppNative(auth, appId, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_rm_revoked_app")]
    private static extern void AuthRmRevokedAppNative(Authenticator auth, [MarshalAs(UnmanagedType.LPStr)] String appId, IntPtr userData, FfiResultCb oCb);

    public static void AuthRevokedApps(Authenticator auth, Action<FfiResult, AppExchangeInfo[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthRevokedAppsNative(auth, userData, OnFfiResultArrayOfAppExchangeInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_revoked_apps")]
    private static extern void AuthRevokedAppsNative(Authenticator auth, IntPtr userData, FfiResultArrayOfAppExchangeInfoCb oCb);

    public static void AuthRegisteredApps(Authenticator auth, Action<FfiResult, RegisteredApp[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthRegisteredAppsNative(auth, userData, OnFfiResultArrayOfRegisteredAppCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_registered_apps")]
    private static extern void AuthRegisteredAppsNative(Authenticator auth, IntPtr userData, FfiResultArrayOfRegisteredAppCb oCb);

    public static void AuthAppsAccessingMutableData(Authenticator auth, IntPtr mdName, ulong mdTypeTag, Action<FfiResult, AppAccess[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthAppsAccessingMutableDataNative(auth, mdName, mdTypeTag, userData, OnFfiResultArrayOfAppAccessCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_apps_accessing_mutable_data")]
    private static extern void AuthAppsAccessingMutableDataNative(Authenticator auth, IntPtr mdName, ulong mdTypeTag, IntPtr userData, FfiResultArrayOfAppAccessCb oCb);

    public static void AuthUnregisteredDecodeIpcMsg(String msg, Action<uint, byte[]> oUnregistered, Action<FfiResult, String> oErr) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oUnregistered, oErr)));
        AuthUnregisteredDecodeIpcMsgNative(msg, userData, OnUIntArrayOfByteAndFfiResultStringCb0, OnUIntArrayOfByteAndFfiResultStringCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_unregistered_decode_ipc_msg")]
    private static extern void AuthUnregisteredDecodeIpcMsgNative([MarshalAs(UnmanagedType.LPStr)] String msg, IntPtr userData, UIntArrayOfByteAndFfiResultStringCb0 oUnregistered, UIntArrayOfByteAndFfiResultStringCb1 oErr);

    public static void AuthDecodeIpcMsg(Authenticator auth, String msg, Action<uint, AuthReq> oAuth, Action<uint, ContainersReq> oContainers, Action<uint, byte[]> oUnregistered, Action<uint, ShareMDataReq, MetadataResponse> oShareMdata, Action<FfiResult, String> oErr) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oAuth, oContainers, oUnregistered, oShareMdata, oErr)));
        AuthDecodeIpcMsgNative(auth, msg, userData, OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb0, OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb1, OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb2, OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb3, OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb4);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_decode_ipc_msg")]
    private static extern void AuthDecodeIpcMsgNative(Authenticator auth, [MarshalAs(UnmanagedType.LPStr)] String msg, IntPtr userData, UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb0 oAuth, UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb1 oContainers, UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb2 oUnregistered, UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb3 oShareMdata, UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb4 oErr);

    public static void EncodeShareMdataResp(Authenticator auth, ShareMDataReq req, uint reqId, bool isGranted, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeShareMdataRespNative(auth, req, reqId, isGranted, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_share_mdata_resp")]
    private static extern void EncodeShareMdataRespNative(Authenticator auth, ShareMDataReq req, uint reqId, [MarshalAs(UnmanagedType.Bool)] bool isGranted, IntPtr userData, FfiResultStringCb oCb);

    public static void AuthRevokeApp(Authenticator auth, String appId, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthRevokeAppNative(auth, appId, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_revoke_app")]
    private static extern void AuthRevokeAppNative(Authenticator auth, [MarshalAs(UnmanagedType.LPStr)] String appId, IntPtr userData, FfiResultStringCb oCb);

    public static void AuthFlushAppRevocationQueue(Authenticator auth, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthFlushAppRevocationQueueNative(auth, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_flush_app_revocation_queue")]
    private static extern void AuthFlushAppRevocationQueueNative(Authenticator auth, IntPtr userData, FfiResultCb oCb);

    public static void EncodeUnregisteredResp(uint reqId, bool isGranted, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeUnregisteredRespNative(reqId, isGranted, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_unregistered_resp")]
    private static extern void EncodeUnregisteredRespNative(uint reqId, [MarshalAs(UnmanagedType.Bool)] bool isGranted, IntPtr userData, FfiResultStringCb oCb);

    public static void EncodeAuthResp(Authenticator auth, AuthReq req, uint reqId, bool isGranted, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeAuthRespNative(auth, req, reqId, isGranted, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_auth_resp")]
    private static extern void EncodeAuthRespNative(Authenticator auth, AuthReq req, uint reqId, [MarshalAs(UnmanagedType.Bool)] bool isGranted, IntPtr userData, FfiResultStringCb oCb);

    public static void EncodeContainersResp(Authenticator auth, ContainersReq req, uint reqId, bool isGranted, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeContainersRespNative(auth, req, reqId, isGranted, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_containers_resp")]
    private static extern void EncodeContainersRespNative(Authenticator auth, ContainersReq req, uint reqId, [MarshalAs(UnmanagedType.Bool)] bool isGranted, IntPtr userData, FfiResultStringCb oCb);

    public static void AuthInitLogging(String outputFileNameOverride, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthInitLoggingNative(outputFileNameOverride, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_init_logging")]
    private static extern void AuthInitLoggingNative([MarshalAs(UnmanagedType.LPStr)] String outputFileNameOverride, IntPtr userData, FfiResultCb oCb);

    public static void AuthOutputLogPath(String outputFileName, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AuthOutputLogPathNative(outputFileName, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "auth_output_log_path")]
    private static extern void AuthOutputLogPathNative([MarshalAs(UnmanagedType.LPStr)] String outputFileName, IntPtr userData, FfiResultStringCb oCb);

    private delegate void FfiResultAccountInfoCb(IntPtr arg0, FfiResult arg1, AccountInfo arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultAccountInfoCb))]
    #endif
    private static void OnFfiResultAccountInfoCb(IntPtr arg0, FfiResult arg1, AccountInfo arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, AccountInfo>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfAppAccessCb(IntPtr arg0, FfiResult arg1, AppAccess[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfAppAccessCb))]
    #endif
    private static void OnFfiResultArrayOfAppAccessCb(IntPtr arg0, FfiResult arg1, AppAccess[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, AppAccess[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfAppExchangeInfoCb(IntPtr arg0, FfiResult arg1, AppExchangeInfo[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfAppExchangeInfoCb))]
    #endif
    private static void OnFfiResultArrayOfAppExchangeInfoCb(IntPtr arg0, FfiResult arg1, AppExchangeInfo[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, AppExchangeInfo[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfRegisteredAppCb(IntPtr arg0, FfiResult arg1, RegisteredApp[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfRegisteredAppCb))]
    #endif
    private static void OnFfiResultArrayOfRegisteredAppCb(IntPtr arg0, FfiResult arg1, RegisteredApp[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, RegisteredApp[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultCb(IntPtr arg0, FfiResult arg1);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultCb))]
    #endif
    private static void OnFfiResultCb(IntPtr arg0, FfiResult arg1) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult>) handle.Target;
        handle.Free();
        cb(arg1);
    }

    private delegate void FfiResultStringCb(IntPtr arg0, FfiResult arg1, String arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultStringCb))]
    #endif
    private static void OnFfiResultStringCb(IntPtr arg0, FfiResult arg1, String arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, String>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void NoneAndFfiResultAuthenticatorCb0(IntPtr arg0);

    #if __IOS__
    [MonoPInvokeCallback(typeof(NoneAndFfiResultAuthenticatorCb0))]
    #endif
    private static void OnNoneAndFfiResultAuthenticatorCb0(IntPtr arg0) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action, Action<FfiResult, Authenticator>>) handle.Target;
        handle.Free();
        cb.Item1();
    }

    private delegate void NoneAndFfiResultAuthenticatorCb1(IntPtr arg0, FfiResult arg1, Authenticator arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(NoneAndFfiResultAuthenticatorCb1))]
    #endif
    private static void OnNoneAndFfiResultAuthenticatorCb1(IntPtr arg0, FfiResult arg1, Authenticator arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action, Action<FfiResult, Authenticator>>) handle.Target;
        handle.Free();
        cb.Item2(arg1, arg2);
    }

    private delegate void UIntArrayOfByteAndFfiResultStringCb0(IntPtr arg0, uint arg1, byte[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntArrayOfByteAndFfiResultStringCb0))]
    #endif
    private static void OnUIntArrayOfByteAndFfiResultStringCb0(IntPtr arg0, uint arg1, byte[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, byte[]>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item1(arg1, arg2);
    }

    private delegate void UIntArrayOfByteAndFfiResultStringCb1(IntPtr arg0, FfiResult arg1, String arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntArrayOfByteAndFfiResultStringCb1))]
    #endif
    private static void OnUIntArrayOfByteAndFfiResultStringCb1(IntPtr arg0, FfiResult arg1, String arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, byte[]>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item2(arg1, arg2);
    }

    private delegate void UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb0(IntPtr arg0, uint arg1, AuthReq arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb0))]
    #endif
    private static void OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb0(IntPtr arg0, uint arg1, AuthReq arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthReq>, Action<uint, ContainersReq>, Action<uint, byte[]>, Action<uint, ShareMDataReq, MetadataResponse>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item1(arg1, arg2);
    }

    private delegate void UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb1(IntPtr arg0, uint arg1, ContainersReq arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb1))]
    #endif
    private static void OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb1(IntPtr arg0, uint arg1, ContainersReq arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthReq>, Action<uint, ContainersReq>, Action<uint, byte[]>, Action<uint, ShareMDataReq, MetadataResponse>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item2(arg1, arg2);
    }

    private delegate void UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb2(IntPtr arg0, uint arg1, byte[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb2))]
    #endif
    private static void OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb2(IntPtr arg0, uint arg1, byte[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthReq>, Action<uint, ContainersReq>, Action<uint, byte[]>, Action<uint, ShareMDataReq, MetadataResponse>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item3(arg1, arg2);
    }

    private delegate void UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb3(IntPtr arg0, uint arg1, ShareMDataReq arg2, MetadataResponse arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb3))]
    #endif
    private static void OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb3(IntPtr arg0, uint arg1, ShareMDataReq arg2, MetadataResponse arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthReq>, Action<uint, ContainersReq>, Action<uint, byte[]>, Action<uint, ShareMDataReq, MetadataResponse>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item4(arg1, arg2, arg3);
    }

    private delegate void UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb4(IntPtr arg0, FfiResult arg1, String arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb4))]
    #endif
    private static void OnUIntAuthReqAndUIntContainersReqAndUIntArrayOfByteAndUIntShareMDataReqMetadataResponseAndFfiResultStringCb4(IntPtr arg0, FfiResult arg1, String arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthReq>, Action<uint, ContainersReq>, Action<uint, byte[]>, Action<uint, ShareMDataReq, MetadataResponse>, Action<FfiResult, String>>) handle.Target;
        handle.Free();
        cb.Item5(arg1, arg2);
    }

}
