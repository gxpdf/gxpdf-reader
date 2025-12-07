#[derive(Default,Clone)]
pub struct GxRenderContext{
    pub rotation: i32,
    pub scale: f32,
    pub target_width: i32,
    pub target_height: i32
}

impl GxRenderContext{
    pub fn new(rotation:i32,scale:f32) -> Self{
        let mut gx_render_context = GxRenderContext::default();
        gx_render_context.rotation = rotation;
        gx_render_context.scale = scale;
        gx_render_context.target_width = -1;
        gx_render_context.target_height = -1;
        gx_render_context
    }

    pub fn set_target_size(&mut self,target_width:i32,target_height:i32){
        self.target_width = target_width;
        self.target_height = target_height;
    }
    
    #[allow(unused)]
    pub fn compute_transformed_size(&mut self,width_points:f32,height_points:f32) -> (i32,i32){
         let (scaled_width,scaled_height) = self.compute_scaled_size(width_points, height_points);
         let transformed_width;
         let transformed_height;
         if self.rotation == 90 || self.rotation == 270{
            transformed_width = scaled_height;
            transformed_height = scaled_width;
         }else{
            transformed_width = scaled_width;
            transformed_height = scaled_height;
         }
         (transformed_width,transformed_height)
    }
    
    pub fn compute_scaled_size(&mut self,width_points:f32,height_points:f32) -> (i32,i32){
        let scaled_width ;
        let scaled_height ;
        //其实仅执行了这个if，else完全没机会执行，后面的height的else也完全没执行
        if self.target_width >= 0 {
            if self.rotation == 90 || self.rotation == 270{
                scaled_width = self.target_height;
            }else{
                scaled_width = self.target_width;
            }
        }else{
            scaled_width = (width_points * self.scale + 0.5) as i32;
        }
        
        if self.target_height >= 0{
            if self.rotation == 90 || self.rotation == 270{
                scaled_height = self.target_width;
            }else{
                scaled_height = self.target_height;
            }
        }else{
            scaled_height = (height_points * self.scale + 0.5) as i32;
        }

        (scaled_width,scaled_height)
    }

    #[allow(unused)]
    pub fn compute_scales(&mut self,width_points:f32,height_points:f32) -> (f32,f32){
        let (scaled_width,scaled_height) = self.compute_scaled_size(width_points, height_points);
        (scaled_width as f32 / width_points,scaled_height as f32 / height_points)
    }
}


