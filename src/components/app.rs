use std::path::PathBuf;

use gtk::prelude::*;
use relm4::prelude::*;

use crate::{components::gx_window::{GxWindow, GxWindowInit}, utils::gx_job_scheduler::GxScheduler};

#[allow(unused)]
pub struct App {
    file_path: Option<PathBuf>,
    main_box: AsyncController<GxWindow>,
    job_scheduler: GxScheduler ,
}

#[relm4::component(pub async)]
impl AsyncComponent for App {
    type Init = Option<String>;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    view! {
        main_window = gtk::ApplicationWindow{
            set_maximized: true,
            set_title: Some("gxpdf reader"),
            #[wrap(Some)]
            set_child = model.main_box.widget(),
        } 
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<App>,
    ) -> AsyncComponentParts<Self> {
        let mut file_path: Option<PathBuf> = None;
        if let Some(init_file_path) = init{
            file_path= Some(PathBuf::from(init_file_path));
        }
        let job_scheduler = GxScheduler::scheduler_new().await;
        let window_init = GxWindowInit{
            file_path:file_path.clone(),
            search_string:None,
            job_scheduler:Some(job_scheduler.clone()),
            start_page:0,
        };                                             
        let main_box= GxWindow::builder().
            launch(window_init).detach();
        let model = App {
            file_path, 
            main_box, 
            job_scheduler,
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }


}

