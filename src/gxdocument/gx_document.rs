use std::path::PathBuf;

use crate::{components::gx_window::{CURRENTTAB, GXPDFDOCUMENT}, gxdocument::gx_pdf_document::GxPageSize};

#[derive(Default,Clone,Debug)]
pub struct GxDocument{
    //pub cache_loaded: bool,
    pub file_path: PathBuf,//代替了uri
    //pub file_size: i64,
    pub max_height: f64,
    //pub max_label: i32,
    pub max_width: f64,
    pub min_height: f64,
    pub min_width: f64,
    //pub modified: bool,
    //pub page_labels: Vec<String>,
    pub page_sizes: Vec<GxPageSize>,
    pub uniform: bool,
    pub uniform_height: f64,
    pub uniform_width: f64,
}

impl GxDocument{
    pub async fn gx_document_get_page_numbers(&self) -> Option<i32>{
        if let Some(document_mutex) 
            = GXPDFDOCUMENT.get(){
            let document_guard = document_mutex.read().await;
            if let Some(current_tab) = CURRENTTAB.get(){
                if let Some(gx_pdf_document) 
                    = document_guard.get(current_tab){
                    if let Some(pdf_document) = &gx_pdf_document.pdf_document{
                        let page_numbers = pdf_document.pages().len();
                        return Some(page_numbers as i32);
                    }
                }               
            }
        }
        None
    }
    
    //pub async fn gx_document_get_page_size(&self,page_index:i32) -> Option<(f32,f32)>{
    //    if page_index < 0 || page_index 
    //}
}