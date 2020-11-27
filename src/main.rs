mod functions;
use std::str::FromStr;
use druid::widget::prelude::*;
use druid::kurbo::{Rect, Circle, Point};
use druid::widget::{Flex, Label, Switch, Either, Slider};
use druid::{AppLauncher, WindowDesc, Widget, WidgetExt, RenderContext, Data, Lens, Color};

const SPEED: f64 = 0.7;
const SPACING: f64 = 0.01;
const TIMESPACING: f64 = 0.01;
const SI60: f64 = 0.8660254037844386;
const CO60: f64 = 0.5;


struct AnimWidget {
    hex_grid: Vec<Vec<f64>>,
    hex_array_tn_minus_1: Vec<Vec<f64>>,
    hex_array_tn: Vec<Vec<f64>>,
    hex_array_temp: Vec<Vec<f64>>,
    n: u32,
    n_max: u32,
    cmap: [[u8; 3]; 256],
}

impl Widget<ApplicationData> for AnimWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut ApplicationData, _env: &Env) {
        if let Event::MouseDown(_) = event {
            self.n = 0;
            ctx.request_anim_frame();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &ApplicationData, _env: &Env) {
        if let LifeCycle::AnimFrame(_interval) = event {
            self.n += 1;
            if self.n < self.n_max {
                self.next_frame();
                std::thread::sleep(std::time::Duration::from_millis(10));
                ctx.request_anim_frame();
            }
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &ApplicationData, _data: &ApplicationData, _env: &Env) {}

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &ApplicationData, _env: &Env) -> Size {
        bc.constrain((700.0, 700.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &ApplicationData, _env: &Env) {
        let hex_grid = &self.hex_grid;
        let to_draw = &self.hex_array_tn;
        let xr: usize = to_draw[0].len();
        let yr: usize = to_draw.len();
        let mut image_vec: Vec<u8> = vec!(0; xr * yr * 3);
        let flattened: Vec<f64> = to_draw.clone().into_iter().flatten().collect();
        let max_val: f64 = flattened.clone().into_iter().map(|x| x.abs()).collect::<Vec<f64>>().iter().cloned().fold(0./0., f64::max)*1.2;
        for i_y in 0..yr {
            for i_x in 0..xr {
                if hex_grid[i_y][i_x] != 0.0 {
                let cols = functions::determine_color(to_draw[i_y][i_x], &self.cmap, 0.0, 2.0*max_val);
                image_vec[i_y*xr*3 + i_x*3 + 0] = cols[0];
                image_vec[i_y*xr*3 + i_x*3 + 1] = cols[1];
                image_vec[i_y*xr*3 + i_x*3 + 2] = cols[2];
                }
            }
        }
        let img = ctx.make_image(xr, xr, &image_vec, druid::piet::ImageFormat::Rgb).expect("Yekis!");
        ctx.draw_image(&img, Rect{x0: 0.0, y0: 0.0, x1: 700.0, y1: 700.0}, druid::piet::InterpolationMode::Bilinear);
    }
}

impl AnimWidget {
    fn next_frame(&mut self) {
        for i_y in 1..self.hex_grid.len() - 1 {
            for i_x in 1..self.hex_grid[0].len() - 1 {
                if self.hex_grid[i_y][i_x] == 1.0 {
                    if i_y%2 == 0 {
                        self.hex_array_temp[i_y][i_x] = 2.0*self.hex_array_tn[i_y][i_x] - self.hex_array_tn_minus_1[i_y][i_x] + (SPEED*TIMESPACING/SPACING).powi(2)*
                            (2.0/3.0*(self.hex_array_tn[i_y-1][i_x-1] + self.hex_array_tn[i_y-1][i_x] + self.hex_array_tn[i_y][i_x-1] + self.hex_array_tn[i_y][i_x+1] +
                            self.hex_array_tn[i_y+1][i_x-1] + self.hex_array_tn[i_y+1][i_x]) - 4.0*self.hex_array_tn[i_y][i_x]);
                    } else {
                        self.hex_array_temp[i_y][i_x] = 2.0*self.hex_array_tn[i_y][i_x] - self.hex_array_tn_minus_1[i_y][i_x] + (SPEED*TIMESPACING/SPACING).powi(2)*
                            (2.0/3.0*(self.hex_array_tn[i_y-1][i_x] + self.hex_array_tn[i_y-1][i_x+1] + self.hex_array_tn[i_y][i_x-1] + self.hex_array_tn[i_y][i_x+1] +
                            self.hex_array_tn[i_y+1][i_x] + self.hex_array_tn[i_y+1][i_x+1]) - 4.0*self.hex_array_tn[i_y][i_x]);
                    }
                    
                }

            }            
        }
        self.hex_array_tn_minus_1 = self.hex_array_tn.clone();
        self.hex_array_tn = self.hex_array_temp.clone();
    }
}

#[derive(Clone, Data, Lens)]
struct ApplicationData {
    cc_active: bool,
    cc_size: f64,
    window_selection: bool,
}

struct LiveCursor {
    punkt: Point,
}

impl Widget<ApplicationData> for LiveCursor {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut ApplicationData, _env: &Env) {
        if let Event::MouseMove(yekis) = event {
            self.punkt = yekis.pos;
            ctx.request_anim_frame();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &ApplicationData, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &ApplicationData, _data: &ApplicationData, _env: &Env) {}

    fn layout(&mut self, _layout_ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &ApplicationData, _env: &Env) -> Size {
        bc.constrain((700.0, 700.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ApplicationData, _env: &Env) {
        let bounds = ctx.size().to_rect();
        let boxy = Rect{x0: 30.0, x1: 80.0, y0: 30.0, y1: 80.0};
        
        ctx.fill(bounds, &Color::rgb8(36, 146, 36));
        ctx.fill(boxy, &Color::rgb8(136, 16, 36));

        if data.cc_active && ctx.is_hot() {
            let circleboy = Circle{center: self.punkt, radius: data.cc_size}.segment(data.cc_size - 1.0, 0.0, 6.3);
            ctx.fill(circleboy, &Color::rgb8(6, 16, 136));
        }
        
    }
}


fn main() {
    // Loading colormap
    let cmap_path: &str = "C:/Users/Soulyin/Desktop/Rust/data/berlin.csv";
    // Loading txt file with vertices data
    let input_path: &str = "C:/Users/Soulyin/Desktop/Blender/Wave Simulation/edges_data.txt";
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
    let range_x_left: usize = (min_x / SPACING).abs().ceil() as usize + 20;
    let range_x_right: usize = (max_x / SPACING).abs().ceil() as usize + 20;
    let range_y_down: usize = (min_y / (SPACING*SI60)).abs().ceil() as usize + 20;
    let range_y_up: usize = (max_y / (SPACING*SI60)).abs().ceil() as usize + 20;

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
    let mut hex_array_tn = vec!(vec!(0.0; range_x_left + range_x_right + 1); range_y_down + range_y_up + 1);
    hex_array_tn[67*3*3][47*3*3] = 1.0;
    //hex_array_tn[47*3][67*3] = -1.0;

    let animation_window = AnimWidget{hex_grid: hex_grid.clone(),
        hex_array_tn_minus_1: hex_array_tn_minus_1.clone(),
        hex_array_tn: hex_array_tn.clone(),
        hex_array_temp: hex_array_tn.clone(),
        n: 0,
        n_max: 3000,
        cmap: functions::get_cmap(cmap_path)};
    
    let abnormale_closure = || {
        let button_bar = Flex::column()
            .with_spacer(20.0)
            .with_child(Label::new("Configure").with_text_size(12.0))
            .with_child(Label::new("Initial State").with_text_size(12.0))
            .with_spacer(10.0)
            .with_child(Switch::new().lens(ApplicationData::cc_active))
            .with_flex_spacer(1.0)
            .with_child(Label::new("Size").with_text_size(12.0))
            .with_spacer(10.0)
            .with_child(Slider::new().with_range(0.0, 100.0).lens(ApplicationData::cc_size))
            .with_spacer(40.0)
            .expand_height()
            .background(Color::rgb8(20, 20, 20));
        
        let anim_window = Either::new(|data, _env| data.cc_active, LiveCursor{punkt: Point{x: 100.0, y: 100.0}}, animation_window);
                
        Flex::row()
            .with_child(button_bar.fix_width(100.0))
            .with_spacer(1.0)
            .with_flex_child(anim_window, 1.0)
            .background(Color::rgb8(10, 10, 10))
    };
    
    let appdata = ApplicationData{cc_active: false, cc_size: 30.0, window_selection: true};//, anim_widget_data: Rc::new(RefCell::new(animation_window))};

    AppLauncher::with_window(WindowDesc::new(abnormale_closure).title("WaveSim").window_size((850.0, 850.0)).show_titlebar(false)).launch(appdata).expect("memes");
}
