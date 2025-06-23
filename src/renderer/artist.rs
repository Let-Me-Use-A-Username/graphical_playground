use std::collections::HashMap;

use macroquad::{color::Color, math::Vec2, shapes::{draw_circle, draw_circle_lines, draw_line, draw_poly, draw_poly_lines, draw_rectangle, draw_rectangle_ex, draw_rectangle_lines, draw_triangle, draw_triangle_lines, DrawRectangleParams}, window::clear_background};


type Layer = i32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawType{
    Line,
    Circle,
    CircleLine,
    Rect,
    RectLine,
    RotRect,
    Triangle,
    TriangleLine,
    Polygon,
    PolyLine
}
impl DrawType{
    fn into_vec() -> Vec<DrawType>{
        return vec![DrawType::Rect,
                DrawType::RectLine,
                DrawType::Circle, 
                DrawType::CircleLine, 
                DrawType::Line, 
                DrawType::RotRect, 
                DrawType::Triangle, 
                DrawType::TriangleLine,
                DrawType::Polygon,
                DrawType::PolyLine
                ]
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum DrawCall{
    //x1, y1, x2, y2, thickness, color
    Line(f32, f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Size, color
    Circle(f32, f32, f32, Color),
    //Pos.x, Pos.y, Size, Thickness, Color
    CircleLines(f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color
    Rectangle(f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Thickness, Color
    RectangleLines(f32, f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color, conf
    RotatedRectangle(f32, f32, f32, f32, DrawRectangleParams),
    //V1, V2, V3, color
    Triangle(Vec2, Vec2, Vec2, Color),
    //V1, V2, V3, thickness, color
    TriangleLine(Vec2, Vec2, Vec2, f32, Color),
    //Pos.x, Pos.y , Sides, Radius, Rotation, Color
    Polygon(f32, f32, u8, f32, f32, Color),
    //Pos.x, Pos.y , Sides, Radius, Rotation, Thickness, Color
    PolygonLines(f32, f32, u8, f32, f32, f32, Color),
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
            DrawCall::CircleLines(x, y, size, thickness, color) => {
                draw_circle_lines(*x, *y, *size, *thickness, *color);
            },
            DrawCall::Rectangle(x, y, w, h, color) => {
                draw_rectangle(*x, *y, *w, *h, *color)
            },
            DrawCall::RectangleLines(x, y, w, h, thickness, color) => {
                draw_rectangle_lines(*x, *y, *w, *h, *thickness, *color)
            },
            DrawCall::RotatedRectangle(x, y, w, h, draw_rectangle_params) => {
                draw_rectangle_ex(*x, *y, *w, *h, draw_rectangle_params.clone());
            },
            DrawCall::Triangle(v1, v2, v3, color) => {
                draw_triangle(*v1, *v2, *v3, *color);
            },
            DrawCall::TriangleLine(v1, v2, v3, thickness, color) => {
                draw_triangle_lines(*v1, *v2, *v3, *thickness, *color);
            }
            DrawCall::Polygon(x, y, sides, radius, rotation, color) => {
                draw_poly(*x, *y, *sides, *radius, *rotation, *color);
            },
            DrawCall::PolygonLines(x, y, sides, radius, rotation, thickness, color) => {
                draw_poly_lines(*x, *y, *sides, *radius, *rotation, *thickness, *color);
            },
        }
    }

    #[inline(always)]
    fn get_type(&self) -> DrawType{
        match self{
            DrawCall::Line(_, _, _, _, _, _) => return DrawType::Line,
            DrawCall::Circle(_, _, _, _) => return DrawType::Circle,
            DrawCall::CircleLines(_, _, _, _, _) => return DrawType::CircleLine,
            DrawCall::Rectangle(_, _, _, _, _) => return DrawType::Rect,
            DrawCall::RectangleLines(_, _, _, _, _, _) => return DrawType::RectLine,
            DrawCall::RotatedRectangle(_, _, _, _, _) => return DrawType::RotRect,
            DrawCall::Triangle(_, _, _, _) => return DrawType::Triangle,
            DrawCall::TriangleLine(_, _, _, _, _) => return DrawType::TriangleLine,
            DrawCall::Polygon(_, _, _, _, _, _) => return DrawType::Polygon,
            DrawCall::PolygonLines(_, _, _, _, _, _, _) => return DrawType::PolyLine,
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
    Boss Outline: 6
    Projectiles: 9
    Player: 10

*/
pub struct Artist{
    queue: HashMap<Layer, HashMap<DrawType, Vec<DrawCall>>>,
}

impl Artist{
    pub fn new() -> Artist{
        return Artist{
            queue: HashMap::new(),
        }
    }
    #[inline(always)]
    ///Draws background for given color.
    pub fn draw_background(&self, color: Color){
        clear_background(color);
    }

    /* 
        Draws entities by order of:
            1) Draw type, which is a shape. This optimized GPU sequence.
            2) Layer. In order to provide depth.
            3) Sequence in which the request was made. First to last.
    */
    #[inline(always)]
    pub fn draw(&mut self){
        let mut layers: Vec<i32> = self.queue.keys().cloned().collect();
        layers.sort_by(|a, b| a.cmp(b));
        
        //For draw type
        for draw_type in DrawType::into_vec(){
            //For layer
            for &layer in &layers {
                //Draw
                if let Some(layer_map) = self.queue.get(&layer) {
                    if let Some(calls) = layer_map.get(&draw_type) {
                        for call in calls {
                            call.draw();
                        }
                    }
                }
            }
        }
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
    #[inline(always)]
    pub fn queue_calls(&mut self, calls: Vec<(Layer, DrawCall)>){
        for (layer, call) in calls {
            let drawtype= call.get_type();

            self.add_call(layer, call, drawtype)
        }
    }
}