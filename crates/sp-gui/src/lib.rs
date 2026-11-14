//! The `SecretPiece` desktop application.
//!
//! `sp` and `secretpiece` start this GUI; the console interface lives in the
//! separate `sp-cli` binary. The design is a dark, material-inspired surface
//! with cyan and violet accents.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::path::Path;

use eframe::egui;
use egui::{
    Align, Color32, FontFamily, FontId, Frame, Layout, Margin, RichText, Stroke, TextStyle,
};
use sp_core::{ErrorCode, SpError};
use sp_crypto::KdfParams;
use sp_format::EntryKind;
use sp_vault::{ListEntry, Vault, VaultInfo};

const ACCENT: Color32 = Color32::from_rgb(34, 211, 238);
const VIOLET: Color32 = Color32::from_rgb(167, 139, 250);
const BG: Color32 = Color32::from_rgb(8, 11, 16);
const PANEL: Color32 = Color32::from_rgb(13, 18, 26);
const CARD: Color32 = Color32::from_rgb(19, 26, 38);
const BORDER: Color32 = Color32::from_rgb(34, 45, 62);
const TEXT: Color32 = Color32::from_rgb(231, 238, 247);
const MUTED: Color32 = Color32::from_rgb(139, 155, 176);
const DANGER: Color32 = Color32::from_rgb(248, 113, 113);
const OK: Color32 = Color32::from_rgb(74, 222, 128);

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
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = PANEL;
        style.visuals.extreme_bg_color = Color32::from_rgb(10, 14, 20);
        style.visuals.faint_bg_color = CARD;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1.0_f32, TEXT);
        style.visuals.widgets.inactive.bg_fill = CARD;
        style.visuals.widgets.inactive.weak_bg_fill = CARD;
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(28, 38, 54);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(35, 48, 68);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
        style.visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
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
        cc.egui_ctx.set_style(style);

        Self {
            lang: Lang::Ru,
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
                    .fill(PANEL)
                    .inner_margin(Margin::symmetric(28, 18)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("SecretPiece").size(27.0).strong().color(TEXT));
                        ui.label(
                            RichText::new(self.lang.t(
                                "зашифрованное хранилище · один файл .spv",
                                "encrypted vault · a single .spv file",
                            ))
                            .size(13.0)
                            .color(MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let label = match self.lang {
                            Lang::Ru => "EN",
                            Lang::En => "RU",
                        };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(label).strong().color(TEXT))
                                    .fill(CARD)
                                    .stroke(Stroke::new(1.0_f32, BORDER)),
                            )
                            .clicked()
                        {
                            self.lang = self.lang.flip();
                        }
                        ui.add_space(8.0);
                        if self.browser.is_some()
                            && ui
                                .button(
                                    RichText::new(self.lang.t("Заблокировать", "Lock"))
                                        .color(MUTED),
                                )
                                .clicked()
                        {
                            self.browser = None;
                            self.password.clear();
                        }
                    });
                });
                let line = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    egui::pos2(rect.right(), rect.bottom()),
                );
                gradient(ui.painter(), line, ACCENT, VIOLET, 128);
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.header(ctx);
        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(Margin::same(28)))
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
    #[allow(clippy::too_many_lines)]
    fn unlock_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        if ui.available_width() >= 880.0 {
            ui.horizontal_top(|ui| {
                ui.add_space(8.0);
                hero_panel(ui, self.lang);
                ui.add_space(40.0);
                self.form_card(ui);
            });
        } else {
            ui.vertical_centered(|ui| {
                self.form_card(ui);
            });
        }

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
            .fill(CARD)
            .stroke(Stroke::new(1.0_f32, BORDER))
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
                    .color(TEXT),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(lang.t(
                        "Пароль не покидает это устройство · Argon2id",
                        "Your password never leaves this device · Argon2id",
                    ))
                    .size(12.5)
                    .color(MUTED),
                );
                ui.add_space(18.0);

                field_label(ui, lang.t("Файл хранилища", "Vault file"));
                ui.horizontal(|ui| {
                    let width = ui.available_width() - 96.0;
                    ui.add_sized(
                        [width, 34.0],
                        egui::TextEdit::singleline(&mut self.vault_path).hint_text("vault.spv"),
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
                    .color(MUTED),
                );

                if self.creating {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(lang.t("Оболочка:", "Shell:"))
                                .size(11.0)
                                .color(MUTED),
                        );
                        for ext in ["spv", "bin", "mp3", "jpg"] {
                            let active = self
                                .vault_path
                                .to_ascii_lowercase()
                                .ends_with(&format!(".{ext}"));
                            let button = egui::Button::new(
                                RichText::new(format!(".{ext}"))
                                    .size(11.0)
                                    .color(if active { BG } else { MUTED }),
                            )
                            .fill(if active { ACCENT } else { PANEL })
                            .stroke(Stroke::new(1.0_f32, if active { ACCENT } else { BORDER }))
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
                            .password(!self.show_password),
                    );
                    let toggle = if self.show_password {
                        lang.t("Скрыть", "Hide")
                    } else {
                        lang.t("Показать", "Show")
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(toggle).size(11.0).color(MUTED))
                                .fill(PANEL)
                                .stroke(Stroke::new(1.0_f32, BORDER))
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
                        0 => (MUTED, lang.t("Минимум 8 символов", "At least 8 characters")),
                        1 => (DANGER, lang.t("Слабый пароль", "Weak password")),
                        2 => (
                            Color32::from_rgb(251, 191, 36),
                            lang.t("Средний пароль", "Medium password"),
                        ),
                        3 => (OK, lang.t("Хороший пароль", "Good password")),
                        _ => (ACCENT, lang.t("Отличный пароль", "Excellent password")),
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
                                    .color(if selected { BG } else { TEXT }),
                            )
                            .fill(if selected { ACCENT } else { PANEL })
                            .stroke(Stroke::new(1.0_f32, if selected { ACCENT } else { BORDER }))
                            .corner_radius(8)
                            .min_size(egui::vec2(126.0, 30.0));
                            if ui.add(button).clicked() {
                                self.kdf = choice;
                            }
                        }
                    });
                    ui.add_space(2.0);
                    ui.label(RichText::new(self.kdf.hint(lang)).size(11.0).color(MUTED));

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Повторите пароль", "Repeat password"));
                    ui.add_sized(
                        [ui.available_width(), 34.0],
                        egui::TextEdit::singleline(&mut self.confirm).password(!self.show_password),
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
                    egui::Button::new(RichText::new(primary).size(16.0).strong().color(BG))
                        .fill(ACCENT)
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
                    .add(egui::Button::new(RichText::new(switch).color(VIOLET)).frame(false))
                    .clicked()
                {
                    self.creating = !self.creating;
                    self.error = None;
                }

                if let Some(error) = &self.error {
                    ui.add_space(12.0);
                    ui.label(RichText::new(error).size(12.5).color(DANGER));
                }
            });

        ui.add_space(14.0);
        ui.label(
            RichText::new(lang.t(
                "Подсказка: перетащите .spv в окно, чтобы открыть его.",
                "Tip: drop a .spv file onto the window to open it.",
            ))
            .size(11.0)
            .color(MUTED),
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
            .fill(PANEL)
            .stroke(Stroke::new(1.0_f32, BORDER))
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
                                        .color(MUTED),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(lang.t(
                                        "Добавьте файл или создайте папку.",
                                        "Add a file or create a folder.",
                                    ))
                                    .size(12.0)
                                    .color(MUTED),
                                );
                            });
                            ui.add_space(30.0);
                        }
                        let mut enter: Option<String> = None;
                        for entry in &self.entries {
                            let selected = self.selected.as_deref() == Some(entry.path.as_str());
                            if row(ui, lang, entry, selected) {
                                enter = Some(entry.path.clone());
                            }
                            if ui.ctx().input(|input| input.pointer.any_click()) {
                                self.selected = Some(entry.path.clone());
                            }
                        }
                        if let Some(path) = enter {
                            self.cwd = path;
                            self.refresh();
                        }
                    });
            });

        ui.add_space(10.0);
        if let Some(status) = &self.status {
            let color = if status.error { DANGER } else { OK };
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
                            .color(BG),
                    )
                    .fill(ACCENT)
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
            if ui
                .add_enabled(
                    selected.is_some(),
                    egui::Button::new(lang.t("Извлечь", "Extract")),
                )
                .clicked()
            {
                if let Some(path) = selected.clone() {
                    self.extract(&path, lang);
                }
            }
            if ui
                .add_enabled(
                    selected.is_some(),
                    egui::Button::new(RichText::new(lang.t("Удалить", "Delete")).color(DANGER)),
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
                ACCENT,
            );
            chip(
                ui,
                lang.t("данные", "payload"),
                &human_size(info.payload_bytes),
                VIOLET,
            );
            chip(ui, lang.t("файлов", "files"), &info.files.to_string(), OK);
            chip(
                ui,
                lang.t("поколение", "generation"),
                &info.generation.to_string(),
                MUTED,
            );
        });
    }

    fn breadcrumb(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            let mut target: Option<String> = None;
            if ui
                .add(
                    egui::Button::new(RichText::new(lang.t("Хранилище", "Vault")).color(ACCENT))
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
                ui.label(RichText::new("/").color(MUTED));
                if ui
                    .add(egui::Button::new(RichText::new(part).color(TEXT)).frame(false))
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

    fn add_file(&mut self, path: &Path, lang: Lang) {
        let name = path.file_name().map_or_else(
            || "file".to_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        let remote = self.join(&name);
        match std::fs::read(path) {
            Ok(data) => {
                let size = data.len() as u64;
                match self.vault.put(&remote, &data) {
                    Ok(()) => {
                        self.refresh();
                        self.status = Some(Status {
                            text: match lang {
                                Lang::Ru => format!("Добавлено: {remote} ({})", human_size(size)),
                                Lang::En => format!("Added: {remote} ({})", human_size(size)),
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
                    let response = ui.add(egui::TextEdit::singleline(&mut value));
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

        if let Some(target) = self.pending_delete.clone() {
            let mut confirmed = false;
            let mut cancel = false;
            egui::Window::new(lang.t("Удалить?", "Delete?"))
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(match lang {
                        Lang::Ru => format!("Удалить `{target}`? Это необратимо."),
                        Lang::En => format!("Delete `{target}`? This cannot be undone."),
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Удалить", "Delete")).color(DANGER),
                                )
                                .fill(CARD),
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
            if confirmed {
                match self.vault.remove(&target) {
                    Ok(()) => {
                        self.refresh();
                        self.status = Some(Status {
                            text: match lang {
                                Lang::Ru => format!("Удалено: {target}"),
                                Lang::En => format!("Removed: {target}"),
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
    }
}

fn row(ui: &mut egui::Ui, lang: Lang, entry: &ListEntry, selected: bool) -> bool {
    let mut open = false;
    let frame = if selected {
        Frame::new()
            .fill(Color32::from_rgb(26, 37, 52))
            .corner_radius(9)
            .inner_margin(Margin::symmetric(8, 6))
    } else {
        Frame::new()
            .fill(Color32::TRANSPARENT)
            .corner_radius(9)
            .inner_margin(Margin::symmetric(8, 6))
    };
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (mark, color) = match entry.kind {
                EntryKind::Dir => (lang.t("ПАПКА", "FOLDER"), VIOLET),
                EntryKind::File => (lang.t("ФАЙЛ", "FILE"), ACCENT),
            };
            ui.label(RichText::new(mark).size(10.0).strong().color(color));
            ui.add_space(6.0);
            let name = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            if ui
                .add(egui::Button::new(RichText::new(name).size(15.0).color(TEXT)).frame(false))
                .clicked()
                && entry.kind == EntryKind::Dir
            {
                open = true;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let size = match entry.kind {
                    EntryKind::Dir => "—".to_owned(),
                    EntryKind::File => human_size(entry.size),
                };
                ui.label(RichText::new(size).size(12.0).color(MUTED));
                if entry.kind == EntryKind::Dir {
                    ui.label(RichText::new("›").size(16.0).color(MUTED));
                }
            });
        });
    });
    open
}

fn chip(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(9)
        .inner_margin(Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(11.0).color(MUTED));
                ui.label(RichText::new(value).size(13.0).strong().color(accent));
            });
        });
}

fn hero_panel(ui: &mut egui::Ui, lang: Lang) {
    ui.allocate_ui_with_layout(
        egui::vec2(300.0, 520.0),
        Layout::top_down(Align::Min),
        |ui| {
            let (tile, _) = ui.allocate_exact_size(egui::vec2(58.0, 58.0), egui::Sense::hover());
            ui.painter().rect_filled(tile, 16.0, PANEL);
            let bar = egui::Rect::from_min_size(tile.min, egui::vec2(58.0, 3.0));
            gradient(ui.painter(), bar, ACCENT, VIOLET, 58);
            ui.painter().text(
                tile.center(),
                egui::Align2::CENTER_CENTER,
                "SP",
                FontId::new(21.0, FontFamily::Proportional),
                ACCENT,
            );

            ui.add_space(18.0);
            ui.label(RichText::new("SecretPiece").size(26.0).strong().color(TEXT));
            ui.add_space(4.0);
            ui.label(
                RichText::new(lang.t("Ваш секрет — в кусочках.", "Your secret, in pieces."))
                    .size(13.0)
                    .color(MUTED),
            );
            ui.add_space(22.0);
            bullet(
                ui,
                lang.t(
                    "Один обычный файл, без разметки диска",
                    "One ordinary file, no disk layout",
                ),
                ACCENT,
            );
            bullet(
                ui,
                lang.t(
                    "Argon2id · XChaCha20-Poly1305 · BLAKE3",
                    "Argon2id · XChaCha20-Poly1305 · BLAKE3",
                ),
                VIOLET,
            );
            bullet(
                ui,
                lang.t(
                    "Флешка, HDD/SSD, сетевая папка",
                    "USB drive, HDD/SSD, network folder",
                ),
                ACCENT,
            );
            bullet(
                ui,
                lang.t(
                    "Windows · macOS · Linux · Android",
                    "Windows · macOS · Linux · Android",
                ),
                VIOLET,
            );
            ui.add_space(24.0);
            Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(1.0_f32, BORDER))
                .corner_radius(9)
                .inner_margin(Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "v{} · SPV{}",
                            env!("CARGO_PKG_VERSION"),
                            sp_core::FORMAT_VERSION
                        ))
                        .size(11.0)
                        .color(ACCENT),
                    );
                });
        },
    );
}

fn bullet(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("•").size(14.0).color(color));
        ui.label(RichText::new(text).size(12.5).color(MUTED));
    });
    ui.add_space(6.0);
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
    ui.label(RichText::new(text).size(12.0).color(MUTED));
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
