//! The `SecretPiece` desktop application.
//!
//! `sp` and `secretpiece` start this GUI; the console interface lives in the
//! separate `sp-cli` binary. The design is a dark, material-inspired surface
//! with cyan and violet accents.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
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
    Sunset,
    Forest,
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
            Self::Sunset => Palette {
                bg: Color32::from_rgb(20, 11, 18),
                panel: Color32::from_rgb(29, 16, 24),
                card: Color32::from_rgb(42, 22, 34),
                line: Color32::from_rgb(74, 37, 54),
                ink: Color32::from_rgb(253, 232, 232),
                muted: Color32::from_rgb(201, 163, 173),
                accent: Color32::from_rgb(251, 146, 60),
                violet: Color32::from_rgb(244, 114, 182),
                danger: Color32::from_rgb(239, 68, 68),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(24, 13, 20),
                selected: Color32::from_rgb(58, 28, 43),
            },
            Self::Forest => Palette {
                bg: Color32::from_rgb(7, 18, 13),
                panel: Color32::from_rgb(12, 26, 19),
                card: Color32::from_rgb(19, 36, 26),
                line: Color32::from_rgb(35, 64, 47),
                ink: Color32::from_rgb(227, 245, 233),
                muted: Color32::from_rgb(155, 184, 166),
                accent: Color32::from_rgb(52, 211, 153),
                violet: Color32::from_rgb(163, 230, 53),
                danger: Color32::from_rgb(248, 113, 113),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(9, 23, 15),
                selected: Color32::from_rgb(23, 48, 31),
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
            (Self::Sunset, Lang::Ru) => "Закат",
            (Self::Sunset, Lang::En) => "Sunset",
            (Self::Forest, Lang::Ru) => "Лес",
            (Self::Forest, Lang::En) => "Forest",
        }
    }

    fn is_light(self) -> bool {
        matches!(self, Self::Light)
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Midnight => "midnight",
            Self::Neon => "neon",
            Self::Light => "light",
            Self::Sunset => "sunset",
            Self::Forest => "forest",
        }
    }

    fn from_key(key: &str) -> Self {
        match key {
            "neon" => Self::Neon,
            "light" => Self::Light,
            "sunset" => Self::Sunset,
            "forest" => Self::Forest,
            _ => Self::Midnight,
        }
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

/// Text colour that stays readable on top of the accent fill.
fn on_accent() -> Color32 {
    let p = pal().accent;
    let luminance = 0.299 * f32::from(p.r()) + 0.587 * f32::from(p.g()) + 0.114 * f32::from(p.b());
    if luminance > 150.0 {
        Color32::from_rgb(10, 14, 20)
    } else {
        Color32::WHITE
    }
}

fn set_theme(theme: Theme) {
    *PALETTE.write().unwrap_or_else(PoisonError::into_inner) = theme.palette();
}

/// How many random overwrite passes temporary preview files get on close.
static WIPE_PASSES: AtomicU32 = AtomicU32::new(3);

fn wipe_passes() -> u32 {
    WIPE_PASSES.load(Ordering::Relaxed).clamp(1, 7)
}

fn set_wipe_passes(passes: u32) {
    WIPE_PASSES.store(passes.clamp(1, 7), Ordering::Relaxed);
}

/// Overwrites a file with random bytes `passes` times, flushes and removes it.
fn wipe_temp(path: &Path, passes: u32) {
    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(path) {
        let len = file.metadata().map_or(0, |meta| meta.len());
        let mut buffer = vec![0_u8; 64 * 1024];
        for _ in 0..passes.max(1) {
            let _ = file.seek(SeekFrom::Start(0));
            let mut left = len;
            while left > 0 {
                let take = usize::try_from(left.min(buffer.len() as u64)).unwrap_or(buffer.len());
                if sp_crypto::random_into(&mut buffer[..take]).is_err() {
                    buffer[..take].fill(0xA5);
                }
                if file.write_all(&buffer[..take]).is_err() {
                    break;
                }
                left -= take as u64;
            }
            let _ = file.sync_all();
        }
    }
    let _ = std::fs::remove_file(path);
}

/// Folder for externally opened previews, removed as soon as it is empty.
fn preview_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("secretpiece-preview");
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    dir
}

/// Extensions that are always shown inside the app, never extracted.
fn is_text_name(name: &str) -> bool {
    const TEXT: &[&str] = &[
        "txt", "md", "markdown", "json", "jsonl", "xml", "csv", "tsv", "log", "ini", "cfg", "conf",
        "toml", "yaml", "yml", "rs", "py", "js", "ts", "tsx", "html", "htm", "css", "sh", "bat",
        "ps1", "c", "h", "cpp", "hpp", "go", "java", "kt", "swift", "rb", "php", "sql", "fb2",
        "srt", "vtt", "svg",
    ];
    name.rsplit('.')
        .next()
        .is_some_and(|ext| TEXT.contains(&ext.to_ascii_lowercase().as_str()))
}

/// Opens a file with the desktop default application.
fn open_with_system(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
    }
}

/// Pushes the active palette into egui's style, so every widget follows the theme.
fn apply_theme(ctx: &egui::Context) {
    let p = pal();
    let light = p.bg.r() > 128;
    ctx.set_theme(if light {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    });
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
    style.visuals.selection.stroke = Stroke::new(1.0_f32, p.accent);
    let soft_line = Color32::from_rgba_unmultiplied(p.muted.r(), p.muted.g(), p.muted.b(), 84);
    style.visuals.widgets.noninteractive.bg_fill = p.panel;
    style.visuals.widgets.noninteractive.weak_bg_fill = p.panel;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.muted);
    style.visuals.widgets.inactive.bg_fill = p.card;
    style.visuals.widgets.inactive.weak_bg_fill = p.card;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, soft_line);
    style.visuals.widgets.hovered.bg_fill = p.selected;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.accent);
    style.visuals.widgets.active.bg_fill = p.selected;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, p.accent);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    style.visuals.widgets.open.bg_fill = p.selected;
    style.visuals.widgets.open.weak_bg_fill = p.selected;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.line);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.ink);
    style.visuals.window_stroke = Stroke::new(1.0_f32, p.line);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.corner_radius = egui::CornerRadius::same(9);
    }
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(11.0, 7.0);
    style.spacing.interact_size.y = 30.0;
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
    ctx.set_style(style.clone());
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
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
    fn t(self, ru: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Ru => ru,
            Self::En => en,
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::En => "en",
        }
    }

    fn from_key(key: &str) -> Self {
        match key {
            "en" => Self::En,
            _ => Self::Ru,
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
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([720.0, 520.0])
            .with_resizable(true)
            .with_title("SecretPiece"),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "SecretPiece",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[allow(clippy::struct_excessive_bools)]
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
    info: Option<String>,
    browser: Option<Browser>,
    settings_open: bool,
    remote: Remote,
    rekey_open: bool,
    rekey_old: String,
    rekey_new: String,
    rekey_confirm: String,
}

/// SSH/SFTP settings for keeping a vault on a server.
#[derive(Clone, Default)]
struct Remote {
    host: String,
    port: String,
    user: String,
    path: String,
    status: Option<String>,
}

impl Remote {
    fn target(&self) -> Option<String> {
        let host = self.host.trim();
        let path = self.path.trim();
        if host.is_empty() || path.is_empty() {
            return None;
        }
        let user = self.user.trim();
        let host = if user.is_empty() {
            host.to_owned()
        } else {
            format!("{user}@{host}")
        };
        Some(format!("{host}:{path}"))
    }

    fn port(&self) -> String {
        let port = self.port.trim();
        if port.is_empty() {
            "22".to_owned()
        } else {
            port.to_owned()
        }
    }

    /// Runs `ssh`/`scp` with key or ssh-agent auth; passwords are never stored.
    fn run(program: &str, args: &[String]) -> Result<String, String> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("{program}: {error}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(stderr
                .trim()
                .lines()
                .last()
                .unwrap_or("ssh failed")
                .to_owned())
        }
    }
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
        let theme = cc
            .storage
            .and_then(|storage| storage.get_string("theme"))
            .map_or(Theme::Midnight, |key| Theme::from_key(&key));
        let lang = cc
            .storage
            .and_then(|storage| storage.get_string("lang"))
            .map_or(Lang::Ru, |key| Lang::from_key(&key));
        set_theme(theme);
        apply_theme(&cc.egui_ctx);
        let vault_path = std::env::args()
            .skip(1)
            .find(|argument| !argument.starts_with('-'))
            .unwrap_or_default();
        let wipe = cc
            .storage
            .and_then(|storage| storage.get_string("wipe_passes"))
            .and_then(|value| value.parse().ok())
            .unwrap_or(3_u32);
        set_wipe_passes(wipe);
        let remote = cc
            .storage
            .and_then(|storage| storage.get_string("remote"))
            .map(|raw| {
                let mut parts = raw.split('\u{1f}');
                Remote {
                    host: parts.next().unwrap_or_default().to_owned(),
                    port: parts.next().unwrap_or_default().to_owned(),
                    user: parts.next().unwrap_or_default().to_owned(),
                    path: parts.next().unwrap_or_default().to_owned(),
                    status: None,
                }
            })
            .unwrap_or_default();
        Self {
            lang,
            theme,
            vault_path,
            password: String::new(),
            confirm: String::new(),
            creating: false,
            show_password: false,
            kdf: KdfChoice::Default,
            error: None,
            info: None,
            browser: None,
            settings_open: false,
            remote,
            rekey_open: false,
            rekey_old: String::new(),
            rekey_new: String::new(),
            rekey_confirm: String::new(),
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

    /// Settings window: temp-file wipe passes and the SSH server tab.
    #[allow(clippy::too_many_lines)]
    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let lang = self.lang;
        let mut open = true;
        egui::Window::new(lang.t("Настройки", "Settings"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_width(540.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(lang.t("Временные файлы", "Temporary files"))
                        .size(13.0)
                        .strong()
                        .color(accent()),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(lang.t("Проходов затирания:", "Wipe passes:")).size(12.0));
                    let mut passes = wipe_passes();
                    if ui
                        .add(egui::DragValue::new(&mut passes).range(1..=7).speed(0.1))
                        .changed()
                    {
                        set_wipe_passes(passes);
                    }
                });
                ui.label(
                    RichText::new(lang.t(
                        "Файл для внешнего просмотра пишется во временную папку и после закрытия перезаписывается случайными байтами столько раз, затем удаляется.",
                        "A file opened externally is written to a temporary folder, then overwritten with random bytes this many times before deletion.",
                    ))
                    .size(10.5)
                    .color(muted()),
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(
                    RichText::new(lang.t("Сервер (SSH / SCP)", "Server (SSH / SCP)"))
                        .size(13.0)
                        .strong()
                        .color(accent()),
                );
                ui.label(
                    RichText::new(lang.t(
                        "Любой пользователь, root не нужен. Пароль не хранится: работает ключ или ssh-agent.",
                        "Any user, root is not required. No password is stored: key or ssh-agent auth only.",
                    ))
                    .size(10.5)
                    .color(muted()),
                );
                ui.add_space(6.0);
                egui::Grid::new("remote-grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new(lang.t("Хост", "Host")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.host)
                                .hint_text("example.com")
                                .desired_width(320.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Порт", "Port")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.port)
                                .hint_text("22")
                                .desired_width(80.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Пользователь", "User")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.user)
                                .hint_text("user")
                                .desired_width(200.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Путь к сейфу", "Vault path")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.path)
                                .hint_text("/home/user/vault.spv")
                                .desired_width(320.0),
                        );
                        ui.end_row();
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(lang.t("Проверить", "Test")).clicked() {
                        self.remote_test();
                    }
                    if ui.button(lang.t("Скачать", "Download")).clicked() {
                        self.remote_download();
                    }
                    if ui.button(lang.t("Загрузить", "Upload")).clicked() {
                        self.remote_upload();
                    }
                });
                if let Some(status) = &self.remote.status {
                    ui.add_space(4.0);
                    ui.label(RichText::new(status).size(11.0).color(muted()));
                }
            });
        self.settings_open = open;
    }

    fn remote_host_target(&self) -> Option<String> {
        let host = self.remote.host.trim();
        if host.is_empty() {
            return None;
        }
        let user = self.remote.user.trim();
        Some(if user.is_empty() {
            host.to_owned()
        } else {
            format!("{user}@{host}")
        })
    }

    fn remote_test(&mut self) {
        let Some(target) = self.remote_host_target() else {
            self.remote.status = Some(self.lang.t("Укажите хост.", "Enter a host.").to_owned());
            return;
        };
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=8".to_owned(),
            "-p".to_owned(),
            self.remote.port(),
            target,
            "echo ok".to_owned(),
        ];
        self.remote.status = Some(match Remote::run("ssh", &args) {
            Ok(_) => self
                .lang
                .t("Соединение работает.", "Connection works.")
                .to_owned(),
            Err(error) => format!("{}: {error}", self.lang.t("Ошибка SSH", "SSH error")),
        });
    }

    fn remote_download(&mut self) {
        let Some(target) = self.remote.target() else {
            self.remote.status = Some(
                self.lang
                    .t(
                        "Нужны хост и путь к сейфу.",
                        "Host and vault path are required.",
                    )
                    .to_owned(),
            );
            return;
        };
        let local = preview_dir().join(format!("remote-{}.spv", std::process::id()));
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=10".to_owned(),
            "-P".to_owned(),
            self.remote.port(),
            target,
            local.to_string_lossy().into_owned(),
        ];
        self.remote.status = Some(match Remote::run("scp", &args) {
            Ok(_) => {
                self.vault_path = local.to_string_lossy().into_owned();
                self.creating = false;
                self.error = None;
                self.lang
                    .t(
                        "Сейф скачан, введите пароль.",
                        "Vault downloaded, enter the password.",
                    )
                    .to_owned()
            }
            Err(error) => format!("{}: {error}", self.lang.t("Ошибка SCP", "SCP error")),
        });
    }

    fn remote_upload(&mut self) {
        let Some(target) = self.remote.target() else {
            self.remote.status = Some(
                self.lang
                    .t(
                        "Нужны хост и путь к сейфу.",
                        "Host and vault path are required.",
                    )
                    .to_owned(),
            );
            return;
        };
        let local = self.vault_path.trim().to_owned();
        if local.is_empty() || !Path::new(&local).exists() {
            self.remote.status = Some(
                self.lang
                    .t(
                        "Сначала откройте локальный сейф.",
                        "Open a local vault first.",
                    )
                    .to_owned(),
            );
            return;
        }
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=10".to_owned(),
            "-P".to_owned(),
            self.remote.port(),
            local,
            target,
        ];
        self.remote.status = Some(match Remote::run("scp", &args) {
            Ok(_) => self
                .lang
                .t("Сейф загружен на сервер.", "Vault uploaded to the server.")
                .to_owned(),
            Err(error) => format!("{}: {error}", self.lang.t("Ошибка SCP", "SCP error")),
        });
    }

    /// Rewrites the vault with a new password and Argon2id profile.
    fn rekey(&mut self) {
        let path = self.vault_path.trim().to_owned();
        if path.is_empty() {
            self.error = Some(self.lang.t("Укажите сейф.", "Choose a vault.").to_owned());
            return;
        }
        if self.rekey_new.chars().count() < 8 || self.rekey_new != self.rekey_confirm {
            self.error = Some(
                self.lang
                    .t(
                        "Новый пароль короче 8 символов или не совпадает.",
                        "The new password is shorter than 8 characters or does not match.",
                    )
                    .to_owned(),
            );
            return;
        }
        let path = Path::new(&path);
        let temp = path.with_extension("rekey");
        let backup = path.with_extension("rekey.bak");
        let _ = std::fs::remove_file(&temp);
        let _ = std::fs::remove_file(&backup);
        let result = (|| -> Result<(), SpError> {
            let mut old = Vault::open(path, self.rekey_old.as_bytes())?;
            let entries = old.list(None);
            let mut fresh = Vault::create(&temp, self.rekey_new.as_bytes(), self.kdf.params())?;
            for entry in entries.iter().filter(|entry| entry.kind == EntryKind::Dir) {
                fresh.mkdir(&entry.path)?;
            }
            for entry in entries.iter().filter(|entry| entry.kind == EntryKind::File) {
                let mut data = old.get(&entry.path)?;
                let put = fresh.put(&entry.path, &data);
                data.fill(0);
                put?;
            }
            fresh.close();
            old.close();
            Ok(())
        })();
        match result {
            Ok(()) => {
                let swapped =
                    std::fs::rename(path, &backup).is_ok() && std::fs::rename(&temp, path).is_ok();
                if swapped {
                    let _ = std::fs::remove_file(&backup);
                    self.error = None;
                    self.info = Some(
                        self.lang
                            .t(
                                "Сейф пересобран с новым паролем и профилем Argon2id.",
                                "Vault repacked with the new password and Argon2id profile.",
                            )
                            .to_owned(),
                    );
                    self.rekey_open = false;
                    self.rekey_old.clear();
                    self.rekey_new.clear();
                    self.rekey_confirm.clear();
                } else {
                    let _ = std::fs::rename(&backup, path);
                    let _ = std::fs::remove_file(&temp);
                    self.error = Some(
                        self.lang
                            .t(
                                "Не удалось заменить файл сейфа.",
                                "Could not replace the vault file.",
                            )
                            .to_owned(),
                    );
                }
            }
            Err(error) => {
                let _ = std::fs::remove_file(&temp);
                self.error = Some(describe(self.lang, &error));
            }
        }
    }

    fn rekey_window(&mut self, ctx: &egui::Context) {
        if !self.rekey_open {
            return;
        }
        let lang = self.lang;
        let mut open = true;
        egui::Window::new(lang.t("Сменить пароль / пересобрать", "Change password / repack"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(lang.t(
                        "Сейф расшифровывается и пересобирается с новым паролем и выбранным профилем Argon2id. Данные внутри сохраняются.",
                        "The vault is decrypted and repacked with a new password and the chosen Argon2id profile. The data inside is kept.",
                    ))
                    .size(11.0)
                    .color(muted()),
                );
                ui.add_space(6.0);
                egui::Grid::new("rekey-grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new(lang.t("Старый пароль", "Old password")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.rekey_old)
                                .password(true)
                                .desired_width(260.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Новый пароль", "New password")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.rekey_new)
                                .password(true)
                                .desired_width(260.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Повтор", "Repeat")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.rekey_confirm)
                                .password(true)
                                .desired_width(260.0),
                        );
                        ui.end_row();
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    for choice in [KdfChoice::Light, KdfChoice::Default, KdfChoice::Paranoid] {
                        let active = self.kdf == choice;
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(choice.label(lang)).size(11.0).color(if active {
                                        on_accent()
                                    } else {
                                        ink()
                                    }),
                                )
                                .fill(if active { accent() } else { input_bg() })
                                .corner_radius(7),
                            )
                            .clicked()
                        {
                            self.kdf = choice;
                        }
                    }
                });
                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(lang.t("Пересобрать", "Repack"))
                                .strong()
                                .color(on_accent()),
                        )
                        .fill(accent())
                        .corner_radius(8),
                    )
                    .clicked()
                {
                    self.rekey();
                }
            });
        self.rekey_open = open;
    }

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .exact_height(78.0)
            .frame(
                Frame::new()
                    .fill(panel())
                    .inner_margin(Margin::symmetric(24, 14)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.horizontal(|ui| {
                    logo_tile(ui, 42.0);
                    ui.add_space(11.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("SecretPiece")
                                .size(23.0)
                                .strong()
                                .color(ink()),
                        );
                        ui.label(
                            RichText::new(self.lang.t(
                                "зашифрованное хранилище · один файл",
                                "encrypted vault · a single file",
                            ))
                            .size(11.5)
                            .color(muted()),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.browser.is_some() {
                            let lock = egui::Button::new(
                                RichText::new(self.lang.t("Заблокировать", "Lock"))
                                    .size(11.5)
                                    .color(ink()),
                            )
                            .fill(card_bg())
                            .stroke(Stroke::new(1.0_f32, line()))
                            .corner_radius(9)
                            .min_size(egui::vec2(128.0, 30.0));
                            if ui.add(lock).clicked() {
                                self.browser = None;
                                self.password.clear();
                            }
                            ui.add_space(8.0);
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(self.lang.t("Настройки", "Settings"))
                                        .size(11.5)
                                        .color(ink()),
                                )
                                .fill(card_bg())
                                .stroke(Stroke::new(1.0_f32, line()))
                                .corner_radius(9)
                                .min_size(egui::vec2(92.0, 30.0)),
                            )
                            .clicked()
                        {
                            self.settings_open = true;
                        }
                        ui.add_space(12.0);
                        language_switch(ui, &mut self.lang);
                        ui.add_space(14.0);
                        let mut picked = None;
                        for theme in [
                            Theme::Light,
                            Theme::Sunset,
                            Theme::Forest,
                            Theme::Neon,
                            Theme::Midnight,
                        ] {
                            if theme_swatch(ui, theme, self.theme == theme, self.lang) {
                                picked = Some(theme);
                            }
                            ui.add_space(5.0);
                        }
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(self.lang.t("Тема", "Theme"))
                                .size(11.0)
                                .color(muted()),
                        );
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
                gradient(ui.painter(), line, accent(), violet(), 160);
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.header(ctx);
        if let Some(browser) = self.browser.as_mut() {
            let dropped = ctx.input(|input| input.raw.dropped_files.clone());
            if let Some(path) = dropped.iter().find_map(|file| file.path.clone()) {
                if path.is_file() {
                    browser.add_file(&path, self.lang);
                }
            }
        }
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(bg())
                    .inner_margin(Margin::symmetric(24, 18)),
            )
            .show(ctx, |ui| {
                paint_glow(ui);
                if let Some(browser) = self.browser.as_mut() {
                    browser.ui(ui, self.lang);
                } else {
                    self.unlock_ui(ui);
                }
            });
        self.settings(ctx);
        self.rekey_window(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("theme", self.theme.key().to_owned());
        storage.set_string("wipe_passes", wipe_passes().to_string());
        storage.set_string(
            "remote",
            format!(
                "{}\u{1f}{}\u{1f}{}\u{1f}{}",
                self.remote.host, self.remote.port, self.remote.user, self.remote.path
            ),
        );
        storage.set_string("lang", self.lang.key().to_owned());
    }
}

impl App {
    fn unlock_ui(&mut self, ui: &mut egui::Ui) {
        let card_width = (ui.available_width() - 24.0).clamp(330.0, 470.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    logo_tile(ui, 48.0);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("SecretPiece")
                            .size(28.0)
                            .strong()
                            .color(ink()),
                    );
                    ui.label(
                        RichText::new(self.lang.t(
                            "Ваш секрет — в кусочках. Один файл, ноль следов на диске.",
                            "Your secret, in pieces. One file, no disk layout.",
                        ))
                        .size(12.5)
                        .color(muted()),
                    );
                    ui.add_space(14.0);
                    self.form_card(ui, card_width);
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let total = 3.0 * 150.0 + 2.0 * 8.0;
                        ui.add_space((ui.available_width() - total).max(0.0) / 2.0);
                        for (label, color) in [
                            ("Argon2id", accent()),
                            ("XChaCha20-Poly1305", violet()),
                            ("BLAKE3", good()),
                        ] {
                            Frame::new()
                                .fill(card_bg())
                                .stroke(Stroke::new(1.0_f32, line()))
                                .corner_radius(8)
                                .inner_margin(Margin::symmetric(10, 5))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(label).size(10.5).color(color));
                                });
                            ui.add_space(8.0);
                        }
                    });
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(self.lang.t(
                            "Перетащите сейф в окно, чтобы открыть его.",
                            "Drop a vault onto the window to open it.",
                        ))
                        .size(11.0)
                        .color(muted()),
                    );
                });
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
    fn form_card(&mut self, ui: &mut egui::Ui, width: f32) {
        let lang = self.lang;
        let inner = width - 40.0;
        Frame::new()
            .fill(card_bg())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(22)
            .inner_margin(Margin::same(20))
            .shadow(egui::epaint::Shadow {
                offset: [0, 12],
                blur: 30,
                spread: 0,
                color: Color32::from_black_alpha(if self.theme.is_light() { 26 } else { 90 }),
            })
            .show(ui, |ui| {
                ui.set_width(inner);
                let (bar, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 3.0),
                    egui::Sense::hover(),
                );
                gradient(ui.painter(), bar, accent(), violet(), 120);
                ui.add_space(8.0);
                ui.label(
                    RichText::new(if self.creating {
                        lang.t("СОЗДАНИЕ", "CREATE")
                    } else {
                        lang.t("ВХОД", "SIGN IN")
                    })
                    .size(10.0)
                    .strong()
                    .color(accent()),
                );
                ui.add_space(2.0);
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
                ui.label(
                    RichText::new(lang.t(
                        "Пароль не покидает это устройство · Argon2id",
                        "Your password never leaves this device · Argon2id",
                    ))
                    .size(12.0)
                    .color(muted()),
                );
                ui.add_space(16.0);

                field_label(ui, lang.t("Файл хранилища", "Vault file"));
                ui.horizontal(|ui| {
                    let width = ui.available_width() - 96.0;
                    ui.add_sized(
                        [width, 38.0],
                        egui::TextEdit::singleline(&mut self.vault_path)
                            .hint_text("vault.spv")
                            .margin(egui::vec2(10.0, 9.0))
                            .background_color(input_bg()),
                    );
                    if secondary_button(ui, lang.t("Обзор…", "Browse…")).clicked() {
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
                ui.add_space(3.0);
                ui.label(
                    RichText::new(lang.t(
                        "Любой файл: .spv, .bin, .mp3, .pdf — внутри всё равно контейнер SPV1.",
                        "Any file: .spv, .bin, .mp3, .pdf — inside it is still an SPV1 container.",
                    ))
                    .size(10.5)
                    .color(muted()),
                );

                if self.creating {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(lang.t("Оболочка:", "Shell:"))
                            .size(11.0)
                            .color(muted()),
                    );
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for ext in ["spv", "bin", "mp3", "jpg", "pdf", "zip", "iso", "dat"] {
                            let active = self
                                .vault_path
                                .to_ascii_lowercase()
                                .ends_with(&format!(".{ext}"));
                            let button = egui::Button::new(
                                RichText::new(format!(".{ext}"))
                                    .size(11.0)
                                    .color(if active { on_accent() } else { muted() }),
                            )
                            .fill(if active { accent() } else { input_bg() })
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
                    let width = ui.available_width() - 82.0;
                    ui.add_sized(
                        [width, 38.0],
                        egui::TextEdit::singleline(&mut self.password)
                            .password(!self.show_password)
                            .margin(egui::vec2(10.0, 9.0))
                            .background_color(input_bg()),
                    );
                    let toggle = if self.show_password {
                        lang.t("Скрыть", "Hide")
                    } else {
                        lang.t("Показать", "Show")
                    };
                    if secondary_button(ui, toggle).clicked() {
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
                    ui.painter().rect_filled(rect, 3.0, line());
                    let ratio = f32::from(score) / 4.0;
                    let filled = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width() * ratio, rect.height()),
                    );
                    ui.painter().rect_filled(filled, 3.0, color);
                    ui.add_space(3.0);
                    ui.label(RichText::new(label).size(11.0).color(color));

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Стойкость ключа", "Key hardness"));
                    let gap = 6.0;
                    let button_width = (ui.available_width() - gap * 2.0) / 3.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for choice in [KdfChoice::Light, KdfChoice::Default, KdfChoice::Paranoid] {
                            let selected = self.kdf == choice;
                            let button = egui::Button::new(
                                RichText::new(choice.label(lang))
                                    .size(12.0)
                                    .color(if selected { on_accent() } else { ink() }),
                            )
                            .fill(if selected { accent() } else { input_bg() })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if selected { accent() } else { line() },
                            ))
                            .corner_radius(8)
                            .min_size(egui::vec2(button_width, 30.0));
                            if ui.add(button).clicked() {
                                self.kdf = choice;
                            }
                        }
                    });
                    ui.add_space(2.0);
                    ui.label(RichText::new(self.kdf.hint(lang)).size(10.5).color(muted()));

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Повторите пароль", "Repeat password"));
                    ui.add_sized(
                        [ui.available_width(), 38.0],
                        egui::TextEdit::singleline(&mut self.confirm)
                            .password(!self.show_password)
                            .margin(egui::vec2(10.0, 9.0))
                            .background_color(input_bg()),
                    );
                }

                ui.add_space(18.0);
                let ready = !self.creating
                    || (self.password.chars().count() >= 8 && self.password == self.confirm);
                let primary = if self.creating {
                    lang.t("Создать сейф", "Create vault")
                } else {
                    lang.t("Открыть", "Unlock")
                };
                let button = egui::Button::new(
                    RichText::new(primary)
                        .size(15.5)
                        .strong()
                        .color(on_accent()),
                )
                .fill(accent())
                .corner_radius(10)
                .min_size(egui::vec2(ui.available_width(), 44.0));
                if ui.add_enabled(ready, button).clicked() {
                    self.unlock();
                }
                ui.add_space(6.0);
                let switch = if self.creating {
                    lang.t("У меня уже есть хранилище", "I already have a vault")
                } else {
                    lang.t("Создать новое хранилище", "Create a new vault")
                };
                if ui
                    .add(
                        egui::Button::new(RichText::new(switch).size(12.0).color(violet()))
                            .frame(false),
                    )
                    .clicked()
                {
                    self.creating = !self.creating;
                    self.error = None;
                }
                if !self.creating && !self.vault_path.trim().is_empty() {
                    ui.add_space(2.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(
                                    lang.t(
                                        "Сменить пароль / пересобрать",
                                        "Change password / repack",
                                    ),
                                )
                                .size(11.0)
                                .color(muted()),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.rekey_open = true;
                        self.rekey_old.clear();
                        self.rekey_new.clear();
                        self.rekey_confirm.clear();
                    }
                }

                if let Some(info) = &self.info {
                    ui.add_space(10.0);
                    Frame::new()
                        .fill(card_bg())
                        .stroke(Stroke::new(1.0_f32, good()))
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(10, 7))
                        .show(ui, |ui| {
                            ui.label(RichText::new(info).size(12.0).color(good()));
                        });
                }

                if let Some(error) = &self.error {
                    ui.add_space(10.0);
                    Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(
                            danger().r(),
                            danger().g(),
                            danger().b(),
                            22,
                        ))
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(10, 7))
                        .show(ui, |ui| {
                            ui.label(RichText::new(error).size(12.0).color(danger()));
                        });
                }
            });
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
    pending_external: Option<String>,
    preview: Option<Preview>,
    external: Option<ExternalPreview>,
    shred_on_delete: bool,
    sort: SortBy,
    sort_desc: bool,
}

/// Sort order of the vault browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortBy {
    Name,
    Size,
    Date,
    Kind,
}

impl SortBy {
    fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Name, Lang::Ru) => "Имя",
            (Self::Name, Lang::En) => "Name",
            (Self::Size, Lang::Ru) => "Размер",
            (Self::Size, Lang::En) => "Size",
            (Self::Date, Lang::Ru) => "Дата",
            (Self::Date, Lang::En) => "Date",
            (Self::Kind, Lang::Ru) => "Тип",
            (Self::Kind, Lang::En) => "Kind",
        }
    }
}

/// A decrypted file written to a temporary folder and opened externally.
struct ExternalPreview {
    path: String,
    temp: PathBuf,
}

impl Drop for ExternalPreview {
    fn drop(&mut self) {
        wipe_temp(&self.temp, wipe_passes());
    }
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
            pending_external: None,
            preview: None,
            external: None,
            shred_on_delete: true,
            sort: SortBy::Name,
            sort_desc: false,
        };
        browser.refresh();
        browser
    }

    fn refresh(&mut self) {
        self.entries = self.vault.list(Some(&self.cwd));
        let sort = self.sort;
        let desc = self.sort_desc;
        self.entries.sort_by(|a, b| {
            let dirs = match (a.kind, b.kind) {
                (EntryKind::Dir, EntryKind::File) => std::cmp::Ordering::Less,
                (EntryKind::File, EntryKind::Dir) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            if dirs != std::cmp::Ordering::Equal {
                return dirs;
            }
            let order = match sort {
                SortBy::Name => a
                    .path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&a.path)
                    .to_lowercase()
                    .cmp(&b.path.rsplit('/').next().unwrap_or(&b.path).to_lowercase()),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Date => a.mtime.cmp(&b.mtime),
                SortBy::Kind => a.path.rsplit('.').next().cmp(&b.path.rsplit('.').next()),
            };
            if desc { order.reverse() } else { order }
        });
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

        ui.add_space(8.0);
        if let Some(status) = &self.status {
            let color = if status.error { danger() } else { good() };
            Frame::new()
                .fill(card_bg())
                .stroke(Stroke::new(1.0_f32, color))
                .corner_radius(8)
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.label(RichText::new(&status.text).size(12.5).color(color));
                });
        }

        self.dialogs(ui, lang);
    }

    #[allow(clippy::too_many_lines)]
    fn actions(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(lang.t("Добавить файл", "Add file"))
                            .size(13.0)
                            .strong()
                            .color(on_accent()),
                    )
                    .fill(accent())
                    .corner_radius(9)
                    .min_size(egui::vec2(0.0, 32.0)),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.add_file(&path, lang);
                }
            }
            if secondary_button(ui, lang.t("Новая папка", "New folder")).clicked() {
                self.new_folder = Some(String::new());
            }
            let selected = self.selected.clone();
            let selected_file = self.selected_file().map(ToOwned::to_owned);
            if ui
                .add_enabled(
                    selected_file.is_some(),
                    egui::Button::new(RichText::new(lang.t("Просмотр", "Preview")).size(13.0))
                        .fill(card_bg())
                        .stroke(Stroke::new(1.0_f32, line()))
                        .corner_radius(9)
                        .min_size(egui::vec2(0.0, 32.0)),
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
                    egui::Button::new(RichText::new(lang.t("Извлечь", "Extract")).size(13.0))
                        .fill(card_bg())
                        .stroke(Stroke::new(1.0_f32, line()))
                        .corner_radius(9)
                        .min_size(egui::vec2(0.0, 32.0)),
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
                    egui::Button::new(
                        RichText::new(lang.t("Удалить", "Delete"))
                            .size(13.0)
                            .color(danger()),
                    )
                    .fill(card_bg())
                    .stroke(Stroke::new(1.0_f32, line()))
                    .corner_radius(9)
                    .min_size(egui::vec2(0.0, 32.0)),
                )
                .clicked()
            {
                self.pending_delete = selected;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(lang.t("Проверить целостность", "Verify integrity"))
                                .size(12.5)
                                .color(accent()),
                        )
                        .fill(card_bg())
                        .stroke(Stroke::new(1.0_f32, accent()))
                        .corner_radius(9)
                        .min_size(egui::vec2(0.0, 32.0)),
                    )
                    .on_hover_text(lang.t(
                        "Расшифровать и проверить каждый файл",
                        "Decrypt and authenticate every file",
                    ))
                    .clicked()
                {
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
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(lang.t(
                        "или перетащите файлы в окно",
                        "or drop files onto the window",
                    ))
                    .size(10.5)
                    .color(muted()),
                );
                ui.add_space(10.0);
                let mut changed = false;
                let arrow = if self.sort_desc { "↓" } else { "↑" };
                ui.menu_button(
                    RichText::new(format!("{} {arrow}", self.sort.label(lang))).size(11.5),
                    |ui| {
                        for by in [SortBy::Name, SortBy::Size, SortBy::Date, SortBy::Kind] {
                            if ui
                                .selectable_label(self.sort == by, by.label(lang))
                                .clicked()
                            {
                                if self.sort == by {
                                    self.sort_desc = !self.sort_desc;
                                } else {
                                    self.sort = by;
                                }
                                changed = true;
                            }
                        }
                    },
                );
                if changed {
                    self.refresh();
                }
            });
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
        let name = path.rsplit('/').next().unwrap_or(path).to_owned();
        match self.vault.get(path) {
            Ok(mut data) => {
                if is_text_name(&name) || preview_text(&data).is_some() {
                    self.preview = Some(Preview {
                        path: path.to_owned(),
                        data,
                        hex: false,
                    });
                } else {
                    data.fill(0);
                    self.pending_external = Some(path.to_owned());
                }
            }
            Err(error) => {
                self.status = Some(Status {
                    text: describe(lang, &error),
                    error: true,
                });
            }
        }
    }

    fn open_external(&mut self, path: &str, lang: Lang) {
        let name = path.rsplit('/').next().unwrap_or(path);
        let safe: String = name
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                    ch
                } else {
                    '_'
                }
            })
            .collect();
        let temp = preview_dir().join(format!("{}-{safe}", std::process::id()));
        match self.vault.get(path) {
            Ok(mut data) => {
                let result = std::fs::write(&temp, &data).and_then(|()| open_with_system(&temp));
                data.fill(0);
                match result {
                    Ok(()) => {
                        self.external = Some(ExternalPreview {
                            path: path.to_owned(),
                            temp,
                        });
                        self.status = Some(Status {
                            text: match lang {
                                Lang::Ru => format!(
                                    "Открыто во временной папке, после закрытия файл затирается {} раз(а).",
                                    wipe_passes()
                                ),
                                Lang::En => format!(
                                    "Opened in a temporary folder; it is wiped {} time(s) on close.",
                                    wipe_passes()
                                ),
                            },
                            error: false,
                        });
                    }
                    Err(error) => {
                        self.status = Some(Status {
                            text: format!(
                                "{}: {error}",
                                lang.t("Не удалось открыть файл", "Cannot open the file")
                            ),
                            error: true,
                        });
                    }
                }
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
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
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
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
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
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
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
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
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

        if let Some(path) = self.pending_external.clone() {
            let mut confirmed = false;
            let mut cancelled = false;
            egui::Window::new(lang.t("Открыть вне хранилища?", "Open outside the vault?"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(460.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Файл будет расшифрован во временную папку и открыт системным приложением. Пока он открыт, копия лежит на диске.",
                            "The file will be decrypted into a temporary folder and opened with the system app. While it is open, a copy sits on disk.",
                        ))
                        .size(12.5)
                        .color(ink()),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(match lang {
                            Lang::Ru => format!(
                                "После закрытия копия затирается случайными байтами {} раз(а) и удаляется. Число проходов меняется в настройках.",
                                wipe_passes()
                            ),
                            Lang::En => format!(
                                "On close the copy is overwritten with random bytes {} time(s) and deleted. Passes are configurable in settings.",
                                wipe_passes()
                            ),
                        })
                        .size(11.0)
                        .color(muted()),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Открыть", "Open"))
                                        .strong()
                                        .color(on_accent()),
                                )
                                .fill(accent())
                                .corner_radius(8),
                            )
                            .clicked()
                        {
                            confirmed = true;
                        }
                        if ui.button(lang.t("Отмена", "Cancel")).clicked() {
                            cancelled = true;
                        }
                    });
                });
            if confirmed {
                self.open_external(&path, lang);
                self.pending_external = None;
            } else if cancelled {
                self.pending_external = None;
            }
        }

        if let Some(external_path) = self.external.as_ref().map(|external| external.path.clone()) {
            let mut wipe_now = false;
            Frame::new()
                .fill(card_bg())
                .stroke(Stroke::new(1.0_f32, accent()))
                .corner_radius(8)
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(match lang {
                                Lang::Ru => format!(
                                    "Временно открыт: {} ({} проход(ов) при закрытии)",
                                    external_path,
                                    wipe_passes()
                                ),
                                Lang::En => format!(
                                    "Open temporarily: {} ({} pass(es) on close)",
                                    external_path,
                                    wipe_passes()
                                ),
                            })
                            .size(11.5)
                            .color(ink()),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(lang.t("Затереть сейчас", "Wipe now")).clicked()
                            {
                                wipe_now = true;
                            }
                        });
                    });
                });
            if wipe_now {
                self.external = None;
                self.status = Some(Status {
                    text: lang
                        .t(
                            "Временная копия затёрта и удалена.",
                            "Temporary copy wiped and deleted.",
                        )
                        .to_owned(),
                    error: false,
                });
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

fn logo_tile(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, size * 0.3, accent());
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "SP",
        FontId::new(size * 0.34, FontFamily::Proportional),
        Color32::WHITE,
    );
}

fn theme_swatch(ui: &mut egui::Ui, theme: Theme, active: bool, lang: Lang) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
    let palette = theme.palette();
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, palette.bg);
    painter.circle_filled(
        egui::pos2(rect.left() + 8.0, rect.top() + 8.0),
        4.0,
        palette.accent,
    );
    painter.circle_filled(
        egui::pos2(rect.right() - 8.0, rect.bottom() - 8.0),
        4.0,
        palette.violet,
    );
    painter.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0_f32, palette.line),
        egui::StrokeKind::Inside,
    );
    if active {
        painter.rect_stroke(
            rect.expand(2.0),
            8.0,
            Stroke::new(2.0_f32, palette.accent),
            egui::StrokeKind::Outside,
        );
    } else if response.hovered() {
        painter.rect_stroke(
            rect.expand(1.0),
            7.0,
            Stroke::new(1.0_f32, line()),
            egui::StrokeKind::Outside,
        );
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text(theme.label(lang)).clicked()
}

fn language_switch(ui: &mut egui::Ui, lang: &mut Lang) {
    Frame::new()
        .fill(card_bg())
        .stroke(Stroke::new(1.0_f32, line()))
        .corner_radius(9)
        .inner_margin(Margin::symmetric(3, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (value, code) in [(Lang::Ru, "RU"), (Lang::En, "EN")] {
                    let active = *lang == value;
                    let button = egui::Button::new(
                        RichText::new(code).size(11.5).strong().color(if active {
                            bg()
                        } else {
                            muted()
                        }),
                    )
                    .fill(if active {
                        accent()
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(Stroke::NONE)
                    .corner_radius(6)
                    .min_size(egui::vec2(34.0, 24.0));
                    if ui.add(button).clicked() {
                        *lang = value;
                    }
                }
            });
        });
}

fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(ink()))
            .fill(card_bg())
            .stroke(Stroke::new(1.0_f32, line()))
            .corner_radius(9)
            .min_size(egui::vec2(0.0, 30.0)),
    )
}

/// Soft radial glow built as a triangle fan, so there are no visible rings.
fn radial_glow(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
    max_alpha: u8,
) {
    const SEGMENTS: u8 = 64;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(
        center,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), max_alpha),
    );
    for index in 0..=SEGMENTS {
        let angle = std::f32::consts::TAU * f32::from(index) / f32::from(SEGMENTS);
        let position = center + egui::vec2(angle.cos(), angle.sin()) * radius;
        mesh.colored_vertex(position, Color32::TRANSPARENT);
    }
    for index in 0..SEGMENTS {
        mesh.add_triangle(0, u32::from(index) + 1, u32::from(index) + 2);
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Soft corner glows that sit behind the content.
fn paint_glow(ui: &egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let light = pal().bg.r() > 128;
    let accent_alpha = if light { 10 } else { 34 };
    let violet_alpha = if light { 8 } else { 28 };
    let accent_color = accent();
    let violet_color = violet();
    radial_glow(
        painter,
        egui::pos2(rect.left() + 40.0, rect.top() - 60.0),
        rect.width().max(560.0) * 0.7,
        accent_color,
        accent_alpha,
    );
    radial_glow(
        painter,
        egui::pos2(rect.right() - 30.0, rect.bottom() + 70.0),
        rect.width().max(560.0) * 0.65,
        violet_color,
        violet_alpha,
    );
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
    fn language_keys_roundtrip() {
        assert_eq!(Lang::from_key("ru"), Lang::Ru);
        assert_eq!(Lang::from_key("en"), Lang::En);
        assert_eq!(Lang::from_key("xx"), Lang::Ru);
        assert_eq!(Lang::En.key(), "en");
    }

    #[test]
    fn theme_keys_roundtrip() {
        assert_eq!(Theme::from_key("neon"), Theme::Neon);
        assert_eq!(Theme::from_key("light"), Theme::Light);
        assert_eq!(Theme::from_key("midnight"), Theme::Midnight);
        assert_eq!(Theme::from_key("sunset"), Theme::Sunset);
        assert_eq!(Theme::from_key("forest"), Theme::Forest);
        assert_eq!(Theme::Neon.key(), "neon");
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
    fn text_files_stay_in_the_app() {
        assert!(is_text_name("notes.md"));
        assert!(is_text_name("book.fb2"));
        assert!(is_text_name("data.JSON"));
        assert!(!is_text_name("book.epub"));
        assert!(!is_text_name("photo.jpg"));
        assert!(!is_text_name("movie.mp4"));
    }

    #[test]
    fn disguise_swaps_only_the_extension() {
        assert_eq!(swap_extension("a", "bin"), "a.bin");
        assert_eq!(swap_extension("", "jpg"), "vault.jpg");
        assert_eq!(
            swap_extension("D:/my.vaults/secret", "spv"),
            "D:/my.vaults/secret.spv"
        );
    }
}
