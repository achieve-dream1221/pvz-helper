use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use anyhow::{bail, Result};
use tracing::{error, info, instrument, trace, warn};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId};

use crate::address;
use crate::address::Address;

#[derive(Debug)]
pub struct PVZHelper {
    handle: HANDLE, // 进程句柄
    pub running: Arc<AtomicBool>, // 后台运行无cd模式
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>, // 多线程句柄
}

impl Drop for PVZHelper {
    #[instrument(skip(self))]
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle).is_ok().then(|| trace!("CloseHandle: {:?}", self.handle));
        }
    }
}

impl PVZHelper {
    pub fn new(title: &str) -> Self {
        Self::build(title).unwrap()
    }
    #[instrument(ret, err)]
    pub fn build(title: &str) -> Result<Self> {
        let handle = Self::get_handle(title)?;
        Ok(PVZHelper {
            handle,
            running: Arc::new(AtomicBool::new(false)),
            join_handle: Arc::new(Mutex::new(None)),
        })
    }

    #[instrument(skip(self), ret, name = "修改阳光")]
    pub fn modify_sun(&self, value: u32) -> bool {
        Self::write_memory(self.handle, address::SUN, value).is_ok()
    }

    #[instrument(skip(self), name = "修改银币")]
    pub fn modify_sliver_coin(&self, value: u32) -> bool {
        let ret = Self::write_memory(self.handle, address::SLIVER_COIN, value).is_ok();
        if ret {
            info!("银币修改成功");
        } else {
            warn!("银币修改失败");
        }
        ret
    }
    #[instrument(skip(self), name = "修改冷却时间")]
    pub fn modify_cd(&mut self, no_cd: bool) {
        if !no_cd {
            self.running.store(false, Ordering::Relaxed); // 更改线程运行状态
            if let Some(h) = self.join_handle.lock().unwrap().take() {
                h.join().unwrap(); // 等待关闭线程
            }
        }
        if self.join_handle.lock().unwrap().is_some() { // 确保只有一个线程运行该操作
            info!("modify_cd thread is already running...");
            return;
        }
        self.running.store(true, Ordering::Relaxed); // 更改线程运行状态
        let running = self.running.clone();
        let handle = self.handle;
        let h = thread::Builder::new().name("modify_cd".into())
            .spawn(move || {
                info!("modify_cd thread is running...");
                while running.load(Ordering::Relaxed) {
                    if let Ok(final_addr) = Self::read_memory(handle, address::CD) {
                        unsafe {
                            if let Err(e) = ReadProcessMemory(handle, final_addr as _, &final_addr as *const u32 as _, 4, None) {
                                error!("ReadProcessMemory failed: {:?}",e );
                                running.store(false,Ordering::Relaxed);
                                break;
                            }
                            for i in 0..14 {
                                if let Err(e) = WriteProcessMemory(handle, (final_addr + 0x70 + (0x50) * i) as _, &no_cd as *const bool as _, 4, None) {
                                    error!("WriteProcessMemory failed: {:?}",e );
                                    running.store(false,Ordering::Relaxed);
                                    break;
                                }
                            }
                        }
                        thread::sleep(std::time::Duration::from_millis(500));
                    } else {
                        break;
                    }
                }
                info!("modify_cd stop...");
            }).unwrap();
        self.join_handle.lock().unwrap().replace(h);
    }

    fn read_memory(handle: HANDLE, addr: Address) -> Result<u32> {
        let buf = 0u32;
        let new_addr = 0u32;
        unsafe {
            ReadProcessMemory(handle, addr.0, &buf as *const u32 as _, 4, None)?;
            ReadProcessMemory(handle, (buf + addr.1) as _, &new_addr as *const u32 as _, 4, None)?;
        }
        Ok(new_addr + addr.2)
    }

    fn write_memory(handle: HANDLE, addr: Address, value: u32) -> Result<()> {
        let final_addr = Self::read_memory(handle, addr)?;
        unsafe {
            Ok(WriteProcessMemory(handle, final_addr as _, &value as *const u32 as _, 4, None)?)
        }
    }

    fn get_handle(title: &str) -> Result<HANDLE> {
        let hwnd = unsafe { FindWindowW(None, PCWSTR::from_raw(HSTRING::from(title).as_ptr())) };
        if hwnd.0 == 0 { bail!("{} 未找到, 请先进入游戏!", title) }
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        trace!("pid: {}", pid);
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid)? };
        if handle.is_invalid() { bail!("OpenProcess failed") }
        Ok(handle)
    }
}
