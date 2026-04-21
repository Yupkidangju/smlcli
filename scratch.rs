use glob::Pattern;

fn main() {
    let p1 = Pattern::new("*rm *-rf *\\**");
    println!("{:?}", p1);
    let p2 = Pattern::new("*rm *-rf *[*]**");
    println!("{:?}", p2);
    let p3 = Pattern::new("*rm *-rf *\\**");
    println!("{:?}", p3);
}
