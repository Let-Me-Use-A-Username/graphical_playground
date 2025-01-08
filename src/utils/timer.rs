#[derive(Debug, Clone, Copy)]
pub struct Timer{
    start: f64,
    duration: f64,
    set: bool,
    cooldown: Option<f64>,
    last_set: Option<f64>
}

impl Timer{
    pub fn new() -> Self{
        return Timer { start: 0.0, duration: 0.0, set: false, cooldown: None, last_set: None}
    }

    pub fn set(&mut self, start: f64, duration: f64, cooldown: Option<f64>){
        self.start = start;
        self.duration = duration;
        self.set = true;
        self.cooldown = cooldown;
        self.last_set = Some(start);
    }

    pub fn has_expired(&self, now: f64) -> Option<bool>{
        if self.set{
            return Some(now > self.start + self.duration)
        }
        return None
    }

    pub fn can_be_set(&self, now: f64) -> bool{
        if let Some(cooldown) = self.cooldown{
            if let Some(last) = self.last_set{
                return now > last + self.duration + cooldown
            }
        }
        return !self.set
    }

    pub fn reset(&mut self){
        self.set = false;
    }

}
