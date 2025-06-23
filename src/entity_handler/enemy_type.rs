

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EnemyType{
    Circle,
    Triangle,
    Rect,
    Hexagon,
    CircleBoss,
    TriangleBoss
}

impl EnemyType{
    pub fn is_boss(&self) -> bool{
        match self{
            EnemyType::CircleBoss => true,
            EnemyType::TriangleBoss => true,
            _ => {false}
        }
    }
}