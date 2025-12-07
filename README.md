<h3 align="center">
    A new pdf reader based on relm4 and pdfium-render library.
    Its goal is to provide a feature-rich pdf reader with a focus on performance and ease of use and cross windows,macos,linux .
    It works on linux now!
</h3>

<br>

## Get Started
### Linux
- Install from source code:
```
    Clone the repository: `git clone https://github.com/gxpdf/gxpdf-reader.git`
    Build the project: `cargo build --release`
    cp -rv pdfium-lib/libpdfium.so ./target/release/
    Run the application: `./target/release/gxpdf-reader test/annotations-test.pdf`
```
- Run with cargo run
```
    Clone the repository: `git clone https://github.com/gxpdf/gxpdf-reader.git`
    Build the project for debug: `cargo build`
    cp -rv pdfium-lib/libpdfium.so ./target/debug/
    Run the application: `cargo run -- test/annotations-test.pdf`
```

## License

The License will be added later.
## Copyright

Copyright (C) 2025, gxpdf.com 
