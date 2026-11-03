fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/keystats.ico");
    res.set("FileDescription", "按键统计-keystats");
    res.set("ProductName", "按键统计-keystats");
    res.set("OriginalFilename", "keystats.exe");
    res.compile().expect("嵌入程序图标失败");
}
