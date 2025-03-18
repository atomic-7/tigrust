use std::io::Write;
use svg::node::element::Group;
use svg::node::element::{Rectangle, Text, Line};
pub struct Graph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub view_width: u32,
    pub view_height: u32,
    pub highlighted: usize,
}

impl Graph {
    // TODO: add flag for daylight savings
    pub fn new(highlight: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1920, 
            height: 1200,
            view_width: 1920, // these dimensions should match the dimensions the plot takes up
            view_height: 1200, // on screenn
            highlighted: highlight,
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
        let margin_left = 30.; // how much space should be kept to the side
        let margin_right = 30.;
        let interval = (((self.width as f32) - margin_left - margin_right) / prices.len() as f32) - 1.0;
        let barspacing = 5;
        let barwidth = interval - barspacing as f32;
        let big_font = 1.75;   // in em

        // TODO: Evaluate relative units to hopefully get scaling out of the box
        let mut bars = Group::new();
        let mut price_tags = Group::new();
        let mut times = Group::new()
            .set("font-size", "1em");
        for (idx, price) in prices.iter().enumerate() {
            let scaled_price = map_range(*price, floor, ceil, 0.0, bottom as f32);
            // xcoord := p.x + (p.width-p.w) + idx*int(interval)
            let x_coord = margin_left + (self.width - self.view_width) as f32 + (idx as f32) * interval;
            let mut bar = 
                Rectangle::new()
                .set("x", x_coord)
                .set("y", (bottom as f32 - scaled_price) as u32)
                .set("height", scaled_price)
                .set("width", barwidth as u32);
            if idx == self.highlighted {
                bar = bar.set("fill", "black");
            }

            bars = bars.add(bar);
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
                time = time.set("font-size", format!("{}em", big_font));
            }
            times = times.add(time);
        }

        //
        let mut help_lines = Group::new()
            .set("stroke", "black")
            .set("stroke-width", 1.0);
        let mut axis_numbers = Group::new()
            .set("font-size", format!("{}em", big_font));
        let mut idx = 0;
        while floor + idx as f32 * step < max_price {
            let line_height = map_range(floor + idx as f32 * step, floor as f32, ceil as f32, self.y as f32, bottom as f32);
            help_lines = help_lines.add(
                Line::new()
                    .set("x1", self.x + (self.width - self.view_width) / 2 + margin_left as u32)
                    .set("y1", bottom as f32 - line_height)
                    .set("x2", self.x + self.view_width - (self.width - self.view_width) / 2 - margin_right as u32)
                    .set("y2", bottom as f32 - line_height)
            );
            axis_numbers = axis_numbers.add(
                Text::new(format!("{:.0}", (floor + idx as f32 * step) * 100.0))
                    .set("x", self.x + 20)
                    .set("y", bottom as f32 - line_height - 10.)
            );
            idx += 1;
        }

        let outer_group = Group::new()
            .set("fill", "lightgray")
            .set("stroke", "black")
            .add(bars)
            .add(price_tags)
            .add(times)
            .add(help_lines)
            .add(axis_numbers);

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
