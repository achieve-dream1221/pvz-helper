use anyhow::{bail, Result};
use tracing::{instrument, trace};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId};
use crate::address;

use crate::address::{Addr, Address};

pub struct PVZHelper<'a> {
    title: &'a str,
    handle: HANDLE,
}

impl<'a> PVZHelper<'a> {
    pub fn new(title: &'a str) -> Result<Self> {
        let handle = Self::get_handle(title)?;
        Ok(PVZHelper {
            title,
            handle,
        })
    }
    
    pub fn change_sun(&self, value: u32){
        Self::write_memory(self.handle, address::SUN, value);
    }
    #[instrument(ret(level = tracing::Level::TRACE), err)]
    pub fn read_memory(handle: HANDLE, addr: Address) -> anyhow::Result<Addr> {
        let mut buf = [0u8; 4];
        let mut v: u32;
        let mut addr_first_value = [0u8; 4];
        unsafe {
            ReadProcessMemory(handle, addr.0, buf.as_mut_ptr().cast(), 4, None)?;
            v = std::mem::transmute(buf); // [u8;] => u32
            ReadProcessMemory(handle, (v + addr.1) as _, addr_first_value.as_mut_ptr().cast(), 4, None)?;
            // v = std::mem::transmute(addr_first_value);
            // ReadProcessMemory(handle, (v + addr.2) as _, buf.as_mut_ptr().cast(), 4, None)?;
        }
        unsafe {
            v = std::mem::transmute(addr_first_value);
        }
        Ok((v + addr.2) as _)
    }

    #[instrument]
    pub fn write_memory(handle: HANDLE, addr: Address, value: u32) -> Result<()> {
        let final_addr = Self::read_memory(handle, addr)?;
        unsafe {
            Ok(WriteProcessMemory(handle, final_addr, &value as *const u32 as _, 4, None)?)
        }
    }

    #[instrument(ret, err)]
    pub fn get_handle(title: &str) -> anyhow::Result<HANDLE> {
        let hwnd = unsafe { FindWindowW(None, PCWSTR::from_raw(HSTRING::from(title).as_ptr())) };
        if hwnd.0 == 0 {
            bail!("{} not found", title)
        }
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        trace!("pid: {}", pid);
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid)? };
        Ok(handle)
    }
}



