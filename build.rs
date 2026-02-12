fn main() {
    #[cfg(feature = "gui")]
    {
        let slint_path = std::path::PathBuf::from("ui/app_window.slint");
        slint_build::compile(&slint_path).expect("Slint 编译失败");
    }
}
