use std::fmt;

use macroquad::time::get_time;

use super::timer::SimpleTimer;


/* 
    Struct that represents abilities or effects that can be used a certain amount of times
    that also recharges based on an interval.
*/
pub struct RechargebleCounter{
    usages: u32,                    //Total count of usages
    counter_interval: u32,          //Interval for charge and discharge
    remaining: Option<i32>,         //Remaining time in usage

    timer: Option<SimpleTimer>,
    timer_duration: Option<f64>
}
impl RechargebleCounter{
    pub fn new(usages: u32, interval: u32, contain_timer: bool, timer_duration: Option<f64>) -> RechargebleCounter{
        assert!(usages > interval);

        let mut timer = None;
        
        if contain_timer{
            if timer_duration.is_some(){
                timer = Some(SimpleTimer::new(timer_duration.clone().unwrap()));
            }
        }

        return RechargebleCounter{
            usages: usages,
            counter_interval: interval,
            remaining: Some((usages - interval) as i32),
            timer: timer,
            timer_duration: timer_duration
        }
    }

    ///Reset remaining usages.
    pub fn reset(&mut self){
        self.remaining = Some((self.usages - self.counter_interval) as i32);
    }

    ///Remove a charge, if charges are available. `remaining` has to be Some(val) and val > 0
    pub fn discharge(&mut self){
        match self.remaining{
            Some(val) => {
                let res = val - (self.counter_interval as i32);

                if res <= 0 {
                    self.remaining = None;
                }
                else{
                    self.remaining = Some(res);
                }
            },
            None => (),
        }
    }

    ///Add a charge.
    fn charge(&mut self){
        if let Some(remain) = self.remaining{
            let new_remain = remain + self.counter_interval as i32;

            if new_remain <= self.usages as i32{
                self.remaining = Some(new_remain);
            }
        }
        else{
            self.remaining = Some(self.counter_interval as i32);
        }
    }

    ///Signals whether usage is valid (based on remaining charges).
    pub fn allow(&mut self) -> bool{
        return self.remaining.is_some_and(|x| x > 0)
    }

    ///Essentially, adds charges based on the intertal timer.
    pub fn update(&mut self){
        let now = get_time();

        if let Some(ref mut timer) = self.timer {
            if timer.expired(now) {
                // Reset the timer if it's expired
                if let Some(duration) = self.timer_duration {
                    timer.set(now, duration);
                    self.charge();
                }
            }
        }
    }
}

impl fmt::Debug for RechargebleCounter{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RechargebleCounter")
            .field("usages", &self.usages)
            .field("counter_interval", &self.counter_interval)
            .field("remaining", &self.remaining).finish()
    }
}