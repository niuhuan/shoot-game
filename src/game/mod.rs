//! 游戏核心模块
//! 包含游戏状态、卷轴系统、碰撞检测等

mod states;
mod scroll;
mod collision;

pub use states::*;
pub use scroll::*;
pub use collision::*;
