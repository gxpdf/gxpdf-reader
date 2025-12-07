use std::{collections::HashMap, path::PathBuf, sync::OnceLock};
use tokio::sync::Mutex;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::{gxdocument::gx_pdf_document::GxPdfDocument, 
    gxview::gx_view::{GxView, GxViewInit}, 
    utils::{gx_job_scheduler::GxScheduler, gx_pdfium::GxPdfium}
};

//CURRENTTAB  initially sets to 0. Each time a new tab is opened, 
//this value is set to the value of TABKEYS.
//If another tab is clicked, is it reset via GxWindow's tabkey?
//Both TABKEYS and CURRENTTAB are initialized in gx_pdfium's init_gxpdfium_and_tabkey.
pub static CURRENTTAB:OnceLock<i32> = OnceLock::new();

pub static DEFAULT_MAX_SCALE: f32 = 5.0;
pub static DEFAULT_MIN_SCALE: f32 = 0.25;
pub static DEFAULT_PIXBUF_CACHE_SIZE: usize = 52428800; // 50MB 
//HashMap is for multiple tabs.
pub static GXPDFDOCUMENT:OnceLock<Mutex<HashMap<i32,GxPdfDocument<'static>>>> = OnceLock::new();
pub static GX_VIEW_PAGES_DEFAULT_LEN: i32 = 25;
pub static MAX_PRELOADED_PAGES: i32 = 3;
// the max size of cairo's image
pub static MAX_IMAGE_SIZE:i32 = 32767;
pub static MIN_SCALE: f32 = 0.05409;
//TABKEYS initially sets to 0, each new tab increases this value by 1.
pub static TABKEYS: OnceLock<i32> = OnceLock::new();
pub static ZOOM_IN_FACTOR:f32 = 1.2;
pub static ZOOM_OUT_FACTOR:f32 = 1.0/ZOOM_IN_FACTOR;

#[derive(Debug)]
pub enum GxWindowMsg {
}

pub struct GxWindowInit{
    pub file_path: Option<PathBuf>,
    #[allow(unused)]
    pub search_string: Option<String>,
    pub job_scheduler: Option<GxScheduler>,   
    pub start_page:i32,
}

#[allow(unused)]
pub struct GxWindow{
    file_path: Option<PathBuf>,
    search_string: Option<String>,
    job_scheduler: Option<GxScheduler>,
    tabkey: i32,
    view: AsyncController<GxView>,
}
#[relm4::component(pub async)]
impl AsyncComponent for GxWindow{
    type Init = GxWindowInit;
    type Input = GxWindowMsg;
    type Output = ();
    type CommandOutput = ();
    view! {
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,
            append =  model.view.widget(),
        },
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        // The following two lines initialize GXPDFIUM and GXPDFDOCUMENT, 
        // and initialize the file_path of GXPDFDOCUMENT
        GxPdfium::init_tabkey();
        GxPdfDocument::init_gxpdfdocument_and_set_static_file_path(
            init.file_path.clone()).await;
        let tabkey = *(CURRENTTAB.get().unwrap());
        let view_init = GxViewInit{
            file_path:init.file_path.clone(),
            job_scheduler:init.job_scheduler.clone(),
            start_page:init.start_page,
        };
        let view = GxView::builder().
            launch(view_init).detach();
        let model = GxWindow{
            file_path:init.file_path.clone(), 
            search_string:init.search_string,
            job_scheduler:init.job_scheduler,
            tabkey:tabkey,
            view,
        };
        GxPdfDocument::static_open_file().await;
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
    async fn update(&mut self, 
        _msg: Self::Input, 
        _sender: AsyncComponentSender<Self>, 
        _root: &Self::Root) {
    }
    async fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        _msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }
}

