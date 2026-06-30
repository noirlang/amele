fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("packaging/windows/amele.ico");
        res.compile().unwrap();
    }
}
