using System;
using System.Collections.Generic;

namespace SafeApp {
    public partial class MockAuthBindings {
       public IntPtr TestCreateApp() {
            var ret = TestCreateAppNative(out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }

        public IntPtr TestCreateAppWithAccess(List<ContainerPermissions> accessInfo) {
            var ret = TestCreateAppWithAccessNative(accessInfo.ToArray(), (IntPtr) accessInfo.Count, out IntPtr app);
            if (ret != 0) {
                throw new InvalidOperationException();
            }

            return app;
        }
    }
}