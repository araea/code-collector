use winres::WindowsResource;

fn main() {
    if cfg!(target_os = "windows") {
        WindowsResource::new()
            // 设置图标文件路径
            .set_icon("assets/icon.ico")
            // 设置产品名称
            .set("ProductName", "Code Collector")
            // 设置文件描述
            .set("FileDescription", "Code Collection Tool")
            .compile()
            .unwrap();
    }
}
