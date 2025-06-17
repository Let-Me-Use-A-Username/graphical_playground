use std::collections::HashMap;

use async_trait::async_trait;
use macroquad::{audio::{self, PlaySoundParams, Sound}, file::set_pc_assets_folder};

use crate::event_system::{event::{Event, EventType}, interface::Subscriber};


#[derive(Eq, Hash, PartialEq, Clone, Debug)]
///Unique Identifier for Sounds.
pub enum SoundType{
    PlayerIdle,
    PlayerDeath,
    PlayerHit,
    PlayerMoving,
    PlayerDrifting,
    PlayerFiring,

    EnemyDeath,
    EnemyHit,
    EnemyFiring
}
impl SoundType{
    fn from_player(self) -> bool{
        match self{
            SoundType::PlayerIdle => true,
            SoundType::PlayerDeath => true,
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
            SoundType::PlayerDeath,
            SoundType::PlayerHit,       //ok
            SoundType::PlayerMoving,
            SoundType::PlayerDrifting,  //ok
            SoundType::PlayerFiring,    //ok

            SoundType::EnemyDeath,      //ok
            SoundType::EnemyHit,
            SoundType::EnemyFiring      //ok
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
    fn get_parameters(&self) -> PlaySoundParams{
        return PlaySoundParams { 
            looped: self.looped, 
            volume: self.volume 
        }
    }

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

    fn is_one_shot(&self) -> bool{
        if self.once{

            if self.looped{
                return false
            }
            else{
                return true
            }
        }

        return false
    }
}

pub struct Accoustic{
    sounds: HashMap<SoundType, SoundRecord>
}
impl Accoustic{
    pub async fn new() -> Accoustic{
        set_pc_assets_folder("assets");

        // let player_death = audio::load_sound(&"audio/sounds/player_death.wav").await.unwrap();
        // let player_idle = audio::load_sound(&"audio/sounds/player_idle.wav").await.unwrap();
        let player_hit = audio::load_sound(&"audio/sounds/player_hit.wav").await.unwrap();
        // let player_moving = audio::load_sound(&"audio/sounds/player_moving.wav").await.unwrap();
        let player_drifting = audio::load_sound(&"audio/sounds/player_drifting.wav").await.unwrap();
        let player_firing = audio::load_sound(&"audio/sounds/player_firing.wav").await.unwrap();
        
        // let enemy_death = audio::load_sound(&"audio/sounds/enemy_death.wav").await.unwrap();
        // let enemy_firing = audio::load_sound(&"audio/sounds/enemy_firing.wav").await.unwrap();

        let mut sounds = HashMap::new();
        
        // sounds.insert(SoundType::PlayerDeath, SoundRecord { sound: player_death, is_playing: false, looped: false, volume: 100.0 });
        // sounds.insert(SoundType::PlayerIdle, SoundRecord { sound: player_idle, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerHit, SoundRecord { sound: player_hit, is_playing: false, looped: false, volume: 100.0 });
        // sounds.insert(SoundType::PlayerMoving, SoundRecord { sound: player_moving, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerDrifting, SoundRecord { sound: player_drifting, is_playing: false, looped: false, volume: 100.0 });
        sounds.insert(SoundType::PlayerFiring, SoundRecord { sound: player_firing, is_playing: false, looped: false, volume: 100.0 });

        // sounds.insert(SoundType::EnemyDeath, SoundRecord { sound: enemy_death, is_playing: false, looped: false, volume: 100.0 });
        // sounds.insert(SoundType::EnemyFiring, enemy_firing);

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

                if options.looped{
                    record.is_playing = true;
                    record.looped = true;
                }

                if stype.clone().from_player(){
                    self.handle_player_audio(stype);
                }
                
                audio::play_sound(&sound, options);
            }
        }
    }

    fn stop_sound(&mut self, stype: SoundType){
        if let Some(record) = self.sounds.get_mut(&stype){
            
            if record.is_playing{
                audio::stop_sound(&record.sound);
                record.is_playing = false;
            }
        }
    }

    fn play_once(&self, stype: SoundType){
        if let Some(record) = self.sounds.get(&stype){
            audio::play_sound_once(&record.sound);
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
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<(SoundType, SoundRequest)>(){
                        let stype = data.0.clone();
                        let srequest = data.1.clone();

                        if srequest.is_one_shot(){
                            self.play_once(stype);
                        }
                        else{
                            let params = srequest.get_params();
                            self.play_sound(stype, Some(params));
                        }
                    }
                }
            },
            _ => {}
        }
    }
}