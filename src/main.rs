mod functions;
use std::sync::Arc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use druid::widget::prelude::*;
use druid::kurbo::{Rect, Circle, Point};
use druid::widget::{Flex, Label, Switch, Either, Slider, Padding};
use druid::{AppLauncher, WindowDesc, Widget, WidgetExt, RenderContext, Data, Lens, Color, TimerToken};


const SPEED: f64 = 0.7;
const SPACING: f64 = 0.01;
const TIMESPACING: f64 = 0.01;
const SI60: f64 = 0.8660254037844386;
const CO60: f64 = 0.5;


#[derive(Clone, Data)]
struct SimulationData {
    hex_grid: Arc<Vec<Vec<f64>>>,
    hex_tnm1: Arc<Vec<Vec<f64>>>,
    hex_tn: Arc<Vec<Vec<f64>>>,
    hex_temp: Arc<Vec<Vec<f64>>>,
    n: u32,
    n_max: u32,
    cmap: Arc<[[u8; 3]; 256]>,
}

impl SimulationData {
    fn calc_next_frame(&mut self) {
        {
            let hex_tn = &self.hex_tn;
            let hex_temp = Arc::make_mut(&mut self.hex_temp);

            for i_y in 1..self.hex_grid.len() - 1 {
                for i_x in 1..self.hex_grid[0].len() - 1 {
                    if self.hex_grid[i_y][i_x] == 1.0 {
                        if i_y%2 == 0 {
                            hex_temp[i_y][i_x] = 2.0*hex_tn[i_y][i_x] - self.hex_tnm1[i_y][i_x] + (SPEED*TIMESPACING/SPACING).powi(2)*
                                (2.0/3.0*(hex_tn[i_y-1][i_x-1] + hex_tn[i_y-1][i_x] + hex_tn[i_y][i_x-1] + hex_tn[i_y][i_x+1] +
                                hex_tn[i_y+1][i_x-1] + hex_tn[i_y+1][i_x]) - 4.0*hex_tn[i_y][i_x]);
                        } else {
                            hex_temp[i_y][i_x] = 2.0*hex_tn[i_y][i_x] - self.hex_tnm1[i_y][i_x] + (SPEED*TIMESPACING/SPACING).powi(2)*
                                (2.0/3.0*(hex_tn[i_y-1][i_x] + hex_tn[i_y-1][i_x+1] + hex_tn[i_y][i_x-1] + hex_tn[i_y][i_x+1] +
                                hex_tn[i_y+1][i_x] + hex_tn[i_y+1][i_x+1]) - 4.0*hex_tn[i_y][i_x]);
                        }                    
                    }
                }            
            }
        }
        self.hex_tnm1 = self.hex_tn.clone();
        self.hex_tn = self.hex_temp.clone();
    }

    fn add_initial(&mut self, x_perc_pos: f64, y_perc_pos: f64) {
        let hex_grid = &self.hex_grid; 
        let hex_tn = Arc::make_mut(&mut self.hex_tn);
        let x_len = hex_grid[0].len();
        let y_len = hex_grid.len();
        let x_pos = ((x_len as f64)*x_perc_pos).floor() as usize;
        let y_pos = ((y_len as f64)*y_perc_pos).floor() as usize;
        if hex_grid[y_pos][x_pos] == 1.0 {
            hex_tn[y_pos][x_pos] = 1.0;
        } else {
            println!("Outside");
        }
    }
}

#[derive(Clone, Data, Lens)]
struct AppData {
    edit_active: bool,
    cc_size: f64,
    anim_data: SimulationData,
    anim_iter: u64, // Time in milliseconds
    anim_paused: bool,
    anim_height: f64,
}

struct SimulationWidget {
    timer_id: TimerToken,
    last_update: Instant,
    cell_ratio: f64,
}

impl Widget<AppData> for SimulationWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_paint();
                self.last_update = Instant::now();
            }
            Event::MouseDown(_mouse_event) => {
                data.anim_paused = false;
                let deadline = Duration::from_millis(data.anim_iter);
                self.last_update = Instant::now();
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    if !data.anim_paused {
                        data.anim_data.calc_next_frame();
                        ctx.request_paint();
                    }
                    let deadline = Duration::from_millis(data.anim_iter);
                    self.last_update = Instant::now();
                    self.timer_id = ctx.request_timer(deadline);
                }
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppData, _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                self.cell_ratio = (data.anim_data.hex_grid[0].len() as f64) / (data.anim_data.hex_grid.len() as f64 * SI60);
                ctx.request_layout();
                ctx.request_paint();
                self.last_update = Instant::now();
            }
            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _env: &Env) {
        if old_data.anim_height != data.anim_height {
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, _bc: &BoxConstraints, data: &AppData, _env: &Env) -> Size {
        Size::new(data.anim_height*self.cell_ratio, data.anim_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let hex_grid = &data.anim_data.hex_grid;
        let to_draw = &data.anim_data.hex_tn;
        let xr: usize = to_draw[0].len();
        let yr: usize = to_draw.len();
        let mut image_vec: Vec<u8> = vec!(0; xr * yr * 3);
        let max_val: f64 = functions::get_max_abs(to_draw);
        for i_y in 0..yr {
            for i_x in 0..xr {
                if hex_grid[i_y][i_x] != 0.0 {
                let cols = functions::determine_color(to_draw[i_y][i_x], &data.anim_data.cmap, 0.0, 2.0*max_val);
                image_vec[i_y*xr*3 + i_x*3 + 0] = cols[0];
                image_vec[i_y*xr*3 + i_x*3 + 1] = cols[1];
                image_vec[i_y*xr*3 + i_x*3 + 2] = cols[2];
                }
            }
        }
        let img = ctx.make_image(xr, yr, &image_vec, druid::piet::ImageFormat::Rgb).expect("Yekis!");
        ctx.draw_image(&img, Rect{x0: 0.0, y0: 0.0, x1: data.anim_height*self.cell_ratio, y1: data.anim_height}, druid::piet::InterpolationMode::Bilinear);
    }
}

struct LiveCursor {
    punkt: Point,
    cell_ratio: f64,
}

impl Widget<AppData> for LiveCursor {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseMove(yekis) => {
                self.punkt = yekis.pos;
                ctx.request_anim_frame();
            }
            Event::MouseDown(mouse_event) => {
                let cursor_x_percent_pos: f64 = mouse_event.pos.x / (data.anim_height*self.cell_ratio);
                let cursor_y_percent_pos: f64 = mouse_event.pos.y / data.anim_height;
                data.anim_data.add_initial(cursor_x_percent_pos, cursor_y_percent_pos);
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppData, _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                self.cell_ratio = (data.anim_data.hex_grid[0].len() as f64) / (data.anim_data.hex_grid.len() as f64 * SI60);
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _env: &Env) {
        if old_data.anim_height != data.anim_height {
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, _bc: &BoxConstraints, data: &AppData, _env: &Env) -> Size {
        Size::new(data.anim_height*self.cell_ratio, data.anim_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let hex_grid = &data.anim_data.hex_grid;
        let to_draw = &data.anim_data.hex_tn;
        let xr: usize = to_draw[0].len();
        let yr: usize = to_draw.len();
        let mut image_vec: Vec<u8> = vec!(0; xr * yr * 3);
        let max_val: f64 = functions::get_max_abs(to_draw);
        for i_y in 0..yr {
            for i_x in 0..xr {
                if hex_grid[i_y][i_x] != 0.0 {
                let cols = functions::determine_color(to_draw[i_y][i_x], &data.anim_data.cmap, 0.0, 2.0*max_val);
                image_vec[i_y*xr*3 + i_x*3 + 0] = cols[0];
                image_vec[i_y*xr*3 + i_x*3 + 1] = cols[1];
                image_vec[i_y*xr*3 + i_x*3 + 2] = cols[2];
                }
            }
        }
        let img = ctx.make_image(xr, yr, &image_vec, druid::piet::ImageFormat::Rgb).expect("Yekis!");
        ctx.draw_image(&img, Rect{x0: 0.0, y0: 0.0, x1: data.anim_height*self.cell_ratio, y1: data.anim_height}, druid::piet::InterpolationMode::Bilinear);
        
        if data.edit_active && ctx.is_hot() {
            let circleboy = Circle{center: self.punkt, radius: data.cc_size}.segment(data.cc_size - 1.0, 0.0, 6.3);
            ctx.fill(circleboy, &Color::rgb8(6, 16, 136));
        }        
    }
}


fn build_ui() -> impl Widget<AppData> {
    let button_bar_edit = Flex::column()
        .with_child(Label::new("Size").with_text_size(12.0))
        .with_spacer(10.0)
        .with_child(Slider::new().with_range(0.0, 100.0).lens(AppData::cc_size))
        .with_spacer(40.0);
    
    let button_bar_anim = Flex::column()
        .with_child(Label::new("Window Size").with_text_size(12.0))
        .with_spacer(10.0)
        .with_child(Slider::new().with_range(100.0, 1000.0).lens(AppData::anim_height))
        .with_spacer(40.0);

    let flex_button = Either::new(|data, _env| data.edit_active,
        button_bar_edit,
        button_bar_anim);

    let button_bar = Flex::column()
        .with_spacer(20.0)
        .with_child(Label::new("Configure").with_text_size(12.0))
        .with_child(Label::new("Initial State").with_text_size(12.0))
        .with_spacer(10.0)
        .with_child(Switch::new().lens(AppData::edit_active))
        .with_spacer(30.0)
        .with_child(Label::new("Window Size").with_text_size(12.0))
        .with_spacer(10.0)
        .with_child(Slider::new().with_range(100.0, 1000.0).lens(AppData::anim_height))
        .with_flex_spacer(1.0)
        .with_child(flex_button)
        .background(Color::rgb8(20, 20, 20));

    let cursor_window = LiveCursor{
        punkt: Point{x: 100.0, y: 100.0},
        cell_ratio: 1.0};

    let simu_window = SimulationWidget{
        timer_id: TimerToken::INVALID,
        last_update: Instant::now(),
        cell_ratio: 1.0};

    let anim_window = Flex::column()
        .with_child(Either::new(|data, _env| data.edit_active,
            Padding::new(20.0, cursor_window).background(Color::rgb8(104, 104, 104)),
            Padding::new(20.0, simu_window).background(Color::rgb8(104, 104, 104))
        )).with_flex_spacer(0.0);
    
    
    Flex::row()
        .with_child(button_bar.fix_width(100.0))
        .with_spacer(1.0)
        .with_flex_child(anim_window, 1.0)
        .with_spacer(1.0)
        .with_child(Label::new("Kek"))
        .background(Color::rgb8(10, 10, 10))
}

fn main() {
    // Setting up relative paths
    let exe_path: std::path::PathBuf = std::env::current_exe().expect("Could not locate executable");
    let head_path: String = String::from(exe_path.parent().expect("").parent().expect("").parent().expect("").to_str().expect(""));
    // Loading colormap
    let mut cmap_path_temp: String = head_path.clone();
    cmap_path_temp.push_str("/data/cmaps/berlin.csv");
    let cmap_path: &str = &cmap_path_temp;
    // Loading txt file with vertices data
    let mut input_path_temp: String = head_path.clone();
    input_path_temp.push_str("/data/shapes/edges_data.txt");
    let input_path: &str = &input_path_temp;
    let data_array: Vec<Vec<String>> = functions::csv_parse(input_path, ' ');
    // Setting up float vectors to fill
    let mut x1_array: Vec<f64> = Vec::new();
    let mut y1_array: Vec<f64> = Vec::new();
    let mut x2_array: Vec<f64> = Vec::new();
    let mut y2_array: Vec<f64> = Vec::new();
    // Filling vectors with string vectors data
    for i in 0..data_array.len() {
        x1_array.push(f64::from_str(&data_array[i][0]).unwrap());
        y1_array.push(f64::from_str(&data_array[i][1]).unwrap());
        x2_array.push(f64::from_str(&data_array[i][2]).unwrap());
        y2_array.push(f64::from_str(&data_array[i][3]).unwrap());
    }
    // Find minima and maxima of x and y arays
    let min_x: f64 = functions::min_element_f64(&x1_array, &x2_array);
    let max_x: f64 = functions::max_element_f64(&x1_array, &x2_array);
    let min_y: f64 = functions::min_element_f64(&y1_array, &y2_array);
    let max_y: f64 = functions::max_element_f64(&y1_array, &y2_array);
    // Settung up boundaries for the array
    let range_x_left: usize = (min_x / SPACING).abs().ceil() as usize + 3;
    let range_x_right: usize = (max_x / SPACING).abs().ceil() as usize + 3;
    let range_y_down: usize = (min_y / (SPACING*SI60)).abs().ceil() as usize + 3;
    let range_y_up: usize = (max_y / (SPACING*SI60)).abs().ceil() as usize + 3;

    // Setting up hex grid
    let mut hex_grid: Vec<Vec<f64>> = Vec::with_capacity(range_y_down + range_y_up + 1);
    // Filling hexgrid with 1 inside and 0 outside the boundary
    for i_y in 0..range_y_down + range_y_up + 1 {
        let mut grid_entry: Vec<f64> = Vec::with_capacity(range_x_left + range_x_right + 1);
        let inter = functions::get_vec_intersect((i_y as f64 - range_y_down as f64)*SPACING*SI60, &x1_array, &y1_array, &x2_array, &y2_array);
        // Get coordinates and mark inside and outside
        for i_x in 0..range_x_left + range_x_right + 1 {
            let (x, _y) = functions::get_cord(i_x, i_y, range_x_left, range_y_down);
            let bigger: usize = functions::amount_bigger(x, &inter);
            // Marking inside and outside
            if bigger%2 == 0 {
                grid_entry.push(0.0);
            } else {
                grid_entry.push(1.0);
            }
        }

        hex_grid.push(grid_entry);
    }
    // Assign temporary hex grid
    let mut temp_hex_grid = hex_grid.clone();
    // Fill temporary hex grid with borders (value 2)
    for i in 1..hex_grid.len() - 2 {
        for j in 1..hex_grid[0].len() - 2 {
            if hex_grid[i][j] == 0.0 {
                if functions::neighbour_sum(&hex_grid, i, j) != 0.0 {
                    temp_hex_grid[i][j] = 2.0;
                }
            }
        }
    }
    // Moving temporary back into normal hex grid
    hex_grid = temp_hex_grid;
    // Creating hex array for t0
    let hex_array_tn_minus_1 = vec!(vec!(0.0; range_x_left + range_x_right + 1); range_y_down + range_y_up + 1);
    // Creating hex array for t1
    let hex_array_tn = vec!(vec!(0.0; range_x_left + range_x_right + 1); range_y_down + range_y_up + 1);
    // hex_array_tn[67*3*3][47*3*3] = 1.0;
    // hex_array_tn[47*3][67*3] = -1.0;

    let window = WindowDesc::new(build_ui);

    AppLauncher::with_window(window)
        .launch(AppData {
            edit_active: true,
            cc_size: 30.0,
            anim_data: SimulationData{
                hex_grid: Arc::new(hex_grid),
                hex_tnm1: Arc::new(hex_array_tn_minus_1),
                hex_tn: Arc::new(hex_array_tn.clone()),
                hex_temp: Arc::new(hex_array_tn),
                n: 0,
                n_max: 3000,
                cmap: Arc::new(functions::get_cmap(cmap_path))},
            anim_iter: 50, // Time in milliseconds
            anim_paused: false,
            anim_height: 700.0})
        .expect("launch failed");
}
