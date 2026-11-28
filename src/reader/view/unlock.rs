use super::super::{Reader, chrome::Input, tabs::Lock};
use crate::ui::{
    icon::Icon,
    kit::{Kind, RADIUS, Status, clickable, dot, icon, mono, status, text_button},
    theme::Theme,
};
use gpui::{AnyElement, Context, Div, FontWeight, IntoElement, div, prelude::*, px};
use std::path::Path;

const MASK: char = '\u{2022}';

impl Reader {
    pub(super) fn unlock(&self, lock: Lock, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.chrome.theme;
        let Some(tab) = self.tabs.current() else {
            return div().into_any_element();
        };
        let heading = |title: String, body: &str| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.ink)
                        .child(title),
                )
                .child(div().text_color(theme.muted).child(body.to_string()))
        };
        let frame = div()
            .mt(px(120.0))
            .w(px(360.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(icon(Icon::Lock, theme.muted).size(px(28.0)));
        if lock == Lock::Cancelled {
            let index = self.tabs.active;
            return frame
                .child(heading(
                    format!("{} is locked", tab.title()),
                    "Nothing from this file is shown until it is unlocked.",
                ))
                .child(div().flex().child(status(&theme, Status::Warn, "Locked")))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            text_button("unlock-ask", "Enter password", &theme, Kind::Primary)
                                .on_click(cx.listener(|this, _, _, cx| this.ask_password(cx))),
                        )
                        .child(
                            text_button("unlock-close", "Close tab", &theme, Kind::Quiet).on_click(
                                cx.listener(move |this, _, window, cx| {
                                    this.close_tab(index, window, cx)
                                }),
                            ),
                        ),
                )
                .into_any_element();
        }
        let checking = lock == Lock::Checking;
        let wrong = lock == Lock::Wrong;
        let focused = self.chrome.input == Input::Password && !checking;
        let border = if wrong {
            theme.fail
        } else if focused {
            theme.accent
        } else {
            theme.hairline
        };
        let shown = self.chrome.password_shown;
        frame
            .child(heading(
                "This PDF is password-protected".into(),
                "Enter its password to open it.",
            ))
            .child(
                mono(display_path(&tab.path))
                    .text_color(theme.faint)
                    .overflow_hidden()
                    .text_ellipsis(),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.muted)
                            .child("Password"),
                    )
                    .child(
                        clickable("unlock-field")
                            .h(px(32.0))
                            .pl(px(10.0))
                            .pr(px(4.0))
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .rounded(px(RADIUS))
                            .border_1()
                            .border_color(border)
                            .bg(theme.raised)
                            .cursor_text()
                            .on_click(cx.listener(|this, _, _, cx| this.focus_password(cx)))
                            .child(self.password_text(&theme, focused, shown))
                            .child(
                                clickable("unlock-reveal")
                                    .flex_none()
                                    .h(px(24.0))
                                    .px(px(6.0))
                                    .flex()
                                    .items_center()
                                    .rounded(px(RADIUS))
                                    .text_size(px(12.0))
                                    .text_color(theme.muted)
                                    .cursor_pointer()
                                    .hover({
                                        let hover = theme.hover;
                                        move |style| style.bg(hover)
                                    })
                                    .child(if shown { "Hide" } else { "Show" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.toggle_password_shown(cx)
                                    })),
                            ),
                    )
                    .when(wrong, |field| {
                        field.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .text_size(px(12.0))
                                .text_color(theme.fail)
                                .child(dot(theme.fail, 8.0))
                                .child("That password did not open the file. Try again."),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(if checking {
                        clickable("unlock-open")
                            .h(px(32.0))
                            .px(px(14.0))
                            .flex()
                            .flex_none()
                            .items_center()
                            .rounded(px(RADIUS))
                            .bg(theme.pressed)
                            .text_color(theme.muted)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Opening")
                    } else {
                        text_button("unlock-open", "Open", &theme, Kind::Primary)
                            .px(px(14.0))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.submit_password(window, cx)),
                            )
                    })
                    .child(
                        text_button("unlock-cancel", "Cancel", &theme, Kind::Quiet)
                            .on_click(cx.listener(|this, _, _, cx| this.cancel_password(cx))),
                    ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.faint)
                    .child("Kept in memory until you close this tab. Never saved or logged."),
            )
            .into_any_element()
    }

    fn password_text(&self, theme: &Theme, focused: bool, shown: bool) -> Div {
        let field = &self.chrome.password;
        let show = |text: &str| -> String {
            if shown {
                text.to_string()
            } else {
                std::iter::repeat_n(MASK, text.chars().count()).collect()
            }
        };
        let caret = || div().w(px(1.5)).h(px(16.0)).mx(px(1.0)).bg(theme.accent);
        let text = |content: String| div().flex_none().text_color(theme.ink).child(content);
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .overflow_hidden()
            .whitespace_nowrap()
            .when(field.selected(), |line| {
                line.child(text(show(&field.text)).bg(theme.select).rounded(px(2.0)))
            })
            .when(!field.selected(), |line| {
                line.child(text(show(field.before_cursor())))
                    .when(focused, |line| line.child(caret()))
                    .child(text(show(field.after_cursor())))
            })
    }
}

fn display_path(path: &Path) -> String {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    match home
        .as_deref()
        .and_then(|home| path.strip_prefix(home).ok())
    {
        Some(inside) => format!("~/{}", inside.display()),
        None => path.display().to_string(),
    }
}
