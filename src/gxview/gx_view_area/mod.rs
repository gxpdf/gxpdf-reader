//gx_view_area/mod.rs
pub mod imp;


use gtk::{
    glib::{self }, prelude::WidgetExt, subclass::prelude::*
};

use crate::{gxdocument::gx_pdf_document::GxPageSize, 
    gxview::{gx_pixbuf_cache::GxPixbufCache,
    gx_view::GxHeightToPageCache}
};



glib::wrapper! {
    pub struct GxViewArea(ObjectSubclass<imp::GxViewArea>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for GxViewArea{
    fn default() -> Self {
        glib::Object::new()
    }
}

impl GxViewArea{
    pub fn gx_view_area_all_page_redraw(&self,
        scroll_x:f32,
    ) {
        self.imp().scroll_x.replace(scroll_x);
        self.queue_resize();
        self.queue_draw();
    }


    pub fn gx_view_area_get_gx_view_pages_len(&self) -> i32{
        self.imp().gx_view_pages.borrow().len() as i32
    }

    pub fn gx_view_area_page_redraw(&self,
        pixbuf_cache: &GxPixbufCache,
        page_index:i32,
    ) {

        let gx_view_page_lens = self.gx_view_area_get_gx_view_pages_len();
        for i in 0..gx_view_page_lens{
            let page = &self.imp().gx_view_pages.borrow()[i as usize];
            if page.gx_view_page_get_page_index() == page_index{
                 let mut texture = None;
                if let Some(cache_job_info) 
                    = pixbuf_cache.gx_pixbuf_cache_find_job_cache_unmut(page_index){
                        texture = cache_job_info.texture.clone();
                }
                //set texture会调用redraw
                page.gx_view_page_set_texture(texture);               
            }
        }       
    }

//    pub fn gx_view_area_set_pages(&self,
//        start_gx_view_page_widget_index:i32,
//        end_gx_view_page_widget_index:i32,
//        start_page:i32,end_page:i32,
//        page_numbers:i32,scale:f32,rotation:i32,
//        height_to_page_cache: &GxHeightToPageCache,
//        page_size: GxPageSize,
//        texture:Option<Texture>,rendered_index:i32
//    ){
//
//        self.imp().start_page.replace(start_page);
//        self.imp().end_page.replace(end_page);
//        self.imp().scale.replace(scale);
//        self.imp().rotation.replace(rotation);
//
//        let gx_view_page_lens = self.gx_view_area_get_gx_view_pages_len();
//        for i in 0..gx_view_page_lens{
//            let page = &self.imp().gx_view_pages.borrow()[i as usize];
//            let mut page_index = -1;
//            if start_gx_view_page_widget_index <= end_gx_view_page_widget_index{
//                if start_gx_view_page_widget_index <= i &&
//                    i <= end_gx_view_page_widget_index{
//                    page_index = start_page - start_gx_view_page_widget_index + i as i32;
//                }
//            }else{
//                if i <= end_gx_view_page_widget_index{
//                    page_index = end_page - end_gx_view_page_widget_index + i as i32;
//                }else if start_gx_view_page_widget_index <= i{
//                    page_index = start_page - start_gx_view_page_widget_index + i as i32;
//                }
//            }
//            if page_index != -1{
//                let width = page_size.width;
//                let height = page_size.height;
//                let height_offset = height_to_page_cache.height_to_page[page_index as usize];
//                if page_index == rendered_index{
//                    page.gx_view_page_set_page(page_index, page_numbers,scale,rotation,
//                        width,height,height_offset,texture.clone());
//                    self.queue_resize();
//                }
//
//            }
//        }
//    }

    pub fn gx_view_area_size_allocate(&self,
        start_gx_view_page_widget_index:i32,
        end_gx_view_page_widget_index:i32,
        start_page:i32,end_page:i32,
        page_numbers:i32,scale:f32,rotation:i32,
        height_to_page_cache: &GxHeightToPageCache,
        uniform:bool,uniform_width:f32,uniform_height:f32,
        page_sizes: &Vec<GxPageSize>,scroll_y:f32,
        pixbuf_cache: &GxPixbufCache,
    ) {
        self.imp().start_page.replace(start_page);
        self.imp().end_page.replace(end_page);
        self.imp().scale.replace(scale);
        self.imp().rotation.replace(rotation);
        self.imp().scroll_y.replace(scroll_y);

        let gx_view_page_lens = self.gx_view_area_get_gx_view_pages_len();
        for i in 0..gx_view_page_lens{
            let page = &self.imp().gx_view_pages.borrow()[i as usize];
            let mut page_index = -1;
            if start_gx_view_page_widget_index <= end_gx_view_page_widget_index{
                if start_gx_view_page_widget_index <= i &&
                    i <= end_gx_view_page_widget_index{
                    page_index = start_page - start_gx_view_page_widget_index + i as i32;
                }
            }else{
                if i <= end_gx_view_page_widget_index{
                    page_index = end_page - end_gx_view_page_widget_index + i as i32;
                }else if start_gx_view_page_widget_index <= i{
                    page_index = start_page - start_gx_view_page_widget_index + i as i32;
                }
            }
            if page_index != -1{
                let page_size;
                if uniform{
                    page_size 
                        = GxPageSize { width:uniform_width, height:uniform_height};
                }else if page_sizes.len() > 0{
                    if let Some(temp_page_size) 
                        = page_sizes.get(page_index as usize){
                            page_size = temp_page_size.clone();
                    }else{
                        return;
                    }
                }else{
                    return;
                }
 
                let width = page_size.width;
                let height = page_size.height;
                let height_offset = height_to_page_cache.height_to_page[page_index as usize];
                let mut texture = None;
                if let Some(cache_job_info) 
                    = pixbuf_cache.gx_pixbuf_cache_find_job_cache_unmut(page_index){
                        texture = cache_job_info.texture.clone();
                }
                page.gx_view_page_set_page(page_index, page_numbers,scale,rotation,
                    width,height,height_offset,texture);
            }
        }       
        self.queue_resize();
    }
    
    pub fn gx_view_area_set_device_scale(&self,device_scale:f32){
        self.imp().device_scale.replace(device_scale);
    }

}