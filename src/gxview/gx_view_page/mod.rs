//gx_view_page/mod.rs
pub mod imp;
pub mod gx_view_page_data;


use gtk::{
    Snapshot, gdk::{Rectangle, Texture, prelude::TextureExt}, 
    glib::{self, subclass::types::ObjectSubclassIsExt }, 
    graphene::Rect, prelude::{SnapshotExt, WidgetExt}
};

use crate::{gxview::gx_document_model::GxDocumentModel};



glib::wrapper! {
    pub struct GxViewPage(ObjectSubclass<imp::GxViewPage>)
    @extends gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for GxViewPage{
    fn default() -> Self {
        glib::Object::new()
    }
}

impl GxViewPage{
    pub fn new(_document_model:GxDocumentModel,texture:Option<Texture>) -> GxViewPage{
        let gx_view_page = GxViewPage::default();
        gx_view_page.imp().texture.replace(texture);
        //gx_view_page.imp().document_model = Some(Arc::new(document_model));
        gx_view_page
    }

    pub fn gx_view_page_get_page_index(&self) -> i32{
        *self.imp().page_index.borrow()
    }

    pub fn gx_view_page_set_page(&self,page_index:i32,
        page_numbers:i32,scale:f32,rotation:i32,
        width:f32,height:f32,height_offset:f32,
        texture:Option<Texture>){
        if page_index >= 0 && page_index < page_numbers {
            self.imp().page_index.replace(page_index);
        }else{
            self.imp().page_index.replace(-1);
        }

        self.imp().scale.replace(scale);
        self.imp().rotation.replace(rotation);
        self.imp().width.replace(width);
        self.imp().height.replace(height);
        self.imp().height_offset.replace(height_offset);
        self.imp().texture.replace(texture);
    }

    //pub fn gx_view_page_queue_size(&self,page_index:i32,
    //    page_numbers:i32,scale:f32,rotation:i32,
    //    width:f32,height:f32,height_offset:f32,
    //    texture:Option<Texture>){
    //    if *self.imp().page_index.borrow() == page_index{
    //        return;
    //    }
    //    if page_index >= 0 && page_index < page_numbers {
    //        self.imp().page_index.replace(page_index);
    //    }else{
    //        self.imp().page_index.replace(-1);
    //    }

    //    self.imp().scale.replace(scale);
    //    self.imp().rotation.replace(rotation);
    //    self.imp().width.replace(width);
    //    self.imp().height.replace(height);
    //    self.imp().height_offset.replace(height_offset);
    //    self.imp().texture.replace(texture);

    //    self.queue_resize();
    //}


    pub fn gx_view_page_get_page_extents(&self,gx_view_area_width:i32,
        gx_view_spacing:f32) -> Rectangle{
        let (scro_width,scro_height) 
            = self.gx_view_page_get_scaled_and_rotation_size(); 
        let mut page_area = Rectangle::new(0,0,scro_width,scro_height);
        
        let x =  gx_view_spacing;
        let left_width = 0.0 as f32;
        let x = x + left_width.max(gx_view_area_width as f32 - 
                (scro_width as f32 + gx_view_spacing * 2.0)) / 2.0;
        page_area.set_x(x as i32);

        let mut scaled_height_offset = self.gx_view_page_get_scaled_and_y_offset();
        let page_index = *self.imp().page_index.borrow();
        scaled_height_offset 
            = scaled_height_offset + (page_index + 1) as f32 * gx_view_spacing;
        page_area.set_y(scaled_height_offset as i32); 
        
        page_area
    }

    pub fn gx_view_page_set_texture(&self,texture:Option<Texture>){
        self.imp().texture.replace(texture);
        self.queue_draw();
    }

    fn gx_view_page_get_scaled_and_rotation_size(&self) -> (i32,i32){
        let scale = *self.imp().scale.borrow();
//        let scaled_width = *self.imp().width.borrow() *
//            scale + 0.5;
        let scaled_width = *self.imp().width.borrow() *
            scale ;
 
        let scaled_width = scaled_width as i32;

//        let scaled_height = *self.imp().height.borrow() *
//            scale + 0.5;

        let scaled_height = *self.imp().height.borrow() *
            scale ;
 
        let scaled_height = scaled_height as i32;
        
        let rotation = *self.imp().rotation.borrow();
        if rotation == 0 || rotation == 180{
            return (scaled_width,scaled_height);
        }else{
            return (scaled_height,scaled_width);
        }
    }

    #[allow(unused)]
    fn gx_view_page_get_scaled_and_rotation_max_size(&self,max_width:f32,
        max_height: f32) -> (i32,i32){
        let scale = *self.imp().scale.borrow();
        let scaled_width = max_width*
            scale + 0.5;
        let scaled_width = scaled_width as i32;

        let scaled_height = max_height *
            scale + 0.5;
        let scaled_height = scaled_height as i32;
        
        let rotation = *self.imp().rotation.borrow();
        if rotation == 0 || rotation == 180{
            return (scaled_width,scaled_height);
        }else{
            return (scaled_height,scaled_width);
        }
    }

    //这个时候rotation已经定死了，如果rotation更改了，那结构体中的原始值要重新获取
    fn gx_view_page_get_scaled_and_y_offset(&self) -> f32{
        let scaled_height_offset = *self.imp().height_offset.borrow() *
            *self.imp().scale.borrow() + 0.5;
        scaled_height_offset
    }


    fn gx_view_page_draw_surface(&self,snapshot:&Snapshot,
        texture:&Texture,area:&mut Rect){
        let mut scale_texture = false;
        if texture.height() == area.height().floor() as i32{
            scale_texture = true;
        }


        //这4行让area和texture的大小一致，从而显示pdf是清晰的
        let width = area.width() - 4.0;
        let height = area.height() - 4.0;
        let x = area.x();
        let y = area.y();
        let area = Rect::new(x,y,width,height);
        //这三行才让显示清晰了
        //let scale_x = texture.width() as f32 / area.width() as f32;
        //let scale_y = texture.height() as f32 / area.height() as f32;
        //let area = area.scale(scale_x, scale_y);

        snapshot.save();
        if scale_texture{
            snapshot.append_texture(texture, &area);
            //snapshot.append_scaled_texture(texture, ScalingFilter::Nearest, &area);//如果gtk4-rs用到0.10.0就可以改用这行了
        }else{
            snapshot.append_texture(texture, &area);
        }
        snapshot.restore();
    }

}