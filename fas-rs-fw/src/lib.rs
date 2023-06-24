pub mod macros;
pub mod prelude;
mod scheduler;

use std::{error::Error, time::Duration};

pub use scheduler::Scheduler;

/// 这里的[`self::FrameTime`]可能是 `帧渲染间隔` 或 `帧渲染时间`
/// 一般来说, 后者比较难从系统获得
pub type FrameTime = Duration;
pub type TargetFps = u32;
pub type Fps = u32;

/// 帧传感器接口
/// `Frame Aware` 意为感知帧变化
/// 目前没有发现通用且高效的获取[`self::FrameTime`]方法, 需要针对不同设备实现
pub trait VirtualFrameSensor: Send {
    /// 设备是否支持此实现
    fn support() -> bool
    where
        Self: Sized;
    /// 在此实现构造函数
    /// 初始化操作(比如创建线程/任务也要在这里完成)
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    /// 获取指定数量的历史[`self::FrameTime`]
    /// 如果目前数据还没收集好就堵塞，[`self::Scheduler`]没有sleep，会马上响应
    fn frametimes(&self, target_fps: TargetFps) -> Vec<FrameTime>;
    /// 获取指定时间内的历史[`self::Fps`]的平均
    /// 不要堵塞
    fn fps(&self) -> Fps;
    /// 很多时候, 监视帧状态是开销较大的
    /// 因此[`self::Scheduler`]在每次从调度中退出后
    /// 会调用此方法关闭监视
    fn pause(&self) -> Result<(), Box<dyn Error>>;
    /// [`self::Scheduler`]在每次开始调度时调用此方法
    /// `frametime_count`是每次要求数据的量, `fps_time`是取这段时间的平均fps
    fn resume(&self, frametime_count: usize, fps_time: Duration) -> Result<(), Box<dyn Error>>;
}

/// 性能控制器接口
/// 控制设备性能状态的控制器
/// 这些实现尽量不要堵塞
pub trait VirtualPerformanceController: Send {
    /// 设备是否支持此实现
    fn support() -> bool
    where
        Self: Sized;
    /// 在此实现构造函数
    /// 因为会被[`self::support_controller`]调用创建实例
    /// 初始化操作(比如创建线程/任务也要在这里完成)
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    /// 限制一级性能
    fn limit(&self);
    /// 释放一级性能
    fn release(&self);
    /// [`self::Scheduler`]每次开始调度的时候会调用此方法初始化(插入)控制器
    fn plug_in(&self) -> Result<(), Box<dyn Error>>;
    /// 还原(拔出)控制器
    fn plug_out(&self) -> Result<(), Box<dyn Error>>;
}