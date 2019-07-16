#[derive(Debug)]
struct Point {
    x: u32,
    y: u32,
    name: String
}

fn get_points() -> Vec<Point> {
    let mut pts = Vec::new();
    pts.push(Point { x: 0, y: 0, name: "skb".to_owned()});
    pts
}

fn main() {
    println!("{:?}", get_points());
}


