use worm_rewrite_rust::settings::AppSettings;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("settings-default") => {
            let settings = AppSettings::default();
            println!(
                "{}",
                serde_json::to_string_pretty(&settings).expect("settings serialization failed")
            );
        }
        Some("--help") | Some("-h") | None => print_help(),
        Some(other) => {
            eprintln!("Bilinmeyen komut: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!(
        "worm-rewrite-rust teknik CLI\n\n\
         Komutlar:\n\
           settings-default        Varsayilan ayarlari JSON olarak yazdir\n\n\
         Not: UI bu crate'e daha sonra Tauri tarafindan baglanacak."
    );
}
