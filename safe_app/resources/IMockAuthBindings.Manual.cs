using System;
using System.Collections.Generic;

namespace SafeApp {
  public interface IMockAuthBindings {
    IntPtr TestCreateApp();
    IntPtr TestCreateAppWithAccess(List<ContainerPermissions> accessInfo);
  }
}