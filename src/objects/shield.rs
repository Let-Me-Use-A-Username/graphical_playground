use std::any::Any;

use async_trait::async_trait;
use macroquad::{color::BLUE, math::Vec2};

use crate::{collision_system::collider::{CircleCollider, Collider}, event_system::interface::{Drawable, Object, Updatable}, renderer::artist::DrawCall};


pub struct Shield{
    pos: Vec2,
    size: usize,
    pub collider: CircleCollider,
    active: bool
}
impl Shield{
    pub fn new(pos: Vec2, size: usize) -> Shield{
        return Shield { 
            pos: pos, 
            size: size, 
            collider: CircleCollider::new(pos.x, pos.y, size as f32), 
            active: false 
        }
    }

    pub fn set_active(&mut self, active: bool){
        self.active = active;
    }

    pub fn is_active(&self) -> bool{
        return self.active
    }

    pub fn collider(&self, other: &dyn Collider) -> bool{
        return other.collide_with_circle(&self.collider)
    }
}

impl Object for Shield{
    fn get_pos(&self) -> Vec2 {
        return self.pos
    }

    fn as_any(&self) -> &dyn std::any::Any {
        return self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any{
        return self
    }
}

#[async_trait]
impl Updatable for Shield{
    async fn update(&mut self, _delta: f32, mut _params: Vec<Box<dyn Any + Send>>){
        if let Some(data) = _params.pop(){
            if let Some(new_pos) = data.downcast_ref::<Vec2>(){
                self.pos = *new_pos;
                self.collider.update(*new_pos);
            }
        }
    }
}

impl Drawable for Shield{
    fn get_draw_call(&self) -> DrawCall {
        return DrawCall::Circle(
            self.pos.x, 
            self.pos.y, 
            self.size as f32, 
            BLUE)
    }

    fn should_emit(&self) -> bool {
        return false
    }
}