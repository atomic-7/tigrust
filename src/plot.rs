use std::io::Write;
use svg::node::element::Group;
use svg::node::element::{Rectangle, Text};
pub struct Graph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub view_width: u32,
    pub view_height: u32,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1650,
            height: 1200,
            view_width: 1650,
            view_height: 1200,
        }
    }
    pub fn plot<W: Write>(&self, writer: W, prices: &Vec<f32>) -> Result<(), std::io::Error> {
        // TODO: make sure the values are integers
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
        let time_big_font = 1.75;   // in em

        // TODO: Evaluate relative units to hopefully get scaling out of the box
        let mut bars = Group::new();
        let mut price_tags = Group::new();
        let mut times = Group::new()
            .set("font-size", "1em");
        for (idx, price) in prices.iter().enumerate() {
            let scaled_price = map_range(*price, floor, ceil, 0.0, bottom as f32);
            // xcoord := p.x + (p.width-p.w) + idx*int(interval)
            let x_coord = (self.x as f32) + (self.width - self.view_width) as f32 + (idx as f32) * interval;
            bars = bars.add(
                Rectangle::new()
                .set("x", x_coord)
                .set("y", (bottom as f32 - scaled_price) as u32)
                .set("height", scaled_price)
                .set("width", barwidth as u32)
            );
            // TODO: adjust the alignment of the text over the bars
            // look at css text-anchor middle and dominant baseline
            price_tags = price_tags.add(
                Text::new(format!("{:.1}", 100. * price))
                .set("x", x_coord + barwidth / 2.0)
                .set("y", bottom as f32 - scaled_price - 10.)
            );
            let mut time = Text::new(idx.to_string())
                .set("x", x_coord + (barwidth / 2.0))
                .set("y", if idx %3 == 0 { bottom + 36 } else { bottom + 31 });
            if idx % 3 == 0 {
                time = time.set("font-size", format!("{}em", time_big_font));
            }
            times = times.add(time);
        }

        let outer_group = Group::new()
            .set("fill", "lightgray")
            .set("stroke", "black")
            .add(bars)
            .add(price_tags)
            .add(times);

        let doc = svg::Document::new()
            // TODO: make the viewbox dynamic, tigraph uses 1100 here
            .set("preserveAspectRatio", String::from("xMaxYMin"))
            .set("viewBox", (0, 0, self.view_width, self.view_height)) // viewbox mapped to the size of the svg elem
            .set("height", self.height)
            .set("width", self.width) // height in user coordinate system
            // TODO: make an enum for the aspect ratios
            .add(outer_group);
        
        svg::write(writer, &doc)
    }
}

fn map_range(input: f32, src_low: f32, src_high: f32, dst_low: f32, dst_high: f32) -> f32 {
    dst_low + (dst_high - dst_low) * (input - src_low) / (src_high - src_low)
}
