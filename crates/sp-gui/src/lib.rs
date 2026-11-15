//! The `SecretPiece` desktop application.
//!
//! `sp` and `secretpiece` start this GUI; the console interface lives in the
//! separate `sp-cli` binary. The design is a dark, material-inspired surface
//! with cyan and violet accents.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::path::Path;
use std::sync::{PoisonError, RwLock};

use eframe::egui;
use egui::{
    Align, Color32, FontFamily, FontId, Frame, Layout, Margin, RichText, Stroke, TextStyle,
};
use sp_core::{ErrorCode, SpError};
use sp_crypto::KdfParams;
use sp_format::EntryKind;
use sp_vault::{ListEntry, Vault, VaultInfo};

/// Colour set of the desktop app. Themes swap the whole palette at runtime.
#[derive(Debug, Clone, Copy)]
struct Palette {
    bg: Color32,
    panel: Color32,
    card: Color32,
    line: Color32,
    ink: Color32,
    muted: Color32,
    accent: Color32,
    violet: Color32,
    danger: Color32,
    good: Color32,
    input: Color32,
    selected: Color32,
}

/// Visual theme offered in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Midnight,
    Neon,
    Light,
}

impl Theme {
    const fn palette(self) -> Palette {
        match self {
            Self::Midnight => Palette {
                bg: Color32::from_rgb(8, 11, 16),
                panel: Color32::from_rgb(13, 18, 26),
                card: Color32::from_rgb(19, 26, 38),
                line: Color32::from_rgb(34, 45, 62),
                ink: Color32::from_rgb(231, 238, 247),
                muted: Color32::from_rgb(139, 155, 176),
                accent: Color32::from_rgb(34, 211, 238),
                violet: Color32::from_rgb(167, 139, 250),
                danger: Color32::from_rgb(248, 113, 113),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(10, 14, 20),
                selected: Color32::from_rgb(26, 37, 52),
            },
            Self::Neon => Palette {
                bg: Color32::from_rgb(12, 6, 24),
                panel: Color32::from_rgb(20, 10, 36),
                card: Color32::from_rgb(28, 14, 48),
                line: Color32::from_rgb(66, 32, 102),
                ink: Color32::from_rgb(245, 235, 255),
                muted: Color32::from_rgb(180, 160, 210),
                accent: Color32::from_rgb(255, 79, 216),
                violet: Color32::from_rgb(34, 211, 238),
                danger: Color32::from_rgb(255, 99, 132),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(16, 8, 30),
                selected: Color32::from_rgb(44, 22, 72),
            },
            Self::Light => Palette {
                bg: Color32::from_rgb(244, 246, 251),
                panel: Color32::from_rgb(255, 255, 255),
                card: Color32::from_rgb(255, 255, 255),
                line: Color32::from_rgb(214, 222, 236),
                ink: Color32::from_rgb(17, 24, 39),
                muted: Color32::from_rgb(91, 107, 131),
                accent: Color32::from_rgb(37, 99, 235),
                violet: Color32::from_rgb(124, 58, 237),
                danger: Color32::from_rgb(220, 38, 38),
                good: Color32::from_rgb(22, 163, 74),
                input: Color32::from_rgb(248, 250, 253),
                selected: Color32::from_rgb(226, 235, 250),
            },
        }
    }

    fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Midnight, Lang::Ru) => "Ночь",
            (Self::Midnight, Lang::En) => "Night",
            (Self::Neon, Lang::Ru) => "Неон",
            (Self::Neon, Lang::En) => "Neon",
            (Self::Light, Lang::Ru) => "Свет",
            (Self::Light, Lang::En) => "Light",
        }
    }

    fn is_light(self) -> bool {
        matches!(self, Self::Light)
    }
}

static PALETTE: RwLock<Palette> = RwLock::new(Theme::Midnight.palette());

fn pal() -> Palette {
    *PALETTE.read().unwrap_or_else(PoisonError::into_inner)
}

fn accent() -> Color32 {
    pal().accent
}

fn violet() -> Color32 {
    pal().violet
}

fn bg() -> Color32 {
    pal().bg
}

fn panel() -> Color32 {
    pal().panel
}

fn card_bg() -> Color32 {
    pal().card
}

fn line() -> Color32 {
    pal().line
}

fn ink() -> Color32 {
    pal().ink
}

fn muted() -> Color32 {
    pal().muted
}

fn danger() -> Color32 {
    pal().danger
}

fn good() -> Color32 {
    pal().good
}

fn input_bg() -> Color32 {
    pal().input
}

fn selected_bg() -> Color32 {
    pal().selected
}

fn set_theme(theme: Theme) {
    *PALETTE.write().unwrap_or_else(PoisonError::into_inner) = theme.palette();
}

/// Pushes the active palette into egui's style, so every widget follows the theme.
fn apply_theme(ctx: &egui::Context) {
    let p = pal();
    let light = p.bg.r() > 128;
    let mut style = (*ctx.style()).clone();
    style.visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    style.visuals.panel_fill = p.bg;
    style.visuals.window_fill = p.panel;
    style.visuals.extreme_bg_color = p.input;
    style.visuals.faint_bg_color = p.card;
    style.visuals.override_text_color = Some(p.ink);
    style.visuals.selection.bg_fill = p.accent;
    style.visuals.selection.stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.widgets.inactive.bg_fill = p.card;
    style.visuals.widgets.inactive.weak_bg_fill = p.card;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.widgets.hovered.bg_fill = p.selected;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.widgets.active.bg_fill = p.selected;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, p.accent);
    style.visuals.widgets.open.bg_fill = p.selected;
    style.visuals.window_stroke = Stroke::new(1.0_f32, p.line);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(30.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(15.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
    ]
    .into();
    ctx.set_style(style);
}

/// Interface language of the desktop app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Russian.
    Ru,
    /// English.
    En,
}

impl Lang {
    const fn flip(self) -> Self {
        match self {
            Self::Ru => Self::En,
            Self::En => Self::Ru,
        }
    }

    fn t(self, ru: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Ru => ru,
            Self::En => en,
        }
    }
}

/// Starts the graphical application.
///
/// # Errors
///
/// Returns an [`eframe::Error`] when the window or the renderer cannot start.
pub fn main_entry() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 720.0])
            .with_min_inner_size([800.0, 560.0])
            .with_title("SecretPiece"),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "SecretPiece",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    lang: Lang,
    theme: Theme,
    vault_path: String,
    password: String,
    confirm: String,
    creating: bool,
    show_password: bool,
    kdf: KdfChoice,
    error: Option<String>,
    browser: Option<Browser>,
}

/// Argon2id effort offered in the create form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KdfChoice {
    Light,
    Default,
    Paranoid,
}

impl KdfChoice {
    const fn params(self) -> KdfParams {
        match self {
            Self::Light => KdfParams::LIGHT,
            Self::Default => KdfParams::DEFAULT,
            Self::Paranoid => KdfParams::PARANOID,
        }
    }

    fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Light, Lang::Ru) => "Быстро",
            (Self::Light, Lang::En) => "Light",
            (Self::Default, Lang::Ru) => "Стандарт",
            (Self::Default, Lang::En) => "Default",
            (Self::Paranoid, Lang::Ru) => "Параноид",
            (Self::Paranoid, Lang::En) => "Paranoid",
        }
    }

    fn hint(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Light, Lang::Ru) => "16 МиБ · слабые устройства",
            (Self::Light, Lang::En) => "16 MiB · low-end devices",
            (Self::Default, Lang::Ru) => "64 МиБ · рекомендуется",
            (Self::Default, Lang::En) => "64 MiB · recommended",
            (Self::Paranoid, Lang::Ru) => "256 МиБ · максимум",
            (Self::Paranoid, Lang::En) => "256 MiB · maximum",
        }
    }
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        set_theme(Theme::Midnight);
        apply_theme(&cc.egui_ctx);
        Self {
            lang: Lang::Ru,
            theme: Theme::Midnight,
            vault_path: String::new(),
            password: String::new(),
            confirm: String::new(),
            creating: false,
            show_password: false,
            kdf: KdfChoice::Default,
            error: None,
            browser: None,
        }
    }

    fn unlock(&mut self) {
        self.error = None;
        let mut path = self.vault_path.trim().to_owned();
        if path.is_empty() {
            self.error = Some(
                self.lang
                    .t("Укажите файл хранилища.", "Choose a vault file.")
                    .to_owned(),
            );
            return;
        }
        if self.creating && !path.to_ascii_lowercase().ends_with(".spv") {
            path.push_str(".spv");
            self.vault_path.clone_from(&path);
        }
        if self.password.is_empty() {
            self.error = Some(
                self.lang
                    .t(
                        "Пароль не может быть пустым.",
                        "The password must not be empty.",
                    )
                    .to_owned(),
            );
            return;
        }
        if self.creating {
            if self.password.chars().count() < 8 {
                self.error = Some(
                    self.lang
                        .t(
                            "Пароль короче 8 символов. Возьмите длиннее.",
                            "The password is shorter than 8 characters. Use a longer one.",
                        )
                        .to_owned(),
                );
                return;
            }
            if self.password != self.confirm {
                self.error = Some(
                    self.lang
                        .t("Пароли не совпадают.", "The passwords do not match.")
                        .to_owned(),
                );
                return;
            }
            if Path::new(&path).exists() {
                self.error = Some(
                    self.lang
                        .t(
                            "Файл уже существует. Выберите другое имя.",
                            "The file already exists. Choose another name.",
                        )
                        .to_owned(),
                );
                return;
            }
        }
        let result = if self.creating {
            Vault::create(
                Path::new(&path),
                self.password.as_bytes(),
                self.kdf.params(),
            )
        } else {
            Vault::open(Path::new(&path), self.password.as_bytes())
        };
        match result {
            Ok(vault) => {
                self.password.clear();
                self.confirm.clear();
                self.browser = Some(Browser::new(vault));
            }
            Err(error) => self.error = Some(describe(self.lang, &error)),
        }
    }

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .exact_height(96.0)
            .frame(
                Frame::new()
                    .fill(panel())
                    .inner_margin(Margin::symmetric(28, 18)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.horizontal(|ui| {
                    let (tile, _) =
                        ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
                    gradient(ui.painter(), tile, accent(), violet(), 38);
                    ui.painter().text(
                        tile.center(),
                        egui::Align2::CENTER_CENTER,
                        "SP",
                        FontId::new(15.0, FontFamily::Proportional),
                        bg(),
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("SecretPiece")
                                .size(24.0)
                                .strong()
                                .color(ink()),
                        );
                        ui.label(
                            RichText::new(self.lang.t(
                                "зашифрованное хранилище · один файл",
                                "encrypted vault · a single file",
                            ))
                            .size(12.0)
                            .color(muted()),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.browser.is_some()
                            && ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(self.lang.t("Заблокировать", "Lock"))
                                            .size(12.0)
                                            .color(muted()),
                                    )
                                    .fill(card_bg())
                                    .stroke(Stroke::new(1.0_f32, line()))
                                    .corner_radius(8),
                                )
                                .clicked()
                        {
                            self.browser = None;
                            self.password.clear();
                        }
                        ui.add_space(10.0);
                        let label = match self.lang {
                            Lang::Ru => "EN",
                            Lang::En => "RU",
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(label).size(12.0).strong().color(ink()),
                                )
                                .fill(card_bg())
                                .stroke(Stroke::new(1.0_f32, line()))
                                .corner_radius(8)
                                .min_size(egui::vec2(40.0, 28.0)),
                            )
                            .clicked()
                        {
                            self.lang = self.lang.flip();
                        }
                        ui.add_space(12.0);
                        let mut picked = None;
                        for theme in [Theme::Light, Theme::Neon, Theme::Midnight] {
                            if theme_chip(ui, theme, self.theme == theme, self.lang) {
                                picked = Some(theme);
                            }
                            ui.add_space(4.0);
                        }
                        if let Some(theme) = picked {
                            self.theme = theme;
                            set_theme(theme);
                            apply_theme(ctx);
                        }
                    });
                });
                let line = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    egui::pos2(rect.right(), rect.bottom()),
                );
                gradient(ui.painter(), line, accent(), violet(), 128);
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.header(ctx);
        egui::CentralPanel::default()
            .frame(Frame::new().fill(bg()).inner_margin(Margin::same(28)))
            .show(ctx, |ui| {
                if let Some(browser) = self.browser.as_mut() {
                    browser.ui(ui, self.lang);
                } else {
                    self.unlock_ui(ui);
                }
            });
    }
}

impl App {
    fn unlock_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            let (tile, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
            gradient(ui.painter(), tile, accent(), violet(), 56);
            ui.painter().text(
                tile.center(),
                egui::Align2::CENTER_CENTER,
                "SP",
                FontId::new(20.0, FontFamily::Proportional),
                bg(),
            );
            ui.add_space(10.0);
            ui.label(
                RichText::new("SecretPiece")
                    .size(30.0)
                    .strong()
                    .color(ink()),
            );
            ui.label(
                RichText::new(self.lang.t(
                    "Ваш секрет — в кусочках. Один файл, ноль следов на диске.",
                    "Your secret, in pieces. One file, no disk layout.",
                ))
                .size(13.0)
                .color(muted()),
            );
            ui.add_space(16.0);
            self.form_card(ui);
        });

        let dropped = ui.ctx().input(|input| input.raw.dropped_files.clone());
        if let Some(file) = dropped.first() {
            if let Some(path) = &file.path {
                self.vault_path = path.to_string_lossy().into_owned();
                self.creating = false;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn form_card(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        Frame::new()
            .fill(card_bg())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(20)
            .inner_margin(Margin::same(26))
            .show(ui, |ui| {
                ui.set_width(408.0);
                ui.label(
                    RichText::new(if self.creating {
                        lang.t("Новое хранилище", "New vault")
                    } else {
                        lang.t("Открыть хранилище", "Unlock your vault")
                    })
                    .size(23.0)
                    .strong()
                    .color(ink()),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(lang.t(
                        "Пароль не покидает это устройство · Argon2id",
                        "Your password never leaves this device · Argon2id",
                    ))
                    .size(12.5)
                    .color(muted()),
                );
                ui.add_space(18.0);

                field_label(ui, lang.t("Файл хранилища", "Vault file"));
                ui.horizontal(|ui| {
                    let width = ui.available_width() - 96.0;
                    ui.add_sized(
                        [width, 34.0],
                        egui::TextEdit::singleline(&mut self.vault_path)
                            .hint_text("vault.spv")
                            .background_color(input_bg()),
                    );
                    if ui
                        .add(
                            egui::Button::new(lang.t("Обзор…", "Browse…"))
                                .min_size(egui::vec2(88.0, 34.0)),
                        )
                        .clicked()
                    {
                        let dialog = rfd::FileDialog::new();
                        let picked = if self.creating {
                            dialog.set_file_name("vault.spv").save_file()
                        } else {
                            dialog.pick_file()
                        };
                        if let Some(path) = picked {
                            self.vault_path = path.to_string_lossy().into_owned();
                        }
                    }
                });
                ui.label(
                    RichText::new(lang.t(
                        "Любой файл: .spv, .bin, .mp3 — внутри всё равно контейнер SPV1.",
                        "Any file: .spv, .bin, .mp3 — inside it is still an SPV1 container.",
                    ))
                    .size(11.0)
                    .color(muted()),
                );

                if self.creating {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(lang.t("Оболочка:", "Shell:"))
                                .size(11.0)
                                .color(muted()),
                        );
                        for ext in ["spv", "bin", "mp3", "jpg"] {
                            let active = self
                                .vault_path
                                .to_ascii_lowercase()
                                .ends_with(&format!(".{ext}"));
                            let button = egui::Button::new(
                                RichText::new(format!(".{ext}"))
                                    .size(11.0)
                                    .color(if active { bg() } else { muted() }),
                            )
                            .fill(if active { accent() } else { panel() })
                            .stroke(Stroke::new(1.0_f32, if active { accent() } else { line() }))
                            .corner_radius(7)
                            .min_size(egui::vec2(52.0, 26.0));
                            if ui.add(button).clicked() {
                                self.vault_path = swap_extension(&self.vault_path, ext);
                            }
                        }
                    });
                }

                ui.add_space(14.0);
                field_label(ui, lang.t("Пароль", "Password"));
                ui.horizontal(|ui| {
                    let width = ui.available_width() - 78.0;
                    ui.add_sized(
                        [width, 34.0],
                        egui::TextEdit::singleline(&mut self.password)
                            .password(!self.show_password)
                            .background_color(input_bg()),
                    );
                    let toggle = if self.show_password {
                        lang.t("Скрыть", "Hide")
                    } else {
                        lang.t("Показать", "Show")
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(toggle).size(11.0).color(muted()))
                                .fill(panel())
                                .stroke(Stroke::new(1.0_f32, line()))
                                .corner_radius(8)
                                .min_size(egui::vec2(70.0, 34.0)),
                        )
                        .clicked()
                    {
                        self.show_password = !self.show_password;
                    }
                });

                if self.creating {
                    ui.add_space(8.0);
                    let score = password_score(&self.password);
                    let (color, label) = match score {
                        0 => (
                            muted(),
                            lang.t("Минимум 8 символов", "At least 8 characters"),
                        ),
                        1 => (danger(), lang.t("Слабый пароль", "Weak password")),
                        2 => (
                            Color32::from_rgb(251, 191, 36),
                            lang.t("Средний пароль", "Medium password"),
                        ),
                        3 => (good(), lang.t("Хороший пароль", "Good password")),
                        _ => (accent(), lang.t("Отличный пароль", "Excellent password")),
                    };
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 6.0),
                        egui::Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, 3.0, Color32::from_rgb(24, 32, 46));
                    let ratio = f32::from(score) / 4.0;
                    let filled = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width() * ratio, rect.height()),
                    );
                    ui.painter().rect_filled(filled, 3.0, color);
                    ui.add_space(2.0);
                    ui.label(RichText::new(label).size(11.0).color(color));

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Стойкость ключа", "Key hardness"));
                    ui.horizontal(|ui| {
                        for choice in [KdfChoice::Light, KdfChoice::Default, KdfChoice::Paranoid] {
                            let selected = self.kdf == choice;
                            let button = egui::Button::new(
                                RichText::new(choice.label(lang))
                                    .size(12.0)
                                    .color(if selected { bg() } else { ink() }),
                            )
                            .fill(if selected { accent() } else { panel() })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if selected { accent() } else { line() },
                            ))
                            .corner_radius(8)
                            .min_size(egui::vec2(126.0, 30.0));
                            if ui.add(button).clicked() {
                                self.kdf = choice;
                            }
                        }
                    });
                    ui.add_space(2.0);
                    ui.label(RichText::new(self.kdf.hint(lang)).size(11.0).color(muted()));

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Повторите пароль", "Repeat password"));
                    ui.add_sized(
                        [ui.available_width(), 34.0],
                        egui::TextEdit::singleline(&mut self.confirm)
                            .password(!self.show_password)
                            .background_color(input_bg()),
                    );
                }

                ui.add_space(20.0);
                let ready = !self.creating
                    || (self.password.chars().count() >= 8 && self.password == self.confirm);
                let primary = if self.creating {
                    lang.t("Создать сейф", "Create vault")
                } else {
                    lang.t("Открыть", "Unlock")
                };
                let button =
                    egui::Button::new(RichText::new(primary).size(16.0).strong().color(bg()))
                        .fill(accent())
                        .corner_radius(10)
                        .min_size(egui::vec2(ui.available_width(), 42.0));
                if ui.add_enabled(ready, button).clicked() {
                    self.unlock();
                }
                ui.add_space(8.0);
                let switch = if self.creating {
                    lang.t("У меня уже есть хранилище", "I already have a vault")
                } else {
                    lang.t("Создать новое хранилище", "Create a new vault")
                };
                if ui
                    .add(egui::Button::new(RichText::new(switch).color(violet())).frame(false))
                    .clicked()
                {
                    self.creating = !self.creating;
                    self.error = None;
                }

                if let Some(error) = &self.error {
                    ui.add_space(12.0);
                    ui.label(RichText::new(error).size(12.5).color(danger()));
                }
            });

        ui.add_space(14.0);
        ui.label(
            RichText::new(lang.t(
                "Подсказка: перетащите .spv в окно, чтобы открыть его.",
                "Tip: drop a .spv file onto the window to open it.",
            ))
            .size(11.0)
            .color(muted()),
        );
    }
}

struct Status {
    text: String,
    error: bool,
}

struct Browser {
    vault: Vault,
    cwd: String,
    entries: Vec<ListEntry>,
    selected: Option<String>,
    status: Option<Status>,
    new_folder: Option<String>,
    pending_delete: Option<String>,
    pending_add: Option<PendingAdd>,
    preview: Option<Preview>,
    shred_on_delete: bool,
}

/// A file read from the container, waiting for overwrite confirmation.
struct PendingAdd {
    remote: String,
    data: Vec<u8>,
}

/// Decrypted bytes kept only in RAM for the preview window.
struct Preview {
    path: String,
    data: Vec<u8>,
    hex: bool,
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.data.fill(0);
    }
}

impl Browser {
    fn new(vault: Vault) -> Self {
        let mut browser = Self {
            vault,
            cwd: String::new(),
            entries: Vec::new(),
            selected: None,
            status: None,
            new_folder: None,
            pending_delete: None,
            pending_add: None,
            preview: None,
            shred_on_delete: true,
        };
        browser.refresh();
        browser
    }

    fn refresh(&mut self) {
        self.entries = self.vault.list(Some(&self.cwd));
        self.selected = None;
    }

    fn join(&self, name: &str) -> String {
        if self.cwd.is_empty() {
            name.to_owned()
        } else {
            format!("{}/{name}", self.cwd)
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let info = self.vault.info();
        self.actions(ui, lang);
        ui.add_space(10.0);
        if let Ok(info) = &info {
            Self::info_bar(ui, lang, info);
        }
        ui.add_space(12.0);
        self.breadcrumb(ui, lang);
        ui.add_space(8.0);

        let height = ui.available_height() - 46.0;
        Frame::new()
            .fill(panel())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(14)
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(height)
                    .show(ui, |ui| {
                        if self.entries.is_empty() {
                            ui.add_space(30.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Здесь пока пусто", "Nothing here yet"))
                                        .size(16.0)
                                        .color(muted()),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(lang.t(
                                        "Добавьте файл или создайте папку.",
                                        "Add a file or create a folder.",
                                    ))
                                    .size(12.0)
                                    .color(muted()),
                                );
                            });
                            ui.add_space(30.0);
                        }
                        let mut enter: Option<String> = None;
                        let mut preview: Option<String> = None;
                        for entry in &self.entries {
                            let selected = self.selected.as_deref() == Some(entry.path.as_str());
                            let click = row(ui, lang, entry, selected);
                            if click.selected {
                                self.selected = Some(entry.path.clone());
                            }
                            if click.open {
                                enter = Some(entry.path.clone());
                            }
                            if click.preview {
                                preview = Some(entry.path.clone());
                            }
                        }
                        if let Some(path) = enter {
                            self.cwd = path;
                            self.refresh();
                        }
                        if let Some(path) = preview {
                            self.open_preview(&path, lang);
                        }
                    });
            });

        ui.add_space(10.0);
        if let Some(status) = &self.status {
            let color = if status.error { danger() } else { good() };
            ui.label(RichText::new(&status.text).color(color));
        }

        self.dialogs(ui, lang);
    }

    fn actions(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(lang.t("Добавить файл", "Add file"))
                            .strong()
                            .color(bg()),
                    )
                    .fill(accent())
                    .corner_radius(9),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.add_file(&path, lang);
                }
            }
            if ui.button(lang.t("Новая папка", "New folder")).clicked() {
                self.new_folder = Some(String::new());
            }
            let selected = self.selected.clone();
            let selected_file = self.selected_file().map(ToOwned::to_owned);
            if ui
                .add_enabled(
                    selected_file.is_some(),
                    egui::Button::new(lang.t("Просмотр", "Preview")),
                )
                .clicked()
            {
                if let Some(path) = selected_file.clone() {
                    self.open_preview(&path, lang);
                }
            }
            if ui
                .add_enabled(
                    selected_file.is_some(),
                    egui::Button::new(lang.t("Извлечь", "Extract")),
                )
                .clicked()
            {
                if let Some(path) = selected_file.clone() {
                    self.extract(&path, lang);
                }
            }
            if ui
                .add_enabled(
                    selected.is_some(),
                    egui::Button::new(RichText::new(lang.t("Удалить", "Delete")).color(danger())),
                )
                .clicked()
            {
                self.pending_delete = selected;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(lang.t("Проверить", "Verify")).clicked() {
                    match self.vault.verify() {
                        Ok(report) => {
                            self.status = Some(Status {
                                text: match lang {
                                    Lang::Ru => format!(
                                        "Целостность в порядке: {} файл(ов), {}",
                                        report.files,
                                        human_size(report.bytes)
                                    ),
                                    Lang::En => format!(
                                        "Integrity check passed: {} file(s), {}",
                                        report.files,
                                        human_size(report.bytes)
                                    ),
                                },
                                error: false,
                            });
                        }
                        Err(error) => {
                            self.status = Some(Status {
                                text: describe(lang, &error),
                                error: true,
                            });
                        }
                    }
                }
            });
        });
    }

    fn info_bar(ui: &mut egui::Ui, lang: Lang, info: &VaultInfo) {
        ui.horizontal(|ui| {
            chip(
                ui,
                lang.t("файл", "file"),
                &human_size(info.file_size),
                accent(),
            );
            chip(
                ui,
                lang.t("данные", "payload"),
                &human_size(info.payload_bytes),
                violet(),
            );
            chip(
                ui,
                lang.t("файлов", "files"),
                &info.files.to_string(),
                good(),
            );
            chip(
                ui,
                lang.t("поколение", "generation"),
                &info.generation.to_string(),
                muted(),
            );
        });
    }

    fn breadcrumb(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            let mut target: Option<String> = None;
            if ui
                .add(
                    egui::Button::new(RichText::new(lang.t("Хранилище", "Vault")).color(accent()))
                        .frame(false),
                )
                .clicked()
            {
                target = Some(String::new());
            }
            let mut accumulated = String::new();
            for part in self.cwd.split('/').filter(|part| !part.is_empty()) {
                if !accumulated.is_empty() {
                    accumulated.push('/');
                }
                accumulated.push_str(part);
                ui.label(RichText::new("/").color(muted()));
                if ui
                    .add(egui::Button::new(RichText::new(part).color(ink())).frame(false))
                    .clicked()
                {
                    target = Some(accumulated.clone());
                }
            }
            if let Some(target) = target {
                self.cwd = target;
                self.refresh();
            }
        });
    }

    fn selected_file(&self) -> Option<&str> {
        let path = self.selected.as_deref()?;
        self.entries
            .iter()
            .find(|entry| entry.path == path && entry.kind == EntryKind::File)
            .map(|entry| entry.path.as_str())
    }

    fn add_file(&mut self, path: &Path, lang: Lang) {
        let name = path.file_name().map_or_else(
            || "file".to_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        let remote = self.join(&name);
        match std::fs::read(path) {
            Ok(data) => {
                let exists = self
                    .entries
                    .iter()
                    .any(|entry| entry.path == remote && entry.kind == EntryKind::File);
                if exists {
                    self.pending_add = Some(PendingAdd { remote, data });
                } else {
                    self.commit_add(&remote, &data, false, lang);
                }
            }
            Err(_) => {
                self.status = Some(Status {
                    text: lang
                        .t("Не удалось прочитать файл.", "Cannot read the file.")
                        .to_owned(),
                    error: true,
                });
            }
        }
    }

    fn commit_add(&mut self, remote: &str, data: &[u8], overwrite: bool, lang: Lang) {
        let size = data.len() as u64;
        match self.vault.put(remote, data) {
            Ok(()) => {
                self.refresh();
                self.status = Some(Status {
                    text: match (lang, overwrite) {
                        (Lang::Ru, true) => {
                            format!("Перезаписано: {remote} ({})", human_size(size))
                        }
                        (Lang::En, true) => {
                            format!("Overwritten: {remote} ({})", human_size(size))
                        }
                        (Lang::Ru, false) => format!("Добавлено: {remote} ({})", human_size(size)),
                        (Lang::En, false) => format!("Added: {remote} ({})", human_size(size)),
                    },
                    error: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    text: describe(lang, &error),
                    error: true,
                });
            }
        }
    }

    fn open_preview(&mut self, path: &str, lang: Lang) {
        match self.vault.get(path) {
            Ok(data) => {
                self.preview = Some(Preview {
                    path: path.to_owned(),
                    data,
                    hex: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    text: describe(lang, &error),
                    error: true,
                });
            }
        }
    }

    fn extract(&mut self, remote: &str, lang: Lang) {
        let name = remote.rsplit('/').next().unwrap_or(remote);
        let Some(target) = rfd::FileDialog::new().set_file_name(name).save_file() else {
            return;
        };
        match self.vault.get(remote) {
            Ok(data) => match std::fs::write(&target, &data) {
                Ok(()) => {
                    self.status = Some(Status {
                        text: match lang {
                            Lang::Ru => format!("Извлечено: {}", target.display()),
                            Lang::En => format!("Extracted: {}", target.display()),
                        },
                        error: false,
                    });
                }
                Err(error) => {
                    self.status = Some(Status {
                        text: format!("{}: {error}", lang.t("Ошибка записи", "Write error")),
                        error: true,
                    });
                }
            },
            Err(error) => {
                self.status = Some(Status {
                    text: describe(lang, &error),
                    error: true,
                });
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn dialogs(&mut self, ui: &mut egui::Ui, lang: Lang) {
        if let Some(name) = self.new_folder.clone() {
            let mut value = name;
            let mut create = false;
            let mut cancel = false;
            egui::Window::new(lang.t("Новая папка", "New folder"))
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(lang.t("Имя папки", "Folder name"));
                    let response =
                        ui.add(egui::TextEdit::singleline(&mut value).background_color(input_bg()));
                    response.request_focus();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(lang.t("Создать", "Create")).clicked() {
                            create = true;
                        }
                        if ui.button(lang.t("Отмена", "Cancel")).clicked() {
                            cancel = true;
                        }
                    });
                });
            if create {
                let remote = self.join(value.trim());
                match self.vault.mkdir(&remote) {
                    Ok(()) => {
                        self.refresh();
                        self.status = Some(Status {
                            text: match lang {
                                Lang::Ru => format!("Папка создана: {remote}"),
                                Lang::En => format!("Folder created: {remote}"),
                            },
                            error: false,
                        });
                    }
                    Err(error) => {
                        self.status = Some(Status {
                            text: describe(lang, &error),
                            error: true,
                        });
                    }
                }
            }
            self.new_folder = if create || cancel { None } else { Some(value) };
        }

        if let Some(pending) = self.pending_add.as_ref() {
            let mut overwrite = false;
            let mut cancel = false;
            egui::Window::new(lang.t("Файл уже есть", "File already exists"))
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(match lang {
                        Lang::Ru => format!(
                            "`{}` уже лежит в этой папке сейфа.\nПерезаписать его новыми данными?",
                            pending.remote
                        ),
                        Lang::En => format!(
                            "`{}` already exists in this vault folder.\nOverwrite it with the new data?",
                            pending.remote
                        ),
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Старая версия исчезнет из индекса сразу.",
                            "The old version disappears from the index immediately.",
                        ))
                        .size(11.5)
                        .color(muted()),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Перезаписать", "Overwrite"))
                                        .strong()
                                        .color(bg()),
                                )
                                .fill(accent())
                                .corner_radius(8),
                            )
                            .clicked()
                        {
                            overwrite = true;
                        }
                        if ui.button(lang.t("Отмена", "Cancel")).clicked() {
                            cancel = true;
                        }
                    });
                });
            if overwrite {
                if let Some(pending) = self.pending_add.take() {
                    self.commit_add(&pending.remote, &pending.data, true, lang);
                }
            } else if cancel {
                self.pending_add = None;
            }
        }

        if let Some(target) = self.pending_delete.clone() {
            let mut confirmed = false;
            let mut cancel = false;
            let mut shred = self.shred_on_delete;
            egui::Window::new(lang.t("Удалить?", "Delete?"))
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(match lang {
                        Lang::Ru => format!("Удалить `{target}`? Это необратимо."),
                        Lang::En => format!("Delete `{target}`? This cannot be undone."),
                    });
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut shred,
                        lang.t(
                            "Затереть данные — восстановить нельзя",
                            "Shred the data — unrecoverable",
                        ),
                    );
                    ui.label(
                        RichText::new(lang.t(
                            "Перезаписывает куски случайными байтами. На флешках лучший результат даёт полное шифрование диска.",
                            "Overwrites chunks with random bytes. Full-disk encryption is still the strongest option on flash.",
                        ))
                        .size(10.5)
                        .color(muted()),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Удалить", "Delete")).color(danger()),
                                )
                                .fill(card_bg()),
                            )
                            .clicked()
                        {
                            confirmed = true;
                        }
                        if ui.button(lang.t("Отмена", "Cancel")).clicked() {
                            cancel = true;
                        }
                    });
                });
            self.shred_on_delete = shred;
            if confirmed {
                let result = if shred {
                    self.vault.shred(&target)
                } else {
                    self.vault.remove(&target)
                };
                match result {
                    Ok(()) => {
                        self.refresh();
                        self.status = Some(Status {
                            text: match (lang, shred) {
                                (Lang::Ru, true) => {
                                    format!("Затёрто безвозвратно: {target}")
                                }
                                (Lang::En, true) => {
                                    format!("Shredded permanently: {target}")
                                }
                                (Lang::Ru, false) => format!("Удалено: {target}"),
                                (Lang::En, false) => format!("Removed: {target}"),
                            },
                            error: false,
                        });
                    }
                    Err(error) => {
                        self.status = Some(Status {
                            text: describe(lang, &error),
                            error: true,
                        });
                    }
                }
                self.pending_delete = None;
            } else if cancel {
                self.pending_delete = None;
            }
        }

        if let Some(preview) = self.preview.as_mut() {
            let mut open = true;
            egui::Window::new(lang.t("Просмотр из контейнера", "Preview from vault"))
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_size([760.0, 520.0])
                .show(ui.ctx(), |ui| {
                    let text = preview_text(&preview.data);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&preview.path).strong().color(ink()));
                        ui.label(
                            RichText::new(human_size(preview.data.len() as u64))
                                .size(12.0)
                                .color(muted()),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if text.is_some() {
                                let label = if preview.hex {
                                    lang.t("Текст", "Text")
                                } else {
                                    lang.t("Hex", "Hex")
                                };
                                if ui.button(label).clicked() {
                                    preview.hex = !preview.hex;
                                }
                            }
                            ui.label(
                                RichText::new(lang.t(
                                    "данные только в RAM, окно закроется — буфер затрётся",
                                    "RAM only; closing the window wipes the buffer",
                                ))
                                .size(10.5)
                                .color(muted()),
                            );
                        });
                    });
                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let hex = preview.hex || text.is_none();
                            if hex {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(hex_dump(&preview.data, 16 * 1024))
                                            .monospace()
                                            .size(12.0)
                                            .color(ink()),
                                    )
                                    .selectable(true),
                                );
                                if preview.data.len() > 16 * 1024 {
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Показаны первые 16 КиБ.",
                                            "Showing the first 16 KiB.",
                                        ))
                                        .size(11.0)
                                        .color(muted()),
                                    );
                                }
                            } else if let Some(text) = text {
                                let shown = clip(text, 200_000);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(shown).monospace().size(12.5).color(ink()),
                                    )
                                    .selectable(true),
                                );
                                if shown.len() < text.len() {
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Показаны первые 200 КБ текста.",
                                            "Showing the first 200 KB of text.",
                                        ))
                                        .size(11.0)
                                        .color(muted()),
                                    );
                                }
                            }
                        });
                });
            if !open {
                self.preview = None;
            }
        }
    }
}

struct RowClick {
    selected: bool,
    open: bool,
    preview: bool,
}

fn row(ui: &mut egui::Ui, lang: Lang, entry: &ListEntry, selected: bool) -> RowClick {
    let frame = if selected {
        Frame::new()
            .fill(selected_bg())
            .corner_radius(9)
            .inner_margin(Margin::symmetric(8, 6))
    } else {
        Frame::new()
            .fill(Color32::TRANSPARENT)
            .corner_radius(9)
            .inner_margin(Margin::symmetric(8, 6))
    };
    let inner = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (mark, color) = match entry.kind {
                EntryKind::Dir => (lang.t("ПАПКА", "FOLDER"), violet()),
                EntryKind::File => (lang.t("ФАЙЛ", "FILE"), accent()),
            };
            ui.label(RichText::new(mark).size(10.0).strong().color(color));
            ui.add_space(6.0);
            let name = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            ui.label(RichText::new(name).size(15.0).color(ink()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let size = match entry.kind {
                    EntryKind::Dir => "—".to_owned(),
                    EntryKind::File => human_size(entry.size),
                };
                ui.label(RichText::new(size).size(12.0).color(muted()));
                if entry.kind == EntryKind::Dir {
                    ui.label(RichText::new("›").size(16.0).color(muted()));
                } else {
                    ui.label(
                        RichText::new(lang.t("двойной клик — просмотр", "double-click to preview"))
                            .size(10.0)
                            .color(muted()),
                    );
                }
            });
        });
    });
    let response = inner.response.interact(egui::Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let double = response.double_clicked();
    RowClick {
        selected: response.clicked(),
        open: double && entry.kind == EntryKind::Dir,
        preview: double && entry.kind == EntryKind::File,
    }
}

fn theme_chip(ui: &mut egui::Ui, theme: Theme, active: bool, lang: Lang) -> bool {
    let text = if active && theme.is_light() {
        Color32::WHITE
    } else if active {
        bg()
    } else {
        muted()
    };
    ui.add(
        egui::Button::new(RichText::new(theme.label(lang)).size(11.5).color(text))
            .fill(if active { accent() } else { card_bg() })
            .stroke(Stroke::new(1.0_f32, if active { accent() } else { line() }))
            .corner_radius(8)
            .min_size(egui::vec2(52.0, 28.0)),
    )
    .clicked()
}

fn chip(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    Frame::new()
        .fill(card_bg())
        .stroke(Stroke::new(1.0_f32, line()))
        .corner_radius(9)
        .inner_margin(Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(11.0).color(muted()));
                ui.label(RichText::new(value).size(13.0).strong().color(accent));
            });
        });
}

/// Returns the payload as UTF-8 text when it looks like text.
fn preview_text(data: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(data).ok()?;
    if text.contains('\0') {
        return None;
    }
    Some(text)
}

/// Cuts `text` to at most `max` bytes on a char boundary.
fn clip(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Classic hex + ASCII dump, limited to `max` bytes.
fn hex_dump(data: &[u8], max: usize) -> String {
    use std::fmt::Write as _;

    let slice = &data[..data.len().min(max)];
    let mut out = String::with_capacity(slice.len() * 4);
    for (index, chunk) in slice.chunks(16).enumerate() {
        let _ = write!(out, "{:08x}  ", index * 16);
        for byte in chunk {
            let _ = write!(out, "{byte:02x} ");
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push(' ');
        for byte in chunk {
            let printable = byte.is_ascii_graphic() || *byte == b' ';
            out.push(if printable { *byte as char } else { '.' });
        }
        out.push('\n');
    }
    out
}

/// Rough password strength from 0 (empty) to 4 (excellent).
fn password_score(password: &str) -> u8 {
    let len = password.chars().count();
    if len == 0 {
        return 0;
    }
    if len < 8 {
        return 1;
    }
    let mut classes = 0;
    if password.chars().any(char::is_lowercase) {
        classes += 1;
    }
    if password.chars().any(char::is_uppercase) {
        classes += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        classes += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        classes += 1;
    }
    let mut score = 2;
    if len >= 12 {
        score += 1;
    }
    if classes >= 3 {
        score += 1;
    }
    score.min(4)
}

/// Replaces the extension of `path`, keeping folders and the base name.
fn swap_extension(path: &str, ext: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return format!("vault.{ext}");
    }
    let slash = trimmed.rfind(['/', '\\']).map_or(0, |index| index + 1);
    let stem = match trimmed.rfind('.') {
        Some(index) if index > slash => &trimmed[..index],
        _ => trimmed,
    };
    format!("{stem}.{ext}")
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(muted()));
    ui.add_space(4.0);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn gradient(painter: &egui::Painter, rect: egui::Rect, from: Color32, to: Color32, steps: u32) {
    for step in 0..steps {
        let ratio = step as f32 / steps as f32;
        let next = (step + 1) as f32 / steps as f32;
        let color = Color32::from_rgb(
            mix(from.r(), to.r(), ratio),
            mix(from.g(), to.g(), ratio),
            mix(from.b(), to.b(), ratio),
        );
        let strip = egui::Rect::from_min_max(
            egui::pos2(rect.left() + rect.width() * ratio, rect.top()),
            egui::pos2(rect.left() + rect.width() * next, rect.bottom()),
        );
        painter.rect_filled(strip, 0.0, color);
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn mix(from: u8, to: u8, ratio: f32) -> u8 {
    let value = f32::from(from) + (f32::from(to) - f32::from(from)) * ratio;
    value.round().clamp(0.0, 255.0) as u8
}

#[allow(clippy::cast_precision_loss)]
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn describe(lang: Lang, error: &SpError) -> String {
    let base = match (lang, error.code()) {
        (Lang::Ru, ErrorCode::UnlockFailed) => {
            "Не удалось открыть: неверный пароль или повреждён заголовок."
        }
        (Lang::En, ErrorCode::UnlockFailed) => "Cannot unlock: wrong password or a damaged header.",
        (Lang::Ru, ErrorCode::NotFound) => "Файл или папка не найдены.",
        (Lang::En, ErrorCode::NotFound) => "File or folder not found.",
        (Lang::Ru, ErrorCode::Busy) => "Такое имя уже занято.",
        (Lang::En, ErrorCode::Busy) => "That name is already taken.",
        (Lang::Ru, ErrorCode::Integrity) => "Проверка целостности не пройдена: данные повреждены.",
        (Lang::En, ErrorCode::Integrity) => "Integrity check failed: data is damaged.",
        (Lang::Ru, ErrorCode::NoSpace) => "На диске закончилось место.",
        (Lang::En, ErrorCode::NoSpace) => "Not enough space on the drive.",
        (Lang::Ru, ErrorCode::PermissionDenied) => "Доступ запрещён.",
        (Lang::En, ErrorCode::PermissionDenied) => "Permission denied.",
        (Lang::Ru, ErrorCode::Corrupted) => "Хранилище повреждено или это не .spv файл.",
        (Lang::En, ErrorCode::Corrupted) => "The vault is damaged or not a .spv file.",
        (Lang::Ru, _) => "Операция не выполнена.",
        (Lang::En, _) => "The operation failed.",
    };
    format!("{base} [{}]", error.code().exit_status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_human_readable() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2.0 KiB");
        assert_eq!(human_size(3 * 1024 * 1024), "3.0 MiB");
    }

    #[test]
    fn language_flips() {
        assert_eq!(Lang::Ru.flip(), Lang::En);
        assert_eq!(Lang::En.flip(), Lang::Ru);
    }

    #[test]
    fn gradient_colors_interpolate() {
        assert_eq!(mix(0, 255, 0.0), 0);
        assert_eq!(mix(0, 255, 1.0), 255);
        assert_eq!(mix(0, 255, 0.5), 128);
    }

    #[test]
    fn error_messages_are_localized() {
        let error = SpError::unlock();
        assert!(describe(Lang::Ru, &error).contains("пароль"));
        assert!(describe(Lang::En, &error).contains("password"));
    }

    #[test]
    fn password_strength_grows_with_length_and_variety() {
        assert_eq!(password_score(""), 0);
        assert_eq!(password_score("short"), 1);
        assert_eq!(password_score("abcdefgh"), 2);
        assert!(password_score("Correct Horse 42!") >= 3);
        assert_eq!(password_score("Aa1!Bb2@Cc3#Dd4$"), 4);
    }

    #[test]
    fn disguise_swaps_only_the_extension() {
        assert_eq!(swap_extension("D:/vaults/a.spv", "mp3"), "D:/vaults/a.mp3");
        assert_eq!(swap_extension("a", "bin"), "a.bin");
        assert_eq!(swap_extension("", "jpg"), "vault.jpg");
        assert_eq!(
            swap_extension("D:/my.vaults/secret", "spv"),
            "D:/my.vaults/secret.spv"
        );
    }
}
