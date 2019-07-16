//Copy derived trait only works on types with statically-sized length
#[derive(Debug, Copy, Clone)]
struct PointCopy {
    x: u32,
    y: u32
}

#[derive(Debug)]
struct Point {
    x: u32,
    y: u32
}

fn take_ownership(p:Point) {

}

fn take_ownership_copy(p:PointCopy) {

}


fn main() {
    let x = PointCopy { x: 0, y: 0 };
    take_ownership_copy(x);
    println!("{:?}", x);
}