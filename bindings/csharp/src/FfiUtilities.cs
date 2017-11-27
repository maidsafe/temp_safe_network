using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

public class FfiException : Exception {
    private int _code;

    public int Code {
        get { return _code; }
    }

    internal FfiException(FfiResult result)
        : base(result.error)
    {
        _code = result.errorCode;
    }
}

[StructLayout(LayoutKind.Sequential)]
internal class FfiResult {
    public int errorCode;
    [MarshalAs(UnmanagedType.LPStr)]
    public String error;
}

internal static class Utilities {
    public static (Task<T>, IntPtr) PrepareTask<T>() {
        var tcs = new TaskCompletionSource<T>();
        var userData = GCHandle.ToIntPtr(GCHandle.Alloc(tcs));

        return (tcs.Task, userData);
    }

    public static (Task, IntPtr) PrepareTask() {
        return PrepareTask<bool>();
    }

    public static void CompleteTask<T>(IntPtr userData, FfiResult result, T arg) {
        var handle = GCHandle.FromIntPtr(userData);
        var tcs = (TaskCompletionSource<T>) handle.Target;
        handle.Free();

        if (result.errorCode != 0) {
            tcs.SetException(new FfiException(result));
        } else {
            tcs.SetResult(arg);
        }
    }

    public static void CompleteTask(IntPtr userData, FfiResult result) {
        CompleteTask(userData, result, true);
    }
}
