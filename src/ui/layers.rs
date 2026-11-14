/// Global UI stack tiers shared by screens.
pub const GLOBAL_BACKGROUND: i32 = -1;
pub const GLOBAL_SCREEN_CONTENT: i32 = 0;
pub const GLOBAL_OVERLAY: i32 = 100;

#[cfg(not(feature = "demo"))]
pub const GLOBAL_DIALOG: i32 = 50;
