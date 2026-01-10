//! 游戏核心模块
//! 包含游戏状态、卷轴系统、碰撞检测等

mod collision;
mod scroll;
mod states;

pub use collision::*;
pub use scroll::*;
pub use states::*;
