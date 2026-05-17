# Contributors

Bu dosya kisi listesi tutmaz. Canli katkici listesi GitHub gecmisinden README icinde otomatik gosterilir:

- https://github.com/noirlang/worm/graphs/contributors

## Nasil Destek Saglanir?

Worm adli bilisim odakli bir arac oldugu icin katkilarin test edilebilir, tekrar uretilebilir ve guvenli olmasi gerekir.

- Hata bildirirken isletim sistemi, mimari, kullandigin komut, beklenen sonuc, gercek sonuc ve ilgili loglari ekle.
- Disk/RAM edinimiyle ilgili hatalarda gercek kisi verisi veya hassas imaj paylasma; mumkunse kucuk test imaji veya tekrar uretim adimlari ver.
- Agent protokolu, uzak edinim, durdur/devam veya mount akisi degisecekse once davranis etkisini acikla.
- Pull requestlerde `cargo fmt`, `cargo test --locked` ve ilgili UI/agent kontrollerinin gectigini belirt.
- Dokumantasyon katkilari da destek sayilir: eksik kurulum adimi, yanlis platform notu veya anlasilmayan hata mesaji duzeltilebilir.
- Guvenlik acigi suphelerinde herkese acik issue acmadan once maintainer ile ozel kanaldan iletisime gec.

## Support Guidelines

This file does not list individual contributors. The live contributor view is rendered from GitHub history in the README.

- Include OS, architecture, command, expected result, actual result, and logs when reporting bugs.
- Do not share sensitive forensic data; prefer small test images or clear reproduction steps.
- Explain behavior impact before changing agent protocol, remote acquisition, pause/resume/stop, or mount flows.
- For pull requests, note that `cargo fmt`, `cargo test --locked`, and relevant UI/agent checks pass.
- Documentation fixes are valid contributions when they clarify setup, platform behavior, or error handling.
- Report suspected security issues privately instead of opening a public issue first.
