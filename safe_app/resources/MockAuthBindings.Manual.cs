using System;

namespace SafeApp {
    public partial class MockAuthBindings : IMockAuthBindings {
       public IntPtr TestCreateApp() {
            var ret = TestCreateAppNative(out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }

        public IntPtr TestCreateAppWithAccess(ContainerPermissions[] accessInfo) {
            var ret = TestCreateAppWithAccessNative(accessInfo, (ulong) accessInfo.Length, out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }
    }
}