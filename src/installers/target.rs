use crate::resource_handler;

pub const FF7_APPID: u32 = 39140;
pub const FF7_2026_APPID: u32 = 3837340;
pub const FF7_GOG_APPID: u32 = 1698970154;
pub const FF8_APPID: u32 = 39150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallTarget {
    FF7,
    FF8,
}

impl InstallTarget {
    pub fn target_key(self) -> &'static str {
        match self {
            Self::FF7 => "ff7",
            Self::FF8 => "ff8",
        }
    }

    pub fn mod_loader_name(self) -> &'static str {
        match self {
            Self::FF7 => "7th Heaven",
            Self::FF8 => "Junction VIII",
        }
    }

    pub fn mod_loader_repo(self) -> &'static str {
        match self {
            Self::FF7 => "tsunamods-codes/7th-Heaven",
            Self::FF8 => "tsunamods-codes/Junction-VIII",
        }
    }

    pub fn install_dir_name(self) -> &'static str {
        self.mod_loader_name()
    }

    pub fn install_log_name(self) -> &'static str {
        match self {
            Self::FF7 => "7thHeaven.log",
            Self::FF8 => "JunctionVIII.log",
        }
    }

    pub fn launch_binary_name(self) -> &'static str {
        match self {
            Self::FF7 => "Launch 7th Heaven",
            Self::FF8 => "Launch Junction VIII",
        }
    }

    pub fn desktop_template(self) -> &'static str {
        match self {
            Self::FF7 => resource_handler::SEVENTH_HEAVEN_SHORTCUT_FILE,
            Self::FF8 => resource_handler::JUNCTION_VIII_SHORTCUT_FILE,
        }
    }

    pub fn desktop_file_name(self, shortcut_identifier: &str) -> String {
        match self {
            Self::FF7 => format!("7th Heaven {}.desktop", shortcut_identifier),
            Self::FF8 => "Junction VIII.desktop".to_string(),
        }
    }

    pub fn icon_name(self) -> &'static str {
        match self {
            Self::FF7 => "7th-heaven",
            Self::FF8 => "Junction-VIII",
        }
    }

    pub fn icon_file_name(self) -> &'static str {
        match self {
            Self::FF7 => "7th-heaven.png",
            Self::FF8 => "Junction-VIII.png",
        }
    }

    pub fn icon_bytes(self) -> &'static [u8] {
        match self {
            Self::FF7 => resource_handler::LOGO_PNG,
            Self::FF8 => resource_handler::JUNCTION_LOGO_PNG,
        }
    }
}

pub fn shortcut_identifier(app_id: u32) -> &'static str {
    match app_id {
        FF7_APPID => "(2013)",
        FF7_2026_APPID => "(2026)",
        FF7_GOG_APPID => "(GOG)",
        FF8_APPID => "",
        _ => "(Unknown)",
    }
}
