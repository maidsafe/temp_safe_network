using System;

namespace SafeApp {
    public partial class MockAuthBindings {
       public IntPtr TestCreateApp() {
            var ret = TestCreateAppNative(out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }

        public IntPtr TestCreateAppWithAccess(ContainerPermissions[] accessInfo) {
            var ret = TestCreateAppWithAccessNative(accessInfo, (IntPtr) accessInfo.Length, out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }
    }
}