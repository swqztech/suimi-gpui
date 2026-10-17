use crate::generate::{
    Mode, PasswordOptions, Strength, generate_nickname, generate_password, generate_passwords,
    generate_username, password_strength,
};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ThemeMode, Theme};
use gpui_kit::prelude::*;
use gpui_kit::{ClipboardItem, Context, FontWeight, Render, SharedString, Window, div, px, relative, rgb};

const BG: u32 = 0x1a262a;
const PANEL: u32 = 0x121c20;
const GOLD: u32 = 0xd4af5f;
const MUTED: u32 = 0x78878e;
const BORDER: u32 = 0x37484e;
const GREEN: u32 = 0x48c47a;

pub struct SuimiApp {
    mode: Mode,
    password_opts: PasswordOptions,
    username_length: u32,
    output: SharedString,
    strength: Strength,
}

impl SuimiApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let password_opts = PasswordOptions::default();
        let strength = password_strength(&password_opts);
        let output = generate_password(&password_opts).into();
        Self {
            mode: Mode::Password,
            password_opts,
            username_length: 12,
            output,
            strength,
        }
    }

    fn mode_tab(&self) -> usize {
        match self.mode {
            Mode::Password => 0,
            Mode::Username => 1,
            Mode::Nickname => 2,
        }
    }

    fn set_mode(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.mode = match tab {
            0 => Mode::Password,
            1 => Mode::Username,
            _ => Mode::Nickname,
        };
        self.roll_once(cx);
    }

    fn roll_once(&mut self, cx: &mut Context<Self>) {
        self.output = match self.mode {
            Mode::Password => generate_password(&self.password_opts),
            Mode::Username => generate_username(self.username_length),
            Mode::Nickname => generate_nickname(),
        }
        .into();
        if self.mode == Mode::Password {
            self.strength = password_strength(&self.password_opts);
        }
        cx.notify();
    }

    fn roll_six(&mut self, cx: &mut Context<Self>) {
        self.output = match self.mode {
            Mode::Password => generate_passwords(&self.password_opts, 6),
            Mode::Username => (0..6)
                .map(|_| generate_username(self.username_length))
                .collect::<Vec<_>>()
                .join("\n"),
            Mode::Nickname => (0..6)
                .map(|_| generate_nickname())
                .collect::<Vec<_>>()
                .join("\n"),
        }
        .into();
        if self.mode == Mode::Password {
            self.strength = password_strength(&self.password_opts);
        }
        cx.notify();
    }

    fn refresh_strength(&mut self) {
        if self.mode == Mode::Password {
            self.strength = password_strength(&self.password_opts);
        }
    }

    fn bump_length(&mut self, delta: i32, cx: &mut Context<Self>) {
        if self.mode == Mode::Password {
            let next = (self.password_opts.length as i32 + delta).clamp(6, 64) as u32;
            self.password_opts.length = next;
            self.refresh_strength();
        } else if self.mode == Mode::Username {
            let next = (self.username_length as i32 + delta).clamp(4, 32) as u32;
            self.username_length = next;
        }
        cx.notify();
    }

    fn current_length(&self) -> u32 {
        if self.mode == Mode::Password {
            self.password_opts.length
        } else {
            self.username_length
        }
    }
}

impl Render for SuimiApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strength_line = format!(
            "{} · 约 {:.0} bit",
            self.strength.label,
            self.strength.bits
        );
        let length = self.current_length();

        div()
            .size_full()
            .bg(rgb(BG))
            .flex()
            .flex_col()
            .p_5()
            .gap_4()
            .overflow_y_scrollbar()
            .child(header())
            .child(
                TabBar::new("mode-tabs")
                    .selected_index(self.mode_tab())
                    .children([
                        Tab::new().label("密码"),
                        Tab::new().label("用户名"),
                        Tab::new().label("网名"),
                    ])
                    .on_click(cx.listener(|this, index, _, cx| {
                        if this.mode_tab() != *index {
                            this.set_mode(*index, cx);
                        }
                    })),
            )
            .child(output_panel(&self.output))
            .when(self.mode == Mode::Password, |this| {
                this.child(strength_bar(self.strength.ratio, strength_line))
            })
            .when(
                self.mode == Mode::Password || self.mode == Mode::Username,
                |this| this.child(length_row(cx, length)),
            )
            .when(self.mode == Mode::Password, |this| {
                this.child(password_toggles(cx, &self.password_opts))
            })
            .when(self.mode == Mode::Username, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child("以小写字母开头，可含数字与下划线"),
                )
            })
            .when(self.mode == Mode::Nickname, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child("从内置词库随机组合中文网名，可带数字后缀"),
                )
            })
            .child(action_row(cx))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .text_center()
                    .child("Passroll · 字符从勾选集合中均匀抽取"),
            )
    }
}

fn header() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_3xl()
                .text_color(rgb(GOLD))
                .font_weight(FontWeight::BOLD)
                .child("随密"),
        )
        .child(div().text_sm().text_color(rgb(0xd2dade)).child("掷一次，拿一把新钥匙"))
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("本地生成 · 不上传 · 系统安全随机源"),
        )
}

fn output_panel(text: &SharedString) -> impl IntoElement {
    let display: SharedString = if text.is_empty() {
        "—".into()
    } else {
        text.clone()
    };
    div()
        .p_4()
        .rounded_lg()
        .bg(rgb(PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .text_sm()
        .text_color(rgb(0xe6ecf0))
        .child(display)
}

fn strength_bar(ratio: f32, label: String) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .flex_1()
                .h(px(6.))
                .rounded_full()
                .bg(rgb(0x233033))
                .child(
                    div()
                        .h_full()
                        .w(relative(ratio.max(0.02)))
                        .rounded_full()
                        .bg(rgb(GREEN)),
                ),
        )
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
}

fn length_row(cx: &mut Context<SuimiApp>, length: u32) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(div().text_sm().text_color(rgb(MUTED)).child("长度"))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("len-dec")
                        .outline()
                        .label("−")
                        .on_click(cx.listener(|this, _, _, cx| this.bump_length(-1, cx))),
                )
                .child(
                    div()
                        .w(px(28.))
                        .text_center()
                        .text_sm()
                        .text_color(rgb(GOLD))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("{}", length)),
                )
                .child(
                    Button::new("len-inc")
                        .outline()
                        .label("+")
                        .on_click(cx.listener(|this, _, _, cx| this.bump_length(1, cx))),
                ),
        )
}

fn password_toggles(cx: &mut Context<SuimiApp>, opts: &PasswordOptions) -> impl IntoElement {
    let opts = opts.clone();
    div()
        .flex()
        .flex_wrap()
        .gap_3()
        .child(toggle_switch(
            cx,
            "opt-lower",
            "小写 a-z",
            opts.lowercase,
            |this, v| this.password_opts.lowercase = v,
        ))
        .child(toggle_switch(
            cx,
            "opt-upper",
            "大写 A-Z",
            opts.uppercase,
            |this, v| this.password_opts.uppercase = v,
        ))
        .child(toggle_switch(
            cx,
            "opt-digit",
            "数字 0-9",
            opts.digits,
            |this, v| this.password_opts.digits = v,
        ))
        .child(toggle_switch(
            cx,
            "opt-sym",
            "符号 !@#",
            opts.symbols,
            |this, v| this.password_opts.symbols = v,
        ))
        .child(toggle_switch(
            cx,
            "opt-ambig",
            "排除易混 0OIl1",
            opts.exclude_ambiguous,
            |this, v| this.password_opts.exclude_ambiguous = v,
        ))
}

fn toggle_switch(
    cx: &mut Context<SuimiApp>,
    id: &'static str,
    label: &'static str,
    checked: bool,
    set: fn(&mut SuimiApp, bool),
) -> impl IntoElement {
    Switch::new(id)
        .label(label)
        .checked(checked)
        .on_change(cx.listener(move |this, value, _, cx| {
            set(this, *value);
            this.refresh_strength();
            cx.notify();
        }))
}

fn action_row(cx: &mut Context<SuimiApp>) -> impl IntoElement {
    div()
        .flex()
        .gap_3()
        .child(
            Button::new("roll")
                .primary()
                .label("再掷一次")
                .on_click(cx.listener(|this, _, _, cx| this.roll_once(cx))),
        )
        .child(
            Button::new("copy")
                .outline()
                .label("复制")
                .on_click(cx.listener(|this, _, _, cx| {
                    if !this.output.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(this.output.to_string()));
                    }
                })),
        )
        .child(
            Button::new("six")
                .outline()
                .label("一次六个")
                .on_click(cx.listener(|this, _, _, cx| this.roll_six(cx))),
        )
}

pub fn init_theme(cx: &mut gpui_kit::App) {
    Theme::change(ThemeMode::Dark, None, cx);
}
