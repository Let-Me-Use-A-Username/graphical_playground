use macroquad::math::{Rect, Vec2};


fn distance_squared(x1: f32, y1: f32, x2:f32, y2: f32 ) -> f32{
    let dx = x1 - x2;
    let dy = y1 - y2;
    return dx * dx + dy * dy
}

///Each entity is either assigned a circle or rectangular collider.
pub trait Collider{
    fn collides_with(&self, other: &dyn Collider) -> bool;
    fn update(&mut self, pos: Vec2);

    fn collide_with_circle(&self, circle: &CircleCollider) -> bool;
    fn collide_with_rect(&self, rect: &RectCollider) -> bool;
}

pub struct CircleCollider{
    x: f32,
    y: f32,
    radius: f32
}
impl CircleCollider{
    //REVIEW: Perhaps new should be a `Collider` method and have 
    //it return a Collider obj.
    pub fn new(x: f32, y: f32, radius: f32) -> Self{
        return CircleCollider { x, y, radius }
    }
}
impl Collider for CircleCollider{
    fn collides_with(&self, other: &dyn Collider) -> bool{
        return other.collide_with_circle(&self)
    }

    fn update(&mut self, pos: Vec2) {
        self.x = pos.x;
        self.y = pos.y;
    }

    fn collide_with_circle(&self, circle: &CircleCollider) -> bool{
        let total_radius = self.radius + circle.radius;
        return distance_squared(self.x, self.y, circle.x, circle.y) <= total_radius * total_radius
    }

    fn collide_with_rect(&self, rect: &RectCollider) -> bool{
        let closest_x = self.x.max(rect.x).min(rect.x + rect.w);
        let closest_y = self.y.max(rect.y).min(rect.y + rect.h);
        return distance_squared(self.x, self.y,closest_x, closest_y) <= self.radius * self.radius    
    }
}

pub struct RectCollider{
    x: f32, 
    y: f32,
    w: f32,
    h: f32
}
impl RectCollider{
    //REVIEW: Perhaps new should be a `Collider` method and have 
    //it return a Collider obj.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self{
        return RectCollider { x, y, w, h }
    }

    pub fn as_rect(&self) -> Rect{
        return Rect{
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h
        }
    }
}
impl Collider for RectCollider{
    fn collides_with(&self, other: &dyn Collider) -> bool{
        return other.collide_with_rect(&self)
    }

    fn update(&mut self, pos: Vec2) {
        self.x = pos.x;
        self.y = pos.y;
    }

    fn collide_with_circle(&self, circle: &CircleCollider) -> bool{
        return circle.collide_with_rect(&self)
    }

    fn collide_with_rect(&self, rect: &RectCollider) -> bool{
        self.x < rect.x + rect.w && self.x + self.w > rect.x &&
        self.y < rect.y + rect.h && self.y + self.h > rect.y
    }
}


