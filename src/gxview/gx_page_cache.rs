
use crate::{gxdocument::{gx_mapping_list::GxMappingList, 
    gx_pdf_document::GxRectangle}, 
    utils::gx_jobs::GxJobPageDataFlags
};


#[derive(Default,Clone)]
pub struct GxPageCache {
    //pub parent: GObject,
    pub page_list: Vec<GxPageCacheData>,
    pub page_numbers: i32,
    // 当前范围
    pub start_page: i32,
    pub end_page: i32,
    pub flags: GxJobPageDataFlags,
}

impl GxPageCache {
    pub fn page_cache_new(page_numbers:i32) -> Self {
        let mut page_cache = GxPageCache::default();
        page_cache.page_numbers = page_numbers;
        page_cache.flags = GxJobPageDataFlags::default_flags();
        let cache_data = vec![GxPageCacheData::default();page_numbers as usize];
        page_cache.page_list = cache_data;
        page_cache
    }
    
    pub fn page_cache_get_flags(&self) -> GxJobPageDataFlags{
        let flags = self.flags;
        flags 
    }
    pub fn page_cache_set_page_range(&mut self,start:i32,end:i32){
        if self.page_cache_get_flags() ==  GxJobPageDataFlags::NONE{
            return;
        }
        for i in start..=end {
           self.page_cache_schedule_job_if_needed(i); 
        }
        self.start_page = start;
        self.end_page = end;
        //后面的是待修改实现的内容
        //i = 1;
        //pages_to_pre_cache = PRE_CACHE_SIZE * 2;
        //while ((start - i > 0) || (end + i < cache->n_pages)) {
        //        if (end + i < cache->n_pages) {
        //                ev_page_cache_schedule_job_if_needed (cache, end + i);
        //                if (--pages_to_pre_cache == 0)
        //                        break;
        //        }

        //        if (start - i > 0) {
        //                ev_page_cache_schedule_job_if_needed (cache, start - i);
        //                if (--pages_to_pre_cache == 0)
        //                        break;
        //        }
        //        i++;
        //}
    }

    //待完善,暂时可以先不管
    fn page_cache_schedule_job_if_needed(&self,_page_index: i32){
        //let data = &self.page_list;
        //let data = data.get(page_index as usize);
    }   


    pub async  fn page_cache_set_flags(& mut self,flags:GxJobPageDataFlags){
        if self.page_cache_get_flags() == flags{
            return;
        }
        //后面待实现
        /* Update the current range for new flags */
        //ev_page_cache_set_page_range (cache, cache->start_page, cache->end_page);
    }


}
// 页面缓存数据
#[derive(Default,Clone)]
pub struct GxPageCacheData {
    //pub job: Option<GxJob<'a>>,
    pub done: bool,
    pub dirty: bool,
    pub flags: GxJobPageDataFlags,
    pub link_mapping: Option<GxMappingList>,
    pub image_mapping: Option<GxMappingList>,
    pub form_field_mapping: Option<GxMappingList>,
    pub annot_mapping: Option<GxMappingList>,
    pub media_mapping: Option<GxMappingList>,
    //pub text_mapping: Option<Region>,
    pub text_layout: Option<Vec<GxRectangle>>,
    pub text_layout_length: usize,
    pub text: Option<String>,
    //pub text_attrs: Option<AttrList>,
    //pub text_log_attrs: Option<Vec<LogAttr>>,
    //pub text_log_attrs_length: usize,
}