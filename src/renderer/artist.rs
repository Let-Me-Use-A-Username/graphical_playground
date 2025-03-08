use std::collections::VecDeque;

use macroquad::{color::Color, math::Vec2, shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_ex, draw_triangle, DrawRectangleParams}, window::clear_background};

#[derive(Clone, Copy, PartialEq, Eq)]
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

pub struct Artist{
    lines: VecDeque<DrawCall>,
    circles: VecDeque<DrawCall>,
    rectangles: VecDeque<DrawCall>,
    rot_rectangles: VecDeque<DrawCall>,
    triangles: VecDeque<DrawCall>
}

impl Artist{
    pub fn new() -> Artist{
        return Artist{
            lines: VecDeque::new(),
            circles: VecDeque::new(),
            rectangles: VecDeque::new(),
            rot_rectangles: VecDeque::new(),
            triangles: VecDeque::new()
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

        while let Some(rectangle) = self.rectangles.pop_front(){
            rectangle.draw();
        }

        while let Some(circle) = self.circles.pop_front(){
            circle.draw();
        }
        
        while let Some(line) = self.lines.pop_front(){
            line.draw();
        }

        while let Some(rot_rectangle) = self.rot_rectangles.pop_front(){
            rot_rectangle.draw();
        }

        while let Some(triangle) = self.triangles.pop_front(){
            triangle.draw();
        }

        let len = 
            self.lines.len() + 
            self.circles.len() + 
            self.rectangles.len() + 
            self.rot_rectangles.len() +
            self.triangles.len();
        
        if len != 0{
            eprintln!("Queue not clear after drawing all components");
            self.lines.clear();
            self.circles.clear();
            self.rectangles.clear();
            self.rot_rectangles.clear();
            self.triangles.clear();
        }
    }

    #[inline(always)]
    ///Add a single draw call.
    pub fn add_call(&mut self, call: DrawCall, drawtype: DrawType){
        match drawtype{
            DrawType::Line => self.lines.push_back(call),
            DrawType::Circle => self.circles.push_back(call),
            DrawType::Rect => self.rectangles.push_back(call),
            DrawType::RotRect => self.rot_rectangles.push_back(call),
            DrawType::Triangle => self.triangles.push_back(call),
        }
    }

    #[inline(always)]
    ///Add batch of same call types.
    pub fn add_batch_calls(&mut self, calls: Vec<DrawCall>, drawtype: DrawType){
        match drawtype{
            DrawType::Line => self.lines.extend(calls),
            DrawType::Circle => self.circles.extend(calls),
            DrawType::Rect => self.rectangles.extend(calls),
            DrawType::RotRect => self.rot_rectangles.extend(calls),
            DrawType::Triangle => self.triangles.extend(calls),
        }
    }

    ///Add batch of different call types to each queue.
    /// Better approach for components that have complex draw calls like the grid.
    pub fn queue_calls(&mut self, calls: Vec<DrawCall>){
        for call in calls {
            match call {
                DrawCall::Line(_, _, _, _, _, _) => self.lines.push_back(call),
                DrawCall::Circle(_, _, _, _) => self.circles.push_back(call),
                DrawCall::Rectangle(_, _, _, _, _) => self.rectangles.push_back(call),
                DrawCall::RotatedRectangle(_, _, _, _, _) => self.rot_rectangles.push_back(call),
                DrawCall::Triangle(_, _, _, _) => self.triangles.push_back(call),
            }
        }
    }
}