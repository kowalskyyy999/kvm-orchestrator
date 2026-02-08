use std::ffi::CStr;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use crate::domain::Domain;
use crate::sys::sys::{
    virConnect, virConnectClose, virConnectGetCapabilities, virConnectListAllDomains,
    virConnectOpen, virConnectPtr, virConnectRef, virDomainDefineXML, virDomainDefineXMLFlags,
    virDomainPtr, virNodeGetInfo, virNodeInfo,
};
// use crate::virConnectGetCapabilities;
// use crate::virFreeCallback;
// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Connection {
    ptr: NonNull<virConnect>,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub model: String,
    pub memory_kb: u64,
    pub cpus: u32,
    pub mhz: u32,
    pub nodes: u32,
    pub sockets: u32,
    pub cores: u32,
    pub threads: u32,
}

impl Connection {
    pub fn open(uri: &str) -> Result<Self, ()> {
        let c_uri = CString::new(uri).unwrap();
        let ptr = unsafe { virConnectOpen(c_uri.as_ptr()) };

        NonNull::new(ptr).map(|p| Self { ptr: p }).ok_or(())
    }

    pub fn get_ptr(&self) -> NonNull<virConnect> {
        self.ptr
    }

    pub fn as_ptr(&self) -> virConnectPtr {
        self.ptr.as_ptr()
    }

    pub fn capabilities(&self) -> Result<String, String> {
        unsafe {
            let raw = virConnectGetCapabilities(self.as_ptr());

            if raw.is_null() {
                return Err("virConnectGetCapabilities return NULL".to_string());
            }

            let xml = CStr::from_ptr(raw).to_string_lossy().into_owned();

            // virFree(raw as *mut libc::c_void);

            Ok(xml)
        }
    }
    pub fn node_info(&self) -> Result<NodeInfo, String> {
        unsafe {
            let mut info = MaybeUninit::<virNodeInfo>::uninit();

            let ret = virNodeGetInfo(self.as_ptr(), info.as_mut_ptr());
            if ret != 0 {
                return Err("virNodeGetInfo failed".to_string());
            }

            let info = info.assume_init();

            // model is char[32]
            let model = CStr::from_ptr(info.model.as_ptr())
                .to_string_lossy()
                .trim()
                .to_string();

            Ok(NodeInfo {
                model,
                memory_kb: info.memory as u64,
                cpus: info.cpus as u32,
                mhz: info.mhz as u32,
                nodes: info.nodes as u32,
                sockets: info.sockets as u32,
                cores: info.cores as u32,
                threads: info.threads as u32,
            })
        }
    }

    pub fn list_all_domains(&self) -> Result<Vec<Domain>, String> {
        unsafe {
            let mut dom_ptrs: *mut virDomainPtr = std::ptr::null_mut();
            // Flags: 0 usually retrieves all domains
            let count = virConnectListAllDomains(self.as_ptr(), &mut dom_ptrs, 0);

            if count < 0 {
                return Err("virConnectListAllDomains failed".to_string());
            }

            let mut domains = Vec::with_capacity(count as usize);
            for i in 0..count {
                let dom_ptr = *dom_ptrs.add(i as usize);
                if let Some(x) = NonNull::new(dom_ptr) {
                    domains.push(Domain { ptr: x });
                }
            }

            // Free the array itself (but not the domains inside,
            // as they are now owned by our Domain struct)
            libc::free(dom_ptrs as *mut libc::c_void);

            Ok(domains)
        }
    }

    pub fn define_domain(&self, xml: &str) -> Result<Domain, ()> {
        let c_xml = CString::new(xml).map_err(|_| ())?;
        unsafe {
            let ptr = virDomainDefineXMLFlags(self.as_ptr(), c_xml.as_ptr(), 0);

            NonNull::new(ptr).map(|p| Domain { ptr: p }).ok_or(())
        }
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        let ret = unsafe { virConnectRef(self.as_ptr()) };
        if ret != 0 {
            panic!("virConnectRef failed")
        }

        Self { ptr: self.ptr }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            virConnectClose(self.as_ptr());
        }
    }
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}
