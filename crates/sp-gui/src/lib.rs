//! The `SecretPiece` desktop application.
//!
//! `sp` and `secretpiece` start this GUI; the console interface lives in the
//! separate `sp-cli` binary. The design is a dark, material-inspired surface
//! with cyan and violet accents.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{PoisonError, RwLock};

use eframe::egui;
use egui::{
    Align, Color32, FontFamily, FontId, Frame, Layout, Margin, RichText, Stroke, TextStyle,
};
use sp_core::{ErrorCode, SpError};
use sp_crypto::KdfParams;
use sp_crypto::shamir::{self, Share};
use sp_format::{EntryKind, HEADER_SIZE, Header};
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

impl Palette {
    #[allow(clippy::too_many_arguments)]
    const fn make(
        bg: u8,
        panel: u8,
        card: u8,
        line: u8,
        ink: u8,
        muted: u8,
        accent: (u8, u8, u8),
        violet: (u8, u8, u8),
        danger: (u8, u8, u8),
        good: (u8, u8, u8),
    ) -> Self {
        Self {
            bg: Color32::from_rgb(bg, bg, bg),
            panel: Color32::from_rgb(panel, panel, panel),
            card: Color32::from_rgb(card, card, card),
            line: Color32::from_rgb(line, line, line),
            ink: Color32::from_rgb(ink, ink, ink),
            muted: Color32::from_rgb(muted, muted, muted),
            accent: Color32::from_rgb(accent.0, accent.1, accent.2),
            violet: Color32::from_rgb(violet.0, violet.1, violet.2),
            danger: Color32::from_rgb(danger.0, danger.1, danger.2),
            good: Color32::from_rgb(good.0, good.1, good.2),
            input: Color32::from_rgb(
                bg.saturating_sub(2),
                bg.saturating_sub(2),
                bg.saturating_sub(2),
            ),
            selected: Color32::from_rgb(
                panel.saturating_add(8),
                panel.saturating_add(8),
                panel.saturating_add(8),
            ),
        }
    }
}

/// Visual theme offered in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Midnight,
    Neon,
    Light,
    Paper,
    Mint,
    Sunset,
    Forest,
    Sepia,
    Ocean,
    Rose,
    Aurora,
    Cyber,
    Coal,
    Cream,
    Lavender,
    Honey,
    Winter,
    Desert,
    Storm,
    Emerald,
    Ruby,
    Amber,
    Slate,
    Orchid,
    Coffee,
    Meadow,
    Coral,
    Indigo,
    Sand,
    Frost,
    Ember,
    Lilac,
    Olive,
    Sky,
    Plum,
    Copper,
    Charcoal,
    Pearl,
    Cobalt,
    Velvet,
}

impl Theme {
    #[allow(clippy::too_many_lines)]
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
            Self::Paper => Palette {
                bg: Color32::from_rgb(250, 247, 242),
                panel: Color32::from_rgb(255, 253, 249),
                card: Color32::from_rgb(255, 255, 255),
                line: Color32::from_rgb(229, 221, 209),
                ink: Color32::from_rgb(45, 38, 30),
                muted: Color32::from_rgb(120, 108, 94),
                accent: Color32::from_rgb(180, 83, 9),
                violet: Color32::from_rgb(124, 58, 237),
                danger: Color32::from_rgb(185, 28, 28),
                good: Color32::from_rgb(21, 128, 61),
                input: Color32::from_rgb(252, 249, 244),
                selected: Color32::from_rgb(243, 232, 216),
            },
            Self::Mint => Palette {
                bg: Color32::from_rgb(240, 250, 245),
                panel: Color32::from_rgb(255, 255, 255),
                card: Color32::from_rgb(255, 255, 255),
                line: Color32::from_rgb(199, 228, 214),
                ink: Color32::from_rgb(16, 48, 38),
                muted: Color32::from_rgb(82, 116, 102),
                accent: Color32::from_rgb(5, 150, 105),
                violet: Color32::from_rgb(14, 116, 144),
                danger: Color32::from_rgb(190, 30, 45),
                good: Color32::from_rgb(22, 163, 74),
                input: Color32::from_rgb(245, 253, 249),
                selected: Color32::from_rgb(214, 244, 229),
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
            Self::Sepia => Palette {
                bg: Color32::from_rgb(29, 23, 16),
                panel: Color32::from_rgb(42, 32, 23),
                card: Color32::from_rgb(51, 40, 29),
                line: Color32::from_rgb(90, 70, 50),
                ink: Color32::from_rgb(246, 235, 220),
                muted: Color32::from_rgb(194, 170, 142),
                accent: Color32::from_rgb(232, 184, 120),
                violet: Color32::from_rgb(216, 154, 106),
                danger: Color32::from_rgb(255, 138, 106),
                good: Color32::from_rgb(163, 230, 53),
                input: Color32::from_rgb(23, 18, 12),
                selected: Color32::from_rgb(64, 50, 37),
            },
            Self::Ocean => Palette {
                bg: Color32::from_rgb(4, 18, 28),
                panel: Color32::from_rgb(7, 32, 51),
                card: Color32::from_rgb(10, 43, 66),
                line: Color32::from_rgb(24, 74, 102),
                ink: Color32::from_rgb(226, 244, 255),
                muted: Color32::from_rgb(163, 198, 216),
                accent: Color32::from_rgb(93, 214, 255),
                violet: Color32::from_rgb(124, 156, 255),
                danger: Color32::from_rgb(255, 126, 126),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(5, 24, 37),
                selected: Color32::from_rgb(14, 56, 84),
            },
            Self::Rose => Palette {
                bg: Color32::from_rgb(30, 14, 21),
                panel: Color32::from_rgb(44, 20, 32),
                card: Color32::from_rgb(56, 26, 41),
                line: Color32::from_rgb(100, 48, 70),
                ink: Color32::from_rgb(255, 235, 242),
                muted: Color32::from_rgb(224, 182, 198),
                accent: Color32::from_rgb(255, 158, 196),
                violet: Color32::from_rgb(255, 193, 120),
                danger: Color32::from_rgb(255, 117, 136),
                good: Color32::from_rgb(74, 222, 128),
                input: Color32::from_rgb(24, 11, 17),
                selected: Color32::from_rgb(71, 34, 52),
            },
            Self::Aurora => Palette::make(
                8,
                14,
                22,
                36,
                226,
                150,
                (95, 246, 200),
                (138, 123, 255),
                (255, 122, 144),
                (74, 222, 128),
            ),
            Self::Cyber => Palette::make(
                8,
                17,
                26,
                42,
                230,
                170,
                (0, 229, 255),
                (255, 46, 136),
                (255, 99, 132),
                (74, 222, 128),
            ),
            Self::Coal => Palette::make(
                16,
                28,
                40,
                58,
                242,
                180,
                (192, 192, 192),
                (138, 138, 255),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Cream => Palette::make(
                250,
                255,
                255,
                220,
                42,
                107,
                (161, 98, 43),
                (124, 92, 58),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Lavender => Palette::make(
                243,
                255,
                248,
                214,
                36,
                100,
                (109, 74, 255),
                (168, 85, 247),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Honey => Palette::make(
                255,
                253,
                245,
                220,
                44,
                110,
                (145, 107, 0),
                (122, 91, 18),
                (198, 40, 40),
                (22, 163, 74),
            ),
            Self::Winter => Palette::make(
                234,
                255,
                248,
                205,
                16,
                85,
                (14, 107, 168),
                (61, 109, 181),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Desert => Palette::make(
                42,
                59,
                75,
                110,
                247,
                200,
                (232, 184, 120),
                (204, 142, 91),
                (255, 138, 106),
                (163, 230, 53),
            ),
            Self::Storm => Palette::make(
                11,
                21,
                33,
                52,
                226,
                170,
                (159, 179, 200),
                (108, 142, 191),
                (255, 126, 126),
                (74, 222, 128),
            ),
            Self::Emerald => Palette::make(
                4,
                10,
                18,
                36,
                220,
                150,
                (52, 211, 153),
                (110, 231, 183),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Ruby => Palette::make(
                26,
                44,
                60,
                120,
                255,
                200,
                (255, 107, 129),
                (255, 179, 107),
                (255, 117, 136),
                (74, 222, 128),
            ),
            Self::Amber => Palette::make(
                26,
                42,
                58,
                110,
                254,
                200,
                (251, 191, 36),
                (245, 158, 11),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Slate => Palette::make(
                15,
                30,
                45,
                72,
                241,
                170,
                (148, 163, 184),
                (129, 140, 248),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Orchid => Palette::make(
                26,
                44,
                60,
                120,
                247,
                190,
                (233, 168, 255),
                (167, 139, 250),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Coffee => Palette::make(
                31,
                50,
                68,
                110,
                245,
                190,
                (214, 162, 122),
                (168, 123, 93),
                (255, 138, 106),
                (163, 230, 53),
            ),
            Self::Meadow => Palette::make(
                237,
                255,
                246,
                210,
                20,
                85,
                (46, 125, 50),
                (85, 139, 47),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Coral => Palette::make(
                27,
                46,
                62,
                110,
                255,
                190,
                (255, 138, 122),
                (255, 180, 160),
                (255, 117, 136),
                (74, 222, 128),
            ),
            Self::Indigo => Palette::make(
                8,
                18,
                30,
                56,
                232,
                170,
                (140, 158, 255),
                (199, 210, 254),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Sand => Palette::make(
                250,
                255,
                246,
                218,
                46,
                110,
                (141, 110, 60),
                (109, 90, 50),
                (198, 40, 40),
                (22, 163, 74),
            ),
            Self::Frost => Palette::make(
                230,
                255,
                246,
                210,
                12,
                85,
                (2, 119, 189),
                (0, 131, 143),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Ember => Palette::make(
                26,
                44,
                60,
                110,
                255,
                190,
                (255, 112, 67),
                (255, 171, 64),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Lilac => Palette::make(
                245,
                255,
                248,
                214,
                39,
                100,
                (126, 87, 194),
                (149, 117, 205),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Olive => Palette::make(
                20,
                34,
                48,
                78,
                232,
                160,
                (174, 213, 129),
                (156, 204, 101),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Sky => Palette::make(
                232,
                255,
                246,
                210,
                13,
                85,
                (2, 136, 209),
                (92, 107, 192),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Plum => Palette::make(
                28,
                47,
                62,
                120,
                252,
                190,
                (240, 98, 146),
                (186, 104, 200),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Copper => Palette::make(
                27,
                46,
                62,
                110,
                248,
                190,
                (230, 138, 91),
                (199, 124, 60),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Charcoal => Palette::make(
                10,
                23,
                36,
                56,
                236,
                170,
                (176, 190, 197),
                (144, 164, 174),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Pearl => Palette::make(
                247,
                255,
                252,
                220,
                28,
                90,
                (69, 90, 100),
                (92, 107, 192),
                (220, 38, 38),
                (22, 163, 74),
            ),
            Self::Cobalt => Palette::make(
                4,
                11,
                20,
                40,
                227,
                160,
                (68, 138, 255),
                (124, 77, 255),
                (248, 113, 113),
                (74, 222, 128),
            ),
            Self::Velvet => Palette::make(
                23,
                40,
                56,
                110,
                255,
                190,
                (255, 119, 194),
                (179, 136, 255),
                (248, 113, 113),
                (74, 222, 128),
            ),
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
            (Self::Paper, Lang::Ru) => "Бумага",
            (Self::Paper, Lang::En) => "Paper",
            (Self::Mint, Lang::Ru) => "Мята",
            (Self::Mint, Lang::En) => "Mint",
            (Self::Sunset, Lang::Ru) => "Закат",
            (Self::Sunset, Lang::En) => "Sunset",
            (Self::Forest, Lang::Ru) => "Лес",
            (Self::Forest, Lang::En) => "Forest",
            (Self::Sepia, Lang::Ru) => "Сепия",
            (Self::Sepia, Lang::En) => "Sepia",
            (Self::Ocean, Lang::Ru) => "Океан",
            (Self::Ocean, Lang::En) => "Ocean",
            (Self::Rose, Lang::Ru) => "Роза",
            (Self::Rose, Lang::En) => "Rose",
            _ => self.key(),
        }
    }

    fn is_light(self) -> bool {
        matches!(self, Self::Light | Self::Paper | Self::Mint)
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Midnight => "midnight",
            Self::Neon => "neon",
            Self::Light => "light",
            Self::Paper => "paper",
            Self::Mint => "mint",
            Self::Sunset => "sunset",
            Self::Forest => "forest",
            Self::Sepia => "sepia",
            Self::Ocean => "ocean",
            Self::Rose => "rose",
            Self::Aurora => "aurora",
            Self::Cyber => "cyber",
            Self::Coal => "coal",
            Self::Cream => "cream",
            Self::Lavender => "lavender",
            Self::Honey => "honey",
            Self::Winter => "winter",
            Self::Desert => "desert",
            Self::Storm => "storm",
            Self::Emerald => "emerald",
            Self::Ruby => "ruby",
            Self::Amber => "amber",
            Self::Slate => "slate",
            Self::Orchid => "orchid",
            Self::Coffee => "coffee",
            Self::Meadow => "meadow",
            Self::Coral => "coral",
            Self::Indigo => "indigo",
            Self::Sand => "sand",
            Self::Frost => "frost",
            Self::Ember => "ember",
            Self::Lilac => "lilac",
            Self::Olive => "olive",
            Self::Sky => "sky",
            Self::Plum => "plum",
            Self::Copper => "copper",
            Self::Charcoal => "charcoal",
            Self::Pearl => "pearl",
            Self::Cobalt => "cobalt",
            Self::Velvet => "velvet",
        }
    }

    fn from_key(key: &str) -> Self {
        match key {
            "neon" => Self::Neon,
            "light" => Self::Light,
            "paper" => Self::Paper,
            "mint" => Self::Mint,
            "sunset" => Self::Sunset,
            "forest" => Self::Forest,
            "aurora" => Self::Aurora,
            "cyber" => Self::Cyber,
            "coal" => Self::Coal,
            "cream" => Self::Cream,
            "lavender" => Self::Lavender,
            "honey" => Self::Honey,
            "winter" => Self::Winter,
            "desert" => Self::Desert,
            "storm" => Self::Storm,
            "emerald" => Self::Emerald,
            "ruby" => Self::Ruby,
            "amber" => Self::Amber,
            "slate" => Self::Slate,
            "orchid" => Self::Orchid,
            "coffee" => Self::Coffee,
            "meadow" => Self::Meadow,
            "coral" => Self::Coral,
            "indigo" => Self::Indigo,
            "sand" => Self::Sand,
            "frost" => Self::Frost,
            "ember" => Self::Ember,
            "lilac" => Self::Lilac,
            "olive" => Self::Olive,
            "sky" => Self::Sky,
            "plum" => Self::Plum,
            "copper" => Self::Copper,
            "charcoal" => Self::Charcoal,
            "pearl" => Self::Pearl,
            "cobalt" => Self::Cobalt,
            "velvet" => Self::Velvet,
            "sepia" => Self::Sepia,
            "ocean" => Self::Ocean,
            "rose" => Self::Rose,
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

/// Optional per-vault folder for files extracted for external viewing.
static PREVIEW_ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);

fn set_preview_root(root: &str) {
    let trimmed = root.trim();
    let value = if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    };
    *PREVIEW_ROOT.write().unwrap_or_else(PoisonError::into_inner) = value;
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
    let custom = PREVIEW_ROOT
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let dir = custom.unwrap_or_else(|| std::env::temp_dir().join("secretpiece-preview"));
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

/// Tabular files that get a grid view in the preview window.
fn is_table_name(name: &str) -> bool {
    name.rsplit('.')
        .next()
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "csv" | "tsv"))
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

/// Restores a damaged header slot from the healthy one (the two alternating
/// superblocks are what make a partially written container recoverable).
fn repair_headers(path: &Path) -> Result<String, String> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut slots = [[0_u8; HEADER_SIZE]; 2];
    for (index, slot) in slots.iter_mut().enumerate() {
        file.seek(SeekFrom::Start(index as u64 * HEADER_SIZE as u64))
            .map_err(|error| error.to_string())?;
        file.read_exact(slot)
            .map_err(|error| format!("header slot {index}: {error}"))?;
    }
    let good = [
        Header::decode(&slots[0]).is_ok(),
        Header::decode(&slots[1]).is_ok(),
    ];
    match good {
        [true, false] | [false, true] => {
            let broken = usize::from(!good[0]);
            let healthy = 1 - broken;
            file.seek(SeekFrom::Start(broken as u64 * HEADER_SIZE as u64))
                .map_err(|error| error.to_string())?;
            file.write_all(&slots[healthy])
                .map_err(|error| error.to_string())?;
            file.sync_all().map_err(|error| error.to_string())?;
            Ok(format!("header slot {broken} restored from slot {healthy}"))
        }
        [true, true] => Ok("both headers are already healthy".to_owned()),
        _ => Err("both headers are damaged - cannot auto-repair".to_owned()),
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
    publish: bool,
    show_password: bool,
    kdf: KdfChoice,
    error: Option<String>,
    info: Option<String>,
    browser: Option<Browser>,
    settings_open: bool,
    server_open: bool,
    help_open: bool,
    recovery: Recovery,
    verify_status: Option<String>,
    presets: Vec<Preset>,
    preset_name: String,
    new_preset_name: String,
    show_presets: bool,
    remember_password: bool,
    auto_sync: bool,
    background_io: bool,
    remote_session: Option<PathBuf>,
    job: Option<Job>,
    lock_wipe: Option<PathBuf>,
    remote: Remote,
    rekey_open: bool,
    rekey_old: String,
    rekey_new: String,
    rekey_confirm: String,
}

/// A remote operation that runs on a worker thread so the UI never blocks.
enum JobKind {
    Test,
    Download(PathBuf),
    Upload,
}

struct Job {
    rx: std::sync::mpsc::Receiver<Result<String, String>>,
    kind: JobKind,
}

/// Where a vault is published: plain SSH/SCP or an FTP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RemoteKind {
    #[default]
    Ssh,
    Ftp,
}

impl RemoteKind {
    const fn key(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Ftp => "ftp",
        }
    }

    const fn default_port(self) -> &'static str {
        match self {
            Self::Ssh => "22",
            Self::Ftp => "21",
        }
    }
}

/// Remote destination settings. The password lives only in memory.
#[derive(Clone, Default)]
struct Remote {
    kind: RemoteKind,
    host: String,
    port: String,
    user: String,
    password: String,
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
            self.kind.default_port().to_owned()
        } else {
            port.to_owned()
        }
    }

    fn ready(&self) -> bool {
        !self.host.trim().is_empty() && !self.path.trim().is_empty()
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

    fn ssh_test(&self) -> Result<String, String> {
        if self.host.trim().is_empty() {
            return Err("host is empty".to_owned());
        }
        let target = self.host_target();
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=3".to_owned(),
            "-p".to_owned(),
            self.port(),
            target,
            "echo ok".to_owned(),
        ];
        Self::run("ssh", &args)
    }

    fn host_target(&self) -> String {
        let host = self.host.trim();
        let user = self.user.trim();
        if user.is_empty() {
            host.to_owned()
        } else {
            format!("{user}@{host}")
        }
    }

    fn ssh_upload(&self, local: &Path) -> Result<String, String> {
        let Some(target) = self.target() else {
            return Err("host or path is empty".to_owned());
        };
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=3".to_owned(),
            "-P".to_owned(),
            self.port(),
            local.to_string_lossy().into_owned(),
            target,
        ];
        Self::run("scp", &args)
    }

    fn ssh_download(&self, local: &Path) -> Result<String, String> {
        let Some(target) = self.target() else {
            return Err("host or path is empty".to_owned());
        };
        let args = vec![
            "-o".to_owned(),
            "BatchMode=yes".to_owned(),
            "-o".to_owned(),
            "ConnectTimeout=3".to_owned(),
            "-P".to_owned(),
            self.port(),
            target,
            local.to_string_lossy().into_owned(),
        ];
        Self::run("scp", &args)
    }

    fn ftp(&self) -> Result<Ftp, String> {
        Ftp::connect(
            self.host.trim(),
            &self.port(),
            self.user.trim(),
            &self.password,
        )
    }

    fn test(&self) -> Result<String, String> {
        match self.kind {
            RemoteKind::Ssh => self.ssh_test(),
            RemoteKind::Ftp => {
                let mut ftp = self.ftp()?;
                ftp.quit();
                Ok("ok".to_owned())
            }
        }
    }

    fn upload(&self, local: &Path) -> Result<String, String> {
        let local_len = std::fs::metadata(local)
            .map_err(|error| format!("{}: {error}", local.display()))?
            .len();
        match self.kind {
            RemoteKind::Ssh => {
                self.ssh_upload(local)?;
            }
            RemoteKind::Ftp => {
                let bytes = std::fs::read(local)
                    .map_err(|error| format!("{}: {error}", local.display()))?;
                let mut ftp = self.ftp()?;
                ftp.stor(self.path.trim(), &bytes)?;
                let remote = ftp.size(self.path.trim());
                ftp.quit();
                if let Some(remote) = remote {
                    if remote != local_len {
                        return Err(format!("server has {remote} bytes, expected {local_len}"));
                    }
                }
            }
        }
        if local_len == 0 {
            return Err("local vault is empty".to_owned());
        }
        Ok("ok".to_owned())
    }

    fn download(&self, local: &Path) -> Result<String, String> {
        match self.kind {
            RemoteKind::Ssh => self.ssh_download(local),
            RemoteKind::Ftp => {
                let mut ftp = self.ftp()?;
                let bytes = ftp.retr(self.path.trim())?;
                ftp.quit();
                std::fs::write(local, bytes)
                    .map_err(|error| format!("{}: {error}", local.display()))?;
                Ok("ok".to_owned())
            }
        }
    }
}

/// Minimal blocking FTP client with short timeouts, enough for one file.
struct Ftp {
    stream: std::net::TcpStream,
}

impl Ftp {
    fn connect(host: &str, port: &str, user: &str, password: &str) -> Result<Self, String> {
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;

        if host.is_empty() {
            return Err("host is empty".to_owned());
        }
        let address = format!("{host}:{port}");
        let socket = address
            .to_socket_addrs()
            .map_err(|error| format!("{address}: {error}"))?
            .next()
            .ok_or_else(|| format!("{address}: no address"))?;
        let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(4))
            .map_err(|error| format!("{address}: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(6)))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(6)))
            .map_err(|error| error.to_string())?;
        let mut ftp = Self { stream };
        ftp.expect(&[220])?;
        let user = if user.is_empty() { "anonymous" } else { user };
        let password = if password.is_empty() {
            "anonymous@"
        } else {
            password
        };
        let (code, _) = ftp.command(&format!("USER {user}"))?;
        if code == 331 {
            ftp.command(&format!("PASS {password}"))?;
        }
        ftp.command("TYPE I")?;
        Ok(ftp)
    }

    fn line(&mut self) -> Result<String, String> {
        use std::io::Read;

        let mut line = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            let read = self
                .stream
                .read(&mut byte)
                .map_err(|error| format!("FTP: {error}"))?;
            if read == 0 {
                return Err("FTP: connection closed".to_owned());
            }
            if byte[0] == b'\n' {
                break;
            }
            if byte[0] != b'\r' {
                line.push(byte[0]);
            }
        }
        Ok(String::from_utf8_lossy(&line).into_owned())
    }

    fn reply(&mut self) -> Result<(u16, String), String> {
        let mut text = String::new();
        loop {
            let line = self.line()?;
            let code = line
                .get(..3)
                .and_then(|code| code.parse::<u16>().ok())
                .ok_or_else(|| format!("FTP: {line}"))?;
            let multiline = line.as_bytes().get(3) == Some(&b'-');
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(line.trim_end());
            if !multiline {
                return Ok((code, text));
            }
        }
    }

    fn command(&mut self, command: &str) -> Result<(u16, String), String> {
        use std::io::Write;

        self.stream
            .write_all(format!("{command}\r\n").as_bytes())
            .map_err(|error| format!("FTP: {error}"))?;
        let (code, text) = self.reply()?;
        if code >= 400 {
            return Err(text);
        }
        Ok((code, text))
    }

    fn expect(&mut self, expected: &[u16]) -> Result<(), String> {
        let (code, text) = self.reply()?;
        if expected.contains(&code) {
            Ok(())
        } else {
            Err(text)
        }
    }

    fn pasv(&mut self) -> Result<std::net::SocketAddr, String> {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let (code, text) = self.command("PASV")?;
        if code != 227 {
            return Err(format!("FTP: PASV {code}"));
        }
        let start = text.find('(').ok_or_else(|| "FTP: bad PASV".to_owned())?;
        let end = text[start..]
            .find(')')
            .ok_or_else(|| "FTP: bad PASV".to_owned())?;
        let numbers: Vec<u16> = text[start + 1..start + end]
            .split(',')
            .filter_map(|part| part.trim().parse().ok())
            .collect();
        if numbers.len() != 6 {
            return Err("FTP: bad PASV".to_owned());
        }
        let octets = [
            u8::try_from(numbers[0]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[1]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[2]).map_err(|_| "FTP: bad PASV".to_owned())?,
            u8::try_from(numbers[3]).map_err(|_| "FTP: bad PASV".to_owned())?,
        ];
        let port = numbers[4] * 256 + numbers[5];
        Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(octets)), port))
    }

    fn data(&mut self) -> Result<std::net::TcpStream, String> {
        use std::time::Duration;

        let address = self.pasv()?;
        let stream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(4))
            .map_err(|error| format!("FTP data: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(8)))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(8)))
            .map_err(|error| error.to_string())?;
        Ok(stream)
    }

    fn stor(&mut self, remote: &str, bytes: &[u8]) -> Result<(), String> {
        use std::io::Write;

        let mut data = self.data()?;
        let (code, _) = self.command(&format!("STOR {remote}"))?;
        if code != 150 && code != 125 {
            return Err(format!("FTP: STOR {code}"));
        }
        data.write_all(bytes)
            .map_err(|error| format!("FTP: {error}"))?;
        data.shutdown(std::net::Shutdown::Write)
            .map_err(|error| format!("FTP: {error}"))?;
        drop(data);
        let (code, text) = self.reply()?;
        if code == 226 || code == 250 {
            Ok(())
        } else {
            Err(text)
        }
    }

    fn retr(&mut self, remote: &str) -> Result<Vec<u8>, String> {
        use std::io::Read;

        let mut data = self.data()?;
        let (code, _) = self.command(&format!("RETR {remote}"))?;
        if code != 150 && code != 125 {
            return Err(format!("FTP: RETR {code}"));
        }
        let mut bytes = Vec::new();
        data.read_to_end(&mut bytes)
            .map_err(|error| format!("FTP: {error}"))?;
        drop(data);
        let (code, text) = self.reply()?;
        if code == 226 || code == 250 {
            Ok(bytes)
        } else {
            Err(text)
        }
    }

    fn size(&mut self, remote: &str) -> Option<u64> {
        let (code, text) = self.command(&format!("SIZE {remote}")).ok()?;
        if code != 213 {
            return None;
        }
        text.split_whitespace().next()?.parse().ok()
    }

    fn quit(&mut self) {
        let _ = self.command("QUIT");
    }
}

/// State of the Shamir recovery panel (split a secret into shares, recombine).
#[derive(Default)]
struct Recovery {
    secret: String,
    total: u8,
    threshold: u8,
    shares: String,
    restore: String,
    result: String,
}

fn recovery_ui(ui: &mut egui::Ui, rec: &mut Recovery, lang: crate::Lang) {
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(
        RichText::new(lang.t("Ключи восстановления (Shamir)", "Recovery keys (Shamir)"))
            .size(13.0)
            .strong()
            .color(accent()),
    );
    ui.label(
        RichText::new(lang.t(
            "Любые K из N долей восстанавливают секрет. Сохраните доли отдельно.",
            "Any K of N shares restore the secret. Store the shares separately.",
        ))
        .size(10.0)
        .color(muted()),
    );
    ui.horizontal(|ui| {
        ui.label(lang.t("Секрет:", "Secret:"));
        ui.add(
            egui::TextEdit::singleline(&mut rec.secret)
                .password(true)
                .desired_width(160.0),
        );
        ui.label("K");
        ui.add(egui::DragValue::new(&mut rec.threshold).range(2..=255));
        ui.label("N");
        ui.add(egui::DragValue::new(&mut rec.total).range(2..=255));
        if ui.button(lang.t("Разделить", "Split")).clicked() {
            match shamir::split(rec.secret.as_bytes(), rec.threshold, rec.total) {
                Ok(parts) => {
                    rec.shares = parts
                        .iter()
                        .map(|share| {
                            use std::fmt::Write as FmtWrite;
                            let mut body = String::with_capacity(share.y.len() * 2);
                            for byte in &share.y {
                                let _ = write!(body, "{byte:02x}");
                            }
                            format!("{}:{}", share.x, body)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    rec.result.clear();
                }
                Err(error) => rec.result = error.to_string(),
            }
        }
    });
    ui.add(
        egui::TextEdit::multiline(&mut rec.shares)
            .desired_rows(3)
            .hint_text("1:...\n2:...\n3:..."),
    );
    if ui
        .button(lang.t("Сохранить доли в файл", "Save shares to file"))
        .clicked()
        && !rec.shares.is_empty()
    {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("secretpiece-shares.txt")
            .save_file()
        {
            rec.result = std::fs::write(path, rec.shares.as_bytes()).map_or_else(
                |error| error.to_string(),
                |()| lang.t("Доли сохранены.", "Shares saved.").to_owned(),
            );
        }
    }
    ui.add(
        egui::TextEdit::multiline(&mut rec.restore)
            .desired_rows(3)
            .hint_text(lang.t(
                "вставьте доли, по одной в строке",
                "paste shares, one per line",
            )),
    );
    if ui.button(lang.t("Восстановить", "Combine")).clicked() {
        let parsed: Vec<Share> = rec
            .restore
            .lines()
            .filter_map(|line| {
                let (x, y) = line.trim().split_once(':')?;
                let body = (0..y.len())
                    .step_by(2)
                    .filter_map(|at| u8::from_str_radix(y.get(at..at + 2)?, 16).ok())
                    .collect::<Vec<_>>();
                Some(Share {
                    x: x.parse().ok()?,
                    y: body,
                })
            })
            .collect();
        rec.result = match shamir::combine(rec.threshold, &parsed) {
            Ok(secret) => String::from_utf8_lossy(&secret).into_owned(),
            Err(error) => error.to_string(),
        };
    }
    ui.label(RichText::new(&rec.result).size(11.0).color(muted()));
}

/// A saved server destination (SSH or FTP).
#[derive(Clone, Default)]
struct Preset {
    name: String,
    kind: RemoteKind,
    host: String,
    port: String,
    user: String,
    password: String,
    path: String,
}

impl Preset {
    fn encode(&self) -> String {
        [
            self.name.as_str(),
            self.kind.key(),
            self.host.as_str(),
            self.port.as_str(),
            self.user.as_str(),
            self.password.as_str(),
            self.path.as_str(),
        ]
        .join("\u{1f}")
    }

    fn decode(raw: &str) -> Option<Self> {
        let parts: Vec<&str> = raw.split('\u{1f}').collect();
        if parts.len() < 7 {
            return None;
        }
        Some(Self {
            name: parts[0].to_owned(),
            kind: if parts[1] == "ftp" {
                RemoteKind::Ftp
            } else {
                RemoteKind::Ssh
            },
            host: parts[2].to_owned(),
            port: parts[3].to_owned(),
            user: parts[4].to_owned(),
            password: parts[5].to_owned(),
            path: parts[6].to_owned(),
        })
    }

    fn apply(&self, remote: &mut Remote) {
        remote.kind = self.kind;
        remote.host.clone_from(&self.host);
        remote.port.clone_from(&self.port);
        remote.user.clone_from(&self.user);
        remote.password.clone_from(&self.password);
        remote.path.clone_from(&self.path);
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
    #[allow(clippy::too_many_lines)]
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
                let parts: Vec<&str> = raw.split('\u{1f}').collect();
                let (kind, rest) = match parts.first().copied() {
                    Some("ftp") => (RemoteKind::Ftp, &parts[1..]),
                    Some("ssh") => (RemoteKind::Ssh, &parts[1..]),
                    _ => (RemoteKind::Ssh, &parts[..]),
                };
                Remote {
                    kind,
                    host: rest.first().copied().unwrap_or_default().to_owned(),
                    port: rest.get(1).copied().unwrap_or_default().to_owned(),
                    user: rest.get(2).copied().unwrap_or_default().to_owned(),
                    password: String::new(),
                    path: rest.get(3).copied().unwrap_or_default().to_owned(),
                    status: None,
                }
            })
            .unwrap_or_default();
        let presets = cc
            .storage
            .and_then(|storage| storage.get_string("presets"))
            .map(|raw| {
                raw.split('\u{1e}')
                    .filter_map(Preset::decode)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let show_presets = cc
            .storage
            .and_then(|storage| storage.get_string("show_presets"))
            .is_none_or(|value| value == "1");
        let remember_password = cc
            .storage
            .and_then(|storage| storage.get_string("remember_password"))
            .is_some_and(|value| value == "1");
        let auto_sync = cc
            .storage
            .and_then(|storage| storage.get_string("auto_sync"))
            .is_none_or(|value| value == "1");
        Self {
            lang,
            theme,
            vault_path,
            password: String::new(),
            confirm: String::new(),
            creating: false,
            publish: false,
            show_password: false,
            kdf: KdfChoice::Default,
            error: None,
            info: None,
            browser: None,
            settings_open: false,
            server_open: false,
            help_open: false,
            recovery: Recovery {
                total: 3,
                threshold: 2,
                ..Recovery::default()
            },
            verify_status: None,
            presets,
            preset_name: String::new(),
            new_preset_name: String::new(),
            show_presets,
            remember_password,
            auto_sync,
            background_io: cc
                .storage
                .and_then(|storage| storage.get_string("background_io"))
                .is_none_or(|value| value == "1"),
            remote_session: None,
            job: None,
            lock_wipe: None,
            remote,
            rekey_open: false,
            rekey_old: String::new(),
            rekey_new: String::new(),
            rekey_confirm: String::new(),
        }
    }

    #[allow(clippy::too_many_lines)]
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
        if self.creating && Path::new(&path).extension().is_none() {
            path.push_str(".mp4");
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
            if self.publish && !self.remote.ready() {
                self.error = Some(
                    self.lang
                        .t(
                            "Для сервера укажите хост и путь к сейфу.",
                            "Enter the server host and vault path.",
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
                let mut note = None;
                if self.creating && self.publish {
                    note = Some(match self.remote.upload(Path::new(&path)) {
                        Ok(_) => self
                            .lang
                            .t(
                                "Сейф создан и опубликован на сервере. Локальная копия остаётся рабочей.",
                                "Vault created and published to the server. The local copy stays active.",
                            )
                            .to_owned(),
                        Err(error) => format!(
                            "{}: {error}",
                            self.lang.t(
                                "Сейф создан локально, но загрузить не удалось",
                                "Vault created locally, upload failed",
                            )
                        ),
                    });
                }
                self.password.clear();
                self.confirm.clear();
                self.remote.status.clone_from(&note);
                self.info = note;
                self.browser = Some({
                    let mut browser = Browser::new(vault);
                    browser.vault_file = Path::new(&path).to_path_buf();
                    browser
                });
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
                    RichText::new(lang.t("Сервер (SSH / FTP)", "Server (SSH / FTP)"))
                        .size(13.0)
                        .strong()
                        .color(accent()),
                );
                ui.label(
                    RichText::new(lang.t(
                        "SSH работает по ключу или ssh-agent без пароля. FTP принимает логин и пароль; пароль хранится только в памяти.",
                        "SSH uses a key or ssh-agent without a password. FTP accepts login and password; the password stays in memory only.",
                    ))
                    .size(10.5)
                    .color(muted()),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    for kind in [RemoteKind::Ssh, RemoteKind::Ftp] {
                        let selected = self.remote.kind == kind;
                        let label = match kind {
                            RemoteKind::Ssh => "SSH / SCP",
                            RemoteKind::Ftp => "FTP",
                        };
                        let button = egui::Button::new(
                            RichText::new(label)
                                .size(11.5)
                                .color(if selected { on_accent() } else { ink() }),
                        )
                        .fill(if selected { accent() } else { input_bg() })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if selected { accent() } else { line() },
                        ))
                        .corner_radius(7);
                        if ui.add(button).clicked() {
                            self.remote.kind = kind;
                        }
                    }
                });
                ui.add_space(4.0);
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
                                .hint_text(self.remote.kind.default_port())
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
                        ui.label(RichText::new(lang.t("Пароль", "Password")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.password)
                                .password(true)
                                .hint_text(lang.t(
                                    "для FTP (не сохраняется)",
                                    "for FTP (not saved)",
                                ))
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

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(
                    RichText::new(lang.t("Приватность и пресеты", "Privacy and presets"))
                        .size(13.0)
                        .strong()
                        .color(accent()),
                );
                ui.checkbox(
                    &mut self.show_presets,
                    RichText::new(lang.t(
                        "Показывать выбор пресетов у входа",
                        "Show preset picker at sign-in",
                    ))
                    .size(12.0),
                );
                ui.checkbox(
                    &mut self.remember_password,
                    RichText::new(lang.t(
                        "Разрешить сохранять пароль в пресете",
                        "Allow saving the password in a preset",
                    ))
                    .size(12.0),
                );
                ui.checkbox(
                    &mut self.background_io,
                    RichText::new(lang.t(
                        "Фоновые операции с сервером (не подвешивать окно)",
                        "Background server operations (do not freeze the window)",
                    ))
                    .size(12.0),
                );
                ui.checkbox(
                    &mut self.auto_sync,
                    RichText::new(lang.t(
                        "Выгружать сейф на сервер при блокировке",
                        "Upload the vault on lock",
                    ))
                    .size(12.0),
                );
                ui.label(
                    RichText::new(lang.t(
                        "Пароль в пресете хранится открыто в конфиге приложения. Не включайте на чужом компьютере.",
                        "A saved password is stored in plain text in the app config. Do not enable this on a shared computer.",
                    ))
                    .size(10.0)
                    .color(muted()),
                );
                recovery_ui(ui, &mut self.recovery, lang);
                if ui.button(lang.t("Проверить целостность сейфа", "Verify vault integrity")).clicked() {
                    self.verify_status = Some(match self.browser.as_mut() {
                        Some(browser) => match browser.vault.verify() {
                            Ok(report) => format!("{}: {} / {}", lang.t("OK", "OK"), report.files, report.bytes),
                            Err(error) => describe(lang, &error),
                        },
                        None => lang.t("Сейф не открыт", "No vault open").to_owned(),
                    });
                }
                if let Some(status) = &self.verify_status {
                    ui.label(RichText::new(status).size(11.0).color(muted()));
                }
                if ui.button(lang.t(
                    "Автофикс заголовков сейфа",
                    "Auto-repair vault headers",
                )).clicked() {
                    let path = self
                        .browser
                        .as_ref()
                        .map(|browser| browser.vault.path().to_path_buf());
                    self.verify_status = Some(match path {
                        Some(path) => repair_headers(&path).unwrap_or_else(|error| error),
                        None => lang.t("Сейф не открыт", "No vault open").to_owned(),
                    });
                }
            });
        self.settings_open = open;
    }

    fn preset_picker(&mut self, ui: &mut egui::Ui) {
        if !self.show_presets || self.presets.is_empty() {
            return;
        }
        let lang = self.lang;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(lang.t("Пресет:", "Preset:"))
                    .size(11.5)
                    .color(muted()),
            );
            let current = if self.preset_name.is_empty() {
                lang.t("выбрать…", "choose…").to_owned()
            } else {
                self.preset_name.clone()
            };
            let mut chosen = None;
            egui::ComboBox::from_id_salt("preset-picker")
                .selected_text(current)
                .width(260.0)
                .show_ui(ui, |ui| {
                    for (index, preset) in self.presets.iter().enumerate() {
                        let label = format!(
                            "{} · {} · {}",
                            preset.name,
                            match preset.kind {
                                RemoteKind::Ssh => "SSH",
                                RemoteKind::Ftp => "FTP",
                            },
                            preset.host
                        );
                        if ui.selectable_label(false, label).clicked() {
                            chosen = Some(index);
                        }
                    }
                });
            if let Some(index) = chosen {
                if let Some(preset) = self.presets.get(index) {
                    preset.apply(&mut self.remote);
                    self.preset_name.clone_from(&preset.name);
                    self.remote.status = None;
                    self.new_preset_name.clear();
                }
            }
            if !self.preset_name.is_empty()
                && ui
                    .add(
                        egui::Button::new(
                            RichText::new(lang.t("Удалить", "Delete"))
                                .size(11.0)
                                .color(danger()),
                        )
                        .frame(false),
                    )
                    .clicked()
            {
                self.presets
                    .retain(|preset| preset.name != self.preset_name);
                self.preset_name.clear();
            }
        });
    }

    /// Built-in quick help in both languages.
    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        let lang = self.lang;
        let mut open = true;
        egui::Window::new(lang.t("Справка", "Help"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_width(560.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(520.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        let sections = [
                            (
                                lang.t("Создание сейфа", "Creating a vault"),
                                lang.t(
                                    "«Создать новое хранилище» → имя и оболочка (.mp4, .png, .docx, .md…) → пароль от 8 символов. Внутри всё равно контейнер SPV1, расширение — маскировка.",
                                    "Create a new vault → name and shell (.mp4, .png, .docx, .md…) → password of 8+ characters. Inside it is always an SPV1 container; the extension is disguise.",
                                ),
                            ),
                            (
                                lang.t("Сервер SSH / FTP", "SSH / FTP server"),
                                lang.t(
                                    "Кнопка «Сервер» или плашка под входом. SSH использует ключ или ssh-agent автоматически; FTP — логин и пароль. Пресеты сохраняются, пароль — только если разрешено в настройках.",
                                    "The Server button or the plaque under the form. SSH uses a key or ssh-agent automatically; FTP uses login and password. Presets are saved; the password only when allowed in Settings.",
                                ),
                            ),
                            (
                                lang.t("Папки и перемещение", "Folders and moving"),
                                lang.t(
                                    "Дерево контейнера показывает все папки. Отметьте несколько файлов галочками и нажмите «Переместить», затем выберите папку. Двойной клик открывает папку или файл.",
                                    "The vault tree lists every folder. Tick several files, press Move and pick a folder. Double-click opens a folder or a file.",
                                ),
                            ),
                            (
                                lang.t("Просмотр файлов", "Viewing files"),
                                lang.t(
                                    "Текст и таблицы (CSV/TSV) открываются в окне: перенос строк, вид таблицы, копирование, hex. Бинарные файлы можно открыть снаружи; папка распаковки и затирание задаются в настройках контейнера.",
                                    "Text and tables (CSV/TSV) open in a window with wrap, a table view, copy and hex. Binary files can open externally; the extraction folder and wiping are set in vault settings.",
                                ),
                            ),
                            (
                                lang.t("Синхронизация", "Sync"),
                                lang.t(
                                    "При блокировке сейф выгружается на сервер, если включено. Для контейнера можно задать свой путь на сервере.",
                                    "On lock the vault is uploaded when enabled. Each vault can override the server path.",
                                ),
                            ),
                            (
                                lang.t("Приватность", "Privacy"),
                                lang.t(
                                    "Пароль в пресете хранится открыто в конфиге приложения — не включайте на чужом компьютере. Временные файлы затираются случайными байтами.",
                                    "A preset password is stored in plain text in the app config — do not enable it on a shared computer. Temporary files are overwritten with random bytes.",
                                ),
                            ),
                            (
                                lang.t("Форматы", "Formats"),
                                lang.t(
                                    "Хранить можно что угодно: видео, аудио, изображения, docx/pptx, markdown, архивы. Ограничение — свободное место и лимиты индекса, а не тип файла.",
                                    "Store anything: video, audio, images, docx/pptx, markdown, archives. The only limits are free space and index limits, not the file type.",
                                ),
                            ),
                        ];
                        for (title, body) in sections {
                            ui.label(RichText::new(title).size(13.0).strong().color(accent()));
                            ui.label(RichText::new(body).size(11.5).color(muted()));
                            ui.add_space(10.0);
                        }
                        ui.label(
                            RichText::new("SecretPiece · GPL-3.0 · github.com/AmazingCoderA/secretpiece")
                                .size(10.5)
                                .color(muted()),
                        );
                    });
            });
        self.help_open = open;
    }

    /// Server dialog reachable straight from the sign-in screen.
    #[allow(clippy::too_many_lines)]
    fn server_window(&mut self, ctx: &egui::Context) {
        if !self.server_open {
            return;
        }
        let lang = self.lang;
        let mut open = true;
        egui::Window::new(lang.t("Сервер: открыть или создать сейф", "Server: open or create a vault"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_width(560.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(lang.t(
                        "SSH работает по ключу или ssh-agent автоматически. FTP — логин и пароль; пароль не покидает это окно.",
                        "SSH uses a key or ssh-agent automatically. FTP uses login and password; the password never leaves this dialog.",
                    ))
                    .size(10.5)
                    .color(muted()),
                );
                ui.add_space(6.0);
                self.preset_picker(ui);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    for kind in [RemoteKind::Ssh, RemoteKind::Ftp] {
                        let selected = self.remote.kind == kind;
                        let button = egui::Button::new(
                            RichText::new(match kind {
                                RemoteKind::Ssh => "SSH / SCP",
                                RemoteKind::Ftp => "FTP",
                            })
                            .size(11.5)
                            .color(if selected { on_accent() } else { ink() }),
                        )
                        .fill(if selected { accent() } else { input_bg() })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if selected { accent() } else { line() },
                        ))
                        .corner_radius(7);
                        if ui.add(button).clicked() {
                            self.remote.kind = kind;
                        }
                    }
                });
                ui.add_space(4.0);
                egui::Grid::new("server-grid")
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
                                .hint_text(self.remote.kind.default_port())
                                .desired_width(80.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Логин", "Login")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.user)
                                .hint_text("user")
                                .desired_width(200.0),
                        );
                        ui.end_row();
                        ui.label(RichText::new(lang.t("Пароль", "Password")).size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.remote.password)
                                .password(true)
                                .hint_text(lang.t("для FTP", "for FTP"))
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
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(lang.t("Скачать и открыть", "Download and open"))
                                    .color(on_accent()),
                            )
                            .fill(accent())
                            .corner_radius(8),
                        )
                        .clicked()
                    {
                        self.remote_download();
                        if self.remote.status.as_deref().is_some_and(|status| {
                            status.contains(lang.t("скачан", "downloaded"))
                        }) {
                            self.server_open = false;
                        }
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(lang.t("Создать здесь", "Create here")).color(ink()),
                            )
                            .fill(card_bg())
                            .stroke(Stroke::new(1.0_f32, line()))
                            .corner_radius(8),
                        )
                        .clicked()
                    {
                        self.creating = true;
                        self.publish = true;
                        if self.vault_path.trim().is_empty() {
                            "vault.mp4".clone_into(&mut self.vault_path);
                        }
                        self.server_open = false;
                    }
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_preset_name)
                            .hint_text(lang.t("имя пресета", "preset name"))
                            .desired_width(180.0),
                    );
                    if ui.button(lang.t("Сохранить пресет", "Save preset")).clicked() {
                        let name = if self.new_preset_name.trim().is_empty() {
                            self.remote.host.trim().to_owned()
                        } else {
                            self.new_preset_name.trim().to_owned()
                        };
                        if !name.is_empty() {
                            let preset = Preset {
                                name: name.clone(),
                                kind: self.remote.kind,
                                host: self.remote.host.clone(),
                                port: self.remote.port.clone(),
                                user: self.remote.user.clone(),
                                password: if self.remember_password {
                                    self.remote.password.clone()
                                } else {
                                    String::new()
                                },
                                path: self.remote.path.clone(),
                            };
                            self.presets.retain(|item| item.name != name);
                            self.presets.insert(0, preset);
                            self.presets.truncate(20);
                            self.preset_name.clone_from(&name);
                            self.new_preset_name.clear();
                        }
                    }
                    if !self.remember_password {
                        ui.label(
                            RichText::new(lang.t(
                                "пароль не сохраняется (включите в настройках)",
                                "password not saved (enable in settings)",
                            ))
                            .size(10.0)
                            .color(muted()),
                        );
                    }
                });
                if let Some(status) = &self.remote.status {
                    ui.add_space(4.0);
                    ui.label(RichText::new(status).size(11.0).color(muted()));
                }
            });
        self.server_open = open;
    }

    fn spawn_job(
        &mut self,
        kind: JobKind,
        work: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        let (tx, rx) = std::sync::mpsc::channel();
        if self.background_io {
            std::thread::spawn(move || {
                let _ = tx.send(work());
            });
            self.job = Some(Job { rx, kind });
        } else {
            let result = work();
            let _ = tx.send(result);
            self.job = Some(Job { rx, kind });
            self.poll_job();
        }
    }

    fn poll_job(&mut self) {
        let Some(job) = self.job.as_ref() else {
            return;
        };
        let Ok(result) = job.rx.try_recv() else {
            return;
        };
        let Some(job) = self.job.take() else {
            return;
        };
        match job.kind {
            JobKind::Test => {
                self.remote.status = Some(match result {
                    Ok(_) => self
                        .lang
                        .t("Соединение работает.", "Connection works.")
                        .to_owned(),
                    Err(error) => format!("{}: {error}", self.lang.t("Ошибка", "Error")),
                });
            }
            JobKind::Download(local) => {
                self.remote.status = Some(match result {
                    Ok(_) => {
                        self.vault_path = local.to_string_lossy().into_owned();
                        self.remote_session = Some(local);
                        self.creating = false;
                        self.error = None;
                        self.lang
                            .t(
                                "Сейф скачан во временную папку: локальной копии не останется — при блокировке он загрузится обратно и затрётся. Введите пароль.",
                                "Vault downloaded to a temp folder: no local copy is kept — on lock it is uploaded back and wiped. Enter the password.",
                            )
                            .to_owned()
                    }
                    Err(error) => format!("{}: {error}", self.lang.t("Ошибка", "Error")),
                });
            }
            JobKind::Upload => {
                let ok = result.is_ok();
                self.remote.status = Some(match result {
                    Ok(_) => self
                        .lang
                        .t("Сейф загружен на сервер.", "Vault uploaded to the server.")
                        .to_owned(),
                    Err(error) => format!("{}: {error}", self.lang.t("Ошибка", "Error")),
                });
                if ok {
                    if let Some(temp) = self.lock_wipe.take() {
                        let temp_str = temp.to_string_lossy().into_owned();
                        wipe_temp(&temp, wipe_passes());
                        if self.vault_path.trim() == temp_str {
                            self.vault_path.clear();
                        }
                    }
                }
            }
        }
    }

    fn remote_test(&mut self) {
        if self.remote.host.trim().is_empty() {
            self.remote.status = Some(self.lang.t("Укажите хост.", "Enter a host.").to_owned());
            return;
        }
        let remote = self.remote.clone();
        self.remote.status = Some(self.lang.t("Проверяю…", "Testing…").to_owned());
        self.spawn_job(JobKind::Test, move || remote.test());
    }

    fn remote_download(&mut self) {
        if !self.remote.ready() {
            self.remote.status = Some(
                self.lang
                    .t(
                        "Нужны хост и путь к сейфу.",
                        "Host and vault path are required.",
                    )
                    .to_owned(),
            );
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |value| value.as_nanos());
        let local = preview_dir().join(format!("remote-{stamp}-{}.spv", std::process::id()));
        if let Some(parent) = local.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let remote = self.remote.clone();
        let target = local.clone();
        self.remote.status = Some(
            self.lang
                .t("Скачиваю сейф…", "Downloading the vault…")
                .to_owned(),
        );
        self.spawn_job(JobKind::Download(local), move || remote.download(&target));
    }

    fn remote_upload(&mut self) {
        if !self.remote.ready() {
            self.remote.status = Some(
                self.lang
                    .t(
                        "Нужны хост и путь к сейфу.",
                        "Host and vault path are required.",
                    )
                    .to_owned(),
            );
            return;
        }
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
        let remote = self.remote.clone();
        let target = PathBuf::from(&local);
        self.remote.status = Some(
            self.lang
                .t("Загружаю на сервер…", "Uploading to the server…")
                .to_owned(),
        );
        self.spawn_job(JobKind::Upload, move || remote.upload(&target));
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

    #[allow(clippy::too_many_lines)]
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
                                let mut remote = self.remote.clone();
                                let sync = self.remote_session.is_some()
                                    || (self.auto_sync
                                        && self
                                            .browser
                                            .as_ref()
                                            .is_some_and(|browser| browser.settings.auto_sync));
                                if let Some(browser) = &self.browser {
                                    if !browser.settings.remote_path.trim().is_empty() {
                                        browser.settings.remote_path.clone_into(&mut remote.path);
                                    }
                                }
                                self.browser = None;
                                self.password.clear();
                                let can_upload =
                                    sync && remote.ready() && !self.vault_path.trim().is_empty();
                                if can_upload {
                                    self.lock_wipe = self.remote_session.take();
                                    let target = PathBuf::from(self.vault_path.trim());
                                    self.remote.status = Some(
                                        self.lang
                                            .t("Загружаю на сервер…", "Uploading to the server…")
                                            .to_owned(),
                                    );
                                    self.spawn_job(JobKind::Upload, move || remote.upload(&target));
                                } else if !sync {
                                    if let Some(temp) = self.remote_session.take() {
                                        let temp_str = temp.to_string_lossy().into_owned();
                                        wipe_temp(&temp, wipe_passes());
                                        if self.vault_path.trim() == temp_str {
                                            self.vault_path.clear();
                                        }
                                    }
                                }
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
                        ui.add_space(8.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(self.lang.t("Сервер", "Server"))
                                        .size(11.5)
                                        .color(accent()),
                                )
                                .fill(card_bg())
                                .stroke(Stroke::new(1.0_f32, accent()))
                                .corner_radius(9)
                                .min_size(egui::vec2(86.0, 30.0)),
                            )
                            .clicked()
                        {
                            self.server_open = true;
                        }
                        ui.add_space(8.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(self.lang.t("Справка", "Help"))
                                        .size(11.5)
                                        .color(ink()),
                                )
                                .fill(card_bg())
                                .stroke(Stroke::new(1.0_f32, line()))
                                .corner_radius(9)
                                .min_size(egui::vec2(84.0, 30.0)),
                            )
                            .clicked()
                        {
                            self.help_open = true;
                        }
                        if let Some(browser) = self.browser.as_mut() {
                            ui.add_space(8.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(self.lang.t("Контейнер", "Vault"))
                                            .size(11.5)
                                            .color(ink()),
                                    )
                                    .fill(card_bg())
                                    .stroke(Stroke::new(1.0_f32, line()))
                                    .corner_radius(9)
                                    .min_size(egui::vec2(96.0, 30.0)),
                                )
                                .clicked()
                            {
                                browser.container_open = true;
                            }
                        }
                        ui.add_space(12.0);
                        language_switch(ui, &mut self.lang);
                        ui.add_space(14.0);
                        let mut picked = None;
                        ui.menu_button(
                            RichText::new(format!(
                                "{} {}",
                                self.lang.t("Тема", "Theme"),
                                self.theme.label(self.lang)
                            ))
                            .size(11.5),
                            |ui| {
                                for theme in [
                                    Theme::Midnight,
                                    Theme::Neon,
                                    Theme::Light,
                                    Theme::Paper,
                                    Theme::Mint,
                                    Theme::Sunset,
                                    Theme::Forest,
                                    Theme::Sepia,
                                    Theme::Ocean,
                                    Theme::Rose,
                                    Theme::Aurora,
                                    Theme::Cyber,
                                    Theme::Coal,
                                    Theme::Cream,
                                    Theme::Lavender,
                                    Theme::Honey,
                                    Theme::Winter,
                                    Theme::Desert,
                                    Theme::Storm,
                                    Theme::Emerald,
                                    Theme::Ruby,
                                    Theme::Amber,
                                    Theme::Slate,
                                    Theme::Orchid,
                                    Theme::Coffee,
                                    Theme::Meadow,
                                    Theme::Coral,
                                    Theme::Indigo,
                                    Theme::Sand,
                                    Theme::Frost,
                                    Theme::Ember,
                                    Theme::Lilac,
                                    Theme::Olive,
                                    Theme::Sky,
                                    Theme::Plum,
                                    Theme::Copper,
                                    Theme::Charcoal,
                                    Theme::Pearl,
                                    Theme::Cobalt,
                                    Theme::Velvet,
                                ] {
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(16.0, 16.0),
                                            egui::Sense::hover(),
                                        );
                                        let palette = theme.palette();
                                        ui.painter().rect_filled(rect, 4.0, palette.bg);
                                        ui.painter().circle_filled(
                                            rect.center(),
                                            3.5,
                                            palette.accent,
                                        );
                                        if ui
                                            .selectable_label(
                                                self.theme == theme,
                                                theme.label(self.lang),
                                            )
                                            .clicked()
                                        {
                                            picked = Some(theme);
                                        }
                                    });
                                }
                            },
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
        self.poll_job();
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
        self.server_window(ctx);
        self.help_window(ctx);
        self.rekey_window(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("theme", self.theme.key().to_owned());
        storage.set_string("wipe_passes", wipe_passes().to_string());
        storage.set_string(
            "remote",
            format!(
                "{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}",
                self.remote.kind.key(),
                self.remote.host,
                self.remote.port,
                self.remote.user,
                self.remote.path
            ),
        );
        storage.set_string("lang", self.lang.key().to_owned());
        let presets = self
            .presets
            .iter()
            .map(Preset::encode)
            .collect::<Vec<_>>()
            .join("\u{1e}");
        storage.set_string("presets", presets);
        storage.set_string(
            "show_presets",
            if self.show_presets { "1" } else { "0" }.to_owned(),
        );
        storage.set_string(
            "remember_password",
            if self.remember_password { "1" } else { "0" }.to_owned(),
        );
        storage.set_string(
            "background_io",
            if self.background_io { "1" } else { "0" }.to_owned(),
        );
        storage.set_string(
            "auto_sync",
            if self.auto_sync { "1" } else { "0" }.to_owned(),
        );
    }
}

impl App {
    #[allow(clippy::too_many_lines)]
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
                    ui.add_space(10.0);
                    let plaque = egui::Button::new(
                        RichText::new(self.lang.t(
                            "Сервер SSH / FTP — открыть или создать сейф",
                            "SSH / FTP server — open or create a vault",
                        ))
                        .size(12.0)
                        .color(accent()),
                    )
                    .fill(card_bg())
                    .stroke(Stroke::new(1.0_f32, accent()))
                    .corner_radius(10)
                    .min_size(egui::vec2(card_width, 38.0));
                    if ui.add(plaque).clicked() {
                        self.server_open = true;
                    }
                    if self.show_presets && !self.presets.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            self.preset_picker(ui);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(
                                            self.lang.t("Открыть с сервера", "Open from server"),
                                        )
                                        .size(11.5)
                                        .color(on_accent()),
                                    )
                                    .fill(accent())
                                    .corner_radius(8)
                                    .min_size(egui::vec2(0.0, 26.0)),
                                )
                                .clicked()
                            {
                                self.remote_download();
                            }
                        });
                    }
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
                            .hint_text("vault.mp4")
                            .margin(egui::vec2(10.0, 9.0))
                            .background_color(input_bg()),
                    );
                    if secondary_button(ui, lang.t("Обзор…", "Browse…")).clicked() {
                        let dialog = rfd::FileDialog::new();
                        let picked = if self.creating {
                            dialog.set_file_name("vault.mp4").save_file()
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
                        "Любой файл: .mp4, .mp3, .png, .docx, .md — внутри всё равно контейнер SPV1.",
                        "Any file: .mp4, .mp3, .png, .docx, .md — inside it is still an SPV1 container.",
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
                        for ext in [
                            "mp4", "mp3", "opus", "avi", "png", "jpg", "jpeg", "gif", "docx",
                            "pptx", "md", "sh", "pdf", "zip",
                        ] {
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

                    ui.add_space(12.0);
                    field_label(ui, lang.t("Куда сохранить", "Destination"));
                    ui.horizontal_wrapped(|ui| {
                        for (label, kind) in [
                            (lang.t("Локально", "Local"), None),
                            ("SSH", Some(RemoteKind::Ssh)),
                            ("FTP", Some(RemoteKind::Ftp)),
                        ] {
                            let active = kind.map_or(!self.publish, |kind| {
                                self.publish && self.remote.kind == kind
                            });
                            let button = egui::Button::new(
                                RichText::new(label)
                                    .size(11.5)
                                    .color(if active { on_accent() } else { muted() }),
                            )
                            .fill(if active { accent() } else { input_bg() })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if active { accent() } else { line() },
                            ))
                            .corner_radius(7)
                            .min_size(egui::vec2(86.0, 28.0));
                            if ui.add(button).clicked() {
                                match kind {
                                    Some(kind) => {
                                        self.publish = true;
                                        self.remote.kind = kind;
                                    }
                                    None => self.publish = false,
                                }
                            }
                        }
                    });
                    if self.publish {
                        ui.add_space(6.0);
                        self.preset_picker(ui);
                        egui::Grid::new("create-remote")
                            .num_columns(2)
                            .spacing([8.0, 5.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new(lang.t("Хост", "Host")).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.remote.host)
                                        .hint_text("example.com")
                                        .desired_width(260.0),
                                );
                                ui.end_row();
                                ui.label(RichText::new(lang.t("Порт", "Port")).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.remote.port)
                                        .hint_text(self.remote.kind.default_port())
                                        .desired_width(80.0),
                                );
                                ui.end_row();
                                ui.label(RichText::new(lang.t("Логин", "Login")).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.remote.user)
                                        .hint_text("user")
                                        .desired_width(180.0),
                                );
                                ui.end_row();
                                ui.label(RichText::new(lang.t("Пароль", "Password")).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.remote.password)
                                        .password(true)
                                        .hint_text(lang.t("для FTP", "for FTP"))
                                        .desired_width(180.0),
                                );
                                ui.end_row();
                                ui.label(RichText::new(lang.t("Путь на сервере", "Server path")).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.remote.path)
                                        .hint_text("/home/user/vault.spv")
                                        .desired_width(260.0),
                                );
                                ui.end_row();
                            });
                        ui.label(
                            RichText::new(lang.t(
                                "SSH: ключ или ssh-agent подхватываются автоматически. FTP: логин и пароль, пароль не сохраняется.",
                                "SSH: a key or ssh-agent is picked up automatically. FTP: login and password, the password is not saved.",
                            ))
                            .size(10.0)
                            .color(muted()),
                        );
                    } else {
                        ui.label(
                            RichText::new(lang.t(
                                "Можно указать сетевую папку: \\\\server\\share\\vault.mp4.",
                                "A network folder works too: \\\\server\\share\\vault.mp4.",
                            ))
                            .size(10.0)
                            .color(muted()),
                        );
                    }
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
                    if self.creating && self.vault_path.trim().is_empty() {
                        "vault.mp4".clone_into(&mut self.vault_path);
                    }
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

#[allow(clippy::struct_excessive_bools)]
struct Browser {
    vault: Vault,
    cwd: String,
    entries: Vec<ListEntry>,
    filter: String,
    selected: Option<String>,
    anchor: Option<String>,
    status: Option<Status>,
    new_folder: Option<String>,
    pending_delete: Option<String>,
    pending_add: Option<PendingAdd>,
    pending_external: Option<String>,
    preview: Option<Preview>,
    external: Vec<ExternalPreview>,
    shred_on_delete: bool,
    sort: SortBy,
    sort_desc: bool,
    settings: ContainerSettings,
    container_open: bool,
    vault_file: PathBuf,
    checked: BTreeSet<String>,
    move_open: bool,
    move_target: String,
}

/// Per-container options stored inside the vault at `.secretpiece/settings`.
#[derive(Clone)]
struct ContainerSettings {
    external_preview: bool,
    preview_root: String,
    auto_sync: bool,
    remote_path: String,
}

impl Default for ContainerSettings {
    fn default() -> Self {
        Self {
            external_preview: true,
            preview_root: String::new(),
            auto_sync: true,
            remote_path: String::new(),
        }
    }
}

impl ContainerSettings {
    fn encode(&self) -> String {
        [
            if self.external_preview { "1" } else { "0" },
            &self.preview_root,
            if self.auto_sync { "1" } else { "0" },
            &self.remote_path,
        ]
        .join("\u{1f}")
    }

    fn decode(raw: &str) -> Option<Self> {
        let parts: Vec<&str> = raw.split('\u{1f}').collect();
        if parts.len() < 4 {
            return None;
        }
        Some(Self {
            external_preview: parts[0] == "1",
            preview_root: parts[1].to_owned(),
            auto_sync: parts[2] == "1",
            remote_path: parts[3].to_owned(),
        })
    }
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
    wrap: bool,
    table: bool,
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.data.fill(0);
    }
}

impl Browser {
    fn new(mut vault: Vault) -> Self {
        let settings = vault
            .get(".secretpiece/settings")
            .ok()
            .and_then(|data| ContainerSettings::decode(&String::from_utf8_lossy(&data)))
            .unwrap_or_default();
        set_preview_root(&settings.preview_root);
        let mut browser = Self {
            vault,
            cwd: String::new(),
            entries: Vec::new(),
            filter: String::new(),
            selected: None,
            anchor: None,
            status: None,
            new_folder: None,
            pending_delete: None,
            pending_add: None,
            pending_external: None,
            preview: None,
            external: Vec::new(),
            shred_on_delete: true,
            sort: SortBy::Name,
            sort_desc: false,
            settings,
            container_open: false,
            vault_file: PathBuf::new(),
            checked: BTreeSet::new(),
            move_open: false,
            move_target: String::new(),
        };
        browser.refresh();
        browser
    }

    fn refresh(&mut self) {
        self.entries = self.vault.list(Some(&self.cwd));
        self.entries
            .retain(|entry| !entry.path.starts_with(".secretpiece"));
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

    fn all_dirs(&self) -> Vec<String> {
        let mut dirs: Vec<String> = self
            .vault
            .list(None)
            .into_iter()
            .filter(|entry| entry.kind == EntryKind::Dir && !entry.path.starts_with(".secretpiece"))
            .map(|entry| entry.path)
            .collect();
        dirs.sort();
        dirs
    }

    /// Moves every checked file or folder into `target` (empty means root).
    fn move_checked(&mut self, target: &str, lang: Lang) -> Result<usize, String> {
        let target = target.trim().trim_matches('/').to_owned();
        let sources: Vec<String> = self.checked.iter().cloned().collect();
        let all: Vec<(String, EntryKind)> = self
            .vault
            .list(None)
            .into_iter()
            .map(|entry| (entry.path, entry.kind))
            .collect();
        let mut moved = 0_usize;
        for source in sources {
            let base = source.rsplit('/').next().unwrap_or(&source).to_owned();
            let dest = if target.is_empty() {
                base
            } else {
                format!("{target}/{base}")
            };
            if dest == source || dest.starts_with(&format!("{source}/")) {
                continue;
            }
            let kind = all
                .iter()
                .find(|(path, _)| path == &source)
                .map(|(_, kind)| *kind);
            match kind {
                Some(EntryKind::File) => {
                    let mut data = self.vault.get(&source).map_err(|e| describe(lang, &e))?;
                    let put = self.vault.put(&dest, &data);
                    data.fill(0);
                    put.map_err(|e| describe(lang, &e))?;
                    self.vault.shred(&source).map_err(|e| describe(lang, &e))?;
                    moved += 1;
                }
                Some(EntryKind::Dir) => {
                    self.vault.mkdir(&dest).map_err(|e| describe(lang, &e))?;
                    let prefix = format!("{source}/");
                    let children: Vec<(String, EntryKind)> = all
                        .iter()
                        .filter(|(path, _)| path.starts_with(prefix.as_str()))
                        .cloned()
                        .collect();
                    for (path, child_kind) in children {
                        let new_path = format!("{dest}{}", &path[source.len()..]);
                        if child_kind == EntryKind::Dir {
                            self.vault
                                .mkdir(&new_path)
                                .map_err(|e| describe(lang, &e))?;
                        } else {
                            let mut data = self.vault.get(&path).map_err(|e| describe(lang, &e))?;
                            let put = self.vault.put(&new_path, &data);
                            data.fill(0);
                            put.map_err(|e| describe(lang, &e))?;
                        }
                    }
                    self.vault.remove(&source).map_err(|e| describe(lang, &e))?;
                    moved += 1;
                }
                None => {}
            }
        }
        self.checked.clear();
        self.refresh();
        Ok(moved)
    }

    #[allow(clippy::too_many_lines)]
    fn ui(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let info = self.vault.info();
        self.actions(ui, lang);
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(12.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text(lang.t("Поиск в папке…", "Search in folder…"))
                    .desired_width(220.0),
            );
            if ui.button("✕").clicked() {
                self.filter.clear();
            }
        });
        let dirs = self.all_dirs();
        if !dirs.is_empty() {
            ui.add_space(4.0);
            let mut dropped: Option<(String, String)> = None;
            Frame::new()
                .fill(panel())
                .stroke(Stroke::new(1.0_f32, line()))
                .corner_radius(10)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(lang.t("Дерево контейнера", "Vault tree"))
                            .size(11.0)
                            .color(muted()),
                    );
                    egui::ScrollArea::vertical()
                        .max_height(170.0)
                        .show(ui, |ui| {
                            let (_, root_drop) =
                                ui.dnd_drop_zone::<String, _>(Frame::new(), |ui| {
                                    if ui
                                        .selectable_label(self.cwd.is_empty(), "/  ·  root")
                                        .clicked()
                                    {
                                        self.cwd.clear();
                                        self.refresh();
                                    }
                                });
                            if let Some(payload) = root_drop {
                                dropped = Some(((*payload).clone(), String::new()));
                            }
                            for dir in &dirs {
                                let depth = dir.matches('/').count();
                                let name = dir.rsplit('/').next().unwrap_or(dir);
                                ui.horizontal(|ui| {
                                    ui.add_space(
                                        f32::from(u8::try_from(depth.min(20)).unwrap_or(0)) * 14.0,
                                    );
                                    let (_, folder_drop) =
                                        ui.dnd_drop_zone::<String, _>(Frame::new(), |ui| {
                                            let id = egui::Id::new(("tree-dir", dir.clone()));
                                            let response =
                                                ui.dnd_drag_source(id, dir.clone(), |ui| {
                                                    ui.selectable_label(
                                                        self.cwd == *dir,
                                                        format!("📁 {name}"),
                                                    )
                                                });
                                            if response.inner.clicked() {
                                                self.cwd.clone_from(dir);
                                                self.refresh();
                                            }
                                        });
                                    if let Some(payload) = folder_drop {
                                        dropped = Some(((*payload).clone(), dir.clone()));
                                    }
                                });
                            }
                        });
                });
            if let Some((source, target)) = dropped {
                self.checked.clear();
                self.checked.insert(source);
                match self.move_checked(&target, lang) {
                    Ok(_) => {
                        self.status = Some(Status {
                            text: lang.t("Перемещено.", "Moved.").to_owned(),
                            error: false,
                        });
                    }
                    Err(text) => self.status = Some(Status { text, error: true }),
                }
            }
        }
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
                        if ui.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::A))
                        {
                            self.checked = self
                                .entries
                                .iter()
                                .map(|entry| entry.path.clone())
                                .collect();
                        }
                        let query = self.filter.trim().to_lowercase();
                        let (ctrl, shift) =
                            ui.input(|input| (input.modifiers.ctrl, input.modifiers.shift));
                        let mut enter: Option<String> = None;
                        let mut preview: Option<String> = None;
                        for entry in &self.entries {
                            if !query.is_empty() && !entry.path.to_lowercase().contains(&query) {
                                continue;
                            }
                            let selected = self.selected.as_deref() == Some(entry.path.as_str());
                            let drag_id = egui::Id::new(("tree-file", entry.path.clone()));
                            let click = ui
                                .dnd_drag_source(drag_id, entry.path.clone(), |ui| {
                                    row(
                                        ui,
                                        lang,
                                        entry,
                                        selected,
                                        self.checked.contains(&entry.path),
                                    )
                                })
                                .inner;
                            if click.toggle {
                                if self.checked.contains(&entry.path) {
                                    self.checked.remove(&entry.path);
                                } else {
                                    self.checked.insert(entry.path.clone());
                                }
                            }
                            if click.selected {
                                if shift {
                                    if let Some(anchor) = &self.anchor {
                                        if let (Some(from), Some(to)) = (
                                            self.entries.iter().position(|e| &e.path == anchor),
                                            self.entries.iter().position(|e| e.path == entry.path),
                                        ) {
                                            let (start, end) =
                                                if from < to { (from, to) } else { (to, from) };
                                            for item in &self.entries[start..=end] {
                                                self.checked.insert(item.path.clone());
                                            }
                                        }
                                    }
                                } else if ctrl {
                                    if self.checked.contains(&entry.path) {
                                        self.checked.remove(&entry.path);
                                    } else {
                                        self.checked.insert(entry.path.clone());
                                    }
                                } else {
                                    self.checked.clear();
                                    self.anchor = Some(entry.path.clone());
                                }
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
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    for path in paths {
                        self.add_file(&path, lang);
                    }
                }
            }
            if secondary_button(ui, lang.t("Новая папка", "New folder")).clicked() {
                self.new_folder = Some(String::new());
            }
            if secondary_button(ui, lang.t("Выбрать всё", "Select all")).clicked() {
                if self.checked.len() == self.entries.len() {
                    self.checked.clear();
                } else {
                    self.checked = self
                        .entries
                        .iter()
                        .map(|entry| entry.path.clone())
                        .collect();
                }
            }
            let checked = self.checked.len();
            if ui
                .add_enabled(
                    checked > 0,
                    egui::Button::new(
                        RichText::new(format!("{} ({checked})", lang.t("Переместить", "Move")))
                            .size(13.0),
                    )
                    .fill(card_bg())
                    .stroke(Stroke::new(1.0_f32, line()))
                    .corner_radius(9)
                    .min_size(egui::vec2(0.0, 32.0)),
                )
                .clicked()
            {
                self.move_target.clone_from(&self.cwd);
                self.move_open = true;
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
                        wrap: true,
                        table: is_table_name(path),
                    });
                } else if self.settings.external_preview {
                    data.fill(0);
                    self.pending_external = Some(path.to_owned());
                } else {
                    self.preview = Some(Preview {
                        path: path.to_owned(),
                        data,
                        hex: true,
                        wrap: true,
                        table: false,
                    });
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
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |value| value.as_nanos());
        let temp = preview_dir().join(format!("{}-{stamp}-{safe}", std::process::id()));
        match self.vault.get(path) {
            Ok(mut data) => {
                let result = std::fs::write(&temp, &data).and_then(|()| open_with_system(&temp));
                data.fill(0);
                match result {
                    Ok(()) => {
                        self.external.push(ExternalPreview {
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
                    let hex = preview.hex || text.is_none();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&preview.path).strong().color(ink()));
                        ui.label(
                            RichText::new(human_size(preview.data.len() as u64))
                                .size(12.0)
                                .color(muted()),
                        );
                        if let Some(text) = text {
                            ui.label(
                                RichText::new(format!(
                                    "{}: {}",
                                    lang.t("строк", "lines"),
                                    text.lines().count()
                                ))
                                .size(11.0)
                                .color(muted()),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        if text.is_some() {
                            let label = if preview.hex {
                                lang.t("Текст", "Text")
                            } else {
                                lang.t("Hex", "Hex")
                            };
                            if ui.button(label).clicked() {
                                preview.hex = !preview.hex;
                            }
                            if !preview.hex {
                                ui.checkbox(
                                    &mut preview.wrap,
                                    RichText::new(lang.t("Перенос", "Wrap")).size(11.5),
                                );
                                if is_table_name(&preview.path) {
                                    ui.checkbox(
                                        &mut preview.table,
                                        RichText::new(lang.t("Таблица", "Table")).size(11.5),
                                    );
                                }
                            }
                            if ui.button(lang.t("Копировать", "Copy")).clicked() {
                                if let Some(text) = text {
                                    ui.ctx().copy_text(clip(text, 200_000).to_owned());
                                }
                            }
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                    let scroll = if preview.wrap && !hex && !preview.table {
                        egui::ScrollArea::vertical()
                    } else {
                        egui::ScrollArea::both()
                    };
                    scroll.auto_shrink([false, false]).show(ui, |ui| {
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
                                    RichText::new(
                                        lang.t(
                                            "Показаны первые 16 КиБ.",
                                            "Showing the first 16 KiB.",
                                        ),
                                    )
                                    .size(11.0)
                                    .color(muted()),
                                );
                            }
                        } else if preview.table {
                            if let Some(text) = text {
                                let delimiter =
                                    if preview.path.to_ascii_lowercase().ends_with(".tsv") {
                                        '\t'
                                    } else {
                                        ','
                                    };
                                egui::Grid::new("preview-table")
                                    .striped(true)
                                    .min_col_width(48.0)
                                    .show(ui, |ui| {
                                        for (index, line) in text.lines().take(500).enumerate() {
                                            for cell in line.split(delimiter).take(24) {
                                                let rich = RichText::new(cell)
                                                    .monospace()
                                                    .size(12.0)
                                                    .color(ink());
                                                ui.label(if index == 0 {
                                                    rich.strong()
                                                } else {
                                                    rich
                                                });
                                            }
                                            ui.end_row();
                                        }
                                    });
                                if text.lines().count() > 500 {
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Показаны первые 500 строк.",
                                            "Showing the first 500 rows.",
                                        ))
                                        .size(11.0)
                                        .color(muted()),
                                    );
                                }
                            }
                        } else if let Some(text) = text {
                            let shown = clip(text, 200_000);
                            let label = egui::Label::new(
                                RichText::new(shown).monospace().size(12.5).color(ink()),
                            )
                            .selectable(true);
                            if preview.wrap {
                                ui.add(label.wrap());
                            } else {
                                ui.add(label);
                            }
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

        if self.move_open {
            let mut open = true;
            let mut close = false;
            egui::Window::new(lang.t("Переместить в папку", "Move to folder"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .default_width(460.0)
                .show(ui.ctx(), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            lang.t("Выбрано", "Selected"),
                            self.checked.len()
                        ))
                        .size(12.0)
                        .color(muted()),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Целевая папка (пусто — корень):",
                            "Target folder (empty — root):",
                        ))
                        .size(11.5),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.move_target)
                            .hint_text("docs/2026")
                            .desired_width(330.0),
                    );
                    ui.add_space(6.0);
                    let dirs = self.all_dirs();
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            if ui
                                .selectable_label(self.move_target.is_empty(), "/  ·  root")
                                .clicked()
                            {
                                self.move_target.clear();
                            }
                            for dir in &dirs {
                                if ui.selectable_label(self.move_target == *dir, dir).clicked() {
                                    self.move_target.clone_from(dir);
                                }
                            }
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Переместить", "Move")).color(on_accent()),
                                )
                                .fill(accent())
                                .corner_radius(8),
                            )
                            .clicked()
                        {
                            let target = self.move_target.clone();
                            match self.move_checked(&target, lang) {
                                Ok(count) => {
                                    self.status = Some(Status {
                                        text: match lang {
                                            Lang::Ru => format!("Перемещено: {count}."),
                                            Lang::En => format!("Moved: {count}."),
                                        },
                                        error: false,
                                    });
                                }
                                Err(error) => {
                                    self.status = Some(Status {
                                        text: error,
                                        error: true,
                                    });
                                }
                            }
                            close = true;
                        }
                        if ui.button(lang.t("Отмена", "Cancel")).clicked() {
                            close = true;
                        }
                    });
                });
            if close || !open {
                self.move_open = false;
            }
        }

        if self.container_open {
            let mut open = true;
            egui::Window::new(lang.t("Настройки контейнера", "Vault settings"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .default_width(520.0)
                .show(ui.ctx(), |ui| {
                    ui.label(
                        RichText::new(lang.t(
                            "Параметры хранятся внутри этого контейнера (.secretpiece/settings) и переезжают вместе с ним.",
                            "These options live inside this container (.secretpiece/settings) and travel with it.",
                        ))
                        .size(10.5)
                        .color(muted()),
                    );
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut self.settings.external_preview,
                        RichText::new(lang.t(
                            "Открывать нетекстовые файлы во внешнем приложении",
                            "Open non-text files in an external app",
                        ))
                        .size(12.0),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Куда распаковывать для просмотра (пусто — системный TEMP):",
                            "Where to extract for viewing (empty — system TEMP):",
                        ))
                        .size(11.0)
                        .color(muted()),
                    );
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.preview_root)
                                .hint_text("%TEMP%\\secretpiece-preview")
                                .desired_width(300.0),
                        );
                        if ui.button(lang.t("Обзор…", "Browse…")).clicked() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                self.settings.preview_root = dir.to_string_lossy().into_owned();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .button(lang.t("Системный TEMP", "System TEMP"))
                            .clicked()
                        {
                            self.settings.preview_root.clear();
                            set_preview_root("");
                        }
                        if ui
                            .button(lang.t("Рядом с контейнером", "Beside the container"))
                            .clicked()
                        {
                            if let Some(dir) = self.vault_file.parent() {
                                self.settings.preview_root =
                                    dir.to_string_lossy().into_owned();
                                set_preview_root(&self.settings.preview_root);
                            }
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            lang.t("Сейчас", "Now"),
                            preview_dir().display()
                        ))
                        .size(10.5)
                        .color(muted()),
                    );
                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut self.settings.auto_sync,
                        RichText::new(lang.t(
                            "Выгружать этот сейф на сервер при блокировке",
                            "Upload this vault to the server on lock",
                        ))
                        .size(12.0),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Свой путь на сервере для этого контейнера (пусто — общий из настроек):",
                            "Per-vault server path (empty — the one from Settings):",
                        ))
                        .size(11.0)
                        .color(muted()),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings.remote_path)
                            .hint_text("/home/user/vault.spv")
                            .desired_width(330.0),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(lang.t("Сохранить в контейнер", "Save into vault"))
                                        .color(on_accent()),
                                )
                                .fill(accent())
                                .corner_radius(8),
                            )
                            .clicked()
                        {
                            let data = self.settings.encode();
                            let _ = self.vault.mkdir(".secretpiece");
                            let _ = self.vault.put(".secretpiece/settings", data.as_bytes());
                            set_preview_root(&self.settings.preview_root);
                            self.refresh();
                            self.status = Some(Status {
                                text: lang
                                    .t("Настройки контейнера сохранены.", "Vault settings saved.")
                                    .to_owned(),
                                error: false,
                            });
                        }
                        ui.label(
                            RichText::new(lang.t(
                                "После смены папки новые файлы пишутся уже туда.",
                                "New extractions use the new folder.",
                            ))
                            .size(10.0)
                            .color(muted()),
                        );
                    });
                });
            self.container_open = open;
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

        if let Some(external_path) = self.external.last().map(|external| external.path.clone()) {
            let mut wipe_now = false;
            let count = self.external.len();
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
                                    "Временно открыто: {external_path} (файлов: {count}, затирание при закрытии: {} проход(ов))",
                                    wipe_passes()
                                ),
                                Lang::En => format!(
                                    "Open temporarily: {external_path} ({count} file(s), {} pass(es) wiped on close)",
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
                self.external.clear();
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

#[allow(clippy::struct_excessive_bools)]
struct RowClick {
    selected: bool,
    open: bool,
    preview: bool,
    toggle: bool,
}

fn row(
    ui: &mut egui::Ui,
    lang: Lang,
    entry: &ListEntry,
    selected: bool,
    checked: bool,
) -> RowClick {
    let mut checked = checked;
    let mut toggle = false;
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
            toggle = ui.checkbox(&mut checked, "").changed();
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
        toggle,
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
        (Lang::Ru, ErrorCode::Corrupted) => "Хранилище повреждено или это не контейнер SPV1.",
        (Lang::En, ErrorCode::Corrupted) => "The vault is damaged or not an SPV1 container.",
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
        assert_eq!(Theme::from_key("paper"), Theme::Paper);
        assert_eq!(Theme::from_key("mint"), Theme::Mint);
        assert!(Theme::Paper.is_light());
        assert!(Theme::Mint.is_light());
        assert_eq!(Theme::Neon.key(), "neon");
    }

    #[test]
    fn presets_roundtrip() {
        let preset = Preset {
            name: "home".to_owned(),
            kind: RemoteKind::Ftp,
            host: "example.com".to_owned(),
            port: "21".to_owned(),
            user: "user".to_owned(),
            password: "secret".to_owned(),
            path: "/vault.spv".to_owned(),
        };
        let decoded = Preset::decode(&preset.encode()).expect("preset decodes");
        assert_eq!(decoded.name, "home");
        assert_eq!(decoded.kind, RemoteKind::Ftp);
        assert_eq!(decoded.host, "example.com");
        assert_eq!(decoded.password, "secret");
        assert_eq!(decoded.path, "/vault.spv");
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
