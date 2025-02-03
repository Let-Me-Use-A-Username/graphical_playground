


enum Enemy{
    Circle(Circle),
    Rect(Rect),
    Ellipse(Ellipse),
    Triangle(Triangle),
    Hexagon(Hexagon)
}

impl Enemy{
    fn random() -> Self{
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => { Enemy::Circle },
            1 => { Enemy::Rect },
            2 => { Enemy::Ellipse },
            3 => { Enemy::Triangle },
            4 => { Enemy::Hexagon }
        }
    }
}

impl Drawable for Enemy{
    //todo
}

impl Updatable for Enemy{
    //todo
}
