//! 本教程将引导您创建一个简单的 “Hello World” TUI 应用程序，该应用程序在屏幕中间显示一些文本并等待用户按 q 退出。
//! 它演示了使用 Ratatui 开发的任何应用程序都需要执行的必要任务。
//! 我们假设您对终端有基本的了解，并且拥有文本编辑器或 Rust IDE。
//! 如果您没有偏好，VSCode 是一个不错的默认选择。

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{prelude::*, widgets::Paragraph};

use std::io::{stdout, Result};

fn main() -> Result<()> {
    // 首先，应用程序进入备用屏幕，这是一个辅助屏幕，允许您的应用程序呈现所需的任何内容，而不会干扰 shell 中终端应用程序的正常输出。
    stdout().execute(EnterAlternateScreen)?;

    // 接下来，应用程序启用原始模式，这会关闭终端的输入和输出处理。这使您的应用程序可以控制何时将字符打印到屏幕上。
    enable_raw_mode()?;

    // 然后应用程序创建一个后端和 Terminal ，然后清除屏幕。
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // 主程序循环。
    main_loop(&mut terminal)?;

    // 当应用程序完成时，它需要通过离开备用屏幕并禁用原始模式来恢复终端状态。
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// 主程序循环。它做两件事：
/// 1. 绘制界面
/// 2. 处理事件
fn main_loop<B>(terminal: &mut Terminal<B>) -> Result<()>
where
    B: Backend,
{
    loop {
        // `terminal` 上的 `draw` 方法是应用程序与 Ratatui 的主要交互点。
        // `draw` 方法接受带有单个 `Frame` 参数的闭包（匿名方法），并呈现整个屏幕。
        // 您的应用程序将创建一个与终端窗口全尺寸的区域，并呈现一个具有白色前景文本和蓝色背景的新段落。
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
                area,
            );
        })?;

        // Ratatui 绘制框架后，您的应用程序需要检查是否发生了任何事件。这些是键盘按下、鼠标事件、调整大小等。
        // 如果用户按下 `q` 键，应用程序应该跳出循环。
        //
        // 为事件轮询添加一个小的超时，以确保无论是否有待处理的事件，UI 都保持响应（16 毫秒约为 60 fps）。
        // 检查事件类型是否为 `Press` 很重要，否则 Windows 终端将看到每个键两次。
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
