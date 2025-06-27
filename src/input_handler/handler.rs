use macroquad::input::{get_last_key_pressed, KeyCode};



#[allow(dead_code)]
pub struct Handler{
    forwards: KeyCode,
    backwards: KeyCode,
    left: KeyCode,
    right: KeyCode,
    boost: KeyCode,
    drift: KeyCode
}
#[allow(dead_code)]
impl Handler{
    pub fn new(){
        if cfg!(target_os = "windows") {
            println!("Running on Windows");
            Handler::platform_init()
        } else if cfg!(target_os = "linux") {
            println!("Running on Linux");
            Handler::platform_init()
        } else if cfg!(target_os = "macos") {
            println!("Running on macOS");
        } else {
            println!("Other OS: {}", std::env::consts::OS);
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_init() {
        println!("Running on windows function");
    }

    #[cfg(target_os = "linux")]
    fn platform_init() {
        println!("Running on linux function");
    }

    pub fn update(){
        if let Some(key) = get_last_key_pressed(){
            match key{
                KeyCode::A =>{
                    println!("A");
                },
                KeyCode::D =>{
                    println!("D");
                },
                KeyCode::W =>{
                    println!("W");
                },
                KeyCode::S =>{
                    println!("S");
                },
                KeyCode::Space =>{
                    println!("Space");
                },
                KeyCode::LeftShift =>{
                    println!("LeftShift");
                },
                KeyCode::Enter =>{
                    println!("Enter");
                },
                KeyCode::Escape =>{
                    println!("Espace");
                },
                _ => println!("Other key: {:?}", key),
            }
        }
    }
}