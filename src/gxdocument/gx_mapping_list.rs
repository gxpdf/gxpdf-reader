
#[derive(Default,Clone)]
pub struct GxMappingList {
    // 页码
    pub page: u32,
    // 映射列表
    pub list: Vec<String>,
    // 引用计数,暂时先不管了
    //pub ref_count: AtomicI32,
}