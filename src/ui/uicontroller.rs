use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::{color::BLACK, math::Vec2, text::{draw_text_ex, load_ttf_font, Font, TextParams}};

use crate::{entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Publisher, Subscriber}}, utils::globals::Global};



pub struct UIController{
    killed: u64,
    score: f64,
    boost_charges: i32,
    ammo: usize,
    sender: Sender<Event>,
    game_over: bool
}
impl UIController{
    pub fn new(sender: Sender<Event>) -> UIController{
        return UIController{
            killed: 0,
            score: 0.0,
            boost_charges: Global::get_boost_charges() as i32,
            ammo: Global::get_bullet_ammo_size(),
            sender: sender,
            game_over: false
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

    //Note: `draw_text` and `draw_text_ex` HAS to be after `set_default_camera`
    pub async fn draw(&self){
        let height = Global::get_screen_height();
        let width = Global::get_screen_width();
        let padding = 30.0;

        let scoreboard_label_pos = Vec2::new(width /2.0 - padding * 2.0, 0.0 + padding);
        let boost_charges_pos = Vec2::new(width - padding * 12.0, height - padding * 3.0);
        let ammo_pos = Vec2::new(width - padding * 12.0, height - padding * 6.0);

        self.draw_scoreboard(scoreboard_label_pos, padding).await;
        self.draw_boost_charges(boost_charges_pos).await;
        self.draw_ammo(ammo_pos).await;
    }

    async fn draw_ammo(&self, ammo_pos: Vec2){
        let custom_font = load_ttf_font("assets/font.ttf").await.unwrap_or_else(|_| {
            // Fallback to default font if loading fails
            Font::default()
        });

        let message = if self.ammo == 0{
            "Reloading...".to_string()
        }
        else{
            self.ammo.to_string()
        };

        let charges_params = TextParams {
            font: Some(&custom_font),
            font_size: 48,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            rotation: 0.0,
            color: BLACK,
        };

        draw_text_ex(
            &format!("Ammo: {}", message),
            ammo_pos.x,
            ammo_pos.y,
            charges_params,
        );
    }

    async fn draw_boost_charges(&self, boost_charges_pos: Vec2){
        let custom_font = load_ttf_font("assets/font.ttf").await.unwrap_or_else(|_| {
            // Fallback to default font if loading fails
            Font::default()
        });

        let charges_params = TextParams {
            font: Some(&custom_font),
            font_size: 48,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            rotation: 0.0,
            color: BLACK,
        };

        draw_text_ex(
            &format!("Charges: {}", self.boost_charges),
            boost_charges_pos.x,
            boost_charges_pos.y,
            charges_params,
        );
    }

    async fn draw_scoreboard(&self, scoreboard_label_pos: Vec2, padding: f32){
        let custom_font = load_ttf_font("assets/font.ttf").await.unwrap_or_else(|_| {
            // Fallback to default font if loading fails
            Font::default()
        });

        let scoreboard_params = TextParams {
            font: Some(&custom_font),
            font_size: 48,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            rotation: 0.0,
            color: BLACK,
        };

        let kill_params = TextParams {
            font: Some(&custom_font),
            font_size: 24,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            rotation: 0.0,
            color: BLACK,
        };

        draw_text_ex(
            &format!("Score: {}", self.score),
            scoreboard_label_pos.x,
            scoreboard_label_pos.y,
            scoreboard_params,
        );

        draw_text_ex(
            &format!("Kills: {}", self.killed),
            scoreboard_label_pos.x + padding,
            scoreboard_label_pos.y + padding,
            kill_params,
        );
    }

    pub fn game_over(&self) -> bool{
        return self.game_over
    }

    pub fn get_points(&self) -> f64{
        return self.score
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
            EventType::AlterBoostCharges => {
                if let Ok(request) = event.data.lock(){
                    if let Some(data) = request.downcast_ref::<i32>(){
                        let new_data = data.to_owned();

                        let new_counter = self.boost_charges + new_data;

                        if new_counter <= Global::get_boost_charges() as i32{
                            self.boost_charges = new_counter;
                        }
                    }
                }  
            },
            EventType::AlterAmmo => {
                if let Ok(request) = event.data.lock(){
                    if let Some(data) = request.downcast_ref::<i32>(){
                        let new_data = data.to_owned();

                        let new_ammo = {
                            //Ammo reduction
                            if new_data < 0 {
                                self.ammo as i32 + new_data
                            }
                            //Ammo refill
                            else{
                                new_data
                            }  
                        };

                        if new_ammo <= Global::get_bullet_ammo_size() as i32{
                            self.ammo = new_ammo as usize;
                        }
                    }
                } 
            },
            EventType::GameOver => {
                if let Ok(request) = event.data.lock(){
                    if let Some(_) = request.downcast_ref::<i32>(){
                        self.game_over = true;
                    }
                } 
            }
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