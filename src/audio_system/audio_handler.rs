use std::collections::HashMap;

use async_trait::async_trait;
use macroquad::{audio::{self, PlaySoundParams, Sound}, file::set_pc_assets_folder};

use crate::event_system::{event::{Event, EventType}, interface::Subscriber};


#[derive(Eq, Hash, PartialEq, Clone, Debug)]
///Unique Identifier for Sounds.
pub enum SoundType{
    PlayerIdle,         //ok. Fires from player.
    PlayerBoosting,
    PlayerHit,          //ok. Fires from player.
    PlayerMoving,       //ok. Fires from player.
    PlayerDrifting,     //ok. Fires from player.
    PlayerFiring,       //ok. Fires from player.

    ShieldHit,          //ok. Fires from player.

    EnemyDeath,         //ok. Each entity fires individually.

    TriangleFiring,     //ok. Fires from TriangleAssistant.
    RectHit,            //ok. Fires from Rect.
    HexDeflect          //ok. Fires from Entity_Handler
}
impl SoundType{
    fn from_player(self) -> bool{
        match self{
            SoundType::PlayerIdle => true,
            SoundType::PlayerBoosting => true,
            SoundType::PlayerHit => true,
            SoundType::PlayerMoving => true,
            SoundType::PlayerDrifting => true,
            SoundType::PlayerFiring => true,
            _ => false
        }
    }

    fn is_player_state(&self) -> bool {
        match self{
            SoundType::PlayerIdle => true,
            SoundType::PlayerHit => true,
            SoundType::PlayerMoving => true,
            SoundType::PlayerDrifting => true,
            _ => false
        }
    }

    fn into_iter() -> Vec<SoundType>{
        return vec![
            SoundType::PlayerIdle,
            SoundType::PlayerBoosting,
            SoundType::PlayerHit,
            SoundType::PlayerMoving,
            SoundType::PlayerDrifting,
            SoundType::PlayerFiring,
        ]
    }
}

///Record Accoustic holds with information on Sound state.
struct SoundRecord{
    sound: Sound,
    is_playing: bool,
    looped: bool,
    volume: f32
}
impl SoundRecord{
    fn get_default(&self) -> PlaySoundParams{
        return PlaySoundParams { 
            looped: false, 
            volume: 0.1 
        }
    }
}


///Sound Request that entities provide to Accoustic.
#[derive(Clone)]
pub struct SoundRequest{
    once: bool,
    looped: bool,
    volume: f32
}
impl SoundRequest{
    pub fn new(once: bool, looped: bool, volume: f32) -> SoundRequest{
        return SoundRequest { 
            once: once, 
            looped: looped, 
            volume: volume 
        }
    }
    fn get_params(&self) -> PlaySoundParams{
        return PlaySoundParams { 
            looped: self.looped, 
            volume: self.volume 
        }
    }
}

pub struct Accoustic{
    sounds: HashMap<SoundType, SoundRecord>
}
impl Accoustic{
    pub async fn new() -> Accoustic{
        set_pc_assets_folder("assets");

        let player_boosting = audio::load_sound(&"audio/sounds/player_boosting.wav").await.unwrap();
        let player_idle = audio::load_sound(&"audio/sounds/player_idle.wav").await.unwrap();
        let player_hit = audio::load_sound(&"audio/sounds/player_hit.wav").await.unwrap();
        let player_moving = audio::load_sound(&"audio/sounds/player_moving.wav").await.unwrap();
        let player_drifting = audio::load_sound(&"audio/sounds/player_drifting.wav").await.unwrap();
        let player_firing = audio::load_sound(&"audio/sounds/player_firing.wav").await.unwrap();
        
        let shield_hit = audio::load_sound(&"audio/sounds/shield_hit.wav").await.unwrap();

        let enemy_death = audio::load_sound(&"audio/sounds/enemy_death.wav").await.unwrap();

        let enemy_firing = audio::load_sound(&"audio/sounds/triangle_firing.wav").await.unwrap();
        let rect_hit = audio::load_sound(&"audio/sounds/rect_hit.wav").await.unwrap();
        let hex_deflect = audio::load_sound(&"audio/sounds/hex_deflect.wav").await.unwrap();


        let mut sounds = HashMap::new();
        
        sounds.insert(SoundType::PlayerBoosting, SoundRecord { sound: player_boosting, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerIdle, SoundRecord { sound: player_idle, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerHit, SoundRecord { sound: player_hit, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerMoving, SoundRecord { sound: player_moving, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerDrifting, SoundRecord { sound: player_drifting, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerFiring, SoundRecord { sound: player_firing, is_playing: false, looped: false, volume: 100.0 });

        sounds.insert(SoundType::ShieldHit, SoundRecord { sound: shield_hit, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::EnemyDeath, SoundRecord { sound: enemy_death, is_playing: false, looped: false, volume: 100.0 });

        sounds.insert(SoundType::TriangleFiring, SoundRecord { sound: enemy_firing, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::RectHit, SoundRecord { sound: rect_hit, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::HexDeflect, SoundRecord { sound: hex_deflect, is_playing: false, looped: false, volume: 100.0 });

        return Accoustic{
            sounds: sounds
        }
    }

    fn play_sound(&mut self, stype: SoundType, params: Option<PlaySoundParams>){
        if let Some(record) = self.sounds.get_mut(&stype){

            //Set default parameters if none assigned.
            let options = if params.is_some(){
                params.unwrap()
            }
            else{
                record.get_default()
            };
            
            if !record.is_playing{
                let sound = record.sound.to_owned();

                record.looped = true;
                record.is_playing = true;
                record.volume = options.volume;

                if stype.clone().from_player(){
                    self.handle_player_audio(stype);
                }
                audio::play_sound(&sound, options);
            }
        }
    }

    fn stop_sound(&mut self, stype: SoundType){
        if let Some(record) = self.sounds.get_mut(&stype){
            audio::stop_sound(&record.sound);
            record.is_playing = false;
        }
    }

    fn play_once(&mut self, stype: SoundType, volume: f32){
        if let Some(record) = self.sounds.get_mut(&stype){
            record.is_playing = true;
            record.volume = volume;
            audio::play_sound_once(&record.sound, volume);
        }
    }

    ///Stops Players state Sounds. Moving, Hit, Idle, Drifting. Substates aren't regarded.
    fn handle_player_audio(&mut self, active: SoundType){
        let sounds = SoundType::into_iter();

        for sound in sounds{
            let temp_sound = sound.clone();

            if sound.ne(&active) && sound.is_player_state(){
                self.stop_sound(temp_sound);
            }
        }
    }
}


#[async_trait]
impl Subscriber for Accoustic {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::PlaySound => {
                let mut state = None;
                let mut reset = false;
                let mut request = None;
                
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<(SoundType, SoundRequest)>(){
                        let stype = data.0.clone();
                        let srequest = data.1.clone();
                        let volume = srequest.volume;

                        if let Some(entry) = self.sounds.get_mut(&stype){
                            let volume_change = if volume > (entry.volume + 0.01){
                                true
                            }
                            else if volume < (entry.volume - 0.01) {
                                true
                            }
                            else{
                                false
                            };

                            if entry.is_playing && volume_change{
                                reset = true;
                            }
                        }

                        request = Some(srequest);
                        state = Some(stype);
                    }
                }

                if let Some(state) = state{
                    if reset{
                        self.stop_sound(state.clone());
                    }

                    if let Some(req) = request{
                        if let Some(rec) = self.sounds.get_mut(&state){
                            let play = if state.is_player_state() && !rec.is_playing{
                                    if !rec.is_playing{
                                        true
                                    }
                                    else{
                                        false
                                    }
                                }
                                //All enemies. And player when record isn't playing
                                else{
                                    true
                                };

                            if play{
                                match req.once{
                                    true => {
                                        self.play_once(state, req.volume);
                                    },
                                    false => {
                                        self.play_sound(state, Some(req.get_params()));
                                    },
                                }
                            }
                            
                        }
                        
                    }
                }
            },
            _ => {}
        }
    }
}