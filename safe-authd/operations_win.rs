// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    authd::run as authd_run,
    errors::{Error, Result},
};
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
const SERVICE_DISPLAY_NAME: &str = "SAFE Authenticator";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const SERVICE_START_TYPE: ServiceStartType = ServiceStartType::OnDemand;
const SERVICE_LAUNCH_ARGUMENT: &str = "sc-start";

define_windows_service!(ffi_authd_service, authd_service);

pub fn install_authd() -> Result<()> {
    println!("Registering SAFE Authenticator (safe-authd) as a Windows service...");
    install_authd_service()
}

pub fn uninstall_authd() -> Result<()> {
    println!("Unregistering SAFE Authenticator (safe-authd) service...");
    uninstall_authd_service()
}

pub fn start_authd(listen: &str, foreground: bool) -> Result<()> {
    println!("Starting SAFE Authenticator service (safe-authd) from command line...");
    if foreground {
        println!("Initialising SAFE Authenticator services...");
        authd_run(listen, None, None)
    } else {
        // Since the authd service runs as a system process, we need to provide
        // the user's local project directory path which is where certificates are shared through
        let cert_base_path = match directories::ProjectDirs::from("net", "maidsafe", "safe-authd") {
            Some(dirs) => dirs.config_dir().display().to_string(),
            None => return Err(Error::GeneralError(
                "Failed to obtain local project directory path where to write authd certificate to"
                    .to_string(),
            )),
        };

        // The safe_vault also stores the certificate in the user's local project directory, thus let's
        // get the path so we pass it down to the SafeAuthenticator API so it can connect to vault
        let config_dir_path = match directories::ProjectDirs::from("net", "maidsafe", "safe_vault") {
            Some(dirs) => {
                // FIXME: safe_core is appending '\config' to the path provided,
                // so we remove it from the path. It seems to be a bug in safe_core lib:
                // https://github.com/maidsafe/safe_client_libs/issues/1054
                let components = dirs.config_dir().components().collect::<Vec<_>>();
                let path: std::path::PathBuf = components[..components.len()-1].iter().collect();
                path.display().to_string()
            },
            None => return Err(Error::GeneralError("Failed to obtain local project directory path where to read safe_vault certificate from".to_string()))
        };

        let output = process::Command::new("sc")
            .args(&[
                "start",
                SERVICE_NAME,
                listen,
                &cert_base_path,
                &config_dir_path,
            ])
            .output()
            .map_err(|err| {
                Error::GeneralError(format!(
                    "Failed to execute service control manager: {}",
                    err
                ))
            })?;

        if output.status.success() {
            println!("safe-authd service started successfully!");
            Ok(())
        } else {
            match output.status.code() {
                Some(1056) => {
                    // serice control manager exit code 1056 is: An instance of the service is already running
                    Err(Error::AuthdAlreadyStarted(format!(
                        "Failed to start safe-authd service: {}",
                        String::from_utf8_lossy(&output.stdout)
                    )))
                }
                Some(_) | None => Err(Error::GeneralError(format!(
                    "Failed to start safe-authd service: {}",
                    String::from_utf8_lossy(&output.stdout)
                ))),
            }
        }
    }
}

pub fn start_authd_from_sc() -> Result<()> {
    println!("Starting SAFE Authenticator service (safe-authd) from Service Control Manager...");
    // Register generated `ffi_authd_service` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start(SERVICE_NAME, ffi_authd_service).map_err(|err| {
        Error::GeneralError(format!("Failed to start safe-authd service: {:?}", err))
    })
}

pub fn stop_authd() -> Result<()> {
    println!("Stopping SAFE Authenticator service (safe-authd)...");

    // TODO: support for stopping gracefully with invoke_stop_on_service_manager()
    let output = process::Command::new("taskkill")
        .args(&["/F", "/IM", SERVICE_BINARY_FILE_NAME])
        .output()
        .map_err(|err| {
            Error::GeneralError(format!("Failed to stop safe-authd service: {}", err))
        })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| Error::GeneralError(format!("Failed to output stdout: {}", err)))?;
        println!("safe-authd service stopped successfully!");
        Ok(())
    } else {
        Err(Error::GeneralError(format!(
            "Failed to stop safe-authd service: {}",
            String::from_utf8_lossy(&output.stdout)
        )))
    }
}

pub fn restart_authd(listen: &str, foreground: bool) -> Result<()> {
    stop_authd()?;
    start_authd(listen, foreground)?;
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

fn run_service(arguments: Vec<OsString>) -> windows_service::Result<()> {
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

    // First argument should be the endpoint listening address
    let exit_code = if arguments.len() < 2 {
        println!("Listening address not provided as first argument");
        ServiceExitCode::ServiceSpecific(100)
    } else {
        match arguments[1].clone().into_string() {
            Err(_err) => {
                println!("Invalid listening address found as first argument");
                ServiceExitCode::ServiceSpecific(101)
            }
            Ok(listen) => {
                // The second optional argument is the cert base path where to write authd certificates
                let cert_base_path = arguments[2].clone().into_string().ok();

                // The thrid optional argument is the config dir path to pass to SafeAuthenticator
                let config_dir_path = arguments[3].clone().into_string().ok();

                match authd_run(
                    &listen,
                    cert_base_path.as_ref().map(String::as_str),
                    config_dir_path.as_ref().map(String::as_str),
                ) {
                    Ok(()) => ServiceExitCode::Win32(0),
                    Err(err) => {
                        println!("{}", err);
                        ServiceExitCode::ServiceSpecific(102)
                    }
                }
            }
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

fn install_authd_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager =
        ServiceManager::local_computer(None::<&str>, manager_access).map_err(|err| {
            Error::GeneralError(format!(
                "Error when checking if safe-authd is already registered as a service: {:?}",
                err
            ))
        })?;

    // Check there if there is an authd service already registered
    if service_manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        .is_ok()
    {
        return Err(Error::GeneralError(
            "A safe-authd service is already registered, please uninstall it first".to_string(),
        ));
    }

    let service_binary_path = ::std::env::current_exe()
        .map_err(|err| {
            Error::GeneralError(format!("Failed to get safe-authd binary path: {:?}", err))
        })?
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
                    // Unexpected as we deleted it beforehand, service already exists
                    println!(
                        "Detected safe-authd ('{}') is already registered as a service",
                        service_binary_path.display()
                    );
                    Ok(())
                } else {
                    Err(Error::GeneralError(format!(
                        "Failed to register safe-authd as a service: {:?}",
                        err
                    )))
                }
            } else {
                Err(Error::GeneralError(format!(
                    "Failed to register safe-authd as a service: {:?}",
                    err
                )))
            }
        }
        Err(err) => Err(Error::GeneralError(format!(
            "Failed to register safe-authd as a service: {:?}",
            err
        ))),
    }
}

fn uninstall_authd_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager =
        ServiceManager::local_computer(None::<&str>, manager_access).map_err(|err| {
            Error::GeneralError(format!(
                "Eror when connecting to Windows service manager: {:?}",
                err
            ))
        })?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::DELETE;
    let service = service_manager
        .open_service(SERVICE_NAME, service_access)
        .map_err(|err| {
            Error::GeneralError(format!(
                "Failed when attempting to query status of safe-authd service: {:?}",
                err
            ))
        })?;

    let service_status = service.query_status().map_err(|err| {
        Error::GeneralError(format!(
            "Failed when attempting to query status of safe-authd service: {:?}",
            err
        ))
    })?;

    service.delete().map_err(|err| {
        Error::GeneralError(format!(
            "Failed when attempting to unregister safe-authd service: {:?}",
            err
        ))
    })?;

    println!("safe-authd service sucessfully unregistered!");

    if service_status.current_state != ServiceState::Stopped {
        println!("An existing safe-authd service is currently running, let's stop it...");
        stop_authd()?;
    }

    Ok(())
}

#[allow(dead_code)]
fn invoke_stop_on_service_manager() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager =
        ServiceManager::local_computer(None::<&str>, manager_access).map_err(|err| {
            Error::GeneralError(format!(
                "Eror when connecting to Windows service manager: {:?}",
                err
            ))
        })?;

    let service = service_manager
        .open_service(SERVICE_NAME, ServiceAccess::STOP)
        .map_err(|err| {
            Error::GeneralError(format!(
                "Failed when attempting to stop safe-authd service: {:?}",
                err
            ))
        })?;

    let _ = service.stop().map_err(|err| {
        Error::GeneralError(format!(
            "Failed when attempting to stop safe-authd service: {:?}",
            err
        ))
    })?;
    Ok(())
}
