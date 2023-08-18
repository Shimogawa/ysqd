use std::{
    fmt,
    sync::{atomic::AtomicBool, Arc},
    thread,
};

use scrap::{Capturer, Display, Frame};

fn err<E, T>(e: E) -> T
where
    E: fmt::Display,
{
    panic!("无法启动原神，因为 {}", e);
}

fn get_原神_install_path() -> Option<String> {
    let keys = [
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\原神",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\原神",
    ];
    for key in keys.iter() {
        let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        let subkey = match hklm.open_subkey(key) {
            Ok(subkey) => subkey,
            Err(_) => continue,
        };
        let install_path: String = subkey.get_value("InstallPath").unwrap_or_else(err);
        return Some(install_path);
    }
    None
}

fn process_frame<'a>(frame: Frame<'a>) -> bool {
    let mut white_cnt = 0;
    let total_cnt = frame.len() / 4;
    for (_, pixel) in frame.chunks_exact(4).enumerate() {
        let [b, g, r, a] = pixel else { unreachable!() };
        if *r == 255 && *g == 255 && *b == 255 && *a == 255 {
            white_cnt += 1;
        }
    }
    println!(
        "白色像素点占比: {:.2}%",
        white_cnt as f64 / total_cnt as f64 * 100.0
    );
    return white_cnt as f64 / total_cnt as f64 > 0.99;
}

fn main() {
    let display = Display::primary().unwrap_or_else(err);
    let mut capturer = Capturer::new(display).unwrap_or_else(err);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let install_path = get_原神_install_path().unwrap_or_else(|| {
        panic!("检测到你没有安装原神！请立刻前往 https://www.yuanshen.com/ 获取安装包。");
    });
    let exe_path = format!(r"{}\Genshin Impact Game\YuanShen.exe", install_path);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match capturer.frame() {
            Ok(frame) => {
                let is_all_white = process_frame(frame);
                if is_all_white {
                    println!("原神，启动！");
                    std::process::Command::new(exe_path)
                        .spawn()
                        .expect("原神，启动失败！");
                    break;
                }
            }
            Err(_) => {
                continue;
            }
        }
        thread::sleep(std::time::Duration::from_millis(1000));
    }
}
