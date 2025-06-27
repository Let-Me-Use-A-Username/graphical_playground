use macroquad::input::{get_last_key_pressed, is_key_down, KeyCode};



pub struct Handler{
    forwards: KeyCode,
    backwards: KeyCode,
    left: KeyCode,
    right: KeyCode,
    boost: KeyCode,
    drift: KeyCode
}
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
        // if is_key_down(KeyCode::A){
        //     println!("Left")
        // }
        // if is_key_down(KeyCode::D){
        //     println!("Right")
        // }
        // if is_key_down(KeyCode::W){
        //     println!("Forwards")
        // }
        // if is_key_down(KeyCode::S){
        //     println!("Backwards")
        // }
        // if is_key_down(KeyCode::LeftShift){
        //     println!("Left Shift")
        // }
        // if is_key_down(KeyCode::Space){
        //     println!("Space")
        // }
        // if is_key_down(KeyCode::Escape){
        //     println!("Escape")
        // }
        // if is_key_down(KeyCode::Enter){
        //     println!("Enter")
        // }
        if let Some(key) = get_last_key_pressed(){
            match key{
                _ => println!("Other key: {:?}", key),
            }
        }
    }
}