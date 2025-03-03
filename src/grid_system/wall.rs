use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::{color::RED, math::{Rect, Vec2}, shapes::draw_line};

use crate::event_system::{event::{Event, EventType}, interface::Publisher};



pub struct Wall{
    bounds: Rect,
    sender: Sender<Event>
}

impl Wall{
    
    pub fn new(bounds: f32, sender: Sender<Event>) -> Wall{
        return Wall{
            bounds: Rect { 
                x: 0.0, 
                y: 0.0, 
                w: bounds, 
                h: bounds,
            },
            sender: sender
        }
    }

    pub async fn update(&self, params: (Vec2, f32)){
        if self.check_boundaries(params.0, params.1){
            let _ = self.publish(Event::new(true, EventType::PlayerHit)).await;
        }
    }

    //Checks if entity is touching any of 4 boundaries
    #[inline(always)]
    fn check_boundaries(&self, pos: Vec2, size: f32) -> bool{
        let mut collided = false;
        
        // Left and top boundary
        if pos.x < 0.0 || pos.y < 0.0{
            collided = true;
        }
        // Right and bottom boundary
        if pos.x + size > self.bounds.w || pos.y + size > self.bounds.h{
            collided = true;
        }

        if collided {
            return true
        } 
        return false
    }

    #[inline(always)]
    pub fn draw(&self, viewport: Rect){
        let width = self.bounds.w;
        let height = self.bounds.h;
        
        let intersects_left = viewport.x <= 0.0 && viewport.x + viewport.w > 0.0;
        let intersects_right = viewport.x < width && viewport.x + viewport.w >= width;
        let intersects_top = viewport.y < height && viewport.y + viewport.h >= height;
        let intersects_bottom = viewport.y <= 0.0 && viewport.y + viewport.h > 0.0;
        
        //Left
        if intersects_left {
            let y_start = f32::max(viewport.y, 0.0);
            let y_end = f32::min(viewport.y + viewport.h, height);
            draw_line(0.0, y_start, 0.0, y_end, 8.0, RED);
        }
        
        //Top
        if intersects_top {
            let x_start = f32::max(viewport.x, 0.0);
            let x_end = f32::min(viewport.x + viewport.w, width);
            draw_line(x_start, height, x_end, height, 8.0, RED);
        }
        
        //Right
        if intersects_right {
            let y_start = f32::max(viewport.y, 0.0);
            let y_end = f32::min(viewport.y + viewport.h, height);
            draw_line(width, y_start, width, y_end, 8.0, RED);
        }
        
        //Bottom
        if intersects_bottom {
            let x_start = f32::max(viewport.x, 0.0);
            let x_end = f32::min(viewport.x + viewport.w, width);
            draw_line(x_start, 0.0, x_end, 0.0, 8.0, RED);
        }
    }
}

#[async_trait]
impl Publisher for Wall{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}