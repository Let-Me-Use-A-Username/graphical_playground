use std::{error::Error, fs::{self, OpenOptions}, io::{BufRead, BufReader, Write}, path::Path};
use macroquad::{miniquad::conf::{Icon, Platform}, window::Conf};
use serde::{Deserialize, Serialize};
use serde_yaml;


const SETTINGS_PATH: &str = "assets\\settings.yaml";
const CONF_PATH: &str = "assets\\conf.yaml";
const SCOREBOARD_PATH: &str = "assets\\scoreboard.txt";

/* 
    Tinkerer struct holds variables that the player can change via the Settings menu.
    Saves changes to `SETTINGS_PATH` whenever changes have been made. 
*/
pub struct Tinkerer{
    has_player_changes: bool,
    settings: Settings,
    pub conf: WindowConf,
    backup: WindowConf
}

impl Tinkerer{
    pub fn new() -> Result<Tinkerer, TinkererError>{
        let path = Path::new(SETTINGS_PATH);
        let path_as_string = SETTINGS_PATH.to_string();
        
        let mut conf = WindowConf::default();

        if let Ok(previous_conf) = Self::read_conf(){
            conf = previous_conf
        }

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
                            settings: settings,
                            conf: conf.clone(),
                            backup: conf
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
                            settings: Settings::default(),
                            conf: conf.clone(),
                            backup: conf
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

    ///Reads settings if exist.
    fn read_settings(path: &Path) -> Result<Settings, TinkererError>{
        // Read the file
        let contents = fs::read_to_string(path)
            .map_err(TinkererError::IOError)?;
        
        // Parse YAML
        let config: Settings = serde_yaml::from_str(&contents)
            .map_err(TinkererError::InvalidFormat)?;
        
        Ok(config)
    }

    pub fn read_conf() -> Result<WindowConf, TinkererError>{
        // Read the file
        let contents = fs::read_to_string(CONF_PATH)
            .map_err(TinkererError::IOError)?;
        
        // Parse YAML
        let config: WindowConf = serde_yaml::from_str(&contents)
            .map_err(TinkererError::InvalidFormat)?;
        
        Ok(config)
    }


    ///Request to write data.
    pub fn write(&mut self, sounds: AudioSettings, variables: VariablesSettings) -> Result<bool, TinkererError>{
        self.settings.audio = sounds;
        self.settings.variables = variables;

        let temp_settings = Settings::default();

        let conf_path = Path::new(CONF_PATH);

        //If changes made to configurationm
        if self.conf != self.backup{
            self.backup = self.conf.clone();
            
            match self.write_conf(conf_path, self.conf.clone()){
                Ok(res) => println!("Wrote config: {}", res),
                Err(err) => eprintln!("Failed writing conf: {}", err),
            }
        }

        //Check if Settings has been modified and write
        if self.settings != temp_settings{
            let settings_path = Path::new(SETTINGS_PATH);

            self.has_player_changes = true;

            return self.write_settings(settings_path);
        }
        return Ok(false)
    }

    fn write_conf(&self, path: &Path, conf: WindowConf) -> Result<bool, TinkererError>{
        // Write the file
        let content = serde_yaml::to_string(&conf);

        if content.is_ok(){
            match fs::write(path, content.unwrap()){
                Ok(_) => return Ok(true),
                Err(err) => return Err(TinkererError::IOError(err)),
            }
        }

        return Err(TinkererError::NoChanges)
    }

    ///Writes settings to path if changes made
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

    pub fn write_score(&mut self, name: String, score: f64){
        let score_path = SCOREBOARD_PATH;

        let fileopt = OpenOptions::new()
            .create(true)
            .append(true)
            .open(score_path);

        if let Ok(mut file) = fileopt{
            if let Ok(res) = file.write(format!("\n{}, {}", name, score.to_string()).as_bytes()){
                eprintln!("Writted data: {}", res);
            }
            else{
                println!("Error during writting data");
            }
        }
        else{
            eprintln!("Failed opening file.")
        }
    }

    pub fn read_score(&mut self) -> Result<Vec<ScoreboardEntry>, TinkererError>{
        let path = SCOREBOARD_PATH;

        let file = OpenOptions::new().read(true).open(path)
        .map_err(TinkererError::IOError)?;
    
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (i, line_result) in reader.lines().enumerate() {
            println!("line: {}", i);
            let line = line_result.map_err(TinkererError::IOError)?;
            let entry = ScoreboardEntry::from_line(&line)?;
            entries.push(entry);
        }

        Ok(entries)
    }


    pub fn get_audio_settings(&self) -> AudioSettings{
        return self.settings.audio.clone()
    }

    pub fn get_variables(&self) -> VariablesSettings{
        return self.settings.variables.clone()
    }
}







/* 
    Temporary object used to serialize scoreboard data
    into struct.

*/

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ScoreboardEntry{
    pub name: String, 
    pub score: f64
}
impl ScoreboardEntry{
    pub fn from_line(line: &str) -> Result<Self, TinkererError> {
        let parts: Vec<&str> = line.trim().split(',').collect();

        if parts.len() != 2 {
            return Err(TinkererError::Unknown("Found only one part on scoreboard entry.".to_string()));
        }

        let name = parts[0].trim().to_string();
        let score = parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| TinkererError::Unknown(format!("Invalid score in line: {}", line)))?;

        Ok(ScoreboardEntry { name, score })
    }
}
impl Eq for ScoreboardEntry {}

impl PartialOrd for ScoreboardEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse to get highest score first
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for ScoreboardEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Use partial_cmp, and unwrap safely since f64 supports it here
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}



/* 
    Temporary object used to freely clone `Conf` file
    required by macroquad.

    Affects windows configuration.
*/
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct WindowConf {
    pub window_title: String,
    pub window_width: i32,
    pub window_height: i32,
    pub high_dpi: bool,
    pub fullscreen: bool,
    pub sample_count: i32,
    pub window_resizable: bool
}
impl WindowConf{
    pub fn default() -> WindowConf{
        return WindowConf{
            window_title: "Geometrical".to_owned(),
            window_height: 1200,
            window_width: 1400,
            window_resizable: true,
            high_dpi: false,
            fullscreen: false,
            sample_count: 1,
        }
    }

    pub fn into_conf(&self, icon: Option<Icon>, platform: Platform) -> Conf{
        return Conf { 
            window_title: self.window_title.clone(), 
            window_width: self.window_width, 
            window_height: self.window_height, 
            high_dpi: self.high_dpi, 
            fullscreen: self.fullscreen, 
            sample_count: self.sample_count, 
            window_resizable: self.window_resizable, 
            icon: icon, 
            platform: platform }
    }
}


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


/*
    Error types to help with file management mostly.
*/
#[derive(Debug)]
pub enum TinkererError{
    FileNotFound(String),
    PermissionDenied(String),
    IOError(std::io::Error),
    InvalidFormat(serde_yaml::Error),
    Unknown(String),
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
            TinkererError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

impl Error for TinkererError{}