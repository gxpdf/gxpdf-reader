use std::sync::{Arc,  Once};
use std::collections::VecDeque;
use std::usize;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::utils::gx_jobs::{GxJobRenderPdf, GxSurfaceData};
use super::gx_jobs::{GxJobClassAct};

// 优先级枚举
#[derive(Default,Clone,PartialEq,Debug)]
pub enum GxJobPriority {
    #[default]
    Urgent = 0,    /* Rendering current page range */
    High = 1,      /* Rendering current thumbnail range */
    Low = 2,       /* Rendering pages not in current range */
    None = 3,      /* Any other job: load, save, print, ... */
    NPriorities = 4,
}
type GxJobOption = Box<dyn GxJobClassAct>;
pub type AMGxJobOption = Arc<Mutex<GxJobOption>>;
//type AMGxSchedulerJob = Arc<Mutex<GxSchedulerJob>>;
//#[derive(Default)]
//pub struct GxSchedulerJob {
//    gx_job: Arc<Mutex<GxJobOption>>,
//    //priority: GxJobPriority,
//    //key:Uuid,//本数据结构在job_list中的key值,暂时也用不上
//}
//impl Clone for GxSchedulerJob {
//    fn clone(&self) -> Self {
//        GxSchedulerJob {
//            gx_job: Arc::clone(&self.gx_job),
//            //priority: self.priority.clone(), // 假设 GxJobPriority 实现了 Clone
//            //key: self.key.clone(), // Uuid 实现了 Clone
//        }
//    }
//}
#[derive(Debug)]
pub struct GxScheduler {
    //JOB_LIST是全局任务双链表变量，存储所有的任务
    //似乎没用上？先注释掉了
    //job_list:Arc<Mutex<HashMap<Uuid,AMGxSchedulerJob>>>,
    //running_job: AMGxSchedulerJob,
    job_added_tx: Sender<GxJobPriority>,//准备就按GxJobPriority分类了   
    //pub job_list:Arc<Mutex<HashMap<i32,AMGxSchedulerJob>>>,//i32是page_index
    //pub page_index_list:Arc<Mutex<HashSet<i32>>>,//有些时候使用job_list会直接把其他都卡死，所以增加了这个
    job_queue: Arc<Mutex<Vec<VecDeque<AMGxJobOption>>>> ,
    pub job_vec: Arc<Mutex<Vec<AMGxJobOption>>>,
    //用于等待新job被添加到链表中？
    //notify:Arc<Notify>,
    //不用notify了，改用mpsc算了
    once:Arc<Once>,
    //渲染完成的数据就临时保存到rendered_surface中，然后保存到gx_pixbuf_cache的hash_draw_pages中
    //需要的时候就到hash_draw_pages中调取
    pub rendered_surfaces:Arc<Mutex<Vec<Option<GxSurfaceData>>>>,
}

impl Clone for GxScheduler {
    fn clone(&self) -> Self {
        GxScheduler {
            //job_list: Arc::clone(&self.job_list),
            //running_job: Arc::clone(&self.running_job),
            job_added_tx: self.job_added_tx.clone(),
            job_queue: Arc::clone(&self.job_queue),
            job_vec: Arc::clone(&self.job_vec),
            rendered_surfaces:Arc::clone(&self.rendered_surfaces),
            //notify: Arc::clone(&self.notify),
            once: Arc::clone(&self.once), 
        }
    }
}

impl GxScheduler{
    //pub async fn scheduler_batch_cancel_render_job(&self,
    //    mut start_page:i32,mut end_page:i32,preload_cache_size:i32,
    //    page_numbers:i32){
    //        start_page = start_page - preload_cache_size;
    //        if start_page < 0{
    //            start_page = 0 ;
    //        }

    //        end_page = end_page + preload_cache_size;
    //        if end_page >= page_numbers{
    //            end_page = page_numbers - 1;
    //        }

    //        for priority in GxJobPriority::Urgent as usize.. GxJobPriority::NPriorities as usize{
    //            let vecdqueue 
    //                = &mut (self.job_queue.lock().await[priority]);
    //            for page_index in start_page..end_page{
    //                if job_list.contains_key(&page_index){
    //                    let job = job_list.remove(&page_index).unwrap();
    //                    let _ = page_index_list.remove(&page_index);
    //                    let gx_job = job.lock().await;
    //                }
    //            }
    //        }

    //        //for page_index in start_page..end_page{
    //        //    if job_list.contains_key(&page_index){
    //        //        let job = job_list.remove(&page_index).unwrap();
    //        //        let _ = page_index_list.remove(&page_index);
    //        //        let gx_job = job.lock().await;
    //        //        if let Some(gx_job) = gx_job.gx_job.lock().await.as_ref(){
    //        //            gx_job.cancel();
    //        //        };
    //        //    }
    //        //}
    //}
    //获取某个等级中某个任务,特征是key
    pub async fn scheduler_get_job_by_key(&self,priority:GxJobPriority,key:Uuid) -> Option<AMGxJobOption>{
        let priority_clone= priority.clone() as usize;
        let queue = &mut (self.job_queue.lock().await[priority_clone]);
        let mut job = None;
        for _job in queue.iter_mut(){
            let gx_job = _job.lock().await;
            //if let Some(gx_job) = _job.clone().lock().await.as_ref(){
                if gx_job.get_key() == key{
                    job = Some(_job.clone());
                    break;
                }
            //}
        }
        //for i in 0..queue.len(){
        //    if let Some(gx_job) = queue[i].lock().await.gx_job.clone().lock().await.as_ref(){
        //        if gx_job.get_key() == key{
        //            job = Some(queue[i].clone());
        //            break;
        //        }
        //    }
        //}
        job
    }


    //获取要执行的任务，这个任务应该是当前序列中存储的最高优先级的第1个
    async fn scheduler_get_next_unlocked(&self) -> Option<AMGxJobOption> {
        let mut job = None;
        for i in GxJobPriority::Urgent as usize.. GxJobPriority::NPriorities as usize{
            job = self.job_queue.lock().await[i].pop_front();
            if job.is_some(){
                break;
            }
        }
        //if cfg!(debug_assertions) {
        //    if let Some(job) = job.as_ref(){
        //        if let Some(gx_job) 
        //            = job.clone().lock().await.as_ref(){
        //            //println!("DEBUG JOBS in  job_queue_get_next_unlocked {}",gx_job.get_type_name());
        //        }
        //        else{
        //            //println!("DEBUG JOBS in  job_queue_get_next_unlocked,No jobs is queue");
        //        }
        //    }
        //}
        job

    }
    
    //这个也是方法，但必须是Arc<Self>类型的自己才能调用
    fn scheduler_init(self:  Arc<Self>){
        let self_clone = Arc::clone(&self);
        //task::spawn 与tokio::spawn好像是一样的
        tokio::spawn(async move {
            self_clone.scheduler_thread_proxy().await;
        });
    }

//    pub async fn scheduler_list_push(&self,page_index:i32,job:AMGxSchedulerJob){
//        let list = &mut (self.job_list.lock().await);
//        let mut page_index_list = self.page_index_list.lock().await;
//        list.insert(page_index,job);
//        page_index_list.insert(page_index);
//        
//    }

    pub async fn scheduler_new() -> Self{
        //let job_list =  Arc::new(Mutex::new(HashMap::new()));
        //let running_job = Arc::new(Mutex::new(GxSchedulerJob::default()));
        let queue_urgent = VecDeque::new();
        let queue_high = VecDeque::new();
        let queue_low = VecDeque::new();
        let queue_none = VecDeque::new();
        let job_queue =Arc::new(Mutex::new(vec![
            queue_urgent,queue_high,queue_low,queue_none]));
        let job_vec:Arc<Mutex<Vec<AMGxJobOption>>> = Arc::new(Mutex::new(Vec::new()));
        //let notify = Arc::new(Mutex::new(Notify::new()));
        //let once = Arc::new(Mutex::new(Once::new()));
        //let notify = Arc::new(Notify::new());
        let (job_added_tx ,_job_added_rx)
            = broadcast::channel::<GxJobPriority>(10);
        let once = Arc::new(Once::new());
        let rendered_surfaces = Arc::new(Mutex::new(vec![]));       
    
        //let new_scheduler = GxScheduler { job_list,running_job,job_queue,notify,once };
        let new_scheduler = GxScheduler {job_queue,job_added_tx,job_vec,
            once,rendered_surfaces };

        //这后面两行代码evince本来是放在了 job_scheduler_push_job函数中,但不好弄，
        //还是放到这个地方一次性初始化了
        let arc_scheduler = Arc::new(new_scheduler.clone());
        arc_scheduler.clone().once.call_once(|| {
            arc_scheduler.scheduler_init();
        });
        new_scheduler
    }   

    pub async fn scheduler_push_job(&self,gx_job:Arc<Mutex<GxJobOption>>,priority:GxJobPriority)  {
        let _gx_job_clone = gx_job.clone();
        let priority_clone = priority.clone();
        let _priority_clone= priority_clone as usize;

        //if cfg!(debug_assertions) {
        //    if let Some(gx_job_temp) = gx_job_clone.lock().await.as_ref(){
        //        //println!("DEBUG JOBS in   job_scheduler_push_job{},priority  {}",gx_job_temp.get_type_name(),priority_clone );
        //    }
        //}

        gx_job.lock().await.as_mut().set_priority(priority.clone());
        let job_clone = gx_job.clone();
        //let job_clone1= job_clone.clone();
        //let job_clone_2 = job_clone.clone();
        //self.scheduler_job_list_add(job_clone).await;
        //self.job_queue_push(job_clone_2, priority.clone()).await;
        self.scheduler_queue_push(job_clone, priority.clone()).await;

        //if let Some(gx_job) = gx_job_clone.lock().await.as_mut(){
        //    if let Some(job_render) = gx_job.as_any().downcast_ref::<GxJobRenderPdf>() {
        //        let page_index = job_render.page_index;
        //        self.scheduler_list_push(page_index, job_clone1).await;
        //    }
        //};

    }   

    //这个函数用于将所有任务推入待处理的任务处理器，并分成了4个等级？
    async fn scheduler_queue_push(&self,job:AMGxJobOption,priority:GxJobPriority){
        let priority_clone= priority.clone() as usize;
        //if cfg!(debug_assertions) {
        //    if let Some(gx_job) = job.clone().lock().await.as_ref(){
        //        //println!("Job queue push: {} priority {}", gx_job.get_type_name(), priority_clone);
        //    }
        //}
        let queue = &mut (self.job_queue.lock().await[priority_clone]);
        queue.push_back(job);
        let _ = self.job_added_tx.send(priority);
        //self.notify.notify_waiters();//通知所有等待的任务，已经有job推入queue了？
    }

    //这个函数用于将所有任务推入待处理的任务处理器，并分成了4个等级？
    fn scheduler_queue_try_push(&self,job:AMGxJobOption,priority:GxJobPriority){
        let priority_clone= priority.clone() as usize;
        if let Ok(mut job_queue) = self.job_queue.try_lock(){
            let queue = &mut job_queue[priority_clone];
            queue.push_back(job);
            let _ = self.job_added_tx.send(priority);
        }
        //self.notify.notify_waiters();//通知所有等待的任务，已经有job推入queue了？
    }


    ////弹出某个等级中某个任务,特征是key
    //pub async fn scheduler_remove_job_by_key(&self,priority:GxJobPriority,key:Uuid) -> Option<AMGxJobOption>{
    //    let priority_clone= priority.clone() as usize;
    //    let queue = &mut self.job_queue.lock().await[priority_clone];
    //    let mut job = None;
    //    let mut i = 0;
    //    for _job in queue.iter_mut(){
    //        if let Some(gx_job) = _job.clone().lock().await.as_ref(){
    //            if gx_job.get_key() == key{
    //                break;
    //            }
    //        }
    //        i += 1;
    //    }
    //    if i < queue.len(){
    //        job = queue.remove(i);
    //    }
 
    //    job
    //}
    //弹出某个等级中某个任务,特征是key
    pub fn scheduler_remove_job_by_key(&self,priority:GxJobPriority,key:Uuid) -> Option<AMGxJobOption>{
        let priority_clone= priority.clone() as usize;
        if let Ok(mut job_queue)  =  self.job_queue.try_lock(){
            let queue = &mut job_queue[priority_clone];
            let mut rt_job = None;
            let mut i = 0;   
            for job in queue.iter_mut(){
                if let Ok(job) = job.clone().try_lock(){
                    //if let Some(gx_job) = &*job{
                        if job.get_key() == key{
                            break;
                        }
                    //}
                }
                i += 1;
            }
            if i < queue.len(){
                rt_job = queue.remove(i);
            }
            return rt_job;
        }
        None
    }



    pub async fn scheduler_test_render_job_by_page_index(&self,page_index:i32) -> bool{
        for i in GxJobPriority::Urgent as usize.. GxJobPriority::NPriorities as usize{
            let queue = self.job_queue.lock().await[i].clone();
            for job in queue.iter(){
                let gx_job = job.lock().await;
                //if let Some(gx_job) 
                //    = job.clone().lock().await.as_ref(){
                    if let Some(job_render) =gx_job.as_any().downcast_ref::<GxJobRenderPdf>() {
                        if job_render.page_index == page_index {
                            return true;
                        }

                    }
                //}
            }
        }
        false
    }



    //这个函数应该是实现完整了？
    //evince的参数是GxJob,此处改为了GxSchedulerJob,主要是方便使用clone与lock？
    //还因为将running_job改造为GxSchedulerJob了
    //这个函数主要目的用于设置running_job，并让它run
    //大概流程：如果传入的Job已经cancel，就直接跳出循环
    //如果没有cancel，那就将job拷贝成running_job，然后执行running_job的run
    //如果run的结果返回是false，表示这个job没有run了，也就是已经运行完了，那就跳出循环
    //跳出循环之后设置running_job为default值，就是啥也没有
    async fn scheduler_thread(&self,job: AMGxJobOption){
        let gx_job = job.clone();
        //if cfg!(debug_assertions) {
        //    if let Some(gx_job) = gx_job.lock().await.as_ref(){
        //        //println!("DEBUG_JOBS in job_thread {}",gx_job.get_type_name())
        //    }
        //}
        loop{
            let mut gx_job = gx_job.lock().await;
            //if let Some(gx_job) = gx_job.lock().await.as_mut(){
                //如果任务取消，result设置为false
                if gx_job.is_cancelled(){
                    println!("Some job is cancelled");
                    break;
                }
                else{
                    gx_job.run().await;
                    //if let Some(job_render) 
                    //    = gx_job.as_any().downcast_ref::<GxJobRenderPdf>() {
                    //    gx_job.run().await;
                    //}
                            
                    break;
                }
            //}
        }
    }

    //这个函数作用是没有任务时让线程等待，有任务就调用job_thread，完了再remove
    async fn scheduler_thread_proxy(&self) {
        //let notify_clone = self.notify.clone();
        let mut job_added_rx = self.job_added_tx.subscribe();
        loop {
            //仅为了表示通知收到了
            //let _ = job_added_rx.recv().await.unwrap();
            if job_added_rx.recv().await.is_err() {
                //来不及处理就跳过 
                continue;
            }
            //notify_clone.notified().await;
            if let Some(job) = self.scheduler_get_next_unlocked().await{
                self.scheduler_thread(job).await;
            };
        }
    }

    //其实用不上，全都改为push_job了？
    pub fn scheduler_update_job(&self,job_key:Uuid,
        old_priority:GxJobPriority,new_priority:GxJobPriority)  {
        let priority_clone = new_priority.clone();
        //if cfg!(debug_assertions) {
        //    if let Some(job_temp) = job_clone{
        //        if let Some(gx_job) =job_temp
        //        .lock().await.as_ref(){
        //            //println!("DEBUG JOBS in   job_scheduler_update_job{},priority  {}",gx_job.get_type_name(),priority_clone );                   
        //        }
        //    }
        //}
        if old_priority != new_priority{
            if let Some(amjob) 
                = self.scheduler_remove_job_by_key(old_priority,job_key){
                    if let Ok(mut amjob) = amjob.try_lock(){
                        //if let Some(amjob) = &mut *amjob{
                            amjob.set_priority(new_priority);
                        //}
                    }
                    //amjob.lock().await.as_mut().unwrap().set_priority(new_priority);
                    self.scheduler_queue_try_push(amjob, priority_clone);
                }
        }
    }





}

