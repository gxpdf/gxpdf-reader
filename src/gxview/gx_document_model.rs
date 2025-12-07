use std::path::PathBuf;

use crate::{components::gx_window::{DEFAULT_MAX_SCALE, DEFAULT_MIN_SCALE}, gxdocument::gx_pdf_document::GxPageSize};

#[derive(Default,Clone,Debug)]
pub struct GxDocumentModel {
    //pub document: Arc<Mutex<Option<GxDocument<'a>>>>,
    pub page_numbers: i32,
    pub rotation: i32,
    pub scale: f32,
    pub sizing_mode: GxSizingMode,
    //pub page_layout: GxPageLayout,//暂时先不实现了
    pub continuous: bool,
    pub dual_page: bool,
    pub dual_page_odd_left: bool,
    pub rtl: bool,
    pub fullscreen: bool,
    pub inverted_colors: bool,
    pub max_scale: f32,
    pub min_scale: f32,
    pub current_page: i32,
    //后面这些是直接从GxDocument复制过来的，不用维护两个结构体了，有时候取值很麻烦
    pub file_path: PathBuf,//代替了uri
    pub max_height: f64,
    pub max_width: f64,
    pub min_height: f64,
    pub min_width: f64,
    pub page_sizes: Vec<GxPageSize>,
    pub uniform: bool,
    pub uniform_height: f64,
    pub uniform_width: f64,
}

impl GxDocumentModel { 
    pub fn init() -> Self{
        Self { page_numbers:0,
            current_page:-1, 
            rotation: 0, 
            scale:1.0, 
            sizing_mode:GxSizingMode::FitPage, 
            continuous:true, 
            dual_page:false, 
            dual_page_odd_left:false, 
            rtl:true, 
            fullscreen:false, 
            inverted_colors:false, 
            max_scale: DEFAULT_MAX_SCALE,
            min_scale: DEFAULT_MIN_SCALE,
            file_path: PathBuf::default(),
            max_height: 0.0,
            max_width: 0.0,
            min_height: 0.0,
            min_width: 0.0,
            page_sizes:Vec::new(),
            uniform: false,
            uniform_height: 0.0,
            uniform_width: 0.0,
        }
    }
    
    pub fn set_page(&mut self,page:i32){
        if self.current_page == page{
            return;
        }
        if page < 0 || page >= self.page_numbers{
            return;
        }
        self.current_page = page;
    }
}
#[derive(Default,Clone,Debug)]
pub enum GxSizingMode {
    #[default]
    FitPage,
    FitWidth,
    Free,
    Automatic,
}
#[derive(Default,Clone,Debug)]
pub enum GxPageLayout {
    #[default]
    Single,
    Dual,
    Automatic,
}
