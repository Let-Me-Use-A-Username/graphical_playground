use crate::collision_system::collider::Collider;


pub struct CollisionDetector{
    entities: Vec<Box<dyn Collider>>
}


impl CollisionDetector{
    pub fn detect_collision(){
        
    }
}
//Check adjusent cells for player hit
//Check current cells for projectiles
//Enemy collision?