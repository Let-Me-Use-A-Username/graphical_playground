use std::{collections::HashMap, sync::mpsc::Sender};

use crate::event_system::event::Event;



pub struct TriangleAssistant{
    triangles: HashMap<u64, usize>, //amount of bullet each entity holds
    sender: Sender<Event>,
    amount: usize
}