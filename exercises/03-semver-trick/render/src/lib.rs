use geom::Point;

/// Renders points as an SVG `<polyline>`.
pub fn polyline(points: &[Point]) -> String {
    let coords: Vec<String> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
    format!(r#"<polyline points="{}" />"#, coords.join(" "))
}

pub fn perimeter(points: &[Point]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.distance(b))
        .sum()
}
