use std::sync::atomic::Ordering;

use tracing_subscriber::fmt::time::LocalTime;

mod address;
mod pvz_helper;

fn main() {
    tracing_subscriber::fmt().with_thread_names(true)
        .with_max_level(tracing::Level::TRACE).with_timer(LocalTime::rfc_3339()).init();
    let title = env!("PROGRAM_TITLE");
    let mut helper = pvz_helper::PVZHelper::new(title);
    helper.modify_sun(9999);
    helper.modify_sliver_coin(123);
    helper.modify_cd(true);
    while helper.running.load(Ordering::Relaxed) { // 阻塞程序,防止主线程退出
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
