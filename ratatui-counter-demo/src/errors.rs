use std::panic;

use color_eyre::{config::HookBuilder, eyre};

use crate::tui;

/// 这将标准 color_eyre 恐慌和错误挂钩替换为在打印恐慌或错误之前恢复终端的挂钩。
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // 从 color_eyre PanicHook 转换为标准恐慌钩子
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    // 从 color_eyre EyreHook 转换为 eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            tui::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}
