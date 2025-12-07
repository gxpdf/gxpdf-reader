use std::collections::HashMap;
use crate::gxview::gx_view::DrawPage;

#[derive(Default,Clone,Debug)]
pub struct GxSurfacesCache {
    //parent: GObject,
    // 我们保留指向包含视图的链接，仅用于样式信息
    //pub view: Option<GxView<'a>>,//这个在c语言中是GTKWIDGET，我直接改为了GxView，也不知道有没有其他地方要这么用的
    //pub document: Arc<Mutex<Option<GxDocument<'a>>>>,
    pub hash_draw_pages: HashMap<i32,DrawPage>,//这个存储了所有已经render的pages
    //DEFAULT_PIXBUF_CACHE_SIZE: usize = 52428800;
}

