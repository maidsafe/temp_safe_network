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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.XOR_NAME_LEN)]
    public byte[] name;
    /// Type tag of the mutable data.
    public ulong typeTag;
    /// Flag indicating whether the encryption info (`enc_key` and `enc_nonce`).
    /// is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasEncInfo;
    /// Encryption key. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Encryption nonce. Meaningful only if `has_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_NONCE_LEN)]
    public byte[] encNonce;
    /// Flag indicating whether the new encryption info is set.
    [MarshalAs(UnmanagedType.Bool)]
    public bool hasNewEncInfo;
    /// New encryption key (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_KEY_LEN)]
    public byte[] newEncKey;
    /// New encryption nonce (used for two-phase reencryption). Meaningful only if
    /// `has_new_enc_info` is `true`.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_NONCE_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.XOR_NAME_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SIGN_PUBLIC_KEY_LEN)]
    public byte[] ownerKey;
    /// Data symmetric encryption key
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_KEY_LEN)]
    public byte[] encKey;
    /// Asymmetric sign public key.
    ///
    /// This is the identity of the App in the Network.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SIGN_PUBLIC_KEY_LEN)]
    public byte[] signPk;
    /// Asymmetric sign private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SIGN_SECRET_KEY_LEN)]
    public byte[] signSk;
    /// Asymmetric enc public key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.ASYM_PUBLIC_KEY_LEN)]
    public byte[] encPk;
    /// Asymmetric enc private key.
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.ASYM_SECRET_KEY_LEN)]
    public byte[] encSk;
}

[StructLayout(LayoutKind.Sequential)]
public class AccessContInfo {
    /// ID
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.XOR_NAME_LEN)]
    public byte[] id;
    /// Type tag
    public ulong tag;
    /// Nonce
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SYM_NONCE_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.SIGN_PUBLIC_KEY_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.XOR_NAME_LEN)]
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
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = SafeCore.XOR_NAME_LEN)]
    public byte[] dataMapName;
}

public static class SafeCore {
    #if __IOS__
    private const String DLL_NAME = "__Internal";
    #else
    private const String DLL_NAME = "safe_core";
    #endif

    #region custom declarations
    public const ulong ASYM_PUBLIC_KEY_LEN = 32;
    public const ulong ASYM_SECRET_KEY_LEN = 32;
    public const ulong ASYM_NONCE_LEN = 24;
    public const ulong SYM_KEY_LEN = 32;
    public const ulong SYM_NONCE_LEN = 24;
    public const ulong SIGN_PUBLIC_KEY_LEN = 32;
    public const ulong SIGN_SECRET_KEY_LEN = 64;
    public const ulong XOR_NAME_LEN = 32;
    #endregion

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

}
