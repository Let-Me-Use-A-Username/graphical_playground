use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::{color::YELLOW, math::Vec2, text::{draw_text_ex, load_ttf_font, Font, TextParams}, ui::root_ui};

use crate::{entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Publisher, Subscriber}}, utils::globals::Global};



pub struct UIController{
    killed: u64,
    score: f64,
    sender: Sender<Event>
}
impl UIController{
    pub fn new(sender: Sender<Event>) -> UIController{
        return UIController{
            killed: 0,
            score: 0.0,
            sender: sender
        }
    }

    fn get_new_points(&mut self, enemies: Vec<EnemyType>) -> f64{
        let scores = Global::get_enemy_points();

        let circle_score = scores.get(0).unwrap();
        let triangle_score = scores.get(1).unwrap();
        let rect_score = scores.get(2).unwrap();
        let hexagon_score = scores.get(3).unwrap();
        let boss_score = scores.get(4).unwrap();

        let mut points = 0.0;

        enemies.iter()
            .for_each(|enemy_type| {
                match enemy_type{
                    EnemyType::Circle => points += circle_score,
                    EnemyType::Triangle => points += triangle_score,
                    EnemyType::Rect => points += rect_score,
                    EnemyType::Hexagon => points += hexagon_score,
                    EnemyType::Boss => points += boss_score,
                }
            }); 
        
        self.score += points;
        
        return points
    }

    pub async fn draw(&self){
        {
            //Scoreboard
            let height = Global::get_screen_height();
            let width = Global::get_screen_width();
            let top = Vec2::new(width, height);

            let custom_font = load_ttf_font("assets/font.ttf").await.unwrap_or_else(|_| {
                // Fallback to default font if loading fails
                Font::default()
            });

            // root_ui()
            //     .label(None, &format!("Score: {}", self.score));

            //FIXME: Doesn't work. Artist is a likely culprit. Implement with root_ui ?
            let text_params = TextParams {
                font: Some(&custom_font),
                font_size: 24,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: YELLOW,
            };

            draw_text_ex(
                "Custom Font Text",
                0.0,
                0.0,
                text_params,
            );
        }
    }
}

#[async_trait]
impl Subscriber for UIController {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::AddScorePoints => {
                if let Ok(request) = event.data.lock(){
                    if let Some(data) = request.downcast_ref::<(u64, Vec<EnemyType>)>(){
                        let new_data = data.to_owned();
                        let kills = new_data.0;
                        let points = self.get_new_points(new_data.1);

                        //Append new kills
                        self.killed += kills;
                        self.score += points;
                    }
                }
            },
            _ => {}
        }
    }
}


#[async_trait]
impl Publisher for UIController {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}