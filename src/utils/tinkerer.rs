use std::{error::Error, fs, path::Path};
use serde::{Deserialize, Serialize};
use serde_yaml;


const SETTINGS_PATH: &str = "assets\\settings.yaml";


pub struct Tinkerer{
    has_player_changes: bool,
    settings: Settings
}

impl Tinkerer{
    pub fn new() -> Result<Tinkerer, TinkererError>{
        let path = Path::new(SETTINGS_PATH);
        let path_as_string = SETTINGS_PATH.to_string();
        
        match fs::metadata(path) {
            Ok(metadata) => {
                //Object found but isn't file. Exit.
                if !metadata.is_file() {
                    return Err(TinkererError::FileNotFound(path_as_string));
                }

                //File found and read data.
                match Tinkerer::read_settings(path){
                    Ok(settings) => {
                        return Ok(Tinkerer{
                            has_player_changes: false,
                            settings: settings
                        })
                    },
                    Err(err) => return Err(err),
                }
            }
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        //File not found. Doesn't matter since it will be created on exit.
                        return Ok(Tinkerer{
                            has_player_changes: false,
                            settings: Settings::default()
                        })
                    }
                    //File found but permission error. Exit
                    std::io::ErrorKind::PermissionDenied => {
                        Err(TinkererError::PermissionDenied(e.to_string()))
                    }
                    //Unknown error. Exit.
                    _ => Err(TinkererError::IOError(e)),
                };
            }
        }
    }

    fn read_settings(path: &Path) -> Result<Settings, TinkererError>{
        // Read the file
        let contents = fs::read_to_string(path)
            .map_err(TinkererError::IOError)?;
        
        // Parse YAML
        let config: Settings = serde_yaml::from_str(&contents)
            .map_err(TinkererError::InvalidFormat)?;
        
        Ok(config)
    }


    fn write_settings(&mut self, path: &Path) -> Result<bool, TinkererError>{
        if self.has_player_changes{
            // Write the file
            let settings = self.settings.clone();
            let content = serde_yaml::to_string(&settings);

            if content.is_ok(){
                match fs::write(path, content.unwrap()){
                    Ok(_) => return Ok(true),
                    Err(err) => return Err(TinkererError::IOError(err)),
                }
            }
        
            return Ok(false)
        }

        return Err(TinkererError::NoChanges)
    }

    pub fn get_audio_settings(&self) -> AudioSettings{
        return self.settings.audio.clone()
    }

    pub fn get_variables(&self) -> VariablesSettings{
        return self.settings.variables.clone()
    }

    ///If changes to settings, write data
    pub fn write(&mut self, sounds: AudioSettings, variables: VariablesSettings) -> Result<bool, TinkererError>{
        self.settings.audio = sounds;
        self.settings.variables = variables;

        let temp_settings = Settings::default();

        if self.settings != temp_settings{
            let path = Path::new(SETTINGS_PATH);
            self.has_player_changes = true;

            return self.write_settings(path);
        }
        return Ok(false)
    }
}

#[derive(Debug)]
pub enum TinkererError{
    FileNotFound(String),
    PermissionDenied(String),
    IOError(std::io::Error),
    InvalidFormat(serde_yaml::Error),
    NoChanges
}

impl std::fmt::Display for TinkererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TinkererError::FileNotFound(path) => write!(f, "File not found: {}", path),
            TinkererError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            TinkererError::IOError(path) => write!(f, "IOError: {}", path),
            TinkererError::InvalidFormat(error) => write!(f, "Invalid format: {}", error),
            TinkererError::NoChanges => write!(f, "Nothing to commit"),
        }
    }
}

impl Error for TinkererError{}



#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Settings{
    audio: AudioSettings,
    variables: VariablesSettings
}
impl Settings{
    fn default() -> Settings{
        return Settings{
            audio: AudioSettings::default(),
            variables: VariablesSettings::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AudioSettings{
    pub (crate) master: f32,
    pub (crate) music: f32,
    pub (crate) effects: f32,
}
impl AudioSettings{
    fn default() -> AudioSettings{
        return AudioSettings {
            master: 1.0, 
            music: 1.0,
            effects: 1.0,
        }
    }

    pub fn get_master_volume(&self) -> f32{
        return self.master
    }

    pub fn get_music_volume(&self) -> f32{
        return self.music
    }

    pub fn get_effects_volume(&self) -> f32{
        return self.effects
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VariablesSettings{
    pub (crate) min_steering_effectiveness: f32,
    pub (crate) max_steering_effectiveness: f32,
    pub (crate) rotation_speed_multiplier: f32,
    pub (crate) steering_force_multiplier: f32,
    pub (crate) acceleration_multiplier: f32,
    pub (crate) velocity_zero_threshold: f32,
    pub (crate) friction: (f32, f32),

    pub (crate) drifting_min_steering_effectiveness: f32,
    pub (crate) drifting_max_steering_effectiveness: f32,
    pub (crate) drifting_rotation_speed_multiplier: f32,
    pub (crate) drifting_steering_force_multiplier: f32,
    pub (crate) drifting_acceleration_multiplier: f32,
    pub (crate) drifting_velocity_zero_threshold: f32,
    pub (crate) drifting_friction: (f32, f32)  
}
impl VariablesSettings{
    fn default() -> VariablesSettings{
        return VariablesSettings { 
            drifting_min_steering_effectiveness: 0.3, 
            drifting_max_steering_effectiveness: 1.2, 
            drifting_rotation_speed_multiplier: 3.0, 
            drifting_steering_force_multiplier: 0.5, 
            drifting_acceleration_multiplier: 0.01, 
            drifting_velocity_zero_threshold: 150.0, 
            drifting_friction: (0.2, 1.8), 
            min_steering_effectiveness: 0.1, 
            max_steering_effectiveness: 1.0, 
            rotation_speed_multiplier: 1.35, 
            steering_force_multiplier: 0.3, 
            acceleration_multiplier: 0.4, 
            velocity_zero_threshold: 100.0, 
            friction: (0.7, 1.0)
        }
    }
}