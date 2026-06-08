use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NibsTheme {
    pub name: &'static str,
    pub id: &'static str,
    pub neutral: Color,
    pub ink: Color,
    pub primary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub select_fg: Color,
    pub select_bg: Color,
}

impl NibsTheme {
    pub fn next(self) -> &'static NibsTheme {
        let themes = ALL_THEMES;
        let pos = themes.iter().position(|t| t.id == self.id);
        match pos {
            Some(i) if i + 1 < themes.len() => &themes[i + 1],
            _ => &themes[0],
        }
    }
}

pub const SYSTEM: NibsTheme = NibsTheme {
    name: "System (Terminal Default)",
    id: "system",
    neutral: Color::Reset,
    ink: Color::Reset,
    primary: Color::Magenta,
    accent: Color::LightBlue,
    success: Color::Green,
    warning: Color::Yellow,
    error: Color::Red,
    info: Color::Blue,
    select_fg: Color::Black,
    select_bg: Color::Magenta,
};

pub const NORD: NibsTheme = NibsTheme {
    name: "Nord",
    id: "nord",
    neutral: Color::Rgb(46, 52, 64),
    ink: Color::Rgb(229, 233, 240),
    primary: Color::Rgb(136, 192, 208),
    accent: Color::Rgb(213, 119, 128),
    success: Color::Rgb(163, 190, 140),
    warning: Color::Rgb(208, 135, 112),
    error: Color::Rgb(191, 97, 106),
    info: Color::Rgb(129, 161, 193),
    select_fg: Color::Rgb(46, 52, 64),
    select_bg: Color::Rgb(136, 192, 208),
};

pub const DRACULA: NibsTheme = NibsTheme {
    name: "Dracula",
    id: "dracula",
    neutral: Color::Rgb(29, 30, 40),
    ink: Color::Rgb(248, 248, 242),
    primary: Color::Rgb(189, 147, 249),
    accent: Color::Rgb(255, 121, 198),
    success: Color::Rgb(80, 250, 123),
    warning: Color::Rgb(255, 184, 108),
    error: Color::Rgb(255, 85, 85),
    info: Color::Rgb(139, 233, 253),
    select_fg: Color::Rgb(29, 30, 40),
    select_bg: Color::Rgb(189, 147, 249),
};

pub const CATPPUCCIN: NibsTheme = NibsTheme {
    name: "Catppuccin",
    id: "catppuccin",
    neutral: Color::Rgb(30, 30, 46),
    ink: Color::Rgb(205, 214, 244),
    primary: Color::Rgb(180, 190, 254),
    accent: Color::Rgb(243, 139, 168),
    success: Color::Rgb(166, 209, 137),
    warning: Color::Rgb(244, 184, 228),
    error: Color::Rgb(243, 139, 168),
    info: Color::Rgb(137, 220, 235),
    select_fg: Color::Rgb(30, 30, 46),
    select_bg: Color::Rgb(180, 190, 254),
};

pub const GRUVBOX: NibsTheme = NibsTheme {
    name: "Gruvbox",
    id: "gruvbox",
    neutral: Color::Rgb(40, 40, 40),
    ink: Color::Rgb(235, 219, 178),
    primary: Color::Rgb(131, 165, 152),
    accent: Color::Rgb(251, 73, 52),
    success: Color::Rgb(184, 187, 38),
    warning: Color::Rgb(250, 189, 47),
    error: Color::Rgb(251, 73, 52),
    info: Color::Rgb(211, 134, 155),
    select_fg: Color::Rgb(40, 40, 40),
    select_bg: Color::Rgb(131, 165, 152),
};

pub const TOKYONIGHT: NibsTheme = NibsTheme {
    name: "Tokyo Night",
    id: "tokyonight",
    neutral: Color::Rgb(26, 27, 38),
    ink: Color::Rgb(192, 202, 245),
    primary: Color::Rgb(122, 162, 247),
    accent: Color::Rgb(255, 158, 100),
    success: Color::Rgb(158, 206, 106),
    warning: Color::Rgb(224, 175, 104),
    error: Color::Rgb(247, 118, 142),
    info: Color::Rgb(125, 207, 255),
    select_fg: Color::Rgb(26, 27, 38),
    select_bg: Color::Rgb(122, 162, 247),
};

pub const ONE_DARK: NibsTheme = NibsTheme {
    name: "One Dark",
    id: "one-dark",
    neutral: Color::Rgb(40, 44, 52),
    ink: Color::Rgb(171, 178, 191),
    primary: Color::Rgb(97, 175, 239),
    accent: Color::Rgb(86, 182, 194),
    success: Color::Rgb(152, 195, 121),
    warning: Color::Rgb(229, 192, 123),
    error: Color::Rgb(224, 108, 117),
    info: Color::Rgb(209, 154, 102),
    select_fg: Color::Rgb(40, 44, 52),
    select_bg: Color::Rgb(97, 175, 239),
};

pub const SOLARIZED: NibsTheme = NibsTheme {
    name: "Solarized Dark",
    id: "solarized",
    neutral: Color::Rgb(0, 43, 54),
    ink: Color::Rgb(147, 161, 161),
    primary: Color::Rgb(108, 113, 196),
    accent: Color::Rgb(211, 54, 130),
    success: Color::Rgb(133, 153, 0),
    warning: Color::Rgb(181, 137, 0),
    error: Color::Rgb(220, 50, 47),
    info: Color::Rgb(42, 161, 152),
    select_fg: Color::Rgb(0, 43, 54),
    select_bg: Color::Rgb(108, 113, 196),
};

pub const MONOKAI: NibsTheme = NibsTheme {
    name: "Monokai",
    id: "monokai",
    neutral: Color::Rgb(39, 40, 34),
    ink: Color::Rgb(248, 248, 242),
    primary: Color::Rgb(174, 129, 255),
    accent: Color::Rgb(249, 38, 114),
    success: Color::Rgb(166, 226, 46),
    warning: Color::Rgb(253, 151, 31),
    error: Color::Rgb(249, 38, 114),
    info: Color::Rgb(102, 217, 239),
    select_fg: Color::Rgb(39, 40, 34),
    select_bg: Color::Rgb(174, 129, 255),
};

pub const EVERFOREST: NibsTheme = NibsTheme {
    name: "Everforest",
    id: "everforest",
    neutral: Color::Rgb(45, 53, 59),
    ink: Color::Rgb(211, 198, 170),
    primary: Color::Rgb(167, 192, 128),
    accent: Color::Rgb(214, 153, 182),
    success: Color::Rgb(167, 192, 128),
    warning: Color::Rgb(230, 152, 117),
    error: Color::Rgb(230, 126, 128),
    info: Color::Rgb(131, 192, 146),
    select_fg: Color::Rgb(45, 53, 59),
    select_bg: Color::Rgb(167, 192, 128),
};

pub const KANAGAWA: NibsTheme = NibsTheme {
    name: "Kanagawa",
    id: "kanagawa",
    neutral: Color::Rgb(31, 31, 40),
    ink: Color::Rgb(220, 215, 186),
    primary: Color::Rgb(126, 156, 216),
    accent: Color::Rgb(210, 126, 153),
    success: Color::Rgb(152, 187, 108),
    warning: Color::Rgb(215, 166, 87),
    error: Color::Rgb(232, 36, 36),
    info: Color::Rgb(118, 148, 106),
    select_fg: Color::Rgb(31, 31, 40),
    select_bg: Color::Rgb(126, 156, 216),
};

pub const ROSE_PINE: NibsTheme = NibsTheme {
    name: "Rose Pine",
    id: "rosepine",
    neutral: Color::Rgb(25, 23, 36),
    ink: Color::Rgb(224, 222, 244),
    primary: Color::Rgb(156, 207, 216),
    accent: Color::Rgb(235, 188, 186),
    success: Color::Rgb(49, 116, 143),
    warning: Color::Rgb(246, 193, 119),
    error: Color::Rgb(235, 111, 146),
    info: Color::Rgb(156, 207, 216),
    select_fg: Color::Rgb(25, 23, 36),
    select_bg: Color::Rgb(156, 207, 216),
};

pub const ALL_THEMES: &[NibsTheme] = &[
    SYSTEM, NORD, DRACULA, CATPPUCCIN, GRUVBOX, TOKYONIGHT, ONE_DARK, SOLARIZED, MONOKAI,
    EVERFOREST, KANAGAWA, ROSE_PINE,
];
