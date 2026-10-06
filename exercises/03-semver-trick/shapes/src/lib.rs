use geom::{Color, Point};

pub fn unit_square() -> Vec<Point> {
    vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(0.0, 1.0),
    ]
}

pub fn favorite_color() -> Color {
    Color(0xc0, 0xff, 0xee)
}
