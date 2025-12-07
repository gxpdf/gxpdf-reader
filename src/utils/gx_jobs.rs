use async_trait::async_trait;
use gtk::{cairo::{Format, ImageSurface, RectangleInt}, 
    gdk::{MemoryFormat, MemoryTexture, Texture}, 
    glib::Bytes
};
use pdfium_render::prelude::PdfColor;
use tokio::{sync::{broadcast::{self, Sender}, mpsc}, 
    time::{sleep,Duration}
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use std::{any::Any, fmt::{Debug, Formatter}, 
    sync::atomic::{AtomicBool, AtomicI32, Ordering}
};
use bitflags::bitflags;
use crate::{components::gx_window::{CURRENTTAB, GXPDFDOCUMENT }, 
    gxdocument::{gx_pdf_document::GxRectangle, 
        gx_render_context::GxRenderContext}, 
    gxview::gx_view::DrawPage, 
    utils::{gx_job_scheduler::GxJobPriority, 
        gx_pdfium::GxPdfium}
};

#[async_trait]
pub trait GxJobClassAct: Send + Sync + Any + Debug {
    async fn run(&mut self) -> bool;
    fn is_cancelled(&self) -> bool;
    fn get_type_name(&self) -> &str;
    fn cancel(&self) -> bool;
    fn get_key(&self) -> Uuid;
    fn get_priority(&self) -> GxJobPriority;
    fn set_key(&mut self,key:Uuid) -> bool;
    fn set_priority(&mut self,priority:GxJobPriority) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    async fn emit_finished(&mut self) -> bool;
}


#[allow(unused)]
#[derive(Default,Debug)]
pub struct GxJobLoadPdf {
    pub cancel_token:CancellationToken,
    //pub pdf_document: Arc<Mutex<Option<GxPdfDocument<'a>>>>,
    //pub file_name: Option<PathBuf>,
    //pub password: Option<String>,
    //pub cancelled: bool,
    pub finished:  bool,
    pub failed:   bool,
    pub key: Uuid,
    pub priority: GxJobPriority,
}

#[allow(unused)]
impl GxJobLoadPdf{
    pub fn new() -> Self{
        let cancel_token = CancellationToken::new();
        let mut gx_job_load = GxJobLoadPdf::default();
        gx_job_load.cancel_token = cancel_token;

        let key = Uuid::new_v4();
        gx_job_load.key = key;
        //gx_job_load.file_name = file_name;
        //gx_job_load.pdf_document = pdf_document;
        gx_job_load
    }
    
    //pub fn set_file_name(&mut self,file_name:Option<PathBuf>){
    //    self.file_name = file_name;
    //}
    //
    //pub fn set_password(&mut self,password:Option<String>){
    //    self.password = password;
    //}
}

#[async_trait]
impl GxJobClassAct for GxJobLoadPdf{
    //返回true暂定为成功运行
    async fn run(&mut self) -> bool{

        let clone_token = self.cancel_token.clone();
        let handle_result = tokio::spawn(async move{
            tokio::select! {
                // 监听取消信号
                _ = clone_token.cancelled() => {
                    println!("gx job load pdf run的任务被取消！");
                    false
                    // 这里可以添加资源清理逻辑
                }
                // 实际工作（模拟耗时操作）
                _ = async {
                    //设置静态cache，仅仅是设置page相关的某些数值，并没有设置page本身？
                    //evince中是先判断了一下cache才调用的这个函数，但没明白其意义在哪儿，暂时先不考虑了
                    //这个函数先不放这儿了，改为放到gx_view中的init中调用了，第一次调用应该没问题，就看后面咋样了
                    //GxPdfDocument::set_static_cache().await;
                    //let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
                    //let document_guard = document_mutex.lock().await;
                    //let current_tab = *(CURRENTTAB.get().unwrap());
                } => {
                   true
                }
            }
        }); 
        if handle_result.await.unwrap(){
            self.finished = true;
            return true;
        }else{
            //self.cancelled = true;            
            return false;
        }
    }

    fn is_cancelled(&self) -> bool{
        self.cancel_token.is_cancelled()    
    }
    fn get_type_name(&self) -> &str{
        "job_load"
    }
    fn cancel(&self) -> bool{
        false
    }
    fn get_key(&self) -> Uuid{
        self.key
    }
    fn get_priority(&self) -> GxJobPriority{
        self.priority.clone()
    }
    
    fn set_key(&mut self,key:Uuid) -> bool{
        self.key = key;
        true
    }
    fn set_priority(&mut self,priority:GxJobPriority) -> bool{
        self.priority = priority;
        true
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    async fn emit_finished(&mut self) -> bool{
        false
    }
 
}

pub struct GxAtomicBool{
    pub atomic_bool: AtomicBool,
}
impl Clone for GxAtomicBool{
    fn clone(&self) -> Self{
        Self {
            atomic_bool: AtomicBool::new(self.atomic_bool.load(Ordering::Relaxed)),
        }
    }
}

impl Default for GxAtomicBool{
    fn default() -> Self{
        Self {
            atomic_bool: AtomicBool::new(false),
        }
    }
}

impl Debug for GxAtomicBool{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GxAtomicBool {{ atomic_bool: {:?} }}", self.atomic_bool.load(Ordering::Relaxed))
    }
}

impl GxAtomicBool{
    pub fn set(&self,value:bool){
        self.atomic_bool.store(value,Ordering::Relaxed);
    }
    pub fn get(&self) -> bool{
        self.atomic_bool.load(Ordering::Relaxed)
    }
}

#[allow(unused)]
pub struct GxAtomicI32{
    pub atomic_i32: AtomicI32,
}
impl Clone for GxAtomicI32{
    fn clone(&self) -> Self{
        Self {
            atomic_i32: AtomicI32::new(self.atomic_i32.load(Ordering::Relaxed)),
        }
    }
}

impl Default for GxAtomicI32{
    fn default() -> Self{
        Self {
            atomic_i32: AtomicI32::new(-1),
        }
    }
}

impl Debug for GxAtomicI32{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GxAtomicI32 {{ atomic_i32: {:?} }}", self.atomic_i32.load(Ordering::Relaxed))
    }
}

#[allow(unused)]
impl  GxAtomicI32{
    pub fn load(&self) -> i32{
        self.atomic_i32.load(Ordering::Relaxed)
    }
    pub fn store(&self,value:i32){
        self.atomic_i32.store(value,Ordering::Relaxed);
    }
}

#[allow(unused)]
#[derive(Debug,Clone)]
pub struct GxCancelSender{
    pub sender: Sender<Option<i32>>,//发送的是page_index
}
impl Default for GxCancelSender{
    fn default() -> Self{
        let (sender,_) = broadcast::channel::<Option<i32>>(2);
        Self {
            sender
        }
    }
}

#[derive(Debug,Clone)]
pub struct GxRenderJobFinishedSender{
    pub sender: mpsc::Sender<DrawPage>,
}
impl Default for  GxRenderJobFinishedSender{
    fn default() -> Self{
        let (sender,_) = mpsc::channel::<DrawPage>(1);
        Self {
            sender
        }
    }
}

#[allow(unused)]
#[derive(Debug,Clone)]
pub struct GxSender{
    pub sender: Sender<Option<GxSurfaceData>>,
}
impl Default for GxSender{
    fn default() -> Self{
        let (sender,_) = broadcast::channel::<Option<GxSurfaceData>>(10);
        Self {
            sender
        }
    }
}
//pub fn arc_ptr(token: &CancellationToken) -> *const () {
//    // Safety: CancellationToken 是个 tuple struct，里面就是 Arc<...>。
//    // 这样 transmute 会把 &CancellationToken 转成 &Arc<opaque>。
//    unsafe {
//        let arc: &Arc<()> = mem::transmute(token);
//        Arc::as_ptr(arc) as *const ()
//    }
//}

#[allow(unused)]
#[derive(Default,Clone,Debug)]
pub struct GxJobRenderPdf {
    pub page_index: i32,
    pub rotation: i32,
    pub scale: f32, 
    pub page_ready: bool,
    pub target_width: i32,
    pub target_height: i32,
    //pub surface: Arc<Mutex<Option<Surface>>>,
    //pub surface: Option<SurfaceData>,
    //pub surface: Arc<Mutex<Option<ImageSurfaceDataOwned>>>,
    pub surface: Option<GxSurfaceData>,
    pub include_selection: bool,
    //pub selection: Arc<Mutex<Option<Surface>>>,
    pub selection: Option<GxSurfaceData>,
    //pub selection_region: Arc<Mutex<Option<Region>>>,
    pub selection_region: Option<RectangleInt>,
    //由Region.rectangle,取得RectangleInt,而create_rectangle(rectangle: &RectangleInt) -> Region
    pub selection_points: Option<GxRectangle>,
    //pub selection_style: GxSelectionStyle,
    pub base: Option<PdfColor>,
    pub text: Option<PdfColor>,
    //自己添加的，先用吧,用于取消正在运行的GxjobRender的job，其实是不是放到job结构体中更好？
    pub cancel_token:  CancellationToken,//这个好像很难用上
    pub cancelled: GxAtomicBool,
    pub key:Uuid,
    pub finished: GxAtomicBool,//其实没法用，不知道为啥
    pub priority: GxJobPriority,
    pub finished_sender: GxRenderJobFinishedSender,
    //pub job_sender: GxSender,//准备就按GxJobPriority分类了,用于发送渲染结果
    //pub cancel_sender: GxSender,

}


impl GxJobRenderPdf{
    pub fn new(page:i32,rotation:i32,scale:f32,target_width:i32,target_height:i32,
        finishd_sender:mpsc::Sender<DrawPage>) -> Self{
        let cancel_token = CancellationToken::new();
        let mut gx_job_render = GxJobRenderPdf::default();
        let key = Uuid::new_v4();
        gx_job_render.key = key;
        gx_job_render.cancel_token = cancel_token;
        gx_job_render.page_index = page;
        gx_job_render.rotation = rotation;
        gx_job_render.scale = scale;
        gx_job_render.target_width = target_width;
        gx_job_render.target_height = target_height;
        let finished_sender = GxRenderJobFinishedSender{
            sender:finishd_sender
        };
        gx_job_render.finished_sender = finished_sender;
        //gx_job_render.finished = false;
        gx_job_render
    }
}

#[async_trait]
impl GxJobClassAct for GxJobRenderPdf{
    async fn run(&mut self)-> bool{
        let clone_token = self.cancel_token.clone();
        let page_index = self.page_index as u16;
        let target_width = self.target_width;
        let target_height = self.target_height;
        let rotation = self.rotation;
        let scale = self.scale;

 
        let handle_result = tokio::spawn(async move {
            tokio::select! {
                _ = clone_token.cancelled() => {
                    println!("gx job render pdf run的任务被取消！");
                    None
                }
                result = async {
                    let document_mutex = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
                    let mut document_guard = document_mutex.lock().await;
                    let current_tab = *(CURRENTTAB.get().unwrap());

                    if let Some(gx_pdf_document) = document_guard.get_mut(&current_tab){
                        let pdf_pages  = gx_pdf_document.pdf_document.as_ref().unwrap().pages();
                        let pdf_page = pdf_pages.get(page_index).ok().unwrap();
                        
                        //println!("IN run of GxJobRenderPdf 
                        //    rotation is {},scale is {},width is {},height is {},page_index is {}",
                        //    rotation,scale,target_width,target_height,page_index);
                        let mut gx_render_context = GxRenderContext::new(rotation,scale);
                        gx_render_context.set_target_size(target_width, target_height);
                        let surface = GxPdfium::bindings_page_render(&pdf_page,&mut gx_render_context);
                        let surface_data = GxSurfaceData::create_surface_data(surface,page_index as i32);
                        Some(surface_data)
                        //Some(surface.take_data().unwrap())
                        //self.surface = Some(surface.take_data().unwrap());
                    }else{
                        None
                    }
                   
                } => {
                   //println!("gx job render pdf run的任务被完成！page index is {}",page_index);
                   result 
                }
            } 
        });
        let surface = handle_result.await.unwrap();
        if surface.is_none(){
            //self.surface = Arc::new(Mutex::new(surface));
            //println!("现在发送了一个none的surface,page index is {}",page_index);
            self.cancelled.set(true);
            //let _ = self.job_sender.sender.send(surface);
            return false;
        }else{
            //self.surface = Arc::new(Mutex::new(surface));
            //println!("现在发送了一个surface,page index is {}",page_index);
            self.finished.set(true);
            let draw_page = DrawPage{
                page_index: page_index as i32,
                surface_borrow:surface.clone(),
                scale:self.scale,
            };
            let _ = self.finished_sender.sender.send(draw_page).await;
            self.surface = surface;
            //let _ = self.job_sender.sender.send(surface);
            //self.finished.atomic_bool.store(true, Ordering::Relaxed);

            true
        }
    }




    fn is_cancelled(&self) -> bool{
        //self.cancelled.atomic_bool.load(Ordering::Relaxed)
        self.cancel_token.is_cancelled()
    }
    
    fn get_type_name(&self) -> &str{
        "job_render"
    }
    //正常时候用不了 
    fn cancel(&self) -> bool{
        //println!("现在发送一个取消job的指令,page index is {}",self.page_index);
        //self.cancelled.atomic_bool.store(true, Ordering::Relaxed);
        self.cancel_token.cancel();
        true
    }
    fn get_key(&self) -> Uuid{
        self.key
    }
    fn get_priority(&self) -> GxJobPriority{
        self.priority.clone()
    }
    fn set_key(&mut self,key:Uuid) -> bool{
        self.key = key;
        true
    }
    fn set_priority(&mut self,priority:GxJobPriority) -> bool{
        self.priority = priority;
        true
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    async fn emit_finished(&mut self) -> bool{
        true
       //let mut rx = self.finished_sender.sender.subscribe();
       //let finished_signal = rx.recv().await;
       //if let Ok(_finished_signal) = finished_signal{
       //  return true;
       //}else{
       // return false;
       //} 
    }
 
}

#[derive(Debug)]
pub struct GxSurfaceData{
    //这个结构体用来代替Surface，因为Surface在线程间传输不安全，太扯了
    pub pixels: Vec<u8>,
    pub format: Format,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub page_index:i32,
}
impl Default for GxSurfaceData {
    fn default() -> Self {
        GxSurfaceData {
            pixels: Vec::new(),   // 空向量
            format: Format::ARgb32, 
            width: 0,             // 默认宽度为0
            height: 0,            // 默认高度为0
            stride: 0,            // 默认步长为0
            page_index:0,
        }
    }

}
impl Clone for GxSurfaceData{
    fn clone(&self) -> Self{
        let surface_data = self.pixels.clone();
        let surface_format = self.format;
        let surface_width = self.width;
        let surface_height = self.height;
        let surface_stride = self.stride;
        let clone_surface_data = GxSurfaceData{
            pixels:surface_data,
            format:surface_format,
            width:surface_width,
            height:surface_height,
            stride:surface_stride,
            page_index:self.page_index,
        };
        clone_surface_data       
    }
}

impl GxSurfaceData {
    pub fn create_surface(&self) -> ImageSurface{
        let surface_data = self.pixels.clone();
        let surface_format = self.format;
        let surface_width = self.width;
        let surface_height = self.height;
        let surface_stride = self.stride;
        let temp_surface = ImageSurface::create_for_data(surface_data, 
            surface_format, surface_width, surface_height, surface_stride);
        temp_surface.unwrap()
    }   
    
    pub fn create_surface_data(image_surface:ImageSurface,page_index:i32) -> Self{
        let format = image_surface.format();
        let width = image_surface.width();
        let height = image_surface.height();
        let stride = image_surface.stride();
        let pixels= image_surface.take_data().unwrap().to_vec();
        GxSurfaceData { pixels, format, width, height, stride,page_index}
    }

    pub fn texture_from_surface(&self) -> Option<Texture> {
        let width = self.width;
        let height = self.height;

        if width <= 0 || height <= 0 {
            return None;
        }

        // 获取 stride 和像素数据指针
        let stride = self.stride;
        //let data = surface.data().ok()?; // Borrowed slice

        // 拷贝数据到 Rc<[u8]>，以便传给 glib::Bytes
        // 如果想避免拷贝，可以用 unsafe + from_glib_full 方式
        let bytes = Bytes::from(&self.pixels[..]);

        // 创建 MemoryTexture
        let texture = MemoryTexture::new(
            width,
            height,
            MemoryFormat::B8g8r8a8, // Cairo 的 ImageSurface 大多是 ARGB32，但字节顺序为 BGRA 即 B8G8R8A8
            &bytes,
            stride as usize,
        );

        Some(texture.into())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq,Default)]
    pub struct GxJobPageDataFlags: u32 {
        const NONE = 0;
        const LINKS = 1 << 0;
        const TEXT = 1 << 1;
        const TEXT_MAPPING = 1 << 2;
        const TEXT_LAYOUT = 1 << 3;
        const TEXT_ATTRS = 1 << 4;
        const TEXT_LOG_ATTRS = 1 << 5;
        const IMAGES = 1 << 6;
        const FORMS = 1 << 7;
        const ANNOTS = 1 << 8;
        const MEDIA = 1 << 9;
        const ALL = (1 << 10) - 1;
    }
}

impl GxJobPageDataFlags { 
    pub fn default_flags() -> Self {
        let default_flags  = GxJobPageDataFlags::LINKS 
            | GxJobPageDataFlags::TEXT_MAPPING
            | GxJobPageDataFlags::IMAGES
            | GxJobPageDataFlags::FORMS
            | GxJobPageDataFlags::ANNOTS
            | GxJobPageDataFlags::MEDIA;
        default_flags


    }
}

#[allow(unused)]
#[derive(Default,Debug)]
pub struct GxJobTest{
    cancel_token:CancellationToken,
    key:Uuid,
    priority:GxJobPriority,
}

#[allow(unused)]
#[async_trait]
impl GxJobClassAct for GxJobTest {
    async fn run(&mut self) -> bool {
        let clone_token = self.cancel_token.clone();
        let handle_result = tokio::spawn(async move{
            tokio::select! {
                // 监听取消信号
                _ = clone_token.cancelled() => {
                    println!("任务被取消！");
                    false
                    // 这里可以添加资源清理逻辑
                }
                // 实际工作（模拟耗时操作）
                _ = async {
                    println!("任务开始执行...");
                    for i in 1..=5 {
                        sleep(Duration::from_secs(1)).await;
                        println!("工作进度: {}/5", i);
                    }
                    println!("任务正常完成！");
                } => {
                   true
                }
            }
        }); 
        true 
        //handle_result.await.unwrap()
    }

    fn is_cancelled(&self) -> bool {
       false 
    }
    fn get_type_name(&self) -> &str {
        "test"
    }
    fn cancel(&self) -> bool{
        self.cancel_token.cancel();
        true
    }
    fn get_key(&self) -> Uuid{
        self.key
    }
    fn get_priority(&self) -> GxJobPriority{
        self.priority.clone()
    }
    fn set_key(&mut self,key:Uuid) -> bool{
        self.key = key;
        true
    }
    fn set_priority(&mut self,priority:GxJobPriority) -> bool{
        self.priority = priority;
        true
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    async fn emit_finished(&mut self) -> bool{
        false
    }
 
}

#[allow(unused)]
impl GxJobTest{
    pub fn new() -> Self{
        let cancel_token = CancellationToken::new();
        let mut gx_job_test = GxJobTest::default();
        gx_job_test.cancel_token = cancel_token;
        let key = Uuid::new_v4();
        gx_job_test.key = key;
        gx_job_test
    }
}
