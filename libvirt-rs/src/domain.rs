use std::ffi::CStr;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

// use crate::virConnectGetCapabilities;
// use crate::virFreeCallback;
// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use crate::sys::sys::{
    self, virDomain, virDomainCreate, virDomainFree, virDomainGetInfo, virDomainGetName,
    virDomainInfo, virDomainPtr, virDomainReboot, virDomainShutdown,
};

#[derive(Debug)]
pub struct Domain {
    pub ptr: NonNull<virDomain>,
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub state: String,
    pub max_memory: u64,
    pub memory: u64,
    pub virt_cpu: u32,
    pub cpu_time: u64,
}

impl Domain {
    pub fn as_ptr(&self) -> virDomainPtr {
        self.ptr.as_ptr()
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name_ptr = virDomainGetName(self.ptr.as_ptr());
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        }
    }

    pub fn start(&self) -> Result<(), String> {
        unsafe {
            let ret = virDomainCreate(self.ptr.as_ptr());

            if ret != 0 {
                return Err("The domain failed to start".to_string());
            }

            return Ok(());
        }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        unsafe {
            let ret = virDomainShutdown(self.ptr.as_ptr());

            if ret != 0 {
                return Err("The domain failed to shutdown".to_string());
            }

            return Ok(());
        }
    }

    pub fn reboot(&self) -> Result<(), String> {
        unsafe {
            let ret = virDomainReboot(self.ptr.as_ptr(), 0);

            if ret != 0 {
                return Err("The domain failed to reboot".to_string());
            }

            Ok(())
        }
    }

    pub fn domain_info(&self) -> Result<DomainInfo, String> {
        unsafe {
            let mut info = MaybeUninit::<virDomainInfo>::uninit();

            let ret = virDomainGetInfo(self.ptr.as_ptr(), info.as_mut_ptr());

            if ret != 0 {
                return Err("virDomainGetInfo failed".to_string());
            }

            let info = info.assume_init();

            let state = self.domain_state(info.state);

            // let state = String::from(self.domain_state(info.state));

            Ok(DomainInfo {
                state: state.to_string(),
                max_memory: info.maxMem as u64,
                memory: info.memory as u64,
                virt_cpu: info.nrVirtCpu as u32,
                cpu_time: info.cpuTime as u64,
            })
        }
    }

    fn domain_state(&self, state: u8) -> &'static str {
        match state as u32 {
            sys::virDomainState_VIR_DOMAIN_NOSTATE => "No State",
            sys::virDomainState_VIR_DOMAIN_RUNNING => "Running",
            sys::virDomainState_VIR_DOMAIN_BLOCKED => "Blocked",
            sys::virDomainState_VIR_DOMAIN_PAUSED => "Paused",
            sys::virDomainState_VIR_DOMAIN_SHUTDOWN => "Shutdown",
            sys::virDomainState_VIR_DOMAIN_SHUTOFF => "Shutoff",
            sys::virDomainState_VIR_DOMAIN_CRASHED => "Crashed",
            sys::virDomainState_VIR_DOMAIN_PMSUSPENDED => "Suspened Power Management",
            _ => "Unknown",
        }
    }
}

impl Drop for Domain {
    fn drop(&mut self) {
        unsafe {
            // Important: Decrement the reference count of the domain object
            virDomainFree(self.ptr.as_ptr());
        }
    }
}
