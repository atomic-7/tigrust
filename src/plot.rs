use std::io::Write;
use svg::node::element::Group;
use svg::node::element::Style;
use svg::node::element::Rectangle;
pub struct Graph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1650,
            height: 1200,
        }
    }
    pub fn plot<W: Write>(&self, writer: W, prices: &Vec<f32>) -> Result<(), std::io::Error> {
        let (min_price, max_price) = prices
            .iter()
            .fold((100.0, -100.0), |(acc_min, acc_max), &p| {
                (f32::min(acc_min, p), f32::max(acc_max, p))
            });
        // 2 cent default step, 5 cent if max > 50 ct and 10 if max > 1.20
        let step = if max_price >= 1.2 {
            0.10
        } else if max_price >= 0.5 {
            0.05
        } else {
            0.02
        };
        let floor = min_price - (min_price % step);
        let ceil = max_price + step - (max_price % step);
        let bottom = 1100;  // y corner + height until the bottom of the bars
        let interval = (self.width as f32 / prices.len() as f32) - 1.0;
        let barspacing = 5;
        let barwidth = interval - barspacing as f32;

        // TOOD: figure out how this style componentn is supposed to work
        let default_style = Style::new("fill:lightgray; stroke:black")
            .set("fill", "lightgray")
            .set("stroke", "black");
        

        let mut bars = Group::new();
        for (idx, price) in prices.iter().enumerate() {
            let scaled_price = map_range(*price, floor, ceil, 0.0, bottom as f32);
            // xcoord := p.x + (p.width-p.w) + idx*int(interval)
            let x_coord = (self.x as f32) + (idx as f32) * interval;
            bars = bars.add(
                Rectangle::new()
                .set("x", x_coord)
                .set("y", (bottom as f32 - scaled_price) as u32)
                .set("height", scaled_price)
                .set("width", barwidth as u32)
            );
        }

        let outer_group = Group::new()
            .add(default_style)
            .add(bars);

        let doc = svg::Document::new()
            // TODO: make the viewbox dynamic, tigraph uses 1100 here
            .set("width", self.width) // height in user coordinate system
            .set("height", self.height)
            .set("viewBox", (0, 0, 1650, 1200)) // viewbox mapped to the size of the svg elem
            // TODO: make an enum for the aspect ratios
            .set("preserveAspectRatio", String::from("xMaxYMin"))
            .add(outer_group);
        
        svg::write(writer, &doc)
    }
}

fn map_range(input: f32, src_low: f32, src_high: f32, dst_low: f32, dst_high: f32) -> f32 {
    dst_low + (dst_high - dst_low) * (input - src_low) / (src_high - src_low)
}
