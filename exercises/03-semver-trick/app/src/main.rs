fn main() {
    let square = shapes::unit_square();

    // error[E0308]: mismatched types
    //   expected `geom::Point`, found a different `geom::Point`
    println!("{}", render::polyline(&square));
    println!("perimeter: {}", render::perimeter(&square));

    // This one must keep working: `Color` really did change in 0.2.
    println!("{:?}", shapes::favorite_color());
}
