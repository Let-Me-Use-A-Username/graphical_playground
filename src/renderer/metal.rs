use std::collections::{HashMap, HashSet, VecDeque};

use async_trait::async_trait;
use macroquad::{color::Color, math::{vec2, Vec2}};
use macroquad_particles::{AtlasConfig, BlendMode, ColorCurve, Curve, EmissionShape, Emitter, EmitterConfig, EmittersCache, ParticleShape};

use crate::{event_system::{event::{Event, EventType}, interface::Subscriber}, utils::machine::StateType};

/* 
    MetalArist is also a Batch rendering component, however
    its task is to handle Emitters. MetalArtist registers a
    emitter when an entity presents a pair of State-Config.

    This implmenetation is used for entities that have different 
    emission based on their state.

    MetalArtist draws emitters upon request, and has a 
    different handling if they are a one shot emitter or not.

    One shot emitters are removed after they fired, and their 
    timer expires. In order to draw the full effect.

    Permanent emittes are removed upon request from an entity.
*/
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConfigType{
    PlayerDrifting,
    PlayerHit,
    PlayerMove,
    EnemyDeath,
    RectHit
}
impl ConfigType{
    pub fn get_conf(&self) -> EmitterConfig{
        match self{
            ConfigType::PlayerDrifting => {
                return EmitterConfig {
                    local_coords: false,
                    emission_shape: EmissionShape::Point,
                    one_shot: false,
                    lifetime: 1.0,
                    lifetime_randomness: 0.0,
                    explosiveness: 0.0,
                    amount: 30,
                    emitting: true,
                    initial_direction: Vec2::new(0.0, 1.0),
                    initial_direction_spread: 0.2,
                    initial_velocity: 150.0,
                    initial_velocity_randomness: 0.3,
                    linear_accel: -50.0,
                    initial_rotation: 0.0,
                    initial_rotation_randomness: 0.5,
                    initial_angular_velocity: 0.0,
                    initial_angular_velocity_randomness: 0.2,
                    angular_accel: 0.0,
                    angular_damping: 0.1,
                    size: 13.0,
                    size_randomness: 0.5,
                    blend_mode: BlendMode::Alpha,
                    colors_curve: ColorCurve{
                        start: Color::new(0.1, 0.1, 0.1, 1.0),  // At spawn: dark and opaque
                        mid: Color::new(0.2, 0.2, 0.2, 0.7),  // Mid-life: slightly lighter
                        end: Color::new(0.3, 0.3, 0.3, 0.1),  // End of life: fades out
                    }
                    ,
                    gravity: Vec2::new(0.0, -20.0),
                    ..Default::default()
                }
            },
            ConfigType::PlayerHit => {
                return EmitterConfig{
                    one_shot: false,
                    lifetime: 0.3,
                    lifetime_randomness: 0.7,
                    explosiveness: 0.95,
                    amount: 30,
                    initial_direction_spread: 2.0 * std::f32::consts::PI,
                    initial_velocity: 200.0,
                    size: 3.0,
                    gravity: vec2(0.0, -1000.0),
                    atlas: Some(AtlasConfig::new(4, 4, 8..)),
                    blend_mode: BlendMode::Alpha,
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(0, 0, 0, 255),       // Black  
                        mid: Color::from_rgba(128, 128, 128, 255),  // Medium Gray  
                        end: Color::from_rgba(230, 240, 255, 255),  // Ice White (slightly bluish tint)  
                    },
                    ..Default::default()
                }
            }
            ConfigType::PlayerMove => {
                return EmitterConfig {
                    local_coords: false,
                    lifetime: 1.1,
                    explosiveness: 0.1,
                    one_shot: false,
                    amount: 75,
                    initial_direction: vec2(0.0, -1.0),
                    initial_direction_spread: std::f32::consts::PI,
                    initial_velocity: 100.0,
                    initial_velocity_randomness: 0.0,
                    size: 5.0, 
                    size_randomness: 0.5,
                    blend_mode: BlendMode::Alpha,
                    size_curve: Some(Curve {
                        points: vec![(0.0, 0.5), (0.5, 1.0), (1.0, 0.0)],
                        ..Default::default()
                    }),
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(255, 200, 0, 255),
                        mid: Color::from_rgba(255, 100, 50, 255),
                        end: Color::from_rgba(255, 0, 0, 255),
                    },
                    gravity: vec2(0.0, 2.0),
                    ..Default::default()
                }
            },
            ConfigType::EnemyDeath => {
                return EmitterConfig {
                    local_coords: false,
                    one_shot: true,
                    emitting: false,
                    lifetime: 2.0,           
                    lifetime_randomness: 0.2,
                    explosiveness: 1.0,      
                    initial_direction_spread: 2.0 * std::f32::consts::PI,
                    initial_velocity: 300.0,   
                    initial_velocity_randomness: 0.5,
                    size: 7.0, //10.0            
                    size_randomness: 0.3,    
                    amount: 100,
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(255, 50, 50, 255),  // Brighter red
                        mid: Color::from_rgba(255, 150, 50, 150),   // Orange-red
                        end: Color::from_rgba(227, 228, 225, 70),  // Icewhite
                    },
                    ..Default::default()
                }
            },
            ConfigType::RectHit => {
                return EmitterConfig {
                    local_coords: false, 
                    emission_shape: EmissionShape::Rect {
                        width: 70.0,
                        height: 70.0,
                    },
                    one_shot: true, 
                    amount: 30, 
                    explosiveness: 1.0,
                    lifetime: 0.6,
                    lifetime_randomness: 0.2,
                    shape: ParticleShape::Circle { subdivisions: 10}, 
                    initial_direction: vec2(1.0, 0.0), 
                    initial_direction_spread: 2.0 * std::f32::consts::PI, 
                    initial_velocity: 100.0,
                    initial_velocity_randomness: 0.5,
                    linear_accel: -30.0, 
                    initial_rotation: 0.0,
                    initial_rotation_randomness: 1.0,
                    initial_angular_velocity: 2.0,
                    initial_angular_velocity_randomness: 1.0,
                    angular_accel: 0.0,
                    angular_damping: 0.2,
                    size: 50.0, //5.0
                    size_randomness: 0.5,
                    blend_mode: BlendMode::Additive, 
                    colors_curve: ColorCurve {
                        start: Color::new(1.0, 0.0, 0.0, 0.7),
                        mid: Color::new(1.0, 0.5, 0.5, 0.3),
                        end: Color::new(1.0, 0.0, 0.0, 0.0), // fade out 
                    },
                    ..Default::default()
                }
            }
        }
    }
}

type Identifier = (u64, StateType);

/* 
    Emitter are either one shot or not.
    One shot emitters are spawned via cache and expire on their own.
    Permanent emitters have 1 emitter that is handled by the artist exclusively.
*/
enum EmitterType{
    Emitter(Emitter),
    Cache(EmittersCache)
}
impl EmitterType{
    fn draw_all_cache(&mut self){
        match self{
            EmitterType::Emitter(_) => (),
            EmitterType::Cache(emitters_cache) => emitters_cache.draw(),
        }
    }
}

pub struct MetalArtist {
    cache: HashMap<ConfigType, EmitterType>,
    registrations: HashMap<Identifier, ConfigType>,
    request_queue: VecDeque<(u64, StateType, Vec2)>,
}
impl MetalArtist{
    pub fn new() -> MetalArtist{
        let mut cache_map = HashMap::new();
        cache_map.insert(ConfigType::EnemyDeath, 
            EmitterType::Cache(EmittersCache::new(ConfigType::EnemyDeath.get_conf())));
        cache_map.insert(ConfigType::RectHit, 
            EmitterType::Cache(EmittersCache::new(ConfigType::RectHit.get_conf())));
        cache_map.insert(ConfigType::PlayerDrifting, 
            EmitterType::Emitter(Emitter::new(ConfigType::PlayerDrifting.get_conf())));
        cache_map.insert(ConfigType::PlayerHit, 
            EmitterType::Emitter(Emitter::new(ConfigType::PlayerHit.get_conf())));
        cache_map.insert(ConfigType::PlayerMove, 
            EmitterType::Emitter(Emitter::new(ConfigType::PlayerMove.get_conf())));

        return MetalArtist {
            cache: cache_map,
            registrations: HashMap::new(),
            request_queue: VecDeque::new(),
        }
    }

    /* 
        In contrast to Artist, Metal Artist draws the requests in order.
    */
    pub fn draw(&mut self) {
        let mut have_drawn = HashSet::new();
        
        while let Some((id, state, pos)) = self.request_queue.pop_front() {
            let identifier = (id, state);
            
            if let Some(config_type) = self.registrations.get(&identifier) {
                
                if let Some(cache) = self.cache.get_mut(config_type){
                    match cache{
                        EmitterType::Emitter(emitter) => {
                            emitter.draw(pos);
                            have_drawn.insert(config_type);
                        },
                        EmitterType::Cache(emitters_cache) => {
                            emitters_cache.spawn(pos);
                        },
                    }
                }
            }
        }
        
        for emitter in self.cache.values_mut(){
            emitter.draw_all_cache();
        }

        //Collect EmitterTypes that didn't draw this frame
        let rest_configs: Vec<&mut EmitterType> = self.cache.iter_mut()
            .filter(|(key, _)| !have_drawn.contains(key))
            .map(|(_, val)| val)
            .collect();
        
        //Reset all permanent emitters, do nothing for cached ones
        for em_type in rest_configs{
            match em_type{
                EmitterType::Emitter(emitter) => emitter.reset(),
                EmitterType::Cache(cache) => {
                    if cache.should_clear(){
                        cache.clear_empty();
                    }
                },
            }
        }
    }

    //Drop identifier from everywhere
    #[inline(always)]
    fn drop(&mut self, id: Identifier) -> bool{
        if let Some(_) = self.registrations.remove(&id){
            return true
        }
        return false
    }

    #[inline(always)]
    fn add_emitter(&mut self, id: Identifier, conf: ConfigType) {
        if !self.registrations.contains_key(&id){
            self.registrations.insert(id, conf);
        }
    }

    #[inline(always)]
    fn add_request(&mut self, id: u64, state: StateType, pos: Vec2) {
        self.request_queue.push_back((id, state, pos));
    }

    #[inline(always)]
    pub fn add_batch_request(&mut self, req: Vec<(u64, StateType, Vec2)>) {
        self.request_queue.extend(req);
    }
}

#[async_trait]
impl Subscriber for MetalArtist{
    async fn notify(&mut self, event: &Event){
        match event.event_type{
            EventType::RegisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, vec)) = data.downcast_ref::<(u64, Vec<(StateType, ConfigType)>)>(){
                        vec.iter().for_each(|(state, conf)| self.add_emitter((*id, *state), conf.clone()));
                    }
                }
            },
            //Note: This is now only needed to remove permanent Emitters.
            EventType::UnregisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some(id) = data.downcast_ref::<(u64, StateType)>(){
                        self.drop(*id);
                    }
                }
            },
            //Note: Not used anymore, all calls for emission are received in main loop, in `game_manager`.
            EventType::DrawEmitter => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, state, pos)) = data.downcast_ref::<(u64, StateType, Vec2)>(){
                        self.add_request(*id, *state, *pos);
                    }
                }
            },
            _ => {
                todo!()
            }
        }
    }
}
