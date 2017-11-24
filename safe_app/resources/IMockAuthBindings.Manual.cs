using System;

namespace SafeApp {
    public interface IMockAuthBindings {
        IntPtr TestCreateApp();
        IntPtr TestCreateAppWithAccess(ContainerPermissions[] accessInfo);
    }
}