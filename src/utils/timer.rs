use macroquad::time::get_time;

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

    #[inline(always)]
    pub fn has_expired(&self, now: f64) -> Option<bool>{
        if self.set{
            return Some(now > self.start + self.duration)
        }
        return None
    }

    
    ///Returns true if the cooldown has ended.
    #[inline(always)]
    pub fn can_be_set(&self, now: f64) -> bool{
        if let Some(cooldown) = self.cooldown{
            if let Some(last) = self.last_set{
                return now > last + self.duration + cooldown
            }
        }
        return !self.set
    }

    #[inline(always)]
    pub fn reset(&mut self){
        self.set = false;
    }

}


pub struct SimpleTimer{
    start: f64,
    end: f64,
    set: bool,
    expired: bool
}
impl SimpleTimer{
    pub fn new(exp: f64) -> SimpleTimer{
        let start = get_time();
        let end = start + exp;
        let expired = start >= end;

        return SimpleTimer{
            start: start,
            end: end,
            set: true,
            expired: expired
        }
    }

    pub fn is_set(&self) -> bool{
        return self.set
    }

    pub fn blank() -> SimpleTimer{
        return SimpleTimer{
            start: 0.0,
            end: 0.0,
            set: false,
            expired: false
        }
    }

    pub fn expired(&mut self, now: f64) -> bool{
        return now >= self.end
    }

    pub fn reset(&mut self, now: f64, new_exp: f64){
        self.start = now;
        self.end = self.start + new_exp;
        self.set = true;
        self.expired = self.start >= self.end;
    }
}