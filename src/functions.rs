use std::sync::Arc;
use std::str::FromStr;

const SPACING: f64 = 0.01;
const SI60: f64 = 0.8660254037844386;
const CO60: f64 = 0.5;

/// Returns x and y coordinates if given indices and the offsets
/// 
/// range x left and range y down
pub fn get_cord(i_x: usize, i_y: usize, x_off: usize, y_off: usize) -> (f64, f64) {
    if i_y%2 == 0 {
        return ((i_x as f64 - x_off as f64) * SPACING, (i_y as f64 - y_off as f64) * SPACING * SI60);
    } else {
        return ((i_x as f64 - x_off as f64) * SPACING + CO60 * SPACING, (i_y as f64 - y_off as f64) * SPACING * SI60);
    }
}

pub fn get_slope(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    return (y2 - y1)/(x2 - x1)
}

pub fn get_offset(x1: f64, y1: f64, m: f64) -> f64 {
    return y1 - m*x1
}

/// Returns the coordinates of the interception of two linear functions:
/// 
/// f1(x)=ax+b
/// 
/// f2(x)=cx+d
/// 
/// where f2 has a slope of zero, and therefore c = 0.0
pub fn get_inter(x1: f64, y1: f64, x2: f64, y2: f64, offset: f64) -> (f64, f64) {
    let a: f64 = get_slope(x1, y1, x2, y2);
    let c: f64 = 0.0;

    let b: f64 = get_offset(x1, y1, a);
    let d: f64 = offset;

    let x_res: f64 = (d - b)/(a- c);
    let y_res: f64 = a*x_res + b;

    return (x_res, y_res)
}

/// Returns a sorted list of all intersections on a horizontal line with a
/// certain offset
/// 
/// - y in the tuple is unused, but could be used to make it more secure
pub fn get_vec_intersect(y_offset: f64, x1: &Vec<f64>, y1: &Vec<f64>, x2: &Vec<f64>, y2: &Vec<f64>) -> Vec<f64> {
    let mut res: Vec<f64> = Vec::new();
    
    for i in 0..x1.len() {
        let (x, _y) = get_inter(x1[i], y1[i], x2[i], y2[i], y_offset);
        if ((x1[i] <= x) && (x <= x2[i])) || ((x2[i] <= x) && (x <= x1[i])) {
            res.push(x);
        }
    }

    res.sort_by(|a, b| a.partial_cmp(b).unwrap());
    return res
}

pub fn max_element_f64(vec_1: &Vec<f64>, vec_2: &Vec<f64>) -> f64 {
    let max_v1: f64 = vec_1.iter().cloned().fold(0./0., f64::max);
    let max_v2: f64 = vec_2.iter().cloned().fold(0./0., f64::max);
    let max: f64 ;
    
    if max_v1 < max_v2 {
        max = max_v1;
    } else {
        max = max_v2;
    }
    return max
}

pub fn min_element_f64(vec_1: &Vec<f64>, vec_2: &Vec<f64>) -> f64 {
    let min_v1: f64 = vec_1.iter().cloned().fold(0./0., f64::min);
    let min_v2: f64 = vec_2.iter().cloned().fold(0./0., f64::min);
    let min: f64 ;
    
    if min_v1 < min_v2 {
        min = min_v1;
    } else {
        min = min_v2;
    }
    return min
}

/// Returns the amount of elements in a Vec that are bigger than the input value
pub fn amount_bigger(value: f64, vector: &Vec<f64>) -> usize {
    let mut count: usize = 0;
    for entry in vector {
        if value < *entry {
            count += 1;
        }
    }
    return count
}

/// Returns the sum of the neighbour entries of the hex grid
pub fn neighbour_sum(hex_grid: &Vec<Vec<f64>>, i: usize, j:usize) -> f64 {
    let sum: f64;
    if i%2 == 0 {
        sum = hex_grid[i-1][j-1] + hex_grid[i][j-1] + hex_grid[i-1][j] + hex_grid[i+1][j] + hex_grid[i-1][j+1] + hex_grid[i][j+1];
    } else {
        sum = hex_grid[i][j-1] + hex_grid[i+1][j-1] + hex_grid[i-1][j] + hex_grid[i+1][j] + hex_grid[i][j+1] + hex_grid[i+1][j+1];
    }
    return sum
}

/// Determines color depending on the value and a cmap
pub fn determine_color(value: f64, cmap: &[[u8; 3]; 256], base_level: f64, range: f64) -> [u8; 3] {
    let norm_value: f64 = (value - base_level) / range + 0.5;
    let mut index: usize = (norm_value*255.0).round() as usize;
    if index > 255 {index = 255}
    return [cmap[index][0], cmap[index][1], cmap[index][2]]
}

/// Parses CSV into a string vector
pub fn csv_parse(path: &str, delimiter: char) -> Vec<Vec<String>> {
    // Setting up error message
    let mut error_message: String = String::from_str("Something went wrong reading the following file: ").expect("Could not parse");
    error_message.push_str(path);
    // Loading file with data
    let content = std::fs::read_to_string(path).expect(&error_message);
    // Setting up string vectors to fill
    let mut data_array: Vec<Vec<String>> = Vec::new();
    let mut string_array: Vec<String> = Vec::new();
    let mut string: String = "".to_string();
    // Filling vectors with data
    for character in content.chars() {
        if character == delimiter {
            string_array.push(string.clone());
            string = "".to_string();
        } else if character == '\n' {
            string_array.push(string.clone());
            string = "".to_string();
            data_array.push(string_array.clone());
            string_array = Vec::new();
        } else {
            string.push(character)
        }
    }
    return data_array
}

/// Returns the colormap used for visualisation
pub fn get_cmap(cmap_path: &str) -> [[u8; 3]; 256] {
    // Loading file with cmap data
    let data_array: Vec<Vec<String>> = csv_parse(cmap_path, ',');
    // Setting up u8 array to fill
    let mut rgb_array: [[u8; 3]; 256] = [[0; 3]; 256];
    // Filling array with string data
    for i in 0..data_array.len() {
        rgb_array[i][0] = (f64::from_str(&data_array[i][1]).unwrap()*255.0).round() as u8;
        rgb_array[i][1] = (f64::from_str(&data_array[i][2]).unwrap()*255.0).round() as u8;
        rgb_array[i][2] = (f64::from_str(&data_array[i][3]).unwrap()*255.0).round() as u8;
    }
    return rgb_array
}

/// Returns the maximum absolute value of an Arc<Vec<Vec<f64>>>
pub fn get_max_abs(obj: &Arc<Vec<Vec<f64>>>) -> f64 {
    let len_outer: usize = obj.len();
    let len_inner: usize = obj[0].len();
    let mut current_max: f64 = 0.0;
    
    for i1 in 0..len_outer {
        for i2 in 0..len_inner{
            if obj[i1][i2].abs() > current_max {
                current_max = obj[i1][i2].abs();
            }
        }
    }
    current_max
}

/// Returns the value of a Gaussian
/// 
/// - height: peak height
/// - stdv: standard deviation
/// - x: deviation from the peak
pub fn gaussian(height: f64, stdv: f64, x: f64) -> f64 {
    height*(-1.0*x.powi(2) / (2.0*stdv.powi(2))).exp()
}

/// Gives the distance of two points on the grid
pub fn grid_distance(i_x_1: usize, i_y_1: usize, i_x_2: usize, i_y_2: usize, x_off: usize, y_off: usize) -> f64 {
    let (x_1, y_1) = get_cord(i_x_1, i_y_1, x_off, y_off);
    let (x_2, y_2) = get_cord(i_x_2, i_y_2, x_off, y_off);
    ((x_1 - x_2).powi(2) + (y_1 - y_2).powi(2)).sqrt()
}