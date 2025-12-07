//gx_view_area/imp.rs
use std::cell::RefCell;

use gtk::{gdk::Rectangle, glib, 
    graphene::Rect, prelude::*, 
    subclass::prelude::*
};

use crate::{components::gx_window::GX_VIEW_PAGES_DEFAULT_LEN, 
    gxview::{gx_view_area,gx_view_page::GxViewPage}
};


#[derive(Default)]
pub struct GxViewArea{
    pub requisition_width: RefCell<f32>,
    pub requisition_height: RefCell<f32>,
    pub gx_view_pages: RefCell<Vec<GxViewPage>>,
    pub start_page: RefCell<i32>,
    pub end_page: RefCell<i32>,
    pub spacing: RefCell<f32>,
    pub max_width: RefCell<f32>,
    pub max_height: RefCell<f32>,
    pub scroll_x: RefCell<f32>,
    pub scroll_y: RefCell<f32>,
    pub scale: RefCell<f32>,
    pub device_scale: RefCell<f32>,
    pub rotation: RefCell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for GxViewArea{
    const NAME: &'static str = "GxViewArea";
    type Type = gx_view_area::GxViewArea;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for GxViewArea{
    fn constructed(&self) {
        //let gx_view_area= GxViewArea::default();
        self.spacing.replace(5.0);
        self.scale.replace(1.0);
        self.rotation.replace(0);

        let gx_view_pages:Vec<GxViewPage> = (0..GX_VIEW_PAGES_DEFAULT_LEN).
            map(|_i| GxViewPage::default()).
            collect();
        
        let widget = self.obj();
        let widget = widget.upcast_ref::<gtk::Widget>();
        for page in gx_view_pages.iter(){
            page.set_parent(widget);
        }

        self.gx_view_pages.replace(gx_view_pages);
    }
    
    // 添加 dispose 方法来清理子 widget
    fn dispose(&self) {
        for page in self.gx_view_pages.borrow().iter() {
            page.unparent();
        }
      
    }
}

impl WidgetImpl for GxViewArea{
    fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {

//        if orientation == Orientation::Horizontal{
//            let minimum = *self.requisition_width.borrow() as i32;
//            return (minimum,minimum,-1,-1);
//        }else{
//            let minimum = *self.requisition_height.borrow() as i32;
//            return (minimum,minimum,-1,-1);
//        }
        //let (min_size, nat_size) = if orientation == gtk::Orientation::Horizontal {
        //    (200, 400) // 确保有最小尺寸
        //} else {
        //    (900, 900)
        //};
        let (min_size,nat_size) = (1,1);
        (min_size, nat_size, -1, -1)
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        let view_area = Rectangle::new(0, 0 ,width , height );
        let pages_len =  self.gx_view_pages.borrow().len();
 
        for i in 0..pages_len {
            let page = &self.gx_view_pages.borrow()[i];
            let page_index = page.gx_view_page_get_page_index();
            if page_index < 0 {
                page.set_visible(false);
                continue;
            }
            if page_index < *self.start_page.borrow() || page_index > *self.end_page.borrow() {
                page.set_visible(false);
                continue;
            }
            let mut page_area = page.gx_view_page_get_page_extents(width, 
                *self.spacing.borrow());

            page_area.set_x(page_area.x() - *self.scroll_x.borrow() as i32 
                + (*self.spacing.borrow() / 2.0) as i32);
            page_area.set_y(page_area.y() - *self.scroll_y.borrow() as i32 );
            page_area.set_width(page_area.width() as i32 + 4);
            page_area.set_height(page_area.height() as i32 + 4);

            if let Some(_area) = page_area.intersect(&view_area){
                page.set_visible(true);               
                page.size_allocate(&page_area, baseline);
            }            

        }
    }
    // We override the snapshot virtual function to draw custom graphics
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let gx_view_width = self.obj().width() as f32;
        let gx_view_height = self.obj().height() as f32;
        snapshot.push_clip(&Rect::new(0.0,0.0,gx_view_width,gx_view_height));
        self.parent_snapshot(snapshot);
        snapshot.pop();
    }

  
}