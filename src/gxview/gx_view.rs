//gx_view.rs
use std::cell::{Cell, RefCell};
use std::{collections::HashMap, path::PathBuf};
use std::cmp::max;
use gtk::cairo::ImageSurface;
use gtk::gdk::ModifierType;
use gtk::glib::{clone};
use gtk::{glib, graphene, DrawingArea, 
    EventControllerScroll, EventControllerScrollFlags,
    Scrollbar
};
use gtk::{cairo::{Context},Border, gdk::{Rectangle}};
use gtk::{graphene::Point,Widget,prelude::*};
use relm4::abstractions::DrawHandler;
use relm4::prelude::*;
use crate::components::gx_window::{CURRENTTAB, 
    DEFAULT_PIXBUF_CACHE_SIZE,GXPDFDOCUMENT, 
    MAX_IMAGE_SIZE, MIN_SCALE, ZOOM_IN_FACTOR, 
    ZOOM_OUT_FACTOR
};
use crate::gxdocument::gx_document_info::GxDocumentInfo;
use crate::gxdocument::gx_pdf_document::GxPageSize;
use crate::gxview::gx_page_cache::GxPageCache;
use crate::gxview::gx_pixbuf_cache::{GxPixbufCache, ScrollDirection};
use crate::gxview::gx_surfaces_cache::GxSurfacesCache;
use crate::gxview::gx_view_area::GxViewArea;
use crate::utils::gx_job_scheduler::GxScheduler;
use crate::utils::gx_jobs::{GxRenderJobFinishedSender, GxSurfaceData};

use super::{gx_document_model::{GxPageLayout, GxSizingMode}};

#[allow(unused)]
const DRAG_HISTORY: usize = 10;

#[derive(Default,Clone,Debug)]
pub struct GxHeightToPageCache {
    pub rotation: i32,
    pub dual_even_left: bool,
    pub height_to_page: Vec<f32>,
    pub dual_height_to_page: Vec<f32>,
}
#[derive(Default,PartialEq,Clone)]
pub enum PendingScroll {
    #[default]
    ScrollToKeepPosition,
    ScrollToPagePosition,
    ScrollToCenter,
    ScrollToFindLocation,
}
// Information for middle clicking and moving around the doc
#[derive(Default)]
#[allow(unused)]
pub struct DragInfo {
    pub in_drag: bool,
    pub start: Point,
    pub hadj: f64,
    pub vadj: f64,
    pub drag_timeout_id: u32,
    pub release_timeout_id: u32,
    pub buffer: [Point; DRAG_HISTORY],
    pub momentum: Point,
    pub in_notify: bool,
}
// Autoscrolling
#[derive(Default,Clone)]
pub struct AutoScrollInfo {
    pub autoscrolling: bool,
    pub last_y: u32,
    pub start_y: u32,
    pub timeout_id: u32,
}
// Information for handling selection
#[derive(Default,Clone)]
pub struct SelectionInfo {
    pub in_drag: bool,
    pub start: Point,
    pub selections: Vec<String>,
   // pub style: GxSelectionStyle,
}
#[derive(Default)]
#[allow(unused)]
pub struct ImageDNDInfo{
	pub in_drag: bool,
    pub start: Point,
	//pub image: GxImage,
} 

#[derive(Default)]
#[allow(unused)]
pub struct GxViewWindowChild {
    //pub window: Widget, // GtkWidget *window;
    pub page: u32,           // guint page;
    // Current position
    pub x: i32,              // gint x;
    pub y: i32,              // gint y;
    // EvView root position
    pub parent_x: i32,       // gint parent_x;
    pub parent_y: i32,       // gint parent_y;
    // Document coords
    pub orig_x: f64,         // gdouble orig_x;
    pub orig_y: f64,         // gdouble orig_y;
    pub visible: bool,       // gboolean visible;
    pub moved: bool,         // gboolean moved;
}
#[derive(Default)]
#[allow(unused)]
pub struct AddingAnnotInfo {
    pub start: Option<Point>,        // GdkPoint start;
    pub stop: Option<Point>,          // GdkPoint stop;
    pub adding_annot: bool,      // gboolean adding_annot;
    //pub annot_type: GxAnnotationType, // EvAnnotationType type;
    //pub annot: GxAnnotation<'a>, // EvAnnotation *annot;
}
#[derive(Default)]
#[allow(unused)]
pub struct MovingAnnotInfo {
    pub start: Option<Point>,
    //pub cursor_offset: Option<GxPoint>,
    pub annot_clicked: bool,
    pub moving_annot: bool,
   // pub annot: Option<GxAnnotation<'a>>,
}

/* Information for handling link preview thumbnails */
#[derive(Default)]
#[allow(unused)]
pub struct GxLinkPreview {
    pub left: f64,
    pub top: f64,
    pub popover: Option<Widget>,
    //pub link: Option<GxLink>,
    pub delay_timeout_id: u32,
}

#[derive(Default)]
#[allow(unused)]
pub  enum GxPanAction{
    #[default]
	NONE,
	NEXT,
	PREV,
} 
pub struct GxViewInit{
    pub file_path: Option<PathBuf>,
    pub job_scheduler: Option<GxScheduler>,
    pub start_page:i32,
}

#[derive(Debug)]
pub enum GxViewMsg {
    ScrollChanged((f32, f32)),  // 滚动位置变化消息，大多数时候只用了后一个f32，
    //但zoom的时候两个都用了
    HScrollChanged(f32),//用于水平滚动位置变化信息
    ReSize((i32,i32)),//这两个值是area可见区域的宽度与高度
    RenderFinished(DrawPage),//i32是这个page_index
    //Scroll_Zoom((f32, Option<(f64, f64)>)),
    ScrollZoom((f32,f32)),
    DeviceScaled,
}

#[derive(Default,Clone)]
pub struct GxView {
    pub allow_links_change_zoom: bool,
    pub allocation_height: Cell<f32>,
    pub allocation_width: Cell<f32>,
    pub allocation_x: Cell<f32>,
    pub allocation_y: Cell<f32>,
    pub allocation_border: RefCell<Border>,
    pub cache_loaded: bool,
    pub caret_enabled: bool,
    pub can_zoom_in: bool,
    pub can_zoom_out: bool,
    pub child_focus_idle_id: u32,
    pub clean_upper: f32,
    pub continuous: bool,
    pub current_page: i32,
    pub cursor_blink_time: u32,
    pub cursor_blink_timeout_id: u32,
    pub cursor_line_offset: f32,
    pub cursor_offset: i32,
    pub cursor_page: i32,
    pub cursor_visible: bool,
    pub dual_even_left: bool,
    pub device_scale:f32,
    //pub document_model: GxDocumentModel,//这个成员似乎可以删除，基本所有的成员在gx_view中都有了
    pub end_page: i32,
    pub focused_element_page: u32,
    pub find_page: i32,
    pub find_result: i32,
    pub fullscreen: bool,
    pub hash_draw_pages: HashMap<i32,DrawPage>,
    pub hash_draw_pages_cache: HashMap<i32,DrawPage>,//这个存储了所有已经render的pages
    pub hash_draw_pages_cache_size: usize,//hash_draw_pages中所有surface加起来的字节数，要小于
    pub height_to_page_cache: Option<GxHeightToPageCache>,
    pub highlight_find_results: bool,
    pub hj_page_size: f32,
    pub hj_value: f32,
    pub hscroll_policy: u8,
    pub internal_size_request: bool,
    pub job_scheduler: Option<GxScheduler>,
    pub jump_to_find_result: bool,
    pub key_binding_handled: bool,
    //pub list_seen:HashSet<i32>,
    pub loading: bool,
    pub loading_timeout: u32,
    pub max_height: f32,
    pub max_label: i32,
    pub max_scale: f32,
    pub max_target_page_size: Requisition,
    pub max_width: f32,
    pub min_height: f32,
    pub min_scale: f32,
    pub min_width: f32,
    pub page_cache: GxPageCache,
    pub page_labels: Option<HashMap<i32, String>>,
    pub page_layout: Option<GxPageLayout>,
    pub page_numbers: i32,
    pub page_sizes: Vec<GxPageSize>,//原来是Arc<Mutex<Vec<GxPageSize>>>,暂时不需要Mutex，删了
    pub pages_per_screen:i32,//每页能显示的pdf页数
    pub pending_resize: bool,
    pub pending_scroll: PendingScroll,
    pub pixbuf_cache: GxPixbufCache,
    //pub pixbuf_cache_sender:GxPixbufCacheSender,
    pub pixbuf_cache_size: usize,
    pub pixbuf_cache_updated: bool,
    pub pressed_button: i32,
    pub prev_zoom_gesture_scale: f32,
    pub render_sender:GxRenderJobFinishedSender,
    pub rotation: i32,
    pub rtl: bool,
    pub scale: f32,
    pub scaled_max_height: f32,
    pub scaled_max_width: f32,
    pub scroll_info: Option<AutoScrollInfo>,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub selection_info: SelectionInfo,
    pub sizing_mode: GxSizingMode,
    pub spacing: f32,
    pub start_page: i32,
    pub surfaces_cache: GxSurfacesCache ,
    pub temp_render_pages_hash: HashMap<i32,f32>,//用于临时存储已经render的page的page_index与scale对
    pub total_delta: f32,
    pub uniform: bool,
    pub uniform_height: f32,
    pub uniform_width: f32,
    pub update_cursor_idle_id: u32,
    pub vj_page_size: f32,
    pub vj_value: Cell<f32>,
    pub vscroll_policy: u8,
    pub zoom_center_x: f32,
    pub zoom_center_y: f32,
}


#[relm4::component(pub async)]
impl AsyncComponent for GxView {
    type Init = GxViewInit;
    type Input = GxViewMsg;
    type Output = ();
    type CommandOutput = ();//GxViewCommandResult;
    view! {
        #[name = "box_main"]
        gtk::Box{
            set_orientation: gtk::Orientation::Horizontal,
            //左边先坐一个drawingarea+水平滚动条
            #[name = "content_box"]
            append = &gtk::Box{
                set_orientation: gtk::Orientation::Vertical,
                #[name = "area"]
                append = &GxViewArea{
                    set_focusable:true,
                },
                #[name = "hscrollbar"]
                append = &gtk::Scrollbar {
                    set_orientation: gtk::Orientation::Horizontal,
                },
            },

            //右边的垂直滚动条
            #[name = "vscrollbar"]
            append = &gtk::Scrollbar {
                set_orientation: gtk::Orientation::Vertical,
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = GxView::default();
        model.pixbuf_cache_updated = true;
        let _ = GxDocumentInfo::set_static_pdf_document_info().await;
        model.job_scheduler = init.job_scheduler.clone();
        model.can_zoom_in = true;
        model.can_zoom_out = true;

        let widgets = view_output!();

        let area = &widgets.area;
        area.set_hexpand(true);
        area.set_vexpand(true);
        area.set_focusable(true);

        //new好了pixbuf_cache与page_cache
        model.gx_view_init_document(model.start_page).await;
        model.gx_view_update_scale_limits(area);
        if let Some(native) = area.native(){
            if let Some(surface) = native.surface(){
                //let scale_factor = surface.scale();//如果gtk4-rs用到0.10.0就可以改用这行了
                let scale_factor = surface.scale_factor();
                model.device_scale = scale_factor as f32;
            }else{
                model.device_scale = 1.0;
            }
        }else{
            model.device_scale = 1.0;
        }

        area.gx_view_area_set_device_scale(model.device_scale);

        //设置整个drawingarea的宽度为统一宽度，高度为所有页面加总的高度
        let max_height= model.max_height;
        let upper_height = model.clean_upper ;

        if area.is_realized(){
            let width = area.width();
            let height = area.height();
            model.allocation_width.replace(width as f32);
            model.allocation_height.replace(height as f32);
        }

        //vscrollbar
        let scrollbar = &widgets.vscrollbar;
        let vadjustment = gtk::Adjustment::new(
            0.0, // value
            0.0, // lower
            upper_height as f64, // upper
            max_height as f64 / 5.0, // step_increment,默认为翻滚5次过完一页
            max_height as f64 * 0.8,// page_increment, 
            max_height as f64, //page_size
        );
        let sender_clone = sender.clone();
        vadjustment.connect_value_changed(move |adj| {
            let value = adj.value();
            sender_clone.input(GxViewMsg::ScrollChanged((0.0,value as f32)));
        });
        scrollbar.set_adjustment(Some(&vadjustment));

        //hscrollbar
        let hscrollbar = &widgets.hscrollbar;
        let hadjustment = gtk::Adjustment::new(
            0.0, // value
            0.0, // lower
            0.0, // upper
            0.0, // step_increment,默认为翻滚5次过完一页
            0.0,// page_increment, 
            0.0, //page_size
        );
 
        let sender_clone = sender.clone();
        hadjustment.connect_value_changed(move |adj| {
            let value = adj.value();
            sender_clone.input(GxViewMsg::HScrollChanged(value as f32));
        });
        hscrollbar.set_adjustment(Some(&hadjustment));

        let controller = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL);

        let sender_clone = sender.clone();
        controller.connect_scroll(
            clone!(
                #[weak]
                vadjustment,
                #[upgrade_or]
                glib::Propagation::Stop,
                move |event, dx, dy| {
                    let old = vadjustment.value();
                    let step = vadjustment.step_increment();
                    
                    if let Some(ev) = event.current_event(){
                        let state = ev.modifier_state();
                        if state == ModifierType::CONTROL_MASK{
                            sender_clone.input(GxViewMsg::ScrollZoom((dx as f32,dy as f32)));
                            return glib::Propagation::Stop;
                        }
                    }

                    let mut new = old + dy * step;
                    new = new.clamp(vadjustment.lower(), 
                        vadjustment.upper() - vadjustment.page_size());
                    vadjustment.set_value(new);
                    glib::Propagation::Stop
                }               
            )
        );
        area.add_controller(controller);

        let sender_clone = sender.clone();
        let area_weak = area.downgrade();
        // 使用 idle_add 在下一个 idle 周期获取尺寸
        glib::idle_add_local(move || {
            if let Some(area) = area_weak.upgrade() {
                let width = area.width();
                let height = area.height();
                if width > 0 && height > 0 {
                    sender_clone.input(GxViewMsg::ReSize((width,height)));
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Break
        });


        area.connect_scale_factor_notify(move |_surface|{
            sender.input(GxViewMsg::DeviceScaled);
        }); 

 
        AsyncComponentParts { model, widgets }

    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            GxViewMsg::ScrollChanged(pos) => {
                self.scroll_x = pos.0;
                self.scroll_y = pos.1;
                let area = &widgets.area;
                let vscroll_bar = &widgets.vscrollbar;
                let sender_clone1 = sender.clone();
                self.gx_view_update_self_vj_values(vscroll_bar);
                self.gx_view_get_start_and_end_page();
                self.gx_view_get_start_and_end_page_new(area);
                //let mut self_clone = self.clone();

                self.gx_view_update_range_and_current_page(sender_clone1);
                let scale = self.scale;
                let rotation = self.rotation;
                let device_scale = self.device_scale;
                let pixbuf_cache 
                    = &mut self.pixbuf_cache;
                if pixbuf_cache.scroll_direction == ScrollDirection::Up{
                    pixbuf_cache.
                        gx_pixbuf_cache_add_prev_jobs_if_needed(rotation,scale,device_scale,
                        sender.clone(),self.page_numbers,
                        self.job_scheduler.clone());
                    pixbuf_cache.
                        gx_pixbuf_cache_add_next_jobs_if_needed(rotation,scale,device_scale,
                        sender.clone(),self.page_numbers,
                        self.job_scheduler.clone());
                }else{
                    pixbuf_cache.
                        gx_pixbuf_cache_add_next_jobs_if_needed(rotation,scale,device_scale,
                        sender.clone(),self.page_numbers,
                        self.job_scheduler.clone());
                    pixbuf_cache.
                        gx_pixbuf_cache_add_prev_jobs_if_needed(rotation,scale,device_scale,
                        sender.clone(),self.page_numbers,
                        self.job_scheduler.clone());
                }                           

                //先对area进行size_allocate
                let gx_view_pages_len = area.gx_view_area_get_gx_view_pages_len();
                let start_gx_view_page_widget_index 
                    =  self.start_page %  gx_view_pages_len;
                let end_gx_view_page_widget_index 
                    = self.end_page % gx_view_pages_len;
                if let Some(height_to_page_cache) 
                    = &self.height_to_page_cache{
                   //gtk4系统会在size_allocate后根据需要自动调用queue_draw
                    area.gx_view_area_size_allocate(start_gx_view_page_widget_index, 
                        end_gx_view_page_widget_index, self.start_page, self.end_page, 
                        self.page_numbers,self.scale,self.rotation,
                        height_to_page_cache,self.uniform,self.uniform_width,
                        self.uniform_height,&self.page_sizes,self.scroll_y,
                        &self.pixbuf_cache
                    );
                }
            }
            GxViewMsg::HScrollChanged(scroll_x) => {
                self.scroll_x = scroll_x;
                let area = &widgets.area;
                let hscroll_bar = &widgets.hscrollbar;
                let _adjustment = hscroll_bar.adjustment();
                 self.gx_view_update_adjustment_value(area,
                    gtk::Orientation::Horizontal,hscroll_bar);
                //self.gx_view_update_hscroll_values(hscroll_bar,area);
                area.gx_view_area_all_page_redraw(self.scroll_x);
            }
            GxViewMsg::ReSize((width,_height)) => {
                let hscroll_bar = &widgets.hscrollbar;
                let area = &widgets.area;
                if width > self.max_width as i32{
                    hscroll_bar.set_visible(false);
                    if self.scroll_x != 0.0{
                        self.scroll_x = 0.0;
                        area.gx_view_area_all_page_redraw(self.scroll_x);
                    }
                }else{
                    hscroll_bar.set_visible(true);
                }

                let width = area.width();
                let height = area.height();
                self.allocation_width.replace(width as f32);
                self.allocation_height.replace(height as f32);

                
                let pages_per_screen 
                    = self.allocation_height.get().round() as i32 / self.max_height.round() as i32;
                self.pages_per_screen = pages_per_screen;

                let vscroll_bar = &widgets.vscrollbar;
                self.gx_view_update_vscroll_values(vscroll_bar,area);
                self.gx_view_update_adjustment_value(area,
                    gtk::Orientation::Horizontal,hscroll_bar);
                //self.gx_view_update_hscroll_values(hscroll_bar,area);
                let sender_clone = sender.clone();
                sender_clone.input(GxViewMsg::ScrollChanged((0.0,self.vj_value.get() as f32)));
            },
            GxViewMsg::RenderFinished(render_finished) => {
                let area = &widgets.area;
                let vscroll_bar = &widgets.vscrollbar;
                self.gx_view_update_self_vj_values(vscroll_bar);
                self.gx_view_get_start_and_end_page_new(area);
                let rendered_index = render_finished.page_index;

                let start_page = self.start_page;
                let end_page = self.end_page;
                let preload_cache_size =  self.pixbuf_cache.preload_cache_size;
                if rendered_index < (start_page - preload_cache_size)
                    || render_finished.page_index > (end_page + preload_cache_size){
                    return;
                }

                if let Some(gx_surface_data) = &render_finished.surface_borrow{
                    let texture = gx_surface_data.texture_from_surface();
                    if let Some(cache_job) 
                        = self.pixbuf_cache.gx_pixbuf_cache_find_job_cache(rendered_index){
                        cache_job.texture = texture.clone();
                    }
                    area.gx_view_area_page_redraw(&self.pixbuf_cache,rendered_index);

                }
            },
            GxViewMsg::ScrollZoom(scale) => {
                //scale的两个f32分别对应dx,dy
                let area = &widgets.area;                
                let h_scroll_bar = &widgets.hscrollbar;
                let v_scroll_bar = &widgets.vscrollbar;
                
                let postion = self.gx_view_get_pointer_position(area);
                if let Some(postion) = postion{
                    self.zoom_center_x = postion.0 as f32;
                    self.zoom_center_y = postion.1 as f32;

                    let delta = scale.0 + scale.1;
                    let factor 
                        = (if delta < 0.0 { ZOOM_IN_FACTOR } else { ZOOM_OUT_FACTOR }).powf(delta.abs());
                    if self.gx_view_can_zoom(factor){
                        self.gx_view_zoom(factor,area, h_scroll_bar, v_scroll_bar);
                    }
                }
                let h_value = h_scroll_bar.adjustment().value();
                let v_value = v_scroll_bar.adjustment().value();
                sender.input(GxViewMsg::ScrollChanged((h_value as f32,v_value as f32)));
            },
            GxViewMsg::DeviceScaled => {
                let area = &widgets.area;

                let width = area.width();
                let height = area.height();
                self.allocation_width.replace(width as f32);
                self.allocation_height.replace(height as f32);
 
                if let Some(native) = area.native(){
                    if let Some(surface) = native.surface(){
                        //let scale_factor = surface.scale();//如果gtk4-rs用到0.10.0就可以改用这行了
                        let scale_factor = surface.scale_factor();
                        self.device_scale = scale_factor as f32;
                        area.gx_view_area_set_device_scale(scale_factor as f32);
                    }
                }
            }
        }
    }
}


impl GxView{
    //已完善注释
     //主要作用是设置要渲染的cairo画布的高度,并将高度存储到height_to_page_cache中
    pub fn gx_view_build_height_to_page_cache_and_clean_upper (& mut self) {
        let swap = self.rotation == 90 || self.rotation == 270;
        let uniform = self.uniform;
        let page_numbers = self.page_numbers;
        self.height_to_page_cache.get_or_insert_with(
            || GxHeightToPageCache::default()).height_to_page = Vec::new();
        self.height_to_page_cache.as_mut().unwrap().dual_height_to_page = Vec::new();
        self.height_to_page_cache.as_mut().unwrap().rotation = self.rotation;
        self.height_to_page_cache.as_mut().unwrap().dual_even_left = self.dual_even_left;

        let mut u_width = 0.0 as f32; 
        let mut u_height = 0.0 as f32;
        if uniform { 
            u_width = self.uniform_width;
            u_height = self.uniform_height;
        }
        let mut saved_height = self.spacing ;
        let mut uniform_height = 0.0;
        let mut page_height;
        for i in 0..page_numbers { 
            if uniform {
                if swap {
                    uniform_height = u_width;
                } else {
                    uniform_height = u_height;
                }    
                self.height_to_page_cache.as_mut().unwrap().height_to_page.push(i as f32 * uniform_height);
            }
            else{
                if i < page_numbers {
                    let _page_width = self.page_sizes[i as usize].width;
                    let _page_height = self.page_sizes[i as usize].height;
                    if swap {
                        page_height = _page_width;
                    }
                    else{
                        page_height = _page_height;
                    }
                }
                else{
                    page_height = 0.0 as f32;
                }
                self.height_to_page_cache.as_mut().unwrap().height_to_page.push(saved_height);
                saved_height += page_height ;
            }
        }

        if uniform {
            self.clean_upper = (page_numbers as f32 * uniform_height).round() ;
        }else{
            let _page_width = self.page_sizes[(page_numbers - 1) as usize].width;
            let _page_height = self.page_sizes[(page_numbers - 1) as usize].height;
            //if swap {
            //    page_height = _page_width;
            //}
            //else{
            //    page_height = _page_height;
            //}
            self.clean_upper = saved_height  ;
        }
    }

    fn gx_view_can_zoom(&self,factor:f32) -> bool{
        if factor == 1.0{
            return true;
        }else if factor < 1.0 {
            return self.can_zoom_out;
        }else{
            return self.can_zoom_in;
        }
    }

    fn gx_view_change_page(&mut self,new_page:i32){
        self.current_page = new_page;
        self.pending_scroll = PendingScroll::ScrollToPagePosition;
        //evince中还要更新gtk widget，现在仅仅是在init的时候，先不更新了,暂时不管了
    }

    //已完善注释
    //确认是否已将所有页面的宽度、高度等存储起来了，若为存储就调用函数存储
    //如果调用了函数还是不行，那这个pdf文件就有问题，应该就要退出，并提示用户pdf文件有问题了
    //提示用户的代码需完善
    pub async fn gx_view_check_dimensions(&mut self) -> bool{
        if !self.cache_loaded{
            self.gx_view_set_cache_from_pdfium().await;
        }
        if self.max_width > 0.0 && self.max_height > 0.0{
            return true;
        }
        else{
            return false;
        }
    }

    pub fn gx_view_check_dimensions_after_init(&mut self) -> bool{
        if self.max_width > 0.0 && self.max_height > 0.0{
            return true;
        }
        else{
            return false;
        }
    }
 
    //未完善：未add class,也未restore class一类
    //pub fn gx_view_compute_border(&mut self,area:&DrawingArea){
    //    let style_context = area.style_context();
    //    let border = style_context.border();
    //    self.allocation_border = RefCell::new(border);
    //}


    pub async fn gx_view_draw_new(&mut self,area: &DrawingArea){

        let mut draw_handler = DrawHandler::new_with_drawing_area(area.clone());
        let ctx = draw_handler.get_context();

        if self.start_page == -1{
            return;
        }

        let clip_rect_ext = ctx.clip_extents();
        if clip_rect_ext.is_err(){
            return;
        }
        let mut clip_rect = (0.0,0.0,0.0,0.0);
        if let Ok(clip_rect_ext) = clip_rect_ext{
            if clip_rect_ext.2 == 0.0 || clip_rect_ext.3 == 0.0{
                return;
            }else{
                clip_rect = clip_rect_ext;
            }
        }
        for page_index in self.start_page..self.end_page + 2 {
            if page_index >= self.page_numbers{
                break;
            }
            let mut page_area = self.gx_view_get_page_extents( page_index);
            let vj_value = self.scroll_y.round() as i32;
            if page_index == 0{
                page_area.set_y(page_area.y() - vj_value 
                + self.allocation_border.borrow().top() as i32 );
                //+ self.spacing as i32);                   
            }else{
                let y = page_area.y() - vj_value ;
                page_area.set_y(y);                   
            }
            let _clip_rect_page = Rectangle::new(clip_rect.0.round() as i32,
                clip_rect.1.round() as i32,clip_rect.2.round() as i32,
                clip_rect.3.round() as i32);
        }       
    }



    pub fn gx_view_draw_border(&mut self,area:&DrawingArea){
        let mut draw_handler = DrawHandler::new_with_drawing_area(area.clone());
        let ctx = draw_handler.get_context();

        if self.start_page == -1{
            return;
        }

        let clip_rect_ext = ctx.clip_extents();
        if clip_rect_ext.is_err(){
            return;
        }
        let mut clip_rect = (0.0,0.0,0.0,0.0);
        if let Ok(clip_rect_ext) = clip_rect_ext{
            if clip_rect_ext.2 == 0.0 || clip_rect_ext.3 == 0.0{
                return;
            }else{
                clip_rect = clip_rect_ext;
            }
        }
        for page_index in self.start_page..self.end_page + 2 {
            if page_index >= self.page_numbers{
                break;
            }
            let mut page_area = self.gx_view_get_page_extents( page_index);
            let vj_value = self.scroll_y.round() as i32;
            if page_index == 0{
                page_area.set_y(page_area.y() - vj_value 
                + self.allocation_border.borrow().top() as i32 
                + self.spacing as i32);                   
            }else{
                let y = page_area.y() - vj_value 
                + self.allocation_border.borrow().bottom() as i32 
                + self.allocation_border.borrow().top() as i32 
                + (self.spacing as i32) * (page_index + 1);
                page_area.set_y(y);                   
            }

            //准备draw_border
            //let clip_rect = ctx.clip_extents().ok().unwrap();
            let clip_rect_border = Rectangle::new(clip_rect.0.round() as i32,
                clip_rect.1.round() as i32,clip_rect.2.round() as i32,
                clip_rect.3.round() as i32);
            self.gx_view_draw_one_page_border(page_index,
                &ctx,&mut page_area,&clip_rect_border);
        }
    }   

    pub fn gx_view_draw_one_page_border(&self,
        _page_index:i32,
        cr: &Context,
        page_area: &mut Rectangle,
        //border: &Border,
        expose_area: &Rectangle,
    ){
        if let Some(_overlap) = page_area.intersect(expose_area){
            // 在 GTK4 中，我们需要手动绘制边框
            // 设置边框样式
            cr.set_line_width(2.0);
            cr.set_source_rgb(0.5, 0.5, 0.5); 
            // 绘制矩形边框
            cr.rectangle(page_area.x() as f64, page_area.y() as f64, 
                page_area.width() as f64, page_area.height() as f64);
            cr.stroke().expect("Failed to stroke");

        }
        
    }

    pub fn gx_view_draw_surface(&self,cr: &Context,
        surface:ImageSurface,x:i32,y:i32,
        mut offset_x:i32,mut offset_y:i32,
        target_width:i32,target_height:i32
        ,_page_area:&Rectangle){
            let device_scale_x =1;
            let device_scale_y = 1;

            let width = surface.width() / device_scale_x;
            let height = surface.height() /device_scale_y;

            let _save = cr.save();
            cr.translate(x as f64, y as f64);
           
            if width != target_width || height != target_height {
                let scale_x = target_width as f64 / width as f64;
                let mut scale_y = target_height as f64 /height as f64;
                cr.source().set_filter(gtk::cairo::Filter::Best);

                let epsilon = 0.02;
                if (scale_x - 1.0).abs() < epsilon && (scale_y - 1.0).abs() < epsilon {
                    cr.scale(1.0, 1.0);
                } else {
                    //这个scale_y仅仅在新的pdf没渲染出来的时候用？，如果新的渲染出来了，那么应该执行的
                    //是前面那个if?
                    scale_y = (target_height - self.spacing as i32) as f64 / height as f64;
                    cr.scale(scale_x, scale_y);
                }

                //cr.scale(scale_x as f64, scale_y as f64);
                //cr.scale(1.0,1.0);
                
                offset_x = (offset_x as f64 / scale_x) as i32;
                offset_y = (offset_y as f64 / scale_y) as i32;
            }
            let _paint 
                = cr.set_source_surface(surface, -offset_x as f64, -offset_y as f64);
            let _paint = cr.paint();
            let _restore = cr.restore();
    }

    //返回页码所在页面渲染时候在drawingarea中的的x,y,scaled_width,scaled_height
    pub fn gx_view_get_page_extents(&mut self,page_index:i32) -> Rectangle {
        //let area_alloc = area.allocation();
        let area_alloc = Rectangle::new(self.allocation_x.get() as i32,
            self.allocation_y.get() as i32,self.allocation_width.get() as i32,
            self.allocation_height.get() as i32);
        let (scaled_page_area_width,scaled_page_area_height) = 
            self.gx_view_get_page_size_for_scale_and_rotation(page_index);
        //self.gx_view_compute_border(area);

        let mut page_area = Rectangle::new(0,0,0,0);
        page_area.set_width(scaled_page_area_width.round() as i32  + 
            self.allocation_border.borrow().left() as i32 
            + self.allocation_border.borrow().right() as i32);
        page_area.set_height(scaled_page_area_height.round() as i32 +
            self.allocation_border.borrow().top() as i32 + 
            self.allocation_border.borrow().bottom() as i32);

        let mut x = self.spacing ;
        let y: f32;
        if self.continuous{
            self.gx_view_get_scaled_max_page_size();
            
            let _scaled_max_width = self.scaled_max_width 
                + self.allocation_border.borrow().left() as f32 
                + self.allocation_border.borrow().right() as f32;
            let max = max(0,area_alloc.width() - 
                (scaled_page_area_width + 
                self.allocation_border.borrow().left() as f32 + 
                self.allocation_border.borrow().right() as f32 +
                self.spacing  * 2.0).round() as i32);

            x = x + max as f32 / 2.0;
            y = self.gx_view_get_page_y_offset(page_index) as f32;
        }
        else{
            y = 0.0;
        }
        page_area.set_x(x.round() as i32);
        page_area.set_y(y.round() as i32);
        page_area
    }  


    pub  fn gx_view_get_page_size(&self,page_index:i32) -> (f32,f32){
        if self.uniform{
            return (self.uniform_width,self.uniform_height);
        }
        else{
            let page_width = self.page_sizes[page_index as usize].width;
            let page_height = self.page_sizes[page_index as usize].height;
            return (page_width,page_height);
        }
    }   
    
    //获取页码所在页面的长宽，长度为页面长度*缩放+0.5，宽度也一样
    pub  fn gx_view_get_page_size_for_scale_and_rotation(&self,page_index:i32) -> (f32,f32) {
        let (page_width,page_height) = self.gx_view_get_page_size(page_index);
        let mut width = page_width * self.scale  ;
        let mut height = page_height * self.scale  ;
        if self.rotation == 90 || self.rotation == 270{
            height = page_width * self.scale  ;
            width = page_height * self.scale  ;
        }
        (width,height)       
    }

 
//    //从height_to_page_cache中获取页码所在页面的y坐标
//    pub  fn gx_view_get_page_y_offset(&mut self,page_index:i32) -> f32{
//        let offset = self.gx_view_get_height_to_page(page_index);
//        //offset = offset + (page_index + 1) * self.spacing  as i32 
//        //    + page_index * (self.allocation_border.borrow().top() + 
//        //    self.allocation_border.borrow().bottom()) as i32;  
//        offset
//    }   

    pub fn gx_view_get_page_y_offset(&mut self, page_index: i32) -> f32 {
        let mut offset = self.gx_view_get_height_to_page(page_index);
    
        // 保证和绘制时的 Y 偏移计算一致
        offset += self.spacing * (page_index as f32 + 1.0);
    
        offset += (self.allocation_border.borrow().top()
            + self.allocation_border.borrow().bottom()) as f32 * (page_index as f32);
    
        offset
    }   
    pub  fn gx_view_get_height_to_page(&mut self,page_index:i32) -> f32{
        if self.rotation != self.height_to_page_cache.as_ref().unwrap().rotation{
            self.gx_view_build_height_to_page_cache_and_clean_upper();
        }
        let h = self.height_to_page_cache.as_ref().unwrap().height_to_page[page_index as usize];
        //let height = h * self.scale + 0.5;
        let height = h * self.scale ;
        height  
    }

    //已完善注释
    //获取要渲染的cairo画布的高度，并存到height_to_page_cache中
    pub  fn gx_view_get_height_to_page_cache_and_clean_upper(&mut self) {
        if self.height_to_page_cache.is_none() {
            self.gx_view_build_height_to_page_cache_and_clean_upper();
        }
    }

    pub fn gx_view_get_pointer_position(&mut self,area: &GxViewArea) -> Option<(i32, i32)> {
        // 默认值
        let mut x:i32;
        let mut y:i32;
    
        // 必须 realized
        if !area.is_realized() {
            return None;
        }
    
        let display = area.display();
        let seat = display.default_seat();
        if seat.is_none(){
            return None;
        }

        let device_pointer = seat?.pointer();
        if device_pointer.is_none(){
            return None;
        }
    

        let native = area.native();
        if native.is_none(){
            return None;
        }

        let native_clone = native.clone()?.clone();

        let surface = native.clone()?.surface();
        if surface.is_none(){
            return None;
        }

        let pointer = surface?.device_position(&device_pointer?);
        if pointer.is_none(){
            return None;
        }
        let dx = pointer?.0 as i32;
        let dy = pointer?.1 as i32;
        
        x = dx;
        y = dy;

        // 计算 widget 相对于 native 的偏移
        let point = area.compute_point(&native?.upcast::<Widget>(),
            &graphene::Point::new(0.0, 0.0)); 
        if point.is_none(){
            return None;
        }
    
        x -= point?.x() as i32;
        y -= point?.y() as i32;

        // surface transform 偏移
        let (dx, dy) = native_clone.surface_transform();
    
        x -= dx as i32;
        y -= dy as i32;
    
        Some((x, y))
    }



    pub fn gx_view_get_scaled_max_page_size(&mut self){
        let width = self.max_width * self.scale ;
        let height = self.max_height * self.scale ;
        if self.rotation == 0 || self.rotation == 180{
            self.scaled_max_width = width;
            self.scaled_max_height = height;           
        }else{
            self.scaled_max_width = height;
            self.scaled_max_height = width;           
        }
    }
    

    pub async fn gx_view_get_surface_lists(&mut self,hash_surfaces:HashMap<i32,Option<GxSurfaceData>>)
         {
        let mut temp_render_pages_hash: HashMap<i32,f32> = HashMap::new();
        for (_page_index,surface) in hash_surfaces.into_iter(){
            if let Some(surface) = surface{
                let page_index = surface.page_index;

                let (_page_width,_page_height) 
                    = self.gx_view_get_page_size(page_index);       
                let draw_page = DrawPage{
                    page_index,
                    surface_borrow: Some(surface),
                    //page_width,
                    //page_height,
                    scale: self.scale,
                };
                self.hash_draw_pages_cache.insert(page_index,draw_page);
                //gpc_hash_draw_pages.insert(page_index,draw_page);
                temp_render_pages_hash.insert(page_index,self.scale);
            }
        }
        self.temp_render_pages_hash = temp_render_pages_hash;
        //list_draw_pages

    }   


    pub  fn gx_view_get_start_and_end_page(&mut self){
        let mut current_area = Rectangle::new(0, 0, 0, 0);
        if self.continuous{
            let mut found = false;
            let mut area_max = -1;
            let mut best_current_page  = -1;
            
            let mut current_width = self.allocation_width.get() as i32;
            if current_width == 0 {
                current_width = self.max_width as i32;
            }
            current_area.set_width(current_width);
            let current_y = self.vj_value.get() as i32;
            current_area.set_y(current_y);
            let mut current_height = self.allocation_height.get() as i32;
            if current_height == 0 {
                current_height = self.max_height as i32;
            }
            current_area.set_height(current_height);
            for i in 0..self.page_numbers{
                //返回页码所在页面渲染时候在drawingarea中的的x,y,scaled_width,scaled_height
                let page_area = self.gx_view_get_page_extents(i);
                if let Some(unused_area) = current_area.intersect(&page_area){
                    let area = unused_area.width() * unused_area.height();
                    if !found {
                        area_max = area;

                        self.start_page = i;

                        found = true;
                        best_current_page = i;
                    }
                    if area > area_max {
                        best_current_page = i;
                        area_max = area;
                    }
                    self.end_page = i;
                }else if found && self.current_page <= self.end_page{
                    break;
                }
            }
            
            if self.pending_scroll ==  PendingScroll::ScrollToKeepPosition  {
                    best_current_page = max(best_current_page,self.start_page);
                    if best_current_page >= 0 && self.current_page != best_current_page{
                        self.current_page = best_current_page;
                        //self.document_model.set_page(best_current_page);
                        //self.gx_view_change_page(best_current_page);
                    }
                }
            }       
//        //没有这个if，到最后的时候，pdf顶部会有大片空白
//        if self.start_page > 0{
//            self.start_page = self.start_page -1;
//        }
    }

    pub  fn gx_view_get_start_and_end_page_new (&mut self,
        area: &GxViewArea){
        let mut current_area = Rectangle::new(0, 0, 0, 0);
        if self.continuous{
            let mut found = false;
            let mut area_max = -1;
            let mut best_current_page  = -1;
            
            let mut current_width = area.width();
            if current_width == 0 {
                current_width = self.max_width as i32;
            }
            current_area.set_width(current_width);
            let current_y = self.vj_value.get() as i32;
            current_area.set_y(current_y);
            let mut current_height = area.height();
            if current_height == 0 {
                current_height = self.max_height as i32;
            }
            current_area.set_height(current_height);

            for i in 0..self.page_numbers{
                //返回页码所在页面渲染时候在drawingarea中的的x,y,scaled_width,scaled_height
                let page_area = self.gx_view_get_page_extents(i);

                if let Some(unused_area) = current_area.intersect(&page_area){
                    let area = unused_area.width() * unused_area.height();
                    if !found {
                        area_max = area;

                        self.start_page = i;

                        found = true;
                        best_current_page = i;
                    }
                    if area > area_max {
                        best_current_page = i;
                        area_max = area;
                    }
                    self.end_page = i;
                }else if found && self.current_page <= self.end_page{
                    break;
                }
            }
            
            if self.pending_scroll ==  PendingScroll::ScrollToKeepPosition || 
                self.pending_scroll == PendingScroll::ScrollToFindLocation{
                    best_current_page = max(best_current_page,self.start_page);
                    if best_current_page >= 0 && self.current_page != best_current_page{
                        self.current_page = best_current_page;
                        //self.document_model.set_page(best_current_page);
                        self.gx_view_change_page(best_current_page);
                        //此处待完善?，evince中是直接gtk_widget_queue_resize 
                    }
                }
            }       
//        //没有这个if，到最后的时候，pdf顶部会有大片空白
//        if self.start_page > 0{
//            self.start_page = self.start_page -1;
//        }
    }

    fn gx_view_get_widget_dpi(&self,area: &GxViewArea) -> i32{
        let display = area.display();
        if let Some(native) = area.native(){
            if let Some(surface) = native.surface(){
                if let Some(monitor) 
                    = display.monitor_at_surface(&surface){
                    let geometry = monitor.geometry();
                    let mut is_landscape = false;
                    if geometry.width() > geometry.height(){
                        is_landscape = true;
                    }
                    if is_landscape && geometry.height() >= 1080{
                        return 192;
                    }else{
                        return 96;
                    }
                }else{
                    return 96;
                }
            }else{
                return 96;
            }
        }else{
            return 96;
        }
    }

    //已完善注释
    //设置start_page与end_page，两者相差0,其实不影响，会根据相关的页面宽度重新计算这两个值
    //还设置了current_page，当前为0,还需完善
    //设置所有页面的宽、高缓存，设置所有的页面标签，如目录的罗马标签,设置page_numbers
    //new了pixbuf_cache与page_cache，未操作cache结构体中具体的各种参数
    pub async fn gx_view_init_document(&mut self,start_page:i32){
        self.find_page = -1;
        self.find_result = 0;
        self.spacing = 5.0;
        self.scale = 1.0;
        self.current_page = 0;
        self.pressed_button = -1;
        self.continuous = true;
        self.fullscreen = false;
        self.sizing_mode = GxSizingMode::FitWidth;
        self.find_page = -1;
        self.pixbuf_cache_size = DEFAULT_PIXBUF_CACHE_SIZE;
        self.zoom_center_x = -1.0;
        self.zoom_center_y = -1.0;
        self.find_result = 0;
        self.pending_scroll = PendingScroll::ScrollToKeepPosition;
        self.rotation = 0; 
        self.rtl = false;
        self.fullscreen = false;
        self.dual_even_left = false;//暂定的
                                    
        //已完善注释
        //设置所有页面的宽、高缓存，设置所有的页面标签，如目录的罗马标签,设置page_numbers
        self.gx_view_set_cache_from_pdfium().await;

        self.start_page = start_page;
        self.end_page = start_page ;

        //已完善注释
        //仅new了pixbuf_cache与page_cache，未操作cache结构体中具体的各种参数
        self.gx_view_new_caches().await;
    }
    

    //已完善注释
    //仅new了pixbuf_cache与page_cache，未操作cache结构体中具体的各种参数
    pub async fn gx_view_new_caches(&mut self) {
        //let inverted_colors;
        //
        self.gx_view_get_height_to_page_cache_and_clean_upper();
        //pixbuf_cache用于存储最近几页渲染好的imagesurface
        //此处仅仅是new，还啥也没有呢
        let pixbuf_cache = GxPixbufCache::gx_pixbuf_cache_new(
            self.pixbuf_cache_size,self.page_sizes.clone(),self.uniform,
            self.uniform_width,self.uniform_height,self.job_scheduler.clone(),self.scale,1.0);
        self.pixbuf_cache = pixbuf_cache;
        self.page_cache = GxPageCache::page_cache_new(self.page_numbers);
        
    }




    //已完善注释
    //设置所有页面的宽、高缓存，设置所有的页面标签，如目录的罗马标签,设置page_numbers
    pub async fn gx_view_set_cache_from_pdfium (&mut self){
        let document_mutex 
            = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");
        let mut document_guard = document_mutex.lock().await;
        let current_tab = *(CURRENTTAB.get().unwrap());
        if let Some(gx_pdf_document) = document_guard.get_mut(&current_tab){
            self.uniform = true;
            self.cache_loaded = true;

            let mut custom_label = false;
            let mut vec_page_sizes = Vec::new();
            let mut vec_page_labels = HashMap::new();
            if let Some(pdf_document) 
                = &gx_pdf_document.pdf_document{
                let pages 
                    = pdf_document.pages();
                    
                self.page_numbers = pages.len() as i32;
                for i in 0..self.page_numbers{
                    let page = pages.get(i as u16).unwrap();
                    let width = page.width().value;
                    let height = page.height().value;

                    if i == 0{
                        self.uniform_width = width;
                        self.uniform_height = height;
                        self.max_width = width;
                        self.max_height = height;
                        self.min_width = width;
                        self.min_height = height;
                    }else if self.uniform && (self.uniform_width != width
                        || self.uniform_height != height){
                        for _j in 0 .. i{
                            let gx_page_size = GxPageSize{
                                width: self.uniform_width,
                                height: self.uniform_height,
                            };
                            vec_page_sizes.push(gx_page_size);
                        }
                        self.uniform = false;
                    }

                    if self.uniform == false{
                        let gx_page_size = GxPageSize{
                            width,
                            height,
                        };
                        vec_page_sizes.push(gx_page_size);

                        if width > self.max_width{
                            self.max_width = width;
                        }
                        if width < self.min_width{
                            self.min_width = width;
                        }

                        if height > self.max_height{
                            self.max_height = height;
                        }
                        if height < self.min_height{
                            self.min_height = height;
                        }
                    }

                    let page_label = page.label();

                    if page_label.is_some(){
                        let label = page_label.unwrap();

                        if !custom_label{
                            let real_page_label = format!("{}",i+1);
                            if real_page_label != label.to_string(){
                                custom_label = true;
                            }
                        }

                        self.max_label = self.max_label.max(label.to_string().len() as i32);
                        vec_page_labels.insert(i, label.to_string());
                    }


                }
                self.page_sizes = vec_page_sizes;
                self.page_labels = Some(vec_page_labels);
            }

        }
    }

    fn gx_view_set_max_scale(&mut self,max_scale:f32){
        if self.max_scale == max_scale{
            return;
        }
        self.max_scale = max_scale;
        
        if self.scale > max_scale{
            self.scale = max_scale;
        }
    }
    
    fn gx_view_set_min_scale(&mut self,min_scale:f32){
        if self.min_scale == min_scale{
            return;
        }
        self.min_scale = min_scale;
        
        if self.scale < min_scale{
            self.scale = min_scale;
        }
    }

    fn gx_view_set_scale(&mut self,scale:f32,
        area:&GxViewArea,h_scroll_bar:&Scrollbar,
        v_scroll_bar:&Scrollbar){
        let scale = scale.clamp(self.min_scale, self.max_scale);
        if self.scale == scale{
            return;
        }
        self.scale = scale;
        self.gx_view_size_allocate(area, h_scroll_bar, v_scroll_bar);
        self.gx_view_update_can_zoom();
    }

    fn gx_view_size_allocate(&mut self,area:&GxViewArea,
        h_scroll_bar:&Scrollbar,v_scroll_bar:&Scrollbar){
        self.gx_view_size_request_continuous();//这个函数得到的self.max target_width有用
        self.gx_view_update_adjustment_values(area,h_scroll_bar,v_scroll_bar);
    }

    fn gx_view_size_request_continuous(&mut self){
        let width = (self.max_width * self.scale + 0.5) as i32 
            + self.spacing as i32 * 2;
        let height = (self.max_height * self.scale + 0.5) as i32 
            + self.spacing as i32* 2;

        if self.rotation == 0 || self.rotation == 180{
            self.max_target_page_size.width = width;
            self.max_target_page_size.height = height;
        }else{
            self.max_target_page_size.width = height;
            self.max_target_page_size.height = width;
        }
    }

    fn gx_view_update_adjustment_value(&mut self,
        area:&GxViewArea,orientation:gtk::Orientation,
        scroll_bar:&Scrollbar){
        let max_target_size;
        let allocate_size;
        let adjustment = scroll_bar.adjustment();
        let mut zoom_center;
        let mut new_upper;
        if orientation == gtk::Orientation::Horizontal{
            max_target_size = self.max_target_page_size.width;
            allocate_size = area.width();
            zoom_center = self.zoom_center_x;
            new_upper = max_target_size as f64;
//            new_upper = new_upper * self.scale as f64 
//                + self.allocation_border.borrow().left() as f64 
//                + self.allocation_border.borrow().right() as f64;
            new_upper = new_upper  
                //+ self.allocation_border.borrow().left() as f64 
                //+ self.allocation_border.borrow().right() as f64
                + self.spacing as f64 * 2.0 ;
 
        }else{
            //max_target_size = self.max_target_page_size.height;
            allocate_size = area.height();
            zoom_center = self.zoom_center_y;
            new_upper = self.clean_upper as f64;
            let pg = self.page_numbers;
            let total_spacing = self.spacing  * (pg as f32 + 1.0);
            new_upper = new_upper * self.scale as f64 + total_spacing as f64
                + self.allocation_border.borrow().top() as f64 
                + self.allocation_border.borrow().bottom() as f64;
        }

        if new_upper <= allocate_size as f64{
            scroll_bar.set_visible(false);
            if self.scroll_x != 0.0{
                self.scroll_x = 0.0;
                area.gx_view_area_all_page_redraw(self.scroll_x);
            }
            return;
        }else{
            scroll_bar.set_visible(true);
        }

        let mut value_upper_factor = 1.0;
        let value = adjustment.value();
        let upper = adjustment.upper();
        let page_size = adjustment.page_size();

        if zoom_center < 0.0 {
            zoom_center = page_size as f32 * 0.5;
        }

        if upper != 0.0 {
            value_upper_factor = match self.pending_scroll{
                PendingScroll::ScrollToKeepPosition => value / upper,
                PendingScroll::ScrollToPagePosition => 1.0,
                PendingScroll::ScrollToCenter => (value + zoom_center as f64) / upper,
                PendingScroll::ScrollToFindLocation => 1.0,
            }
        }

        new_upper = new_upper.max(allocate_size as f64);
        let new_page_size = allocate_size as f64;

        let new_value = match self.pending_scroll{
            PendingScroll::ScrollToKeepPosition => {
                let new_value = new_upper * value_upper_factor;
                let new_value = new_value.clamp(0.0, new_upper - new_page_size);
                new_value
            },
            PendingScroll::ScrollToPagePosition => value,
            PendingScroll::ScrollToCenter => {
                if orientation == gtk::Orientation::Horizontal{
                    self.zoom_center_x = -1.0;
                }else{
                    self.zoom_center_y = -1.0;
                }
                let new_value = new_upper * value_upper_factor - zoom_center as f64;
                let new_value = new_value.clamp(0.0, new_upper - new_page_size);
                new_value
            },
            PendingScroll::ScrollToFindLocation => value,
        };
        scroll_bar.adjustment().set_upper(new_upper);
        scroll_bar.adjustment().set_page_size(new_page_size);
        scroll_bar.adjustment().set_value(new_value);
        
    }


    fn gx_view_update_adjustment_values(&mut self,
        area:&GxViewArea,h_scroll_bar:&Scrollbar,
        v_scroll_bar:&Scrollbar,){
        self.gx_view_update_adjustment_value(area,gtk::Orientation::Horizontal,h_scroll_bar);
        self.gx_view_update_adjustment_value(area,gtk::Orientation::Vertical,v_scroll_bar);
    }


    pub fn gx_view_update_allocation_and_vj_value(&mut self,
        area: &GxViewArea,scroll_bar:&Scrollbar, 
        _sender: AsyncComponentSender<GxView>){

        let allocation = area.allocation();
        self.allocation_width.set(allocation.width() as f32);
        self.allocation_height.set(allocation.height() as f32);
        self.allocation_x.set(allocation.x() as f32);
        self.allocation_y.set(allocation.y() as f32);
        let style_context = area.style_context();
        let border = style_context.border();
        self.allocation_border.borrow_mut().set_left(border.left());
        self.allocation_border.borrow_mut().set_right(border.right());
        self.allocation_border.borrow_mut().set_top(border.top());
        self.allocation_border.borrow_mut().set_bottom(border.bottom());
        self.vj_value.set(scroll_bar.adjustment().value() as f32);
    }

    pub fn gx_view_update_allocation(&mut self,
        area: &GxViewArea
        ){
        let allocation = area.allocation();
        self.allocation_width.set(allocation.width() as f32);
        self.allocation_height.set(allocation.height() as f32);
        self.allocation_x.set(allocation.x() as f32);
        self.allocation_y.set(allocation.y() as f32);
        let style_context = area.style_context();
        let border = style_context.border();
        self.allocation_border.borrow_mut().set_left(border.left());
        self.allocation_border.borrow_mut().set_right(border.right());
        self.allocation_border.borrow_mut().set_top(border.top());
        self.allocation_border.borrow_mut().set_bottom(border.bottom());
    }

    fn gx_view_update_can_zoom(&mut self){
        let min_scale = self.min_scale;
        let max_scale = self.max_scale;
        let scale = self.scale;
        let can_zoom_in = scale < max_scale;
        let can_zoom_out = scale > min_scale;
        if self.can_zoom_in != can_zoom_in{
            self.can_zoom_in = can_zoom_in;
        }
        if self.can_zoom_out != can_zoom_out{
            self.can_zoom_out = can_zoom_out;
        }
    }

    pub fn gx_view_update_self_vj_values(&mut self,
        vscroll_bar:&Scrollbar 
        ){
        self.vj_value.set(vscroll_bar.adjustment().value() as f32);
    }


    pub fn gx_view_update_hscroll_values(&mut self,
        hscroll_bar:&Scrollbar,area: &GxViewArea,
        ){
        let _alloc_size = area.width();
        //旧值与factor
        let old_value = hscroll_bar.adjustment().value();
        let old_upper = hscroll_bar.adjustment().upper();
        let old_page_size = hscroll_bar.adjustment().page_size();
		let mut zoom_center = self.zoom_center_x;
        let factor:f64;
        //evince中还是根据不同情况设置不同值
        if zoom_center >= 0.0{
            factor = (old_value + zoom_center as f64) / old_upper;
        }else{
            factor = old_value / old_upper;           
            zoom_center = old_page_size as f32 * 0.5;
        }       
        let new_upper = self.scale * self.max_width + self.spacing * 2.0;
        let new_upper = new_upper as f64;
        //let new_upper = alloc_size as f64;
        let new_page_size = old_page_size;

        //新value
        let new_value:f64;
        if zoom_center >= 0.0{
            new_value = (new_upper * factor - zoom_center as f64).
                clamp(0.0, new_upper - new_page_size);
        }else{
            new_value = new_upper * factor.
                clamp(0.0, new_upper - new_page_size);
        }
        hscroll_bar.adjustment().set_upper(new_upper);
        //hscroll_bar.adjustment().set_page_size(new_page_size);
        hscroll_bar.adjustment().set_value(new_value);
       
        //设置zoom_center_y
        self.zoom_center_x = -1.0;
    }



    pub fn gx_view_update_vscroll_values(&mut self,
        vscroll_bar:&Scrollbar,area: &GxViewArea,
        ){
        //旧值与factor
        let old_value = vscroll_bar.adjustment().value();
        let old_upper = vscroll_bar.adjustment().upper();
		let zoom_center = self.zoom_center_y;
        let factor:f64;
        //evince中还是根据不同情况设置不同值
        if zoom_center >= 0.0{
            factor = (old_value + zoom_center as f64) / old_upper;
        }else{
            factor = old_value / old_upper;           
        }       

        //新upper
        let mut new_upper = self.clean_upper as f64;
        let pg = self.page_numbers;
        let total_spacing = self.spacing  * (pg as f32 + 1.0);
        new_upper = new_upper * self.scale as f64 + total_spacing as f64
            + self.allocation_border.borrow().top() as f64 
            + self.allocation_border.borrow().bottom() as f64;
		let alloc_size = area.height();
        new_upper = new_upper.max(alloc_size as f64);

        //新page_size
        let new_page_size = alloc_size as f64;

        //新value
        let new_value:f64;
        if zoom_center >= 0.0{
            new_value = (new_upper * factor - zoom_center as f64).
                clamp(0.0, new_upper - new_page_size);
        }else{
            new_value = new_upper * factor.
                clamp(0.0, new_upper - new_page_size);
        }
        vscroll_bar.adjustment().set_upper(new_upper);
        vscroll_bar.adjustment().set_page_size(new_page_size);
        vscroll_bar.adjustment().set_value(new_value);
       
        //设置zoom_center_y
        self.zoom_center_y = -1.0;
    }



    //待完善
    pub fn gx_view_update_range_and_current_page(&mut self,
        sender: AsyncComponentSender<Self>,
        ){
        
        //let document_guard = document_mutex.lock().await;
        //if let Some(_gx_pdf_document) = document_guard.get(&current_tab){
            //已完善注释
            //确认是否已将所有页面的宽度、高度等存储起来了，若为存储就调用函数存储
            //如果调用了函数还是不行，那这个pdf文件就有问题，应该就要退出，并提示用户pdf文件有问题了
            //提示用户的代码需完善
            if self.page_numbers <= 0 || !self.gx_view_check_dimensions_after_init(){
                println!("卡在这2-5");
                return;
            }
            //self.gx_view_get_start_and_end_page();
                //println!("卡在这2-6");
            //self.end_page = 1;
            if self.start_page == -1 || self.end_page == -1{
                println!("卡在这2-7");
                return;
            }
            //这个page_cache先不管 
            self.page_cache.page_cache_set_page_range(self.start_page, self.end_page);
            self.pixbuf_cache.gx_pixbuf_cache_set_page_range(self.start_page, 
                self.end_page,self.page_numbers, self.scale,self.device_scale,self.rotation,
                sender, self.job_scheduler.clone());
    }
    
    //待完善
    pub async fn gx_view_update_range_and_current_page_new(&mut self,
        sender: AsyncComponentSender<Self>,
        ){
        let _start_page = self.start_page;
        let _end_page = self.end_page;

        //确认是否已将所有页面的宽度、高度等存储起来了，若为存储就调用函数存储
        //如果调用了函数还是不行，那这个pdf文件就有问题，应该就要退出，并提示用户pdf文件有问题了
        if self.page_numbers <= 0 || !self.gx_view_check_dimensions_after_init(){
            println!("卡在这2-5");
            return;
        }
        self.gx_view_get_start_and_end_page();
        if self.start_page == -1 || self.end_page == -1{
            println!("卡在这2-7");
            return;
        }
        self.page_cache.page_cache_set_page_range(self.start_page, self.end_page);
        let _current_tab = *(CURRENTTAB.get().unwrap());
            let _document_mutex 
            = GXPDFDOCUMENT.get().expect("GXPDFDOCUMENT should be initialized");

        {
           //这个page_cache先不管 
            self.pixbuf_cache.gx_pixbuf_cache_set_page_range(self.start_page, 
                self.end_page,self.page_numbers, self.scale,self.device_scale,self.rotation,
                sender,self.job_scheduler.clone());
           
        }
               
    }

    fn gx_view_update_scale_limits(&mut self,area: &GxViewArea){
        let dpi = self.gx_view_get_widget_dpi(area) as f32 / 72.0;
        
        let max_width = self.max_width;
        let max_height = self.max_height;

        let mut max_scale = f32::sqrt(self.pixbuf_cache_size as f32 
            / (max_width * 4.0 * max_height));
        max_scale = max_scale.min(MAX_IMAGE_SIZE as f32 / max_height);
        max_scale = max_scale.min(MAX_IMAGE_SIZE as f32 / max_width);

        self.gx_view_set_min_scale(MIN_SCALE * dpi);
        self.gx_view_set_max_scale(max_scale);
    }

    fn gx_view_zoom(&mut self,factor:f32,
        area:&GxViewArea,h_scroll_bar:&Scrollbar,
        v_scroll_bar:&Scrollbar){
        self.pending_scroll = PendingScroll::ScrollToCenter;
        let scale = self.scale * factor;
        self.gx_view_set_scale(scale,area, h_scroll_bar, v_scroll_bar);
    }



}



#[derive(Clone,Debug)]
pub struct DrawPage{
    pub page_index: i32,
    //pub page_area: Rectangle,
    pub surface_borrow: Option<GxSurfaceData>,
    //pub page_width: f32,
    //pub page_height: f32,
    pub scale: f32,
}

#[derive(Clone,Debug,Default)]
pub struct Requisition{
    pub width: i32,
    pub height: i32,
}
