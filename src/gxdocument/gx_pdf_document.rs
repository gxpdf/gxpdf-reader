use std::{collections::HashMap, path::PathBuf};

use pdfium_render::prelude::PdfDocument;
use tokio::sync::Mutex;

use crate::{components::gx_window::{CURRENTTAB, GXPDFDOCUMENT, TABKEYS}, 
    utils::gx_pdfium::GxPdfium
};

#[derive(Default,Clone,Debug)]
pub struct GxPageSize{
    pub width:f32,
    pub height:f32,
}

#[derive(Default)]
pub struct GxPdfDocument<'a> {
    pub file_path: Option<PathBuf>,
    pub pdf_document: Option<PdfDocument<'a>>,
    //pub gx_pdfium: Option<GxPdfium<'a>>,
}


impl<'a> GxPdfDocument<'a> {
    //关联函数：初始化静态变量 GXPDFDOCUMENT，并将pdf文件的路径设置到静态变量 GXPDFDOCUMENT中
    pub async fn init_gxpdfdocument_and_set_static_file_path(file_path:Option<PathBuf>) {
        // 初始化 TESTDOCUMENT（如果还没有初始化）
        let tabkeys = *(TABKEYS.get().unwrap());
        let mut hm_gxdocument
            = HashMap::<i32,GxPdfDocument>::new();
        let mut gx_pdf_document = GxPdfDocument::default();

        gx_pdf_document.file_path = file_path;


        hm_gxdocument.insert(tabkeys,gx_pdf_document);
        GXPDFDOCUMENT.get_or_init(|| Mutex::new(hm_gxdocument));
        // 获取文档的可变引用
        //let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
        //let mut document_guard = document_mutex.lock().await;
        //document_guard.as_mut().unwrap().file_path = Some(file_path.to_path_buf());
    }
    
    //关联函数：从静态变量GXPDFDOCUMENT中获取Option<GxPdfDocument<'static>> 
    //pub async fn get_static_pdf_document() -> Option<&'static GxPdfDocument<'static>> {
    //    let current_tab = *(CURRENTTAB.get().unwrap());
    //    let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
    //    let document_guard = document_mutex.lock().await;
    //    document_guard.get(&current_tab)
 
    //}

    //关联函数：从静态变量GXPDFDOCUMENT中获取Option<PathBuf> 
    pub async fn get_static_path_file() -> Option<String> {
        let current_tab = *(CURRENTTAB.get().unwrap());
        let document_mutex 
            = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
        let document_guard = document_mutex.lock().await;
        //let pdf_document =  document_guard.get(&current_tab);
 
        if let Some(gx_pdf_document) 
            = document_guard.get(&current_tab){
            let file_path = gx_pdf_document.file_path.clone();
            Some(file_path.unwrap().to_string_lossy().to_string())
        }
        else{
            None
        }
    }


    //关联函数：通过静态变量打开pdf文件，并将其值设置到静态变量 GXPDFDOCUMENT中
    pub async fn static_open_file(){
        //let gx_pdfium = GXPDFIUM.get().expect("Could not get Pdfium from OnceLock").lock().await;
        let current_tab = *(CURRENTTAB.get().unwrap());
        //let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
        //let mut document_guard = document_mutex.lock().await;
        if let Some(file_path) = GxPdfDocument::get_static_path_file().await{
            let pdf_document 
                = GxPdfium::open_pdf_from_file(file_path, None).await;
            if let Some(pdf_document) = pdf_document{
                let document_mutex 
                    = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
                let mut document_guard 
                    = document_mutex.lock().await;
                if let Some(gx_pdf_document) 
                    = document_guard.get_mut(&current_tab){
                    gx_pdf_document.pdf_document = Some(pdf_document);
                }
            }

        }

    }


    

}


#[derive(Default,Clone,Debug)]
pub struct GxRectangle {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}
#[allow(unused)]
#[derive(Default)]
pub struct GxPoint {
    pub x: f64,
    pub y: f64,
}

#[allow(unused)]
pub struct GxMapping {
    pub area: GxRectangle,
    //pub data: Box<dyn Any>,
}
//impl Default for GxMapping {
//    fn default() -> Self {
//        Self {
//            area: GxRectangle::default(),
//            data: Box::new(()),
//        }
//        
//    }
//}