//gx_view_page/imp.rs
use std::cell::RefCell;

use gtk::{gdk::{Texture, RGBA}, glib, 
    graphene::{Rect, Size}, gsk::RoundedRect, 
    prelude::*, subclass::prelude::*
};

use crate::gxview::gx_view_page;


#[derive(Default)]
pub struct GxViewPage{
    pub page_index:RefCell<i32>,
    pub scale:RefCell<f32>,
    pub device_scale:RefCell<f32>,
    pub rotation:RefCell<i32>,
    pub width:RefCell<f32>,
    pub height: RefCell<f32>,
    pub height_offset: RefCell<f32>,
    //pub document_model: Option<Arc<GxDocumentModel>>,
    //pub pixbuf_cache: GxPixbufCache,
    pub texture: RefCell<Option<Texture>>,

}

#[glib::object_subclass]
impl ObjectSubclass for GxViewPage{
    const NAME: &'static str = "GxViewPage";
    type Type = gx_view_page::GxViewPage;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for GxViewPage{
    fn constructed(&self) {
        //let gx_view_page = GxViewPage::default();
        self.page_index.replace(-1);
        self.scale.replace(1.0);
        self.rotation.replace(0);
    }
    
    // 添加 dispose 方法来清理子 widget
    fn dispose(&self) {
      
    }
}

impl WidgetImpl for GxViewPage{
    fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let target_width ;
        let target_height;
        if *self.page_index.borrow() >= 0{
            if *self.rotation.borrow() == 0 
                || *self.rotation.borrow() == 180{
                target_width = (*self.width.borrow() * *self.scale.borrow() + 0.5 ).round() as i32;
                target_height = (*self.height.borrow() * *self.scale.borrow() + 0.5 ).round() as i32;
            }else{
                target_width = (*self.height.borrow() * *self.scale.borrow() + 0.5 ).round() as i32;
                target_height = (*self.width.borrow() * *self.scale.borrow() + 0.5 ).round() as i32;
            }
            if orientation == gtk::Orientation::Horizontal{
                let minimum = 0;
                let natural = target_width;
                return (minimum,natural,-1,-1);
            } else {
                let minimum = 0;
                let natural = target_height;
                return (minimum,natural,-1,-1);
            }
            
        }
        (0,0,-1,-1)
    }

    //暂时不需要，因为目前还没有那些child widget 
    //fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
    //}
    // We override the snapshot virtual function to draw custom graphics
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let allocated_width = self.obj().width();
        let allocated_height = self.obj().height();
        //gp means grand_parent
        if let Some(gp_native) = self.obj().native(){
            if let Some(gp_surface )= gp_native.surface(){
                let gp_fractional_scale = gp_surface.scale_factor();
                if *self.page_index.borrow() < 0{
                    return;
                }

                let bounds =  Rect::new(0 as f32,0 as f32,allocated_width as f32,allocated_height as f32);
                let outline =  RoundedRect::new(bounds,
                Size::zero(),Size::zero(),Size::zero(),Size::zero()); 
                let border_width = [2.0 as f32,2.0 as f32,2.0 as f32,2.0 as f32];
                //let grey = RGBA::new(0.9, 0.9, 0.9, 1.0);//得到灰色
                let light_blue = RGBA::new(0.263, 0.431, 0.933, 1.0);

                let border_color = [light_blue,light_blue,light_blue,light_blue];                                                 
                //绘制边框
                snapshot.append_border(&outline, &border_width, &border_color);
 
                let page_texture = self.texture.borrow();
                if page_texture.is_none(){
                    return;
                }else{
                    if let Some(ref page_texture) = *page_texture{
                        let _y = *self.height_offset.borrow();
                        let mut area 
                            = Rect::new(
                                2.0 as f32,
                                2.0,
                                (allocated_width * gp_fractional_scale) as f32,
                                (allocated_height * gp_fractional_scale) as f32,
                            );
                        snapshot.save();
                        snapshot.scale(1 as f32 / gp_fractional_scale as f32, 
                            1 as f32 / gp_fractional_scale as f32);
                        self.obj().gx_view_page_draw_surface(snapshot, 
                            &page_texture, &mut area);
                        snapshot.restore();
                        self.parent_snapshot(snapshot);
                    }
                }
            }
        }
        //let (width,height) = self.obj().get_size();
        //snapshot.push_clip(&Rect::new(0 as f32 ,0 as f32,width as f32,height as f32));
        //self.parent_snapshot(snapshot);
        //snapshot.pop();
    }

  
}