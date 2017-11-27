using System;
using System.Runtime.InteropServices;

public enum MDataAction {
    /// Permission to insert new entries.
    Insert,
    /// Permission to update existing entries.
    Update,
    /// Permission to delete existing entries.
    Delete,
    /// Permission to manage permissions.
    ManagePermissions,
}

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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.XOR_NAME_LEN)]
    public byte[] name;
    /// Type tag of the mutable data.
    public ulong typeTag;
    /// Flag indicating whether the encryption info (`enc_key` and `enc_nonce`).
    /// is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasEncInfo;
    /// Encryption key. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Encryption nonce. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_NONCE_LEN)]
    public byte[] encNonce;
    /// Flag indicating whether the new encryption info is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasNewEncInfo;
    /// New encryption key (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_KEY_LEN)]
    public byte[] newEncKey;
    /// New encryption nonce (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_NONCE_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.XOR_NAME_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SIGN_PUBLIC_KEY_LEN)]
    public byte[] ownerKey;
    /// Data symmetric encryption key
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Asymmetric sign public key.
    ///
    /// This is the identity of the App in the Network.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SIGN_PUBLIC_KEY_LEN)]
    public byte[] signPk;
    /// Asymmetric sign private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SIGN_SECRET_KEY_LEN)]
    public byte[] signSk;
    /// Asymmetric enc public key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.ASYM_PUBLIC_KEY_LEN)]
    public byte[] encPk;
    /// Asymmetric enc private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.ASYM_SECRET_KEY_LEN)]
    public byte[] encSk;
}

[StructLayout(LayoutKind.Sequential)]
public class AccessContInfo {
    /// ID
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.XOR_NAME_LEN)]
    public byte[] id;
    /// Type tag
    public ulong tag;
    /// Nonce
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SYM_NONCE_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.SIGN_PUBLIC_KEY_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.XOR_NAME_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeApp.XOR_NAME_LEN)]
    public byte[] dataMapName;
}

[StructLayout(LayoutKind.Sequential)]
public class UserPermissionSet {
    /// User's sign key handle.
    public ulong userH;
    /// User's permission set.
    public PermissionSet permSet;
}

[StructLayout(LayoutKind.Sequential)]
public struct App {
    private IntPtr value;
}

public static class SafeApp {
    #if __IOS__
    private const String DLL_NAME = "__Internal";
    #else
    private const String DLL_NAME = "safe_app";
    #endif

    public const ulong MAIDSAFE_TAG = 5483000;
    public const ulong DIR_TAG = 15000;
    public const String SAFE_MOCK_UNLIMITED_MUTATIONS = "SAFE_MOCK_UNLIMITED_MUTATIONS";
    public const String SAFE_MOCK_IN_MEMORY_STORAGE = "SAFE_MOCK_IN_MEMORY_STORAGE";
    public const String SAFE_MOCK_VAULT_PATH = "SAFE_MOCK_VAULT_PATH";
    public const ulong NULL_OBJECT_HANDLE = 0;

    public static bool IsMockBuild() {
        return IsMockBuildNative();
    }

    [DllImport(DLL_NAME, EntryPoint = "is_mock_build")]
    private static extern bool IsMockBuildNative();

    public static void AppUnregistered(byte[] bootstrapConfig, Action oDisconnectNotifierCb, Action<FfiResult, App> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oDisconnectNotifierCb, oCb)));
        AppUnregisteredNative(bootstrapConfig, (ulong) bootstrapConfig.Length, userData, OnNoneAndFfiResultAppCb0, OnNoneAndFfiResultAppCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_unregistered")]
    private static extern void AppUnregisteredNative([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)] byte[] bootstrapConfig, ulong bootstrapConfigLen, IntPtr userData, NoneAndFfiResultAppCb0 oDisconnectNotifierCb, NoneAndFfiResultAppCb1 oCb);

    public static void AppRegistered(String appId, AuthGranted authGranted, Action oDisconnectNotifierCb, Action<FfiResult, App> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oDisconnectNotifierCb, oCb)));
        AppRegisteredNative(appId, authGranted, userData, OnNoneAndFfiResultAppCb0, OnNoneAndFfiResultAppCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_registered")]
    private static extern void AppRegisteredNative([MarshalAs(UnmanagedType.LPStr)] String appId, AuthGranted authGranted, IntPtr userData, NoneAndFfiResultAppCb0 oDisconnectNotifierCb, NoneAndFfiResultAppCb1 oCb);

    public static void AppReconnect(App app, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppReconnectNative(app, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_reconnect")]
    private static extern void AppReconnectNative(App app, IntPtr userData, FfiResultCb oCb);

    public static void AppAccountInfo(App app, Action<FfiResult, AccountInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppAccountInfoNative(app, userData, OnFfiResultAccountInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_account_info")]
    private static extern void AppAccountInfoNative(App app, IntPtr userData, FfiResultAccountInfoCb oCb);

    public static void AppExeFileStem(Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppExeFileStemNative(userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_exe_file_stem")]
    private static extern void AppExeFileStemNative(IntPtr userData, FfiResultStringCb oCb);

    public static void AppSetAdditionalSearchPath(String newPath, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppSetAdditionalSearchPathNative(newPath, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_set_additional_search_path")]
    private static extern void AppSetAdditionalSearchPathNative([MarshalAs(UnmanagedType.LPStr)] String newPath, IntPtr userData, FfiResultCb oCb);

    public static void AppFree(App app) {
        AppFreeNative(app);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_free")]
    private static extern void AppFreeNative(App app);

    public static void AppResetObjectCache(App app, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppResetObjectCacheNative(app, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_reset_object_cache")]
    private static extern void AppResetObjectCacheNative(App app, IntPtr userData, FfiResultCb oCb);

    public static void AccessContainerRefreshAccessInfo(App app, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AccessContainerRefreshAccessInfoNative(app, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "access_container_refresh_access_info")]
    private static extern void AccessContainerRefreshAccessInfoNative(App app, IntPtr userData, FfiResultCb oCb);

    public static void AccessContainerFetch(App app, Action<FfiResult, ContainerPermissions[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AccessContainerFetchNative(app, userData, OnFfiResultArrayOfContainerPermissionsCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "access_container_fetch")]
    private static extern void AccessContainerFetchNative(App app, IntPtr userData, FfiResultArrayOfContainerPermissionsCb oCb);

    public static void AccessContainerGetContainerMdataInfo(App app, String name, Action<FfiResult, MDataInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AccessContainerGetContainerMdataInfoNative(app, name, userData, OnFfiResultMDataInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "access_container_get_container_mdata_info")]
    private static extern void AccessContainerGetContainerMdataInfoNative(App app, [MarshalAs(UnmanagedType.LPStr)] String name, IntPtr userData, FfiResultMDataInfoCb oCb);

    public static void CipherOptNewPlaintext(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        CipherOptNewPlaintextNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "cipher_opt_new_plaintext")]
    private static extern void CipherOptNewPlaintextNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void CipherOptNewSymmetric(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        CipherOptNewSymmetricNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "cipher_opt_new_symmetric")]
    private static extern void CipherOptNewSymmetricNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void CipherOptNewAsymmetric(App app, ulong peerEncryptKeyH, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        CipherOptNewAsymmetricNative(app, peerEncryptKeyH, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "cipher_opt_new_asymmetric")]
    private static extern void CipherOptNewAsymmetricNative(App app, ulong peerEncryptKeyH, IntPtr userData, FfiResultULongCb oCb);

    public static void CipherOptFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        CipherOptFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "cipher_opt_free")]
    private static extern void CipherOptFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void AppPubSignKey(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppPubSignKeyNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_pub_sign_key")]
    private static extern void AppPubSignKeyNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void SignGenerateKeyPair(App app, Action<FfiResult, ulong, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignGenerateKeyPairNative(app, userData, OnFfiResultULongULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_generate_key_pair")]
    private static extern void SignGenerateKeyPairNative(App app, IntPtr userData, FfiResultULongULongCb oCb);

    public static void SignPubKeyNew(App app, IntPtr data, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignPubKeyNewNative(app, data, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_pub_key_new")]
    private static extern void SignPubKeyNewNative(App app, IntPtr data, IntPtr userData, FfiResultULongCb oCb);

    public static void SignPubKeyGet(App app, ulong handle, Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignPubKeyGetNative(app, handle, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_pub_key_get")]
    private static extern void SignPubKeyGetNative(App app, ulong handle, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void SignPubKeyFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignPubKeyFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_pub_key_free")]
    private static extern void SignPubKeyFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void SignSecKeyNew(App app, IntPtr data, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignSecKeyNewNative(app, data, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_sec_key_new")]
    private static extern void SignSecKeyNewNative(App app, IntPtr data, IntPtr userData, FfiResultULongCb oCb);

    public static void SignSecKeyGet(App app, ulong handle, Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignSecKeyGetNative(app, handle, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_sec_key_get")]
    private static extern void SignSecKeyGetNative(App app, ulong handle, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void SignSecKeyFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignSecKeyFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign_sec_key_free")]
    private static extern void SignSecKeyFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void AppPubEncKey(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppPubEncKeyNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_pub_enc_key")]
    private static extern void AppPubEncKeyNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void EncGenerateKeyPair(App app, Action<FfiResult, ulong, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncGenerateKeyPairNative(app, userData, OnFfiResultULongULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_generate_key_pair")]
    private static extern void EncGenerateKeyPairNative(App app, IntPtr userData, FfiResultULongULongCb oCb);

    public static void EncPubKeyNew(App app, IntPtr data, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncPubKeyNewNative(app, data, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_pub_key_new")]
    private static extern void EncPubKeyNewNative(App app, IntPtr data, IntPtr userData, FfiResultULongCb oCb);

    public static void EncPubKeyGet(App app, ulong handle, Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncPubKeyGetNative(app, handle, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_pub_key_get")]
    private static extern void EncPubKeyGetNative(App app, ulong handle, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void EncPubKeyFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncPubKeyFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_pub_key_free")]
    private static extern void EncPubKeyFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void EncSecretKeyNew(App app, IntPtr data, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncSecretKeyNewNative(app, data, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_secret_key_new")]
    private static extern void EncSecretKeyNewNative(App app, IntPtr data, IntPtr userData, FfiResultULongCb oCb);

    public static void EncSecretKeyGet(App app, ulong handle, Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncSecretKeyGetNative(app, handle, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_secret_key_get")]
    private static extern void EncSecretKeyGetNative(App app, ulong handle, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void EncSecretKeyFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncSecretKeyFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "enc_secret_key_free")]
    private static extern void EncSecretKeyFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void Sign(App app, byte[] data, ulong signSkH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        SignNative(app, data, (ulong) data.Length, signSkH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sign")]
    private static extern void SignNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] data, ulong dataLen, ulong signSkH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void Verify(App app, byte[] signedData, ulong signPkH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        VerifyNative(app, signedData, (ulong) signedData.Length, signPkH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "verify")]
    private static extern void VerifyNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] signedData, ulong signedDataLen, ulong signPkH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void Encrypt(App app, byte[] data, ulong pkH, ulong skH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncryptNative(app, data, (ulong) data.Length, pkH, skH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encrypt")]
    private static extern void EncryptNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] data, ulong dataLen, ulong pkH, ulong skH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void Decrypt(App app, byte[] data, ulong pkH, ulong skH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DecryptNative(app, data, (ulong) data.Length, pkH, skH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "decrypt")]
    private static extern void DecryptNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] data, ulong dataLen, ulong pkH, ulong skH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void EncryptSealedBox(App app, byte[] data, ulong pkH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncryptSealedBoxNative(app, data, (ulong) data.Length, pkH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encrypt_sealed_box")]
    private static extern void EncryptSealedBoxNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] data, ulong dataLen, ulong pkH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void DecryptSealedBox(App app, byte[] data, ulong pkH, ulong skH, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DecryptSealedBoxNative(app, data, (ulong) data.Length, pkH, skH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "decrypt_sealed_box")]
    private static extern void DecryptSealedBoxNative(App app, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] data, ulong dataLen, ulong pkH, ulong skH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void Sha3Hash(byte[] data, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        Sha3HashNative(data, (ulong) data.Length, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "sha3_hash")]
    private static extern void Sha3HashNative([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)] byte[] data, ulong dataLen, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void GenerateNonce(Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        GenerateNonceNative(userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "generate_nonce")]
    private static extern void GenerateNonceNative(IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void IdataNewSelfEncryptor(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataNewSelfEncryptorNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_new_self_encryptor")]
    private static extern void IdataNewSelfEncryptorNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void IdataWriteToSelfEncryptor(App app, ulong seH, IntPtr data, ulong size, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataWriteToSelfEncryptorNative(app, seH, data, size, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_write_to_self_encryptor")]
    private static extern void IdataWriteToSelfEncryptorNative(App app, ulong seH, IntPtr data, ulong size, IntPtr userData, FfiResultCb oCb);

    public static void IdataCloseSelfEncryptor(App app, ulong seH, ulong cipherOptH, Action<FfiResult, IntPtr> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataCloseSelfEncryptorNative(app, seH, cipherOptH, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_close_self_encryptor")]
    private static extern void IdataCloseSelfEncryptorNative(App app, ulong seH, ulong cipherOptH, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void IdataFetchSelfEncryptor(App app, IntPtr name, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataFetchSelfEncryptorNative(app, name, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_fetch_self_encryptor")]
    private static extern void IdataFetchSelfEncryptorNative(App app, IntPtr name, IntPtr userData, FfiResultULongCb oCb);

    public static void IdataSerialisedSize(App app, IntPtr name, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataSerialisedSizeNative(app, name, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_serialised_size")]
    private static extern void IdataSerialisedSizeNative(App app, IntPtr name, IntPtr userData, FfiResultULongCb oCb);

    public static void IdataSize(App app, ulong seH, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataSizeNative(app, seH, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_size")]
    private static extern void IdataSizeNative(App app, ulong seH, IntPtr userData, FfiResultULongCb oCb);

    public static void IdataReadFromSelfEncryptor(App app, ulong seH, ulong fromPos, ulong len, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataReadFromSelfEncryptorNative(app, seH, fromPos, len, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_read_from_self_encryptor")]
    private static extern void IdataReadFromSelfEncryptorNative(App app, ulong seH, ulong fromPos, ulong len, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void IdataSelfEncryptorWriterFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataSelfEncryptorWriterFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_self_encryptor_writer_free")]
    private static extern void IdataSelfEncryptorWriterFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void IdataSelfEncryptorReaderFree(App app, ulong handle, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        IdataSelfEncryptorReaderFreeNative(app, handle, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "idata_self_encryptor_reader_free")]
    private static extern void IdataSelfEncryptorReaderFreeNative(App app, ulong handle, IntPtr userData, FfiResultCb oCb);

    public static void EncodeAuthReq(AuthReq req, Action<FfiResult, uint, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeAuthReqNative(req, userData, OnFfiResultUIntStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_auth_req")]
    private static extern void EncodeAuthReqNative(AuthReq req, IntPtr userData, FfiResultUIntStringCb oCb);

    public static void EncodeContainersReq(ContainersReq req, Action<FfiResult, uint, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeContainersReqNative(req, userData, OnFfiResultUIntStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_containers_req")]
    private static extern void EncodeContainersReqNative(ContainersReq req, IntPtr userData, FfiResultUIntStringCb oCb);

    public static void EncodeUnregisteredReq(byte[] extraData, Action<FfiResult, uint, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeUnregisteredReqNative(extraData, (ulong) extraData.Length, userData, OnFfiResultUIntStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_unregistered_req")]
    private static extern void EncodeUnregisteredReqNative([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)] byte[] extraData, ulong extraDataLen, IntPtr userData, FfiResultUIntStringCb oCb);

    public static void EncodeShareMdataReq(ShareMDataReq req, Action<FfiResult, uint, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        EncodeShareMdataReqNative(req, userData, OnFfiResultUIntStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "encode_share_mdata_req")]
    private static extern void EncodeShareMdataReqNative(ShareMDataReq req, IntPtr userData, FfiResultUIntStringCb oCb);

    public static void DecodeIpcMsg(String msg, Action<uint, AuthGranted> oAuth, Action<uint, byte[]> oUnregistered, Action<uint> oContainers, Action<uint> oShareMdata, Action oRevoked, Action<FfiResult, uint> oErr) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oAuth, oUnregistered, oContainers, oShareMdata, oRevoked, oErr)));
        DecodeIpcMsgNative(msg, userData, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb0, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb1, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb2, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb3, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb4, OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb5);
    }

    [DllImport(DLL_NAME, EntryPoint = "decode_ipc_msg")]
    private static extern void DecodeIpcMsgNative([MarshalAs(UnmanagedType.LPStr)] String msg, IntPtr userData, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb0 oAuth, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb1 oUnregistered, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb2 oContainers, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb3 oShareMdata, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb4 oRevoked, UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb5 oErr);

    public static void AppInitLogging(String outputFileNameOverride, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppInitLoggingNative(outputFileNameOverride, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_init_logging")]
    private static extern void AppInitLoggingNative([MarshalAs(UnmanagedType.LPStr)] String outputFileNameOverride, IntPtr userData, FfiResultCb oCb);

    public static void AppOutputLogPath(String outputFileName, Action<FfiResult, String> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        AppOutputLogPathNative(outputFileName, userData, OnFfiResultStringCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "app_output_log_path")]
    private static extern void AppOutputLogPathNative([MarshalAs(UnmanagedType.LPStr)] String outputFileName, IntPtr userData, FfiResultStringCb oCb);

    public static void MdataInfoNewPrivate(IntPtr name, ulong typeTag, IntPtr secretKey, IntPtr nonce, Action<FfiResult, MDataInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoNewPrivateNative(name, typeTag, secretKey, nonce, userData, OnFfiResultMDataInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_new_private")]
    private static extern void MdataInfoNewPrivateNative(IntPtr name, ulong typeTag, IntPtr secretKey, IntPtr nonce, IntPtr userData, FfiResultMDataInfoCb oCb);

    public static void MdataInfoRandomPublic(ulong typeTag, Action<FfiResult, MDataInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoRandomPublicNative(typeTag, userData, OnFfiResultMDataInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_random_public")]
    private static extern void MdataInfoRandomPublicNative(ulong typeTag, IntPtr userData, FfiResultMDataInfoCb oCb);

    public static void MdataInfoRandomPrivate(ulong typeTag, Action<FfiResult, MDataInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoRandomPrivateNative(typeTag, userData, OnFfiResultMDataInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_random_private")]
    private static extern void MdataInfoRandomPrivateNative(ulong typeTag, IntPtr userData, FfiResultMDataInfoCb oCb);

    public static void MdataInfoEncryptEntryKey(MDataInfo info, byte[] input, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoEncryptEntryKeyNative(info, input, (ulong) input.Length, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_encrypt_entry_key")]
    private static extern void MdataInfoEncryptEntryKeyNative(MDataInfo info, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] input, ulong inputLen, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void MdataInfoEncryptEntryValue(MDataInfo info, byte[] input, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoEncryptEntryValueNative(info, input, (ulong) input.Length, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_encrypt_entry_value")]
    private static extern void MdataInfoEncryptEntryValueNative(MDataInfo info, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] input, ulong inputLen, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void MdataInfoDecrypt(MDataInfo info, byte[] input, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoDecryptNative(info, input, (ulong) input.Length, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_decrypt")]
    private static extern void MdataInfoDecryptNative(MDataInfo info, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] byte[] input, ulong inputLen, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void MdataInfoSerialise(MDataInfo info, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoSerialiseNative(info, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_serialise")]
    private static extern void MdataInfoSerialiseNative(MDataInfo info, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void MdataInfoDeserialise(byte[] ptr, Action<FfiResult, MDataInfo> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataInfoDeserialiseNative(ptr, (ulong) ptr.Length, userData, OnFfiResultMDataInfoCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_info_deserialise")]
    private static extern void MdataInfoDeserialiseNative([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)] byte[] ptr, ulong ptrLen, IntPtr userData, FfiResultMDataInfoCb oCb);

    public static void MdataPut(App app, MDataInfo info, ulong permissionsH, ulong entriesH, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPutNative(app, info, permissionsH, entriesH, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_put")]
    private static extern void MdataPutNative(App app, MDataInfo info, ulong permissionsH, ulong entriesH, IntPtr userData, FfiResultCb oCb);

    public static void MdataGetVersion(App app, MDataInfo info, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataGetVersionNative(app, info, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_get_version")]
    private static extern void MdataGetVersionNative(App app, MDataInfo info, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataSerialisedSize(App app, MDataInfo info, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataSerialisedSizeNative(app, info, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_serialised_size")]
    private static extern void MdataSerialisedSizeNative(App app, MDataInfo info, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataGetValue(App app, MDataInfo info, byte[] key, Action<FfiResult, byte[], ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataGetValueNative(app, info, key, (ulong) key.Length, userData, OnFfiResultArrayOfByteULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_get_value")]
    private static extern void MdataGetValueNative(App app, MDataInfo info, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, IntPtr userData, FfiResultArrayOfByteULongCb oCb);

    public static void MdataListEntries(App app, MDataInfo info, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListEntriesNative(app, info, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_entries")]
    private static extern void MdataListEntriesNative(App app, MDataInfo info, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataListKeys(App app, MDataInfo info, Action<FfiResult, MDataKey[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListKeysNative(app, info, userData, OnFfiResultArrayOfMDataKeyCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_keys")]
    private static extern void MdataListKeysNative(App app, MDataInfo info, IntPtr userData, FfiResultArrayOfMDataKeyCb oCb);

    public static void MdataListValues(App app, MDataInfo info, Action<FfiResult, MDataValue[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListValuesNative(app, info, userData, OnFfiResultArrayOfMDataValueCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_values")]
    private static extern void MdataListValuesNative(App app, MDataInfo info, IntPtr userData, FfiResultArrayOfMDataValueCb oCb);

    public static void MdataMutateEntries(App app, MDataInfo info, ulong actionsH, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataMutateEntriesNative(app, info, actionsH, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_mutate_entries")]
    private static extern void MdataMutateEntriesNative(App app, MDataInfo info, ulong actionsH, IntPtr userData, FfiResultCb oCb);

    public static void MdataListPermissions(App app, MDataInfo info, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListPermissionsNative(app, info, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_permissions")]
    private static extern void MdataListPermissionsNative(App app, MDataInfo info, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataListUserPermissions(App app, MDataInfo info, ulong userH, Action<FfiResult, PermissionSet> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListUserPermissionsNative(app, info, userH, userData, OnFfiResultPermissionSetCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_user_permissions")]
    private static extern void MdataListUserPermissionsNative(App app, MDataInfo info, ulong userH, IntPtr userData, FfiResultPermissionSetCb oCb);

    public static void MdataSetUserPermissions(App app, MDataInfo info, ulong userH, PermissionSet permissionSet, ulong version, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataSetUserPermissionsNative(app, info, userH, permissionSet, version, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_set_user_permissions")]
    private static extern void MdataSetUserPermissionsNative(App app, MDataInfo info, ulong userH, PermissionSet permissionSet, ulong version, IntPtr userData, FfiResultCb oCb);

    public static void MdataDelUserPermissions(App app, MDataInfo info, ulong userH, ulong version, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataDelUserPermissionsNative(app, info, userH, version, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_del_user_permissions")]
    private static extern void MdataDelUserPermissionsNative(App app, MDataInfo info, ulong userH, ulong version, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntriesNew(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntriesNewNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_new")]
    private static extern void MdataEntriesNewNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataEntriesInsert(App app, ulong entriesH, byte[] key, byte[] value, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntriesInsertNative(app, entriesH, key, (ulong) key.Length, value, (ulong) value.Length, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_insert")]
    private static extern void MdataEntriesInsertNative(App app, ulong entriesH, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 4)] byte[] value, ulong valueLen, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntriesLen(App app, ulong entriesH, Action<FfiResult[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntriesLenNative(app, entriesH, userData, OnArrayOfFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_len")]
    private static extern void MdataEntriesLenNative(App app, ulong entriesH, IntPtr userData, ArrayOfFfiResultCb oCb);

    public static void MdataEntriesGet(App app, ulong entriesH, byte[] key, Action<FfiResult, byte[], ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntriesGetNative(app, entriesH, key, (ulong) key.Length, userData, OnFfiResultArrayOfByteULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_get")]
    private static extern void MdataEntriesGetNative(App app, ulong entriesH, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, IntPtr userData, FfiResultArrayOfByteULongCb oCb);

    public static void MdataEntriesForEach(App app, ulong entriesH, Action<byte[], byte[], ulong> oEachCb, Action<FfiResult> oDoneCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(Tuple.Create(oEachCb, oDoneCb)));
        MdataEntriesForEachNative(app, entriesH, userData, OnArrayOfByteArrayOfByteULongAndFfiResultCb0, OnArrayOfByteArrayOfByteULongAndFfiResultCb1);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_for_each")]
    private static extern void MdataEntriesForEachNative(App app, ulong entriesH, IntPtr userData, ArrayOfByteArrayOfByteULongAndFfiResultCb0 oEachCb, ArrayOfByteArrayOfByteULongAndFfiResultCb1 oDoneCb);

    public static void MdataEntriesFree(App app, ulong entriesH, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntriesFreeNative(app, entriesH, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entries_free")]
    private static extern void MdataEntriesFreeNative(App app, ulong entriesH, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntryActionsNew(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntryActionsNewNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entry_actions_new")]
    private static extern void MdataEntryActionsNewNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataEntryActionsInsert(App app, ulong actionsH, byte[] key, byte[] value, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntryActionsInsertNative(app, actionsH, key, (ulong) key.Length, value, (ulong) value.Length, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entry_actions_insert")]
    private static extern void MdataEntryActionsInsertNative(App app, ulong actionsH, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 4)] byte[] value, ulong valueLen, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntryActionsUpdate(App app, ulong actionsH, byte[] key, byte[] value, ulong entryVersion, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntryActionsUpdateNative(app, actionsH, key, (ulong) key.Length, value, (ulong) value.Length, entryVersion, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entry_actions_update")]
    private static extern void MdataEntryActionsUpdateNative(App app, ulong actionsH, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 4)] byte[] value, ulong valueLen, ulong entryVersion, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntryActionsDelete(App app, ulong actionsH, byte[] key, ulong entryVersion, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntryActionsDeleteNative(app, actionsH, key, (ulong) key.Length, entryVersion, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entry_actions_delete")]
    private static extern void MdataEntryActionsDeleteNative(App app, ulong actionsH, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 3)] byte[] key, ulong keyLen, ulong entryVersion, IntPtr userData, FfiResultCb oCb);

    public static void MdataEntryActionsFree(App app, ulong actionsH, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEntryActionsFreeNative(app, actionsH, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_entry_actions_free")]
    private static extern void MdataEntryActionsFreeNative(App app, ulong actionsH, IntPtr userData, FfiResultCb oCb);

    public static void MdataEncodeMetadata(MetadataResponse metadata, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataEncodeMetadataNative(metadata, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_encode_metadata")]
    private static extern void MdataEncodeMetadataNative(MetadataResponse metadata, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void MdataPermissionsNew(App app, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPermissionsNewNative(app, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_permissions_new")]
    private static extern void MdataPermissionsNewNative(App app, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataPermissionsLen(App app, ulong permissionsH, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPermissionsLenNative(app, permissionsH, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_permissions_len")]
    private static extern void MdataPermissionsLenNative(App app, ulong permissionsH, IntPtr userData, FfiResultULongCb oCb);

    public static void MdataPermissionsGet(App app, ulong permissionsH, ulong userH, Action<FfiResult, PermissionSet> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPermissionsGetNative(app, permissionsH, userH, userData, OnFfiResultPermissionSetCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_permissions_get")]
    private static extern void MdataPermissionsGetNative(App app, ulong permissionsH, ulong userH, IntPtr userData, FfiResultPermissionSetCb oCb);

    public static void MdataListPermissionSets(App app, ulong permissionsH, Action<FfiResult, UserPermissionSet[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataListPermissionSetsNative(app, permissionsH, userData, OnFfiResultArrayOfUserPermissionSetCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_list_permission_sets")]
    private static extern void MdataListPermissionSetsNative(App app, ulong permissionsH, IntPtr userData, FfiResultArrayOfUserPermissionSetCb oCb);

    public static void MdataPermissionsInsert(App app, ulong permissionsH, ulong userH, PermissionSet permissionSet, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPermissionsInsertNative(app, permissionsH, userH, permissionSet, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_permissions_insert")]
    private static extern void MdataPermissionsInsertNative(App app, ulong permissionsH, ulong userH, PermissionSet permissionSet, IntPtr userData, FfiResultCb oCb);

    public static void MdataPermissionsFree(App app, ulong permissionsH, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        MdataPermissionsFreeNative(app, permissionsH, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "mdata_permissions_free")]
    private static extern void MdataPermissionsFreeNative(App app, ulong permissionsH, IntPtr userData, FfiResultCb oCb);

    public static void DirFetchFile(App app, MDataInfo parentInfo, String fileName, Action<FfiResult, File, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DirFetchFileNative(app, parentInfo, fileName, userData, OnFfiResultFileULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "dir_fetch_file")]
    private static extern void DirFetchFileNative(App app, MDataInfo parentInfo, [MarshalAs(UnmanagedType.LPStr)] String fileName, IntPtr userData, FfiResultFileULongCb oCb);

    public static void DirInsertFile(App app, MDataInfo parentInfo, String fileName, File file, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DirInsertFileNative(app, parentInfo, fileName, file, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "dir_insert_file")]
    private static extern void DirInsertFileNative(App app, MDataInfo parentInfo, [MarshalAs(UnmanagedType.LPStr)] String fileName, File file, IntPtr userData, FfiResultCb oCb);

    public static void DirUpdateFile(App app, MDataInfo parentInfo, String fileName, File file, ulong version, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DirUpdateFileNative(app, parentInfo, fileName, file, version, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "dir_update_file")]
    private static extern void DirUpdateFileNative(App app, MDataInfo parentInfo, [MarshalAs(UnmanagedType.LPStr)] String fileName, File file, ulong version, IntPtr userData, FfiResultCb oCb);

    public static void DirDeleteFile(App app, MDataInfo parentInfo, String fileName, ulong version, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        DirDeleteFileNative(app, parentInfo, fileName, version, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "dir_delete_file")]
    private static extern void DirDeleteFileNative(App app, MDataInfo parentInfo, [MarshalAs(UnmanagedType.LPStr)] String fileName, ulong version, IntPtr userData, FfiResultCb oCb);

    public static void FileOpen(App app, MDataInfo parentInfo, File file, ulong openMode, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        FileOpenNative(app, parentInfo, file, openMode, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "file_open")]
    private static extern void FileOpenNative(App app, MDataInfo parentInfo, File file, ulong openMode, IntPtr userData, FfiResultULongCb oCb);

    public static void FileSize(App app, ulong fileH, Action<FfiResult, ulong> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        FileSizeNative(app, fileH, userData, OnFfiResultULongCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "file_size")]
    private static extern void FileSizeNative(App app, ulong fileH, IntPtr userData, FfiResultULongCb oCb);

    public static void FileRead(App app, ulong fileH, ulong position, ulong len, Action<FfiResult, byte[]> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        FileReadNative(app, fileH, position, len, userData, OnFfiResultArrayOfByteCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "file_read")]
    private static extern void FileReadNative(App app, ulong fileH, ulong position, ulong len, IntPtr userData, FfiResultArrayOfByteCb oCb);

    public static void FileWrite(App app, ulong fileH, IntPtr data, ulong size, Action<FfiResult> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        FileWriteNative(app, fileH, data, size, userData, OnFfiResultCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "file_write")]
    private static extern void FileWriteNative(App app, ulong fileH, IntPtr data, ulong size, IntPtr userData, FfiResultCb oCb);

    public static void FileClose(App app, ulong fileH, Action<FfiResult, File> oCb) {
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(oCb));
        FileCloseNative(app, fileH, userData, OnFfiResultFileCb);
    }

    [DllImport(DLL_NAME, EntryPoint = "file_close")]
    private static extern void FileCloseNative(App app, ulong fileH, IntPtr userData, FfiResultFileCb oCb);

    private delegate void ArrayOfByteArrayOfByteULongAndFfiResultCb0(IntPtr arg0, byte[] arg1, byte[] arg2, ulong arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(ArrayOfByteArrayOfByteULongAndFfiResultCb0))]
    #endif
    private static void OnArrayOfByteArrayOfByteULongAndFfiResultCb0(IntPtr arg0, byte[] arg1, byte[] arg2, ulong arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<byte[], byte[], ulong>, Action<FfiResult>>) handle.Target;
        handle.Free();
        cb.Item1(arg1, arg2, arg3);
    }

    private delegate void ArrayOfByteArrayOfByteULongAndFfiResultCb1(IntPtr arg0, FfiResult arg1);

    #if __IOS__
    [MonoPInvokeCallback(typeof(ArrayOfByteArrayOfByteULongAndFfiResultCb1))]
    #endif
    private static void OnArrayOfByteArrayOfByteULongAndFfiResultCb1(IntPtr arg0, FfiResult arg1) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<byte[], byte[], ulong>, Action<FfiResult>>) handle.Target;
        handle.Free();
        cb.Item2(arg1);
    }

    private delegate void ArrayOfFfiResultCb(IntPtr arg0, FfiResult[] arg1);

    #if __IOS__
    [MonoPInvokeCallback(typeof(ArrayOfFfiResultCb))]
    #endif
    private static void OnArrayOfFfiResultCb(IntPtr arg0, FfiResult[] arg1) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult[]>) handle.Target;
        handle.Free();
        cb(arg1);
    }

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

    private delegate void FfiResultArrayOfByteCb(IntPtr arg0, FfiResult arg1, IntPtr arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfByteCb))]
    #endif
    private static void OnFfiResultArrayOfByteCb(IntPtr arg0, FfiResult arg1, IntPtr arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, IntPtr>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfByteULongCb(IntPtr arg0, FfiResult arg1, byte[] arg2, ulong arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfByteULongCb))]
    #endif
    private static void OnFfiResultArrayOfByteULongCb(IntPtr arg0, FfiResult arg1, byte[] arg2, ulong arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, byte[], ulong>) handle.Target;
        handle.Free();
        cb(arg1, arg2, arg3);
    }

    private delegate void FfiResultArrayOfContainerPermissionsCb(IntPtr arg0, FfiResult arg1, ContainerPermissions[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfContainerPermissionsCb))]
    #endif
    private static void OnFfiResultArrayOfContainerPermissionsCb(IntPtr arg0, FfiResult arg1, ContainerPermissions[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, ContainerPermissions[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfMDataKeyCb(IntPtr arg0, FfiResult arg1, MDataKey[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfMDataKeyCb))]
    #endif
    private static void OnFfiResultArrayOfMDataKeyCb(IntPtr arg0, FfiResult arg1, MDataKey[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, MDataKey[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfMDataValueCb(IntPtr arg0, FfiResult arg1, MDataValue[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfMDataValueCb))]
    #endif
    private static void OnFfiResultArrayOfMDataValueCb(IntPtr arg0, FfiResult arg1, MDataValue[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, MDataValue[]>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultArrayOfUserPermissionSetCb(IntPtr arg0, FfiResult arg1, UserPermissionSet[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultArrayOfUserPermissionSetCb))]
    #endif
    private static void OnFfiResultArrayOfUserPermissionSetCb(IntPtr arg0, FfiResult arg1, UserPermissionSet[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, UserPermissionSet[]>) handle.Target;
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

    private delegate void FfiResultFileCb(IntPtr arg0, FfiResult arg1, File arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultFileCb))]
    #endif
    private static void OnFfiResultFileCb(IntPtr arg0, FfiResult arg1, File arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, File>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultFileULongCb(IntPtr arg0, FfiResult arg1, File arg2, ulong arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultFileULongCb))]
    #endif
    private static void OnFfiResultFileULongCb(IntPtr arg0, FfiResult arg1, File arg2, ulong arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, File, ulong>) handle.Target;
        handle.Free();
        cb(arg1, arg2, arg3);
    }

    private delegate void FfiResultMDataInfoCb(IntPtr arg0, FfiResult arg1, MDataInfo arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultMDataInfoCb))]
    #endif
    private static void OnFfiResultMDataInfoCb(IntPtr arg0, FfiResult arg1, MDataInfo arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, MDataInfo>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultPermissionSetCb(IntPtr arg0, FfiResult arg1, PermissionSet arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultPermissionSetCb))]
    #endif
    private static void OnFfiResultPermissionSetCb(IntPtr arg0, FfiResult arg1, PermissionSet arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, PermissionSet>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
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

    private delegate void FfiResultUIntStringCb(IntPtr arg0, FfiResult arg1, uint arg2, String arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultUIntStringCb))]
    #endif
    private static void OnFfiResultUIntStringCb(IntPtr arg0, FfiResult arg1, uint arg2, String arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, uint, String>) handle.Target;
        handle.Free();
        cb(arg1, arg2, arg3);
    }

    private delegate void FfiResultULongCb(IntPtr arg0, FfiResult arg1, ulong arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultULongCb))]
    #endif
    private static void OnFfiResultULongCb(IntPtr arg0, FfiResult arg1, ulong arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, ulong>) handle.Target;
        handle.Free();
        cb(arg1, arg2);
    }

    private delegate void FfiResultULongULongCb(IntPtr arg0, FfiResult arg1, ulong arg2, ulong arg3);

    #if __IOS__
    [MonoPInvokeCallback(typeof(FfiResultULongULongCb))]
    #endif
    private static void OnFfiResultULongULongCb(IntPtr arg0, FfiResult arg1, ulong arg2, ulong arg3) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Action<FfiResult, ulong, ulong>) handle.Target;
        handle.Free();
        cb(arg1, arg2, arg3);
    }

    private delegate void NoneAndFfiResultAppCb0(IntPtr arg0);

    #if __IOS__
    [MonoPInvokeCallback(typeof(NoneAndFfiResultAppCb0))]
    #endif
    private static void OnNoneAndFfiResultAppCb0(IntPtr arg0) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action, Action<FfiResult, App>>) handle.Target;
        handle.Free();
        cb.Item1();
    }

    private delegate void NoneAndFfiResultAppCb1(IntPtr arg0, FfiResult arg1, App arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(NoneAndFfiResultAppCb1))]
    #endif
    private static void OnNoneAndFfiResultAppCb1(IntPtr arg0, FfiResult arg1, App arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action, Action<FfiResult, App>>) handle.Target;
        handle.Free();
        cb.Item2(arg1, arg2);
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb0(IntPtr arg0, uint arg1, AuthGranted arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb0))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb0(IntPtr arg0, uint arg1, AuthGranted arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item1(arg1, arg2);
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb1(IntPtr arg0, uint arg1, byte[] arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb1))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb1(IntPtr arg0, uint arg1, byte[] arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item2(arg1, arg2);
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb2(IntPtr arg0, uint arg1);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb2))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb2(IntPtr arg0, uint arg1) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item3(arg1);
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb3(IntPtr arg0, uint arg1);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb3))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb3(IntPtr arg0, uint arg1) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item4(arg1);
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb4(IntPtr arg0);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb4))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb4(IntPtr arg0) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item5();
    }

    private delegate void UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb5(IntPtr arg0, FfiResult arg1, uint arg2);

    #if __IOS__
    [MonoPInvokeCallback(typeof(UIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb5))]
    #endif
    private static void OnUIntAuthGrantedAndUIntArrayOfByteAndUIntAndUIntAndNoneAndFfiResultUIntCb5(IntPtr arg0, FfiResult arg1, uint arg2) {
        var handle = GCHandle.FromIntPtr(arg0);
        var cb = (Tuple<Action<uint, AuthGranted>, Action<uint, byte[]>, Action<uint>, Action<uint>, Action, Action<FfiResult, uint>>) handle.Target;
        handle.Free();
        cb.Item6(arg1, arg2);
    }

}
