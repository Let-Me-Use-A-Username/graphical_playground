use std::collections::{HashMap, VecDeque};

use macroquad::{color::Color, math::Vec2, shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_ex, draw_triangle, DrawRectangleParams}, window::clear_background};

type Layer = i32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawType{
    Line,
    Circle,
    Rect,
    RotRect,
    Triangle
}

#[derive(Clone)]
pub enum DrawCall{
    //x1, y1, x2, y2, thickness, color
    Line(f32, f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Size, color
    Circle(f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color
    Rectangle(f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color, conf
    RotatedRectangle(f32, f32, f32, f32, DrawRectangleParams),
    //V1, V2, V3, color
    Triangle(Vec2, Vec2, Vec2, Color)
}
impl DrawCall{
    #[inline]
    fn draw(&self){
        match self{
            DrawCall::Line(x1, y1, x2, y2, thick, color) => {
                draw_line(*x1, *y1, *x2, *y2, *thick, *color);
            }
            DrawCall::Circle(x, y, size, color) => {
                draw_circle(*x, *y, *size, *color)
            },
            DrawCall::Rectangle(x, y, w, h, color) => {
                draw_rectangle(*x, *y, *w, *h, *color)
            },
            DrawCall::RotatedRectangle(x, y, w, h, draw_rectangle_params) => {
                draw_rectangle_ex(*x, *y, *w, *h, draw_rectangle_params.clone());
            },
            DrawCall::Triangle(v1, v2, v3, color) => {
                draw_triangle(*v1, *v2, *v3, *color);
            }
        }
    }

    #[inline(always)]
    fn get_type(&self) -> DrawType{
        match self{
            DrawCall::Line(_, _, _, _, _, _) => return DrawType::Line,
            DrawCall::Circle(_, _, _, _) => return DrawType::Circle,
            DrawCall::Rectangle(_, _, _, _, _) => return DrawType::Rect,
            DrawCall::RotatedRectangle(_, _, _, _, _) => return DrawType::RotRect,
            DrawCall::Triangle(_, _, _, _) => return DrawType::Triangle,
        }
    }
}

/*
    Artist is a Batch rendering component that assist in the handling
    of draw calls.

    Every draw call is accompanied by the "Layer" it belongs to, lower layer
    means it is drawn earlier and can be "overlapped" by higher layer draws.

    Background: 0
    Grid Background: 1
    Grid Lines: 2
    Wall: 3
    Enemies: 4
    Pickables: 5-7
    Projectiles: 9
    Player: 10

*/
pub struct Artist{
    queue: HashMap<Layer, HashMap<DrawType, Vec<DrawCall>>>,
    // lines: VecDeque<DrawCall>,
    // circles: VecDeque<DrawCall>,
    // rectangles: VecDeque<DrawCall>,
    // rot_rectangles: VecDeque<DrawCall>,
    // triangles: VecDeque<DrawCall>
}

impl Artist{
    pub fn new() -> Artist{
        return Artist{
            queue: HashMap::new(),
            // lines: VecDeque::new(),
            // circles: VecDeque::new(),
            // rectangles: VecDeque::new(),
            // rot_rectangles: VecDeque::new(),
            // triangles: VecDeque::new()
        }
    }
    #[inline(always)]
    ///Draws background for given color.
    pub fn draw_background(&self, color: Color){
        clear_background(color);
    }

    #[inline(always)]
    ///Draws all entities inside each queue.
    pub fn draw(&mut self){
        let mut layers: Vec<i32> = self.queue.keys().cloned().collect();
        layers.sort_by(|a, b| a.cmp(b));

        //Review: Why does this provide increased performance...?
        for draw_type in [DrawType::Rect, DrawType::Circle, DrawType::Line, DrawType::RotRect, DrawType::Triangle] {
            for &layer in &layers {
                if let Some(layer_map) = self.queue.get(&layer) {
                    if let Some(calls) = layer_map.get(&draw_type) {
                        for call in calls {
                            call.draw();
                        }
                    }
                }
            }
        }
        //For each draw layer
        // for layer in layers{
        //     //If an entry exists and has items
        //     if let Some(entry) = self.queue.get(&layer){
        //         //Extend the draw collection based on draw type
        //         for (calltype, calls) in entry{
        //             match calltype{
        //                 DrawType::Line => self.lines.extend(calls.to_owned()),
        //                 DrawType::Circle => self.circles.extend(calls.to_owned()),
        //                 DrawType::Rect => self.rectangles.extend(calls.to_owned()),
        //                 DrawType::RotRect => self.rot_rectangles.extend(calls.to_owned()),
        //                 DrawType::Triangle => self.triangles.extend(calls.to_owned()),
        //             }
        //         }
        //     }
        //     //Once the layer has been broken down into collections, draw them all            
        //     while let Some(line) = self.lines.pop_front(){
        //         line.draw();
        //     }

        //     while let Some(circle) = self.circles.pop_front(){
        //         circle.draw();
        //     }

        //     while let Some(rectangle) = self.rectangles.pop_front(){
        //         rectangle.draw();
        //     }
    
        //     while let Some(rot_rectangle) = self.rot_rectangles.pop_front(){
        //         rot_rectangle.draw();
        //     }
    
        //     while let Some(triangle) = self.triangles.pop_front(){
        //         triangle.draw();
        //     }
    
        //     let len = 
        //         self.lines.len() + 
        //         self.circles.len() + 
        //         self.rectangles.len() + 
        //         self.rot_rectangles.len() +
        //         self.triangles.len();
            
        //     if len != 0{
        //         self.lines.clear();
        //         self.circles.clear();
        //         self.rectangles.clear();
        //         self.rot_rectangles.clear();
        //         self.triangles.clear();
        //     }
        // }
        if self.queue.len() != 0{
            self.queue.clear();
        }
    }

    #[inline(always)]
    ///Add a single draw call.
    pub fn add_call(&mut self, layer: i32, call: DrawCall, drawtype: DrawType){
        //Take entry or insert blank
        let layer = self.queue
            .entry(layer)
            .or_insert_with(|| {
                let hashmap: HashMap<DrawType, Vec<DrawCall>> = HashMap::new();
                hashmap
            });
        
        //Take DrawCall entry or insert blank
        let calls = layer.entry(drawtype)
            .or_insert_with(|| Vec::new());
        
        //Push new DrawCall
        calls.push(call);
    }

    ///Add batch of different call types to each queue.
    /// Better approach for components that have complex draw calls like the grid.
    pub fn queue_calls(&mut self, calls: Vec<(Layer, DrawCall)>){
        for (layer, call) in calls {
            let drawtype= call.get_type();

            self.add_call(layer, call, drawtype)
        }
    }
}