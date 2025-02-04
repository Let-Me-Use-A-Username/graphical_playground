use std::sync::{Arc, Mutex};

#[derive(Debug, Eq, PartialEq)]
pub enum StateType{
    Idle,
    Moving,
    Hit,
    Dash
}

pub struct StateMachine{
    active: Arc<Mutex<StateType>>
}

impl StateMachine{
    pub fn new() -> Self{
        return StateMachine { active: Arc::new(Mutex::new(StateType::Idle))}
    }

    pub fn transition(&mut self, state: StateType){
        self.active = Arc::new(Mutex::new(state));
        println!("Changed state to {:?}", self.active.clone());
    }

    pub fn get_state(&self) -> Arc<Mutex<StateType>>{
        return self.active.clone()
    }
}
