//! 大多数 Ratatui 应用程序中的常见模式是：
//! 1. 初始化终端
//! 2. 循环运行应用程序，直到用户退出应用程序
//! 3. 将终端恢复到原始状态

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

mod errors;
mod tui;

/// `main` 函数通过调用 `tui` 模块（接下来定义）中的方法来设置终端，然后创建并运行应用程序（稍后定义）。
/// 它推迟评估调用 `App::run()` 的结果，直到终端恢复后，以确保在应用程序退出后将任何 `Error` 结果显示给用户。
fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    App::default().run(&mut terminal)?;
    Ok(())
}

/// 调用 `App::default()` 将创建一个 `App` ，其初始化为 `counter` 设置为 0， `exit` 设置为 false 。
#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

/// 大多数应用程序都有一个主循环，一直运行到用户选择退出为止。
/// 循环的每次迭代都会通过调用 `Terminal::draw()` 绘制单个帧，然后更新应用程序的状态。
///
/// 使用新的 run 方法为 App 创建一个 impl 块，该方法将充当应用程序的主循环。
impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }

        Ok(())
    }

    /// 为了呈现 UI，应用程序使用接受 `Frame` 的闭包调用 `Terminal::draw()` 。
    /// `Frame` 上最重要的方法是 `render_widget()` ，它呈现实现 `Widget` 特征的任何类型，
    /// 例如 `Paragraph` 、 `List` 结构实现 `Widget` 特征，以便将与渲染相关的代码组织在一个地方。
    /// 这允许我们调用 `Frame::render_widget()` 并将闭包中的应用程序传递给 `Terminal::draw` 。
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<()> {
        // event::read 函数会阻塞，直到发生事件为止。
        // 如果您的应用程序需要执行 UI 之外的其他任务，那么它应该通过调用 event::poll 来检查是否存在待处理事件，
        // 并设置适合您的应用程序的合理超时时间。有关此内容的更多信息将在以后的章节中介绍。
        match event::read()? {
            // 检查该事件是否为按键事件非常重要，因为 crossterm 还会在 Windows 上发出按键释放和重复事件。
            // 检查它是否等于 KeyEventKind::Press 非常重要，否则您的应用程序可能会看到重复的事件（按键按下、按键重复和按键向上）。
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    /// 用于处理按键事件。
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // KeyCode 表示按下了哪个特定键。
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter()?,
            KeyCode::Right => self.increment_counter()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) -> Result<()> {
        self.counter += 1;
        if self.counter > 2 {
            bail!("counter overflow");
        }
        Ok(())
    }

    fn decrement_counter(&mut self) -> Result<()> {
        self.counter -= 1;
        Ok(())
    }
}

/// 首先，添加一个新的 `impl Widget for &App` 块。
/// 我们在对 App 类型的引用上实现这一点，因为渲染函数不会改变任何状态，并且我们希望能够在调用绘图后使用该应用程序。
///
/// 渲染函数将创建一个带有标题、底部说明文本和一些边框的块。
/// 使用块内的应用程序状态（ `App` 计数器字段的值）渲染 `Paragraph` 小部件。
/// 块和段落将占据小部件的整个大小。
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // 注意 ratatui 还有一个 assert_buffer_eq！可用于比较缓冲区并以更易读的方式显示差异的宏。
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert_eq!(app.exit, true);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn handle_key_event_panic() {
        let mut app = App::default();
        let _ = app.handle_key_event(KeyCode::Left.into());
    }

    #[test]
    fn handle_key_event_overflow() {
        let mut app = App::default();
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert_eq!(
            app.handle_key_event(KeyCode::Right.into())
                .unwrap_err()
                .to_string(),
            "counter overflow"
        );
    }
}
