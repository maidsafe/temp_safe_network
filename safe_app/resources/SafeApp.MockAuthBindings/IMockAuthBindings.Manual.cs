using System;
using System.Collections.Generic;
using SafeApp.Utilities;

namespace SafeApp.MockAuthBindings {
  public interface IMockAuthBindings {
    IntPtr TestCreateApp();
    IntPtr TestCreateAppWithAccess(List<ContainerPermissions> accessInfo);
  }
}