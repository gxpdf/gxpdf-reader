use bitflags::bitflags;
use pdfium_render::prelude::{PdfDocumentMetadataTag, PdfDocumentMetadataTagType, PdfDocumentVersion, PdfPageMode, PdfSecurityHandlerRevision};

use crate::components::gx_window::{CURRENTTAB, GXPDFDOCUMENT};
#[derive(Default)]
pub struct GxDocumentInfo {
    pub pdf_version: Option<PdfDocumentVersion>, // 例如，"pdf-1.5"
    pub title: Option<PdfDocumentMetadataTag>,
    pub author: Option<PdfDocumentMetadataTag>,
    pub subject: Option<PdfDocumentMetadataTag>,
    pub keywords: Option<PdfDocumentMetadataTag>,
    pub creator: Option<PdfDocumentMetadataTag>,
    pub producer: Option<PdfDocumentMetadataTag>,
    //pub linearized: Option<String>,//这个主要用于网络传输，先不实现了
    pub creation_date: Option<PdfDocumentMetadataTag>, // GTime 在 Rust 中可以用 i64 表示
    pub modified_date: Option<PdfDocumentMetadataTag>,
    pub has_security: Option<String>,
    //pub layout: GxDocumentLayout, // 需要定义此枚举，pdfium没提供此功能，暂时先不实现了
    pub page_mode: GxDocumentMode,     // 需要定义此枚举
    //pub ui_hints: u32,            // guint 对应 u32，暂时不用，因为pdfium-render没有提供相关api
    pub page_numbers: i32,      // int 对应 i32
    pub paper_height_mm: f32, // double 对应 f64
    pub paper_width_mm: f32,                          
    pub doc_permissions: GxDocumentPermissions,
    //pub license: Option<GxDocumentLicense>,//暂时先不实现了
    //pub contains_js: GxDocumentContainsJS, // 需要定义此枚举,暂时先不实现了
    // 有效字段的掩码
    pub fields_mask: GxDocumentInfoFields,
}

impl GxDocumentInfo{
    //设置pdf file中的info值到静态变量 GXPDFDOCUMENT中存储起来
    pub async fn set_static_pdf_document_info() -> Option<Self>{
        let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
        let mut document_guard = document_mutex.lock().await;
        let current_tab = *(CURRENTTAB.get().unwrap());
        if let Some(gx_pdf_document) = document_guard.get_mut(&current_tab){
            let mut gx_document_info = GxDocumentInfo::default();
            
            if let Some(pdf_document) = &gx_pdf_document.pdf_document{
                let pdf_version 
                    = pdf_document.version();
                gx_document_info.pdf_version = Some(pdf_version);

                let title 
                    = pdf_document.
                        metadata().get(PdfDocumentMetadataTagType::Title);
                gx_document_info.title = title;

                let author 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::Author);            
                gx_document_info.author = author;
                
                let subject 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::Subject);
                gx_document_info.subject = subject;

                let keywords 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::Keywords);
                gx_document_info.keywords = keywords;
                
                let creator 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::Creator);
                gx_document_info.creator = creator;

                let producer 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::Producer);
                gx_document_info.producer = producer;

                let creation_date 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::CreationDate);
                gx_document_info.creation_date = creation_date;

                let modified_date 
                    = pdf_document
                        .metadata().get(PdfDocumentMetadataTagType::ModificationDate);
                gx_document_info.modified_date = modified_date;

                let has_security 
                    = match pdf_document
                        .permissions().security_handler_revision() {
                    Ok(PdfSecurityHandlerRevision::Unprotected) => Some("no".to_string()),
                    Ok(_) => Some("yes".to_string()),
                    Err(_) => Some("no".to_string())
                };
                gx_document_info.has_security = has_security;

                let pages  
                    = pdf_document.pages();
                let mode  = pages.page_mode();
    	        let page_mode = match mode {
                    PdfPageMode::None => GxDocumentMode::None,
                    PdfPageMode::UnsetOrUnknown => GxDocumentMode::UseOc,
                    PdfPageMode::ShowDocumentOutline => GxDocumentMode::OutLine,
                    PdfPageMode::ShowPageThumbnails => GxDocumentMode::UseThumbs,
                    PdfPageMode::Fullscreen => GxDocumentMode::FullScreen,
                    PdfPageMode::ShowContentGroupPanel => GxDocumentMode::ShowContentGroupPanel,
                    PdfPageMode::ShowAttachmentsPanel => GxDocumentMode::UseAttachments,
                };
                gx_document_info.page_mode = page_mode;

                let page_numbers = pages.len();
                gx_document_info.page_numbers = page_numbers as i32;

                if gx_document_info.page_numbers > 0{
                   let page_one = pages.get(0).ok().unwrap();
                   let page_width_point = page_one.width();
                   let page_height_point = page_one.height();               
                   gx_document_info.paper_width_mm = (page_width_point.value / 72.0) * 25.4;
                   gx_document_info.paper_height_mm = (page_height_point.value / 72.0) * 25.4; 
                }

                let permissions 
                    = pdf_document.permissions();
                let mut doc_permissions 
                    = GxDocumentPermissions::empty();
                if let Ok(_) 
                    = permissions.can_assemble_document() {
                   doc_permissions |= GxDocumentPermissions::CAN_ASSEMBLE_DOCUMENT; 
                }
                if let Ok(_) 
                    = permissions.can_add_or_modify_text_annotations() {
                    doc_permissions |= GxDocumentPermissions::CAN_ADD_OR_MODIFY_TEXT_ANNOTATIONS;
                }
                if let Ok(_) 
                    = permissions.can_create_new_interactive_form_fields() {
                    doc_permissions |= GxDocumentPermissions::CAN_CREATE_NEW_INTERACTIVE_FORM_FIELDS;
                }
                if let Ok(_) 
                    = permissions.can_extract_text_and_graphics() {
                    doc_permissions |= GxDocumentPermissions::CAN_EXTRACT_TEXT_AND_GRAPHICS;
                }
                if let Ok(_) 
                    = permissions.can_fill_existing_interactive_form_fields() {
                    doc_permissions |= GxDocumentPermissions::CAN_FILL_EXISTING_INTERACTIVE_FORM_FIELDS;
                }
                if let Ok(_) 
                    = permissions.can_modify_document_content() {
                    doc_permissions |= GxDocumentPermissions::CAN_MODIFY_DOCUMENT_CONTENT;
                }
                if let Ok(_) 
                    = permissions.can_print_high_quality() {
                    doc_permissions |= GxDocumentPermissions::CAN_PRINT_HIGH_QUALITY;
                }
                if let Ok(_) 
                    = permissions.can_print_only_low_quality() {
                    doc_permissions |= GxDocumentPermissions::CAN_PRINT_ONLY_LOW_QUALITY;
                }
                gx_document_info.doc_permissions = doc_permissions;

                let mut fields_mask = GxDocumentInfoFields::empty();
                fields_mask |= GxDocumentInfoFields::INFO_LAYOUT |
    			    GxDocumentInfoFields::INFO_START_MODE |
                    GxDocumentInfoFields::INFO_PERMISSIONS |
    			    GxDocumentInfoFields::INFO_UI_HINTS |
    			    GxDocumentInfoFields::INFO_LINEARIZED |
    			    GxDocumentInfoFields::INFO_N_PAGES |
    			    GxDocumentInfoFields::INFO_SECURITY |
    		        GxDocumentInfoFields::INFO_PAPER_SIZE;
                if let Some(_) 
                    = gx_document_info.title {
                    fields_mask |= GxDocumentInfoFields::INFO_TITLE;
                }
                if let Some(_) 
                    = gx_document_info.pdf_version {
                    fields_mask |= GxDocumentInfoFields::INFO_VERSION;
                }
                if let Some(_) 
                    = gx_document_info.author {
                    fields_mask |= GxDocumentInfoFields::INFO_AUTHOR;
                }
                if let Some(_) 
                    = gx_document_info.subject {
                    fields_mask |= GxDocumentInfoFields::INFO_SUBJECT;
                }
                if let Some(_) 
                    = gx_document_info.keywords {
                    fields_mask |= GxDocumentInfoFields::INFO_KEYWORDS;
                }
                if let Some(_) 
                    = gx_document_info.creator {
                    fields_mask |= GxDocumentInfoFields::INFO_CREATOR;
                }
                if let Some(_) 
                    = gx_document_info.producer {
                    fields_mask |= GxDocumentInfoFields::INFO_PRODUCER;
                }

                gx_document_info.fields_mask = fields_mask;
                return Some(gx_document_info);
            }
        }
        None

    }
}

#[allow(unused)]
#[derive(Default)]
pub enum GxDocumentLayout {
    #[default]
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoColumnRight,
    TwoPageLeft,
    TwoPageRight,
}
#[derive(Default)]
pub enum GxDocumentMode {
    #[default]
    None,
    ShowContentGroupPanel,//显示内容组面板，不知道啥叫内容组 
    UseOc,//无法识别的边框
    UseThumbs,//展示缩略图面板
    FullScreen,
    UseAttachments,//显示附件面板
    #[allow(unused)]
    Presentation, // 与 FullScreen 具有相同的值
    OutLine,//pdfium独特的mode，展示pdf的文档大纲面板
}
#[allow(unused)]
#[derive(Default)]
pub enum GxDocumentContainsJS {
    #[default]
    Unknown,
    No,
    Yes,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GxDocumentUIHints: u32 {
        const HIDE_TOOLBAR = 1 << 0;
        const HIDE_MENUBAR = 1 << 1;
        const HIDE_WINDOWUI = 1 << 2;
        const FIT_WINDOW = 1 << 3;
        const CENTER_WINDOW = 1 << 4;
        const DISPLAY_DOC_TITLE = 1 << 5;
        const DIRECTION_RTL = 1 << 6;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,Default)]
    pub struct GxDocumentInfoFields: u32 {
	    const INFO_TITLE = 1 << 0;
	    const INFO_VERSION = 1 << 1;//对应的是evince中的FORMAT
	    const INFO_AUTHOR = 1 << 2;
	    const INFO_SUBJECT = 1 << 3;
	    const INFO_KEYWORDS = 1 << 4;
	    const INFO_LAYOUT = 1 << 5;
	    const INFO_CREATOR = 1 << 6;
	    const INFO_PRODUCER = 1 << 7;
	    const INFO_CREATION_DATE = 1 << 8;
	    const INFO_MOD_DATE = 1 << 9;
	    const INFO_LINEARIZED = 1 << 10;
	    const INFO_START_MODE = 1 << 11;
	    const INFO_UI_HINTS = 1 << 12;
	    const INFO_PERMISSIONS = 1 << 13;
	    const INFO_N_PAGES = 1 << 14;
	    const INFO_SECURITY = 1 << 15;
	    const INFO_PAPER_SIZE = 1 << 16;
	    const INFO_LICENSE = 1 << 17;
	    const INFO_CONTAINS_JS = 1 << 18;
	    const INFO_EXTENDED = 1 << 30; /*< skip >*/
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,Default)]
    pub struct  GxDocumentPermissions: u32 {
	    const CAN_ASSEMBLE_DOCUMENT = 1 << 0;
	    const CAN_PRINT_HIGH_QUALITY = 1 << 1;
	    const CAN_PRINT_ONLY_LOW_QUALITY = 1 << 2;
	    const CAN_MODIFY_DOCUMENT_CONTENT = 1 << 3;
	    const CAN_EXTRACT_TEXT_AND_GRAPHICS = 1 << 4;
	    const CAN_ADD_OR_MODIFY_TEXT_ANNOTATIONS = 1 << 5;
	    const CAN_CREATE_NEW_INTERACTIVE_FORM_FIELDS = 1 << 6;
	    const CAN_FILL_EXISTING_INTERACTIVE_FORM_FIELDS = 1 << 7;
    }
}



#[allow(unused)]
pub struct GxDocumentLicense {
    text: Option<String>,
    uri: Option<String>,
    web_statement: Option<String>,
}
