# egui + eframe + wgpu

I'm playing with these trying to get a sense for what it's like to have some wgpu
driven widgets rendered in a browser.

## Getting started

Based on [eframe template](https://github.com/emilk/eframe_template/)

To serve as a web app, run:
```bash
RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve
```
```powershell
$env:RUSTFLAGS="--cfg=web_sys_unstable_apis"; trunk serve
```

`cargo run` will start a desktop app.

## Trouble?

1. Try clearing the application's storage in the browser. I did this in chrome by opening the dev tools, navigating to the "Application" tab, clicking on "Storage" and hitting the "Clear site data" button. Then I refresh the page. 
