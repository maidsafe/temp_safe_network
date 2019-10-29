// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::authd::{run as authd_run, ErrorExt};
use failure::Error;
use std::{ffi::OsString, io, io::Write, process, time::Duration};
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

const SERVICE_BINARY_FILE_NAME: &str = "safe-authd.exe";
const SERVICE_NAME: &str = "safe-authd";
const SERVICE_DISPLAY_NAME: &str = "AAASAFE Authenticator";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const SERVICE_START_TYPE: ServiceStartType = ServiceStartType::OnDemand;
const SERVICE_LAUNCH_ARGUMENT: &str = "start";

define_windows_service!(ffi_authd_service, authd_service);

pub fn install_authd() -> Result<(), Error> {
    println!("Installing SAFE Authenticator (safe-authd) as a Windows service...");
    install_authd_service()
}

pub fn uninstall_authd() -> Result<(), Error> {
    println!("Uninstalling SAFE Authenticator (safe-authd) service...");
    uninstall_authd_service()
}

pub fn start_authd(_listen: &str) -> Result<(), Error> {
    println!("Starting SAFE Authenticator service (safe-authd)...");

    // Register generated `ffi_authd_service` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start(SERVICE_NAME, ffi_authd_service)
        .map_err(|err| format_err!("Failed to start safe-authd service: {:?}", err))
}

pub fn stop_authd() -> Result<(), Error> {
    println!("Stopping SAFE Authenticator service (safe-authd)...");

    // TODO: support for stopping gracefully with invoke_stop_on_service_manager()
    let output = process::Command::new("taskkill")
        .args(&["/F", "/IM", SERVICE_BINARY_FILE_NAME])
        .output()
        .map_err(|err| format_err!("Failed to stop safe-authd process: {}", err))?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| format_err!("Failed to output stdout: {}", err))?;
        println!("safe-authd service stopped successfully!");
        Ok(())
    } else {
        io::stderr()
            .write_all(&output.stderr)
            .map_err(|err| format_err!("Failed to output stderr: {}", err))?;
        Err(format_err!("Failed to stop safe-authd process"))
    }
}

pub fn restart_authd(listen: &str) -> Result<(), Error> {
    stop_authd()?;
    start_authd(listen)?;
    println!("Success, safe-authd restarted!");
    Ok(())
}

// Private helpers

fn authd_service(arguments: Vec<OsString>) {
    // The entry point where execution will start on a background thread after a call to
    // `service_dispatcher::start` from `main`.
    if let Err(err) = run_service(arguments) {
        println!("Failed to start authd service: {}", err);
    }
}

fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                // TODO: send signal to process to stop gracefully
                match stop_authd() {
                    Ok(()) => ServiceControlHandlerResult::NoError,
                    Err(_) => ServiceControlHandlerResult::NoError,
                }
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
    })?;

    // FIXME: receive endpoint listen address from arguments
    let exit_code = match authd_run("https://localhost:33000") {
        Ok(()) => ServiceExitCode::Win32(0),
        Err(err) => {
            println!("{}", err.pretty());
            ServiceExitCode::ServiceSpecific(100)
        }
    };

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code,
        checkpoint: 0,
        wait_hint: Duration::default(),
    })?;

    Ok(())
}

fn install_authd_service() -> Result<(), Error> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager =
        ServiceManager::local_computer(None::<&str>, manager_access).map_err(|err| {
            format_err!(
                "Eror when checking if safe-authd service is installed: {:?}",
                err
            )
        })?;

    let service_binary_path = ::std::env::current_exe()
        .map_err(|err| format_err!("Failed to get safe-authd service binary path: {:?}", err))?
        .with_file_name(SERVICE_BINARY_FILE_NAME);

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: SERVICE_START_TYPE,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path.clone(),
        launch_arguments: vec![OsString::from(SERVICE_LAUNCH_ARGUMENT)],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    match service_manager.create_service(service_info, ServiceAccess::empty()) {
        Ok(_) => {
            println!(
                "The safe-authd service ('{}') was just installed sucessfully!",
                service_binary_path.display()
            );
            Ok(())
        }
        Err(windows_service::Error::Winapi(err)) => {
            if let Some(os_err) = err.raw_os_error() {
                if os_err == 1073 {
                    // service already exists
                    println!(
                        "Detected safe-authd service ('{}') is already installed",
                        service_binary_path.display()
                    );
                    Ok(())
                } else {
                    Err(format_err!(
                        "Failed to install safe-authd service: {:?}",
                        err
                    ))
                }
            } else {
                Err(format_err!(
                    "Failed to install safe-authd service: {:?}",
                    err
                ))
            }
        }
        Err(err) => Err(format_err!(
            "Failed to install safe-authd service: {:?}",
            err
        )),
    }
}

fn uninstall_authd_service() -> Result<(), Error> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .map_err(|err| format_err!("Eror when connecting to service manager: {:?}", err))?;

    let service_access = ServiceAccess::DELETE;
    let service = service_manager
        .open_service(SERVICE_NAME, service_access)
        .map_err(|err| {
            format_err!(
                "Failed when attempting to query status of safe-authd service: {:?}",
                err
            )
        })?;

    service.delete().map_err(|err| {
        format_err!(
            "Failed when attempting to delete safe-authd service: {:?}",
            err
        )
    })?;

    println!("safe-authd sucessfully uninstalled!");
    Ok(())
}

#[allow(dead_code)]
fn invoke_stop_on_service_manager() -> Result<(), Error> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .map_err(|err| format_err!("Eror when connecting to service manager: {:?}", err))?;

    let service = service_manager
        .open_service(SERVICE_NAME, ServiceAccess::STOP)
        .map_err(|err| {
            format_err!(
                "Failed when attempting to stop safe-authd service: {:?}",
                err
            )
        })?;

    let _ = service.stop().map_err(|err| {
        format_err!(
            "Failed when attempting to stop safe-authd service: {:?}",
            err
        )
    })?;
    Ok(())
}
