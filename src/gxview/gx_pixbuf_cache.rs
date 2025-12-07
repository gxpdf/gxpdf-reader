use std::{collections::HashMap, sync::Arc};
use gtk::{cairo::{Format, Region}, gdk::Texture};
use relm4::prelude::*;
use tokio::{sync::{mpsc::{self,}, Mutex }};
use uuid::Uuid;
use crate::{components::gx_window::MAX_PRELOADED_PAGES, gxdocument::gx_pdf_document::GxPageSize, 
    gxview::{gx_view::{DrawPage, GxView, GxViewMsg}}, 
    utils::{gx_job_scheduler::{AMGxJobOption, GxJobPriority, GxScheduler}, 
    gx_jobs::{GxJobClassAct, GxJobRenderPdf, GxSurfaceData}}};

#[derive(Default,Clone,Debug)]
pub struct GxPixbufCache {
    //parent: GObject,
    // 我们保留指向包含视图的链接，仅用于样式信息
    pub end_page: i32,
    //DEFAULT_PIXBUF_CACHE_SIZE: usize = 52428800;
    pub job_list: HashMap<i32,CacheJobInfo>,//Option<CacheJobInfo<'a>>,
    pub job_list_len: u32,
    pub job_list_next: HashMap<i32,CacheJobInfo>,//Option<CacheJobInfo<'a>>,
    pub job_list_prev: HashMap<i32,CacheJobInfo>,//Option<CacheJobInfo<'a>>,
    pub max_size: usize,
    pub page_sizes: Vec<GxPageSize>,
    // preload_cache_size是我们缓存的当前可见区域之前的页数。
    // 通常为1，但在双页情况下可能为2。
    pub preload_cache_size: i32,
    pub scroll_direction: ScrollDirection,
    pub start_page: i32,
    pub uniform: bool,
    pub uniform_width: f32,
    pub uniform_height: f32,
    pub scale:f32,//用于判断用户是否已经缩放了
}

impl GxPixbufCache {
    pub fn gx_pixbuf_cache_add_job(&mut self,page:i32,rotation:i32,
        width:i32,height:i32,priority:GxJobPriority,gpc_list_flag:GPCListFlag,
        sender: AsyncComponentSender<GxView>,
        scale:f32,device_scale:f32,
        job_scheduler:Option<GxScheduler>,
        //region:Option<Region>){
    ){
            //这个函数还未实现，针对hdpi
          	//job_info->device_scale = get_device_scale (pixbuf_cache);
            let (tx,mut rx) = mpsc::channel::<DrawPage>(100);
            let render_job = GxJobRenderPdf::new(page, rotation, 
                scale * device_scale ,
                width * device_scale as i32, 
                height * device_scale as i32,
                tx,
            );
            let boxed = Box::new(render_job.clone())  
                as Box<dyn GxJobClassAct>;
            let job_render_pdf = Arc::new(Mutex::new(boxed));
            let job_render_pdf_clone = job_render_pdf.clone();
            let cache_job_info = CacheJobInfo{
                page_index:page,
                job:Some(job_render_pdf_clone),
                scale:scale * device_scale,
                device_scale:device_scale,
                region:None,
                page_ready:false,
                pending_job:None,
                pending_priority:GxJobPriority::None,
                texture:None,
            };
            match gpc_list_flag{
                GPCListFlag::List => {
                    //self.job_list.lock().await.push(job_render_pdf_clone);
                    self.job_list.insert(page,cache_job_info);
                }
                GPCListFlag::Prev => {
                    self.job_list_prev.insert(page,cache_job_info);
                }
                GPCListFlag::Next => {
                    self.job_list_next.insert(page,cache_job_info);
                }
            }
            tokio::spawn(async move {
                if let Some(job_scheduler) = job_scheduler{
                    job_scheduler.scheduler_push_job(job_render_pdf, 
                        priority.clone()).await;
                }
            });
            tokio::spawn(async move{
                let job_receiver_recv = rx.recv().await;
                if let Some(job_receiver_result) = job_receiver_recv{
                    sender.input(GxViewMsg::RenderFinished(job_receiver_result));                        
                }               
            });
        }

    //基本能用
    fn gx_pixbuf_cache_add_job_if_needed(&mut self,page:i32,rotation:i32,
        priority:GxJobPriority,gpc_list_flag:GPCListFlag,
        sender: AsyncComponentSender<GxView>,
        scale:f32,device_scale:f32,
        job_scheduler:Option<GxScheduler>,
        ){
        //这个get_device_scale待实现
    	//let device_scale = get_device_scale (self);
        //let device_scale = 1.0;
        let (target_width,target_height) = 
            self.gx_pixbuf_cache_get_page_size_for_scale_and_rotation(page,rotation,scale);
        self.gx_pixbuf_cache_add_job(page, rotation, target_width as i32, 
            target_height as i32, priority,gpc_list_flag,sender,
            scale,device_scale,job_scheduler);
        
    }

    //基本能用,未实现上下滚动
    fn gx_pixbuf_cache_add_jobs_if_needed(&mut self,
        rotation:i32,
        sender: AsyncComponentSender<GxView>,
        scale:f32,device_scale:f32,
        job_scheduler:Option<GxScheduler>,
        ){
        for i in 0..self.job_list_len as i32{
            let page_index = self.start_page + i;
            let temp_info = self.job_list.get(&page_index);

            if temp_info.is_none() {
                self.gx_pixbuf_cache_add_job_if_needed(page_index,
                    rotation,GxJobPriority::Urgent,
                    GPCListFlag::List,
                    sender.clone(),scale,device_scale,job_scheduler.clone());               
            }
        }
    }

    pub fn gx_pixbuf_cache_add_next_jobs_if_needed(&mut self,
        rotation:i32,scale:f32,device_scale:f32,
        sender: AsyncComponentSender<GxView>,
        page_numbers:i32,job_scheduler:Option<GxScheduler>){
        for i in 0..self.preload_cache_size as i32{
            let page_index = self.start_page + self.job_list_len as i32 + i;
            if page_index >=0 && page_index < page_numbers{
                let temp_info = self.job_list_next.get(&page_index);
                if temp_info.is_none(){
                    self.gx_pixbuf_cache_add_job_if_needed(page_index,
                        rotation,GxJobPriority::Low,
                        GPCListFlag::Next,sender.clone(),
                        scale,device_scale,job_scheduler.clone());                   
                }
            } 
        }
    }


    pub fn gx_pixbuf_cache_add_prev_jobs_if_needed(&mut self,
        rotation:i32,scale:f32,device_scale:f32,
        sender: AsyncComponentSender<GxView>,
        page_numbers:i32,job_scheduler:Option<GxScheduler>){
        for i in 0..self.preload_cache_size as i32{
            let page_index = self.start_page - (i + 1) ;
            if page_index >=0 && page_index < page_numbers{
                let temp_info = self.job_list_prev.get(&page_index);
                if temp_info.is_none(){
                    self.gx_pixbuf_cache_add_job_if_needed(page_index,
                        rotation,GxJobPriority::Low,
                        GPCListFlag::Prev,sender.clone(),
                        scale,device_scale,job_scheduler.clone());                   
                }
            } 
        }
    }

    #[allow(unused)]
    fn gx_pixbuf_cache_check_job_size_and_unred(&mut self,
        job_info:&mut CacheJobInfo,scale:f32){
        if job_info.job.is_none(){
            return;
        }
    }

    pub fn gx_pixbuf_cache_clear_job_sizes(&mut self,_scale:f32){
        self.job_list.clear();
        self.job_list_prev.clear();
        self.job_list_next.clear();
    }
    ////已完善
    //async fn gx_pixbuf_cache_dispose_job_info(&self,mut job_info: CacheJobInfo){
    //    //job_info.job.as_ref().unwrap().cancel_token.cancel();
    //    job_info.job.as_ref().unwrap().cancel();
    //    let _ = self.job_scheduler.as_ref().unwrap().scheduler_remove_job_by_key(
    //        job_info.job.as_ref().unwrap().priority.clone(),
    //        job_info.job.as_ref().unwrap().key).await;
    //    job_info.job = None;
    //    //job_info.surface = None;
    //    //job_info.region = None;    
    //    job_info.page_ready = false;
    //    //job_info.job = GxJobRenderPdf::default();//其实这条还需不需要？
    //}

    //evince中是找到对应页码的cachejobinfo，并返回 
    pub fn gx_pixbuf_cache_find_job_cache(&mut self,page:i32) -> Option<&mut CacheJobInfo>{
        if page < (self.start_page - self.preload_cache_size) 
            || page > (self.end_page + self.preload_cache_size){
                return None;
            }
        
        if page < self.start_page{
            let page_offset = page - (self.start_page - self.preload_cache_size);
            assert!(page_offset >= 0 
                && page_offset < self.preload_cache_size);
            if page_offset < self.job_list_prev.len() as i32{
                return self.job_list_prev.get_mut(&page);
            }else{
                return None;
            }
        }

        if page > self.end_page{
            let page_offset = page - (self.end_page + 1);
            assert!(page_offset >= 0 
                && page_offset < self.preload_cache_size);
            if page_offset < self.job_list_next.len() as i32{
                return self.job_list_next.get_mut(&page);
            }else{
                return None;
            }
        }
        let page_offset = page - self.start_page;
        assert!(page_offset >= 0 
            && page_offset < self.end_page - self.start_page + 1);
        if page_offset < self.job_list.len() as i32{
            return self.job_list.get_mut(&page);
        }else{
            return None;
        }
    }

    //evince中是找到对应页码的cachejobinfo，并返回 
    pub fn gx_pixbuf_cache_find_job_cache_unmut(&self,page:i32) -> Option<&CacheJobInfo>{
        if page < (self.start_page - self.preload_cache_size) 
            || page > (self.end_page + self.preload_cache_size){
                return None;
            }
        
        if page < self.start_page{
            let page_offset = page - (self.start_page - self.preload_cache_size);
            assert!(page_offset >= 0 
                && page_offset < self.preload_cache_size);
            if page_offset < self.job_list_prev.len() as i32{
                return self.job_list_prev.get(&page);
            }else{
                return None;
            }
        }

        if page > self.end_page{
            let page_offset = page - (self.end_page + 1);
            assert!(page_offset >= 0 
                && page_offset < self.preload_cache_size);
            if page_offset < self.job_list_next.len() as i32{
                return self.job_list_next.get(&page);
            }else{
                return None;
            }
        }
        let page_offset = page - self.start_page;
        assert!(page_offset >= 0 
            && page_offset < self.end_page - self.start_page + 1);
        if page_offset < self.job_list.len() as i32{
            return self.job_list.get(&page);
        }else{
            return None;
        }
    }


    pub fn gx_pixbuf_cache_get_page_size(&self,page_index:i32) -> (f32,f32){
        if self.uniform{
            return (self.uniform_width,self.uniform_height);
        }
        else{
            let page_width = self.page_sizes[page_index as usize].width;
            let page_height = self.page_sizes[page_index as usize].height;
            return (page_width,page_height);
        }
    }

    //已完善
    fn gx_pixbuf_cache_get_page_size_area(&mut self,
        page_index:i32,rotation:i32,scale:f32) -> i32{
        let (width,height) 
            = self.gx_pixbuf_cache_get_page_size_for_scale_and_rotation(page_index,
            rotation,scale);
        let size = height as i32  * Format::stride_for_width(Format::ARgb32, width as u32).unwrap();
        size
    }
    
    pub fn gx_pixbuf_cache_get_page_size_for_scale_and_rotation(&self,
        page_index:i32,rotation:i32,scale:f32) -> (f32,f32) {
        let (page_width,page_height) = self.gx_pixbuf_cache_get_page_size(page_index);
        let mut width = page_width * scale;
        let mut height = page_height * scale ;
        if rotation == 90 || rotation == 270{
            height = page_width * scale ;
            width = page_height * scale ;
        }
        (width,height)       
    }


    //已完善
    fn gx_pixbuf_cache_get_preload_size(&mut self,
        start_page: i32,end_page: i32,rotation:i32,
        scale:f32,page_numbers:i32) -> i32{
        let mut new_preload_cache_size = 0;
        let mut range_size = 0;
        //let page_numbers = self.document_model.page_numbers;


        for i in start_page..=end_page {
            range_size += self.gx_pixbuf_cache_get_page_size_area(i as i32,rotation,scale);
        }

        if range_size >= self.max_size as i32 {
            return new_preload_cache_size;
        }
        
        let mut i = 1;
        while (start_page -i > 0 || end_page + i < page_numbers as i32) 
            && new_preload_cache_size < MAX_PRELOADED_PAGES {
            let mut updated = false;
            let mut page_size:i32;  
            if end_page + i < page_numbers  {
                page_size = self.gx_pixbuf_cache_get_page_size_area(end_page + i,
                   rotation,scale);
                if page_size + range_size <= self.max_size as i32 {
                    range_size += page_size;
                    new_preload_cache_size += 1;
                    updated = true;
                }
                else{
                    break;
                }
            }
            if start_page - i > 0 {
                page_size = self.gx_pixbuf_cache_get_page_size_area(start_page - i,
                    rotation,scale);
                if page_size + range_size <= self.max_size as i32 {
                    range_size += page_size;
                    if !updated {
                        new_preload_cache_size += 1;
                    }
                }
                else{
                    break;
                }
            }
            i += 1;
        }

        new_preload_cache_size
    }


    //已完善
    fn gx_pixbuf_cache_get_scroll_direction(&self, start_page: i32, end_page: i32) -> ScrollDirection{
        let scroll_direction = self.scroll_direction.clone();
        if start_page < self.start_page {
            return ScrollDirection::Up;
        }
        if start_page > self.start_page {
            return ScrollDirection::Down;
        }
        if end_page > self.end_page {
            return ScrollDirection::Down;
        }
        if end_page < self.end_page {
            return ScrollDirection::Up;
        }
        scroll_direction
    }

       
    //待完善 ,evince还有段region相关的代码未处理
    pub fn gx_pixbuf_cache_get_surface(&mut self, page: i32) -> Option<GxSurfaceData> {
       
        // 查找缓存的任务信息
        if let Some(job_info) 
            = self.gx_pixbuf_cache_find_job_cache(page){
            let job = job_info.job.clone();
            if let Some(job_render) = job{
                let job_render= job_render.try_lock();
                if let Ok(job_render) = job_render{
                    let job_render 
                        = job_render.as_any().downcast_ref::<GxJobRenderPdf>().unwrap(); 
                    if job_render.finished.get(){
                        return job_render.surface.clone();
                    }               
                }
            }
        }
        None
    }

    //待完善
    fn gx_pixbuf_cache_move_one_job(&self, 
        job_key:Uuid,old_priority:GxJobPriority,
        page:i32,am_job_clone:CacheJobInfo,
        new_preload_cache_size:i32,start_page:i32,end_page:i32,
        new_prev_job:&mut HashMap<i32,CacheJobInfo>, 
        new_job_list:&mut HashMap<i32,CacheJobInfo>,
        new_next_job:&mut HashMap<i32,CacheJobInfo>,
        job_scheduler:Option<GxScheduler>){

        let new_priority:GxJobPriority; 
        //prev_job
        if page < start_page{
            let page_offset = page - (start_page - new_preload_cache_size);
            assert!(page_offset >= 0 && page_offset < new_preload_cache_size);
            new_priority = GxJobPriority::Low;
            new_prev_job.insert(page,am_job_clone);
        }else if page > end_page{//next job
            let page_offset = page - (end_page + 1);
            assert!(page_offset >= 0 && page_offset < new_preload_cache_size);
            new_priority = GxJobPriority::Low;
            new_next_job.insert(page,am_job_clone);
        }else{//job_list
            let page_offset = page - start_page;
            assert!(page_offset >= 0 && page_offset <= (end_page - start_page + 1));
            new_priority = GxJobPriority::Urgent;
            new_job_list.insert(page,am_job_clone);
        }
        if new_priority != old_priority{
            if let Some(job_scheduler) = job_scheduler{
                job_scheduler.scheduler_update_job(job_key,old_priority,new_priority);           
            }           
        }           
    }



    //已完善
    pub fn gx_pixbuf_cache_new(
		max_size: usize,page_sizes:Vec<GxPageSize>,
        uniform:bool,uniform_width:f32,uniform_height:f32,
        _job_scheduler:Option<GxScheduler>,_scale:f32,
        _device_scale:f32) -> Self{
        let mut pixbuf_cache = GxPixbufCache::default();
        pixbuf_cache.start_page = -1;
        pixbuf_cache.end_page = -1;
        //pixbuf_cache.view = Some(view);
        //let document = model.document.clone();
        //pixbuf_cache.document = document;
        pixbuf_cache.max_size = max_size;
        pixbuf_cache.page_sizes = page_sizes;
        pixbuf_cache.uniform = uniform;
        pixbuf_cache.uniform_width = uniform_width;
        pixbuf_cache.uniform_height = uniform_height;
        pixbuf_cache
    }

    //基本能用
    pub fn gx_pixbuf_cache_set_page_range(&mut self, start_page: i32, 
        end_page: i32,page_numbers:i32,scale:f32,device_scale:f32,
        rotation:i32,
        sender: AsyncComponentSender<GxView> ,
        job_scheduler:Option<GxScheduler>
        ){
        if start_page < 0 || start_page >= page_numbers{
            return;
        }
        if end_page < 0 || end_page >= page_numbers{
            return;
        }
        if start_page > end_page{
            return;
        }
        self.scroll_direction = self.gx_pixbuf_cache_get_scroll_direction(start_page, end_page);
        let ret 
            = self.gx_pixbuf_cache_update_range(start_page, end_page,rotation,scale,page_numbers,job_scheduler.clone());
        if ret{
            self.gx_pixbuf_cache_add_jobs_if_needed(rotation,sender.clone(),scale,device_scale,job_scheduler);
        }
    }


    //已完善 
    fn gx_pixbuf_cache_update_range(&mut self, start_page: i32,end_page: i32,
        rotation:i32,scale:f32,page_numbers:i32,job_scheduler:Option<GxScheduler>) -> bool{
        let new_preload_cache_size 
            = self.gx_pixbuf_cache_get_preload_size (start_page,end_page,
            rotation,scale,page_numbers);

        if self.start_page == start_page &&
            self.end_page == end_page &&
            self.preload_cache_size == new_preload_cache_size &&
            self.scale == scale{
            return false;
        }else{
            self.scale = scale;
        }
	    let new_job_list_len = (end_page - start_page) + 1;
        let mut new_job_list:HashMap<i32,CacheJobInfo> = HashMap::new();
        let mut new_prev_job:HashMap<i32,CacheJobInfo> = HashMap::new();
        let mut new_next_job:HashMap<i32,CacheJobInfo> = HashMap::new();
        //处理上一次的prev_job
        let mut job_list_prev = self.job_list_prev.clone();
        for (_page_index,am_job) in job_list_prev.drain(){
            let am_job_clone = am_job.clone();
            if let Some(am_job) = am_job.job{
                let am_job = am_job.try_lock();
                if let Ok(mut am_job) 
                    = am_job{
                    let render_job 
                        = am_job.as_any_mut().
                        downcast_mut::<GxJobRenderPdf>().unwrap();
                    if !render_job.finished.get(){
                        if render_job.page_index < start_page - new_preload_cache_size 
                            || render_job.page_index > end_page + new_preload_cache_size{
                                    render_job.cancel();
                        }else{
                            let job_key = render_job.get_key();
                            let old_priority = render_job.get_priority();
                            self.gx_pixbuf_cache_move_one_job(job_key,
                            old_priority, 
                            render_job.page_index,am_job_clone, 
                            new_preload_cache_size, 
                            start_page, end_page,
                            &mut new_prev_job, &mut new_job_list, 
                            &mut new_next_job,job_scheduler.clone());
                        }
                    }
                }                       
            }
        }           
            let mut job_list = self.job_list.clone();
            for (_page_index,am_job) in job_list.drain(){
                let am_job_clone = am_job.clone();
                if let Some(am_job) = am_job.job{
                    let am_job = am_job.try_lock();
                    if let Ok(mut am_job) = am_job{
                            let render_job 
                                = am_job.as_any_mut().downcast_mut::<GxJobRenderPdf>().unwrap();
                            if !render_job.finished.get(){
                                if render_job.page_index < start_page - new_preload_cache_size 
                                    || render_job.page_index > end_page + new_preload_cache_size{
                                            render_job.cancel();
                                }else{
                                    let job_key = render_job.get_key();
                                    let old_priority = render_job.get_priority();
                                    self.gx_pixbuf_cache_move_one_job(job_key,old_priority, 
                                    render_job.page_index,am_job_clone, 
                                    new_preload_cache_size,start_page, end_page,
                                    &mut new_prev_job, &mut new_job_list, 
                                    &mut new_next_job,job_scheduler.clone());
                                }
                            }
                    }                   
                }
                
            }
            let mut job_list_next = self.job_list_next.clone();
            for (_page_index,am_job) in job_list_next.drain(){
                let am_job_clone = am_job.clone();
                if let Some(am_job) = am_job.job{
                    let am_job = am_job.try_lock();
                    if let Ok(mut am_job) = am_job{
                            let render_job 
                                = am_job.as_any_mut().downcast_mut::<GxJobRenderPdf>().unwrap();
                            if !render_job.finished.get(){
                                if render_job.page_index < start_page - new_preload_cache_size 
                                    || render_job.page_index > end_page + new_preload_cache_size{
                                            render_job.cancel();
                                }else{
                                    let job_key = render_job.get_key();
                                    let old_priority = render_job.get_priority();
                                    self.gx_pixbuf_cache_move_one_job(job_key,old_priority, 
                                    render_job.page_index,am_job_clone,
                                    new_preload_cache_size, start_page, end_page,
                                    &mut new_prev_job, &mut new_job_list, 
                                    &mut new_next_job,job_scheduler.clone());
                                }
                            }
                    }                   
                }
            }
        self.preload_cache_size = new_preload_cache_size;
        self.job_list_len = new_job_list_len as u32;
        self.job_list = new_job_list;
        self.job_list_prev = new_prev_job;
        self.job_list_next = new_next_job;

        self.start_page = start_page;
        self.end_page = end_page;
        true
    }
    
}

#[derive(Default,Clone,Eq,PartialEq,Debug)]
pub enum ScrollDirection {
    #[default]
    Down,
    Up,
}

#[derive(Default,Clone,Eq,PartialEq,Debug)]
pub enum GPCListFlag{
    #[default]
    List,
    Prev,
    Next,
}




#[derive(Default,Clone,Debug)]
pub struct CacheJobInfo {
    pub job: Option<AMGxJobOption>,
    pub page_ready: bool,
    pub pending_job:Option<AMGxJobOption>,
    pub pending_priority: GxJobPriority,
    pub region: Option<Region>,//需要重新drawn的region
    pub texture: Option<Texture>,
    pub device_scale: f32,
    pub page_index:i32,
    pub scale:f32,
}

impl CacheJobInfo{
}