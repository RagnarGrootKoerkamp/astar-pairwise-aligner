fn main() {
    for v in [1.234, 123.4, 1234.5678] {
        println!("{:>1}", v);
        println!("{:>2}", v);
        println!("{:>3}", v);
        println!("{:>4}", v);
        println!("{:>5}", v);
        println!("{:>6}", v);
        println!("{:>7}", v);
        println!("{:>.1}", v);
        println!("{:>.2}", v);
        println!("{:>.3}", v);
        println!("{:>.4}", v);
        println!("{:>.5}", v);
        println!("{:>.6}", v);
        println!("{:>.7}", v);
        println!("{:>1.}", v);
        println!("{:>2.}", v);
        println!("{:>3.}", v);
        println!("{:>4.}", v);
        println!("{:>5.}", v);
        println!("{:>6.}", v);
        println!("{:>7.}", v);
        println!("{:06.1}", v);
        println!("{:06.2}", v);
        println!("{:06.3}", v);
        println!("{:06.4}", v);
        println!("{:06.5}", v);
        println!("{:06.6}", v);
        println!("{:06.7}", v);
    }
}
