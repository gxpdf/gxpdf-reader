use gtk::{gdk, gio};
use gxpdf_reader::App;
use relm4::prelude::*;

fn main() {
    let app = RelmApp::new("com.gxpdf.Reader");
    initialize_custom_icons();
    rust_i18n::set_locale("en");
    
    let args: Vec<String> = std::env::args().collect();
    let mut init_string: Option<String> = None;
    if args.len() > 1 {
        init_string = Some(args[1].clone());
        app.with_args(vec![]).run_async::<App>(init_string);
    } else {
        //init_string = Some(String::from("test/1.pdf"));
        app.run_async::<App>(init_string);
    }
}

fn initialize_custom_icons() {
    gio::resources_register_include!("icons.gresource").unwrap();
    let display = gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/com/example/Foobar/icons");
}
