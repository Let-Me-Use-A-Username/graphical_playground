use macroquad::math::Vec2;

///Each entity is either assigned a circle or rectangular collider.
pub trait Collider{
    fn collides_with(&self, other: &dyn Collider) -> bool;
    fn update(&mut self, pos: Vec2);

    fn collide_with_circle(&self, circle: &CircleCollider) -> bool;
    fn collide_with_rect(&self, rect: &RectCollider) -> bool;
}

#[derive(Clone, Copy)]
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
        // If no rotation, calculate squared distance and if less or equal to radius, collision 
        if rect.rotation == 0.0 {
            let closest_x = self.x.max(self.x).min(self.x + rect.w);
            let closest_y = self.y.max(self.y).min(self.y + rect.h);
            return distance_squared(self.x, self.y, closest_x, closest_y) <= self.radius * self.radius
        }

        let corners = rect.get_corners();
        //Check if horizontal lines fron circle center, intersect the rect
        if is_point_in_polygon(self.x, self.y, &corners) {
            return true;
        }

        //Edge case to check rectangle corner intersection with circle
        for i in 0..4 {
            let j = (i + 1) % 4;
            let (x1, y1) = corners[i];
            let (x2, y2) = corners[j];
            
            // Find the closest point on each rectangle edge to the circle center 
            let closest = closest_point_on_line_segment(self.x, self.y, x1, y1, x2, y2);
            
            // Check if this closest point is within the circle's radius
            if distance_squared(self.x, self.y, closest.0, closest.1) <= self.radius * self.radius {
                return true;
            }
        }

        return false    }
}

#[derive(Clone, Copy)]
pub struct RectCollider{
    x: f32, 
    y: f32,
    w: f32,
    h: f32,
    rotation: f32
}
impl RectCollider{
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self{
        return RectCollider { x, y, w, h , rotation: 0.0}
    }

    pub fn set_rotation(&mut self, rotation: f32){
        self.rotation = rotation;
    }

    fn get_corners(&self) -> [(f32, f32); 4]{
        let half_width = self.w / 2.0;
        let half_height = self.h / 2.0;

        //Rect center
        let center_x = self.x + half_width;
        let center_y = self.y + half_height;

        let relative_corners = [
            (-half_width, -half_height), //Top left
            (-half_width, half_height), // Bottom left
            (half_width, -half_height), //Top right
            (half_width, half_height), // Bottom right
        ];

        //Why does this translate to world coords
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let mut rotated_corners = [(0.0, 0.0); 4];

        let mut index = 0;
        while let Some((relative_x, relative_y)) = relative_corners.get(index){
            let rotation_x = relative_x * cos_r - relative_y * sin_r;
            let rotation_y = relative_x * sin_r + relative_y * cos_r;

            rotated_corners[index] = (center_x + rotation_x, center_y + rotation_y);
            index += 1;
        }

        return rotated_corners
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

    fn collide_with_circle(&self, circle: &CircleCollider) -> bool {
        // If no rotation, calculate squared distance and if less or equal to radius, collision 
        if self.rotation == 0.0 {
            let closest_x = circle.x.max(self.x).min(self.x + self.w);
            let closest_y = circle.y.max(self.y).min(self.y + self.h);
            return distance_squared(circle.x, circle.y, closest_x, closest_y) <= circle.radius * circle.radius
        }
        
        let corners = self.get_corners();
        //Check if horizontal lines fron circle center, intersect the rect
        if is_point_in_polygon(circle.x, circle.y, &corners) {
            return true;
        }
        
        //Edge case to check rectangle corner intersection with circle
        for i in 0..4 {
            let j = (i + 1) % 4;
            let (x1, y1) = corners[i];
            let (x2, y2) = corners[j];
            
            // Find the closest point on each rectangle edge to the circle center 
            let closest = closest_point_on_line_segment(circle.x, circle.y, x1, y1, x2, y2);
            
            // Check if this closest point is within the circle's radius
            if distance_squared(circle.x, circle.y, closest.0, closest.1) <= circle.radius * circle.radius {
                return true;
            }
        }
        
        return false
    }

    fn collide_with_rect(&self, rect: &RectCollider) -> bool {
        // No rotation, use AABB
        if self.rotation == 0.0 && rect.rotation == 0.0 {
            return self.x < rect.x + rect.w && self.x + self.w > rect.x &&
                   self.y < rect.y + rect.h && self.y + self.h > rect.y
        }
        
        /* 
            Separating Axis Theorem (SAT)
            Checks if any axis exists, that separates the two rects (polygons with 4 sides)
        */
        
        // Get all corners
        let corners1 = self.get_corners();
        let corners2 = rect.get_corners();
        
        // Check for separation along each potential separating axis
        // For two rectangles, we need to check 4 axes (2 from each rectangle)
        
        // Check axes from the first rectangle
        for i in 0..2 {
            let j = (i + 1) % 4;
            let axis = normalize(
                corners1[j].0 - corners1[i].0,
                corners1[j].1 - corners1[i].1
            );
            
            if is_separating_axis(axis, &corners1, &corners2) {
                return false;  // Found a separating axis, no collision
            }
        }
        
        // Check axes from the second rectangle
        for i in 0..2 {
            let j = (i + 1) % 4;
            let axis = normalize(
                corners2[j].0 - corners2[i].0,
                corners2[j].1 - corners2[i].1
            );
            
            if is_separating_axis(axis, &corners1, &corners2) {
                return false;  // Found a separating axis, no collision
            }
        }
        
        // No separating axis found, the rectangles are colliding
        return true
    }
}

fn distance_squared(x1: f32, y1: f32, x2:f32, y2: f32 ) -> f32{
    let dx = x1 - x2;
    let dy = y1 - y2;
    return dx * dx + dy * dy
}

fn normalize(x: f32, y: f32) -> (f32, f32) {
    let len = (x*x + y*y).sqrt();
    if len == 0.0 {
        return (0.0, 0.0);
    }
    (x / len, y / len)
}

fn is_separating_axis(axis: (f32, f32), corners1: &[(f32, f32); 4], corners2: &[(f32, f32); 4]) -> bool {
    // Project all corners onto the axis
    let mut min1 = f32::MAX;
    let mut max1 = f32::MIN;
    let mut min2 = f32::MAX;
    let mut max2 = f32::MIN;
    
    // Project corners of the first rectangle
    for corner in corners1.iter() {
        let proj = corner.0 * axis.0 + corner.1 * axis.1;
        min1 = min1.min(proj);
        max1 = max1.max(proj);
    }
    
    // Project corners of the second rectangle
    for corner in corners2.iter() {
        let proj = corner.0 * axis.0 + corner.1 * axis.1;
        min2 = min2.min(proj);
        max2 = max2.max(proj);
    }
    
    // Check if projections overlap
    max1 < min2 || max2 < min1
}

/* 
    Take the circle's center.

    For each corner, draw a horizontal line.
    
    If the line intersects the rectangle an odd number of times, 
    then the point is inside.
*/
fn is_point_in_polygon(x: f32, y: f32, corners: &[(f32, f32); 4]) -> bool {
    let mut inside = false;

    for i in 0..4 {
        let j = (i + 1) % 4;
        let (xi, yi) = corners[i];
        let (xj, yj) = corners[j];
        
        let intersect = ((yi > y) != (yj > y)) && 
                         (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
    }
    inside
}

/* 
    Takes two corners and the circles position.
    Finds the closest point on each edge to the circle center.
*/
fn closest_point_on_line_segment(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
    // Calculate the closest point on a line segment to a given point
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_squared = dx*dx + dy*dy;
    
    //Zero division catch
    if len_squared == 0.0 {
        return (x1, y1);
    }
    
    let t = ((px - x1) * dx + (py - y1) * dy) / len_squared;
    
    let t_clamped = t.max(0.0).min(1.0);
    
    (x1 + t_clamped * dx, y1 + t_clamped * dy)
}




