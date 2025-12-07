use gtk::cairo::{Format, ImageSurface};
use pdfium_render::prelude::*;
use crate::{components::gx_window::{CURRENTTAB,  TABKEYS}, 
    gxdocument::gx_render_context::GxRenderContext
};

#[allow(unused)]
pub struct GxPdfium {
    //这个结构体根据evince的 _PdfDocument，编写代码编修改
    pdfium: Pdfium,
    //pub document: Option<PdfDocument<'a>>,
    //pub page_numbers: u32,
	//password: Option<String>,
}

#[allow(unused)]
impl GxPdfium {
    pub fn new() -> Self{
        let pdfium = Pdfium::default();
        Self {
            pdfium:pdfium,
            //document:None,
            //page_numbers:0,
            //password:None,
        }       
    }
    //用pdfium库打开pdf文件，并返回PdfDocument
    pub async fn open_pdf_from_file<'a>(file_name: String,
        password:Option<&'a str>) -> Option<PdfDocument<'a>>  {
        let pdfium = Box::new(Pdfium::default());
        let pdfium = Box::leak(pdfium);
        let document 
            = pdfium.load_pdf_from_file(&file_name, password);
        //let page_numbers = document.as_ref().unwrap().pages().len() as u32;
        document.ok()
        //self.document = document.ok();
        //self.page_numbers = page_numbers;
    }
    
//    pub async fn static_open_pdf_from_file<'a>(file_name: String,password:Option<&'a str>) -> Option<PdfDocument<'a>> {
//        let current_tab = *(CURRENTTAB.get().unwrap());
//        let gx_pdfium = GxPdfium::get_gxpdfiums().await;
//        let gx_pdfium = gx_pdfium.get(&current_tab).unwrap();
//        //应该不会有内存泄漏吧？
//        let gx_pdfium = Box::new(gx_pdfium);
//        let gx_pdfium = Box::leak(gx_pdfium);
//        let document = gx_pdfium.pdfium.load_pdf_from_file(&file_name, password);
//        document.ok()
//    }

    //pub async  fn get_pdfium() -> MutexGuard<'static,GxPdfium> {
    //    GXPDFIUM
    //        .get()
    //        .expect("Could not get Pdfium from OnceCell")
    //        .lock()
    //        .await
    //
    //}

    // 初始化静态变量GXPDFIUM、 TABKEYS、 CURRENTTAB
    pub fn init_tabkey() {
        println!("初始化 GXPDFIUM RwLock");
        TABKEYS.get_or_init(|| {-1});
        let tabkeys = TABKEYS.get().unwrap();
        let _ = TABKEYS.set(*tabkeys + 1);        
        let tabkeys = *(TABKEYS.get().unwrap());
        CURRENTTAB.get_or_init(|| {tabkeys});

        //let mut hm_gxpdfium = HashMap::<i32,GxPdfium>::new();
        //hm_gxpdfium.insert(tabkeys,GxPdfium::new());
        //GXPDFIUM.get_or_init(||  { Mutex::new(hm_gxpdfium) });
    }

    // 读取操作（可以并发）
    //pub async fn get_gxpdfiums() -> MutexGuard<'static, HashMap<i32, GxPdfium>>  {
    //    GXPDFIUM.get().expect("Could not get Pdfium from OnceLock").lock().await

    //}

    // 写入操作（独占访问）
    //pub async fn insert_gxpdfium(key: i32, value: GxPdfium) -> bool {
    //    //let rwlock = GXPDFIUM.get_or_init(|| RwLock::new(HashMap::new()));
    //    if let Some(lock) = GXPDFIUM.get(){
    //        let mut hm = lock.lock().await;
    //        hm.insert(key, value);
    //        true
    //    }
    //    else{
    //       false 
    //    }

    //}
    //关联函数
    pub fn document_render(pdf_page:&PdfPage<'_>,gx_render_context:&mut GxRenderContext) -> ImageSurface{
        let page_width_point = pdf_page.width();
        let page_height_point = pdf_page.height();
        let (width,height) = gx_render_context.compute_transformed_size(page_width_point.value, 
            page_height_point.value);
        //gx_render_context.set_target_size(width, height);
        let surface = Self::page_render(pdf_page, width , height , gx_render_context);
        surface
    }
    //关联函数
    pub fn page_render(pdf_page:&PdfPage<'_>,
        width:i32,height:i32,gx_render_context:&mut GxRenderContext) -> ImageSurface{
        let mut render_config = PdfRenderConfig::new();
        match gx_render_context.rotation {
            90 => {
                render_config = render_config.translate(PdfPoints { value: width as f32 }, 
                    PdfPoints { value: 0.0 }).unwrap();
                render_config =  render_config.rotate(PdfPageRenderRotation::Degrees90,true);
            },
            180 => {
                render_config = render_config.translate(PdfPoints { value: width as f32 }, 
                    PdfPoints { value: height as f32 }).unwrap();
                render_config = render_config.rotate(PdfPageRenderRotation::Degrees180,true);
            },
            270 => {
                render_config = render_config.translate(PdfPoints { value: 0.0 },
                     PdfPoints { value: height as f32 }).unwrap();
                render_config = render_config.rotate(PdfPageRenderRotation::Degrees270,true);
            },
            _ => {
                render_config = render_config.translate(PdfPoints { value: 0.0 },
                     PdfPoints { value: 0.0 }).unwrap();
                render_config = render_config.rotate(PdfPageRenderRotation::None, true);
            },
        }
        
        let page_width_points = pdf_page.width();
        let page_height_points = pdf_page.height();
        let (xscale,yscale) = gx_render_context.compute_scales(page_width_points.value,page_height_points.value);
        
        render_config = render_config.scale(xscale, yscale).unwrap();

        
        let bitmap = pdf_page.render_with_config(&render_config).unwrap();

        //为了得到stride，也不知道这样得到的stride对不对
        let bytes = bitmap.as_raw_bytes();
        let height = bitmap.height();
        let stride = bitmap.bindings().FPDFBitmap_GetStride(bitmap.handle().clone());
        //let stride= bytes.len() / height as usize;

        let surface = ImageSurface::create_for_data(bitmap.as_rgba_bytes(),
        Format::ARgb32, width, height, stride as i32).unwrap();

        surface        
    }
    

    pub fn bindings_page_render(pdf_page:&PdfPage<'_>,
        gx_render_context:&mut GxRenderContext)-> ImageSurface{
        //let pdfium = Pdfium::default();
        let bindings = pdf_page.bindings();
        //let bindings = pdfium.bindings();      

        let mut rotate = 0;//0表示正常，1表示90度，2表示180度，3表示270度或者反90度
        match gx_render_context.rotation {
            90 => {
                rotate = 1;
            },
            180 => {
                rotate = 2;
            },
            270 => {
                rotate = 3;
            },
            _ => {
                rotate = 0;
            },
        }


        let page_width_points = pdf_page.width();
        let page_height_points = pdf_page.height();

        let (scaled_width,scaled_height) = gx_render_context.compute_scaled_size(
            page_width_points.value * 300.0 / 72.0, 
            page_height_points.value * 300.0 / 72.0);
       
        //创建确定长、宽的空bitmap
        let bitmap = bindings.FPDFBitmap_Create(scaled_width, scaled_height, 0); 

        //给空bitmap设置背景
        bindings.FPDFBitmap_FillRect(
            bitmap,
            0,
            0,
            scaled_width,
            scaled_height,
            0xFFFFFFFF,//这是白色，0x00000000是黑色
        );
       
        bindings.FPDF_RenderPageBitmap(
            bitmap,
            pdf_page.page_handle(),
            0,
            0,
            scaled_width,
            scaled_height,
            rotate,
            0,
        );

        let buffer = bindings.FPDFBitmap_GetBuffer(bitmap);

        let surface = unsafe {
            ImageSurface::create_for_data_unsafe(
                buffer as *mut u8,
                Format::ARgb32,
                scaled_width,  // 使用缩放后的宽度
                scaled_height, // 使用缩放后的高度
                bindings.FPDFBitmap_GetStride(bitmap),
            )
        };

        let surface = surface.unwrap();
        surface
    }
}