use core::panic;

#[derive(Debug, Clone, Copy)]
pub enum TimerType{
    Unassigned,
    ImmuneTimer
}

#[derive(Debug, Clone, Copy)]
pub struct Timer{
    start: f64,
    duration: f64,
    timer_type: TimerType,
    set: bool,
}

impl Timer{
    pub fn new() -> Self{
        return Timer { start: 0.0, duration: 0.0, timer_type: TimerType::Unassigned, set: false}
    }

    pub fn set(&mut self, start: f64, duration: f64, timer_type: TimerType){
        self.start = start;
        self.duration = duration;
        self.timer_type = timer_type;
        self.set = true;
    }

    pub fn has_expired(&self, now: f64) -> bool{
        if self.set{
            return now - self.start > self.duration;
        }
        panic!("Checking expiration on a Timer which hasn't been setted first.");
    }

    pub fn is_set(&self) -> bool{
        return self.set
    }

    pub fn clear(&mut self){
        self.start = 0.0;
        self.duration = 0.0;
        self.timer_type = TimerType::Unassigned;
        self.set = false;
    }
}