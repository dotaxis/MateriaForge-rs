pub trait InstallTarget {
    fn target_key(&self) -> &'static str;
    fn mod_loader_name(&self) -> &'static str;
    fn mod_loader_repo(&self) -> &'static str;
    fn install_log_name(&self) -> &'static str;
    fn launch_binary_name(&self) -> &'static str;
    fn desktop_template(&self) -> &'static str;
    fn desktop_file_name(&self, app_id: u32) -> String;
    fn icon_name(&self) -> &'static str;
    fn icon_file_name(&self) -> &'static str;
    fn icon_bytes(&self) -> &'static [u8];

    fn github_asset_pattern(&self) -> &'static str {
        ".exe"
    }

    fn install_dir_name(&self) -> &'static str {
        self.mod_loader_name()
    }

    fn launch_binary_with_identifier(&self, app_id: u32) -> String {
        let shortcut_identifier = self.shortcut_identifier(app_id);
        if shortcut_identifier.is_empty() {
            self.launch_binary_name().to_string()
        } else {
            format!("{} {}", self.launch_binary_name(), shortcut_identifier)
        }
    }

    fn shortcut_identifier(&self, _app_id: u32) -> &'static str {
        ""
    }
}
