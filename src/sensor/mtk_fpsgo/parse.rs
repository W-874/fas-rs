use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        mpsc::SyncSender,
        Arc,
    },
    thread,
    time::Instant,
};

use fas_rs_fw::prelude::*;

use super::enable_fpsgo;
use super::FPSGO;

pub(super) fn frametime_thread(
    sender: SyncSender<Vec<FrameTime>>,
    count: Arc<AtomicUsize>,
    pause: Arc<AtomicBool>,
) {
    thread::park();

    let mut buffer = Vec::with_capacity(144);

    loop {
        if pause.load(Ordering::Acquire) {
            buffer.clear();
            thread::park();
        }

        if buffer.len() >= count.load(Ordering::Acquire) {
            sender.send(buffer).unwrap();
            buffer = Vec::with_capacity(144);
        }

        let mut stamps = [0, 0];

        // 获取第一个时间戳
        let fbt_info = fs::read_to_string(Path::new(FPSGO).join("fbt/fbt_info")).unwrap();
        if let Some(stamp) = parse_frametime(&fbt_info) {
            stamps[0] = stamp
        }

        // 轮询(sysfs不可用inotify监听)
        // 值变化后保存为第二个时间戳
        loop {
            let fbt_info = fs::read_to_string(Path::new(FPSGO).join("fbt/fbt_info")).unwrap();
            if let Some(stamp) = parse_frametime(&fbt_info) {
                if stamps[0] < stamp {
                    stamps[1] = stamp;
                    break;
                }
            } else {
                enable_fpsgo().unwrap();
                break;
            }

            // 轮询间隔6ms
            thread::sleep(Duration::from_millis(6));
        }

        // 检查是否解析失败
        if stamps[0] == 0 || stamps[1] == 0 {
            continue; // 失败就重新解析
        }

        let frametime = FrameTime::from_nanos(stamps[1] - stamps[0]);
        buffer.push(frametime);
    }
}

pub(super) fn fps_thread(
    avg_fps: Arc<AtomicU32>,
    time_millis: Arc<AtomicU64>,
    pause: Arc<AtomicBool>,
) {
    thread::park();

    let mut buffer = VecDeque::with_capacity(1024);

    loop {
        if pause.load(Ordering::Acquire) {
            buffer.clear();
            thread::park();
        }

        if let Some((time, _)) = buffer.front() {
            if Instant::now() - *time > Duration::from_millis(time_millis.load(Ordering::Acquire)) {
                buffer.pop_front();
            }
        }

        let avg = buffer
            .iter()
            .map(|(_, fps)| fps)
            .sum::<Fps>()
            .checked_div(buffer.len() as u32)
            .unwrap_or(0);
        avg_fps.store(avg, Ordering::Release);

        thread::sleep(Duration::from_millis(8));

        let fpsgo_status = fs::read_to_string(Path::new(FPSGO).join("fstb/fpsgo_status")).unwrap();
        if let Some(fps) = parse_fps(&fpsgo_status) {
            buffer.push_back((Instant::now(), fps));
        } else {
            enable_fpsgo().unwrap();
            continue;
        }
    }
}

/* 解析第9行:
1(状态)	0		37	19533	0x4c2e00000021	60(屏幕刷新率)	24029340996131(最新帧的vsync时间戳) */
fn parse_frametime(fbt_info: &str) -> Option<u64> {
    let mut parse_line = fbt_info.lines().nth(8)?.split_whitespace();

    let enabled = parse_line.next()?.trim().parse::<u64>().ok()? == 1;

    // fpsgo未开启
    if !enabled {
        return None; // 需要重新读取
    }

    parse_line.nth(5)?.trim().parse().ok()
}

/* 解析需跳过第0行和最后3行，提取第3个元素
tid	bufID		name		currentFPS	targetFPS	FPS_margin	FPS_margin_GPU	FPS_margin_thrs	sbe_state	HWUI
23480	0x5b9800000038	bin.mt.plus	60		60		0		0		0		0		1
8606	0x136d0000000d	curitycenter:ui	60		60		0		0		0		0		1
fstb_self_ctrl_fps_enable:1
fstb_is_cam_active:0
dfps_ceiling:60 */
fn parse_fps(fpsgo_status: &str) -> Option<Fps> {
    let mut lines: Vec<&str> = fpsgo_status.lines().skip(1).collect();
    lines.reverse();
    let lines: Vec<&str> = lines.into_iter().skip(3).collect();

    lines
        .into_iter()
        .filter_map(|line| line.split_whitespace().nth(3)?.parse().ok())
        .max()
}

#[test]
fn test_parse() {
    let fbt_info = r"##clus	max	min	
    3	2	0	
    
    clus	num	c	r	
    0	4	-1	-1
    1	3	-1	-1
    2	1	-1	-1
    enable	idleprefer	max_blc	max_pid	max_bufID	dfps	vsync
    1	0		13	8606	0x136d00000021	120	15827015268850
    
    pid	bufid		perfidx	
    2898	0x9d700000001	0
    8606	0x136d00000021	13
    26994	0x6955000001a7	12##";
    assert_eq!(parse_frametime(fbt_info), Some(15827015268850));

    let fpsgo_status = r"##tid	bufID		name		currentFPS	targetFPS	FPS_margin	FPS_margin_GPU	FPS_margin_thrs	sbe_state	HWUI
    26994	0x69550000025d	bin.mt.plus	60		120		0		0		0		0		1
    26994	0x69550000025c	bin.mt.plus	115		120		0		0		0		0		1
    8606	0x136d0000003f	curitycenter:ui	110		120		0		0		0		0		1
    2756	0x9d9000002ce	com.miui.home	0		120		0		0		0		0		1
    2898	0x9d700000001	ndroid.systemui	-1		10		0		0		0		0		1
    24678	0x12fb00000009	m.omarea.vtools	120		10		0		0		0		0		1
    24678	0x12fb0000000b	m.omarea.vtools	0		10		0		0		0		0		1
    fstb_self_ctrl_fps_enable:1
    fstb_is_cam_active:0
    dfps_ceiling:120##";

    assert_eq!(parse_fps(fpsgo_status), Some(120));
}