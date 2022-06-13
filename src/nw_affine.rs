use crate::prelude::*;

const INF: usize = usize::MAX / 2;

fn print_vec(A: &Vec<Vec<usize>>) {
    //This function prints matrix for debug purposes
    for j in 0..A[0].len() {
        for i in 0..A.len() {
            if A[i][j] > INF / 2 {
                print!("inf");
            } else {
                print!("{}", A[i][j]);
            }
            print!("\t");
        }
        print!("\n");
    }
    print!("\n");
}

fn print_vec2(A: &Vec<Vec<usize>>) {
    //This function prints matrix for debug purposes
    for i in 0..A.len() {
        for j in 0..A[i].len() {
            if A[i][j] > INF / 2 {
                print!("inf");
            } else {
                print!("{}", A[i][j]);
            }
            print!("\t");
        }
        print!("\n");
    }
    print!("\n");
}

pub fn nw_affine(s1: &Sequence, s2: &Sequence) -> (usize) //Needleman-Wunsch algorithm with affine gap penalty
{
    const A: usize = 1; //openning gap cost
    const B: usize = 1; //extapanding gap cost
    const mis_match: usize = 1; //the function implies that match cost is ZERO
    let mut t = vec![vec![INF; s2.len() + 1]; s1.len() + 1];
    let mut u = vec![vec![INF; s2.len() + 1]; s1.len() + 1];
    let mut s = vec![vec![INF; s2.len() + 1]; s1.len() + 1];
    s[0][0] = 0;
    t[0][0] = 0;
    u[0][0] = 0;
    for i in 1..=s1.len() {
        t[i][0] = A + i * B;
        s[i][0] = t[i][0];
    }
    for i in 1..=s2.len() {
        u[0][i] = A + i * B;
        s[0][i] = u[0][i];
    }
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            t[i][j] = min(t[i - 1][j] + B, s[i - 1][j] + A + B);
            u[i][j] = min(u[i][j - 1] + B, s[i][j - 1] + A + B);
            s[i][j] = min(
                min(t[i][j], u[i][j]),
                s[i - 1][j - 1] + if s1[i - 1] == s2[j - 1] { 0 } else { mis_match },
            );
        }
    }

    // print_vec(&t);
    // print_vec(&u);
    // print_vec(&s);

    return s[s1.len()][s2.len()];
}

fn explore_diagonal(
    //Function for exploring (checking matches on the diagonal). Only for standard DTM and compatible algorithms!
    s1: &Sequence,
    s2: &Sequence,
    A: &mut isize, // A = A[j][i]
    i: usize,
    j: usize,
) {
    if *A < 0 {
        return;
    }
    let d = 1 + i * 2;
    let mut x = *A as usize;
    let mut y = x + d / 2 - j;
    while x < s1.len() && y < s2.len() {
        if s1[x] == s2[y] {
            *A += 1;
            x += 1;
            y += 1;
        } else {
            break;
        }
    }
}

pub fn diagonal_transition_affine<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    //Diagonal transition method. Saves all the staes, so we can track the path
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

    const NEG: isize = isize::MIN;
    let mut M = vec![vec![]]; // Main layer
    let mut I = vec![vec![]]; // Insertion layer
    let mut D = vec![vec![]]; // Deletion layer
    let mut w = 1;
    const x: usize = 1; // mismatch cost
    const o: usize = 1; // open gap cost
    const e: usize = 1; // extand gap cost

    let get_j = |s: usize, d: usize, k: isize| -> Option<(usize, usize)> {
        if s < d {
            return None;
        }
        let p = (s - d) as isize + k;
        if p < 0 || p as usize > (s - d) * 2 {
            return None;
        }
        Some((s - d, p as usize))
    };

    M[0] = vec![0isize; w];
    I[0] = vec![NEG; w];
    D[0] = vec![NEG; w];
    explore_diagonal(s1, s2, &mut M[0][0], 0, 0);
    if s2.len() == s1.len() && M[0][0] as usize >= s1.len() {
        return 0;
    }
    for s in 1..=s2.len() {
        w += 2;
        M.push(vec![NEG; w]); //Probably it's better to store all three values in one object - less memory allocations - more speed
        I.push(vec![NEG; w]);
        D.push(vec![NEG; w]);
        let mut k: isize = -(w as isize / 2);
        for j in 0..w {
            I[s][j] = NEG;
            if let Some((i1, j1)) = get_j(s, o + e, k - 1) {
                I[s][j] = M[i1][j1] + 1;
            }
            if let Some((i1, j1)) = get_j(s, e, k - 1) {
                I[s][j] = max(I[i1][j1] + 1, I[s][j]);
            }
            D[s][j] = NEG;
            if let Some((i1, j1)) = get_j(s, o + e, k + 1) {
                D[s][j] = M[i1][j1];
            }
            if let Some((i1, j1)) = get_j(s, e, k + 1) {
                D[s][j] = max(D[i1][j1], D[s][j]);
            }
            // I[s][j] = max(M[s - o - e][k - 1], I[s - e][k - 1]) + 1;

            // D[s][j] = max(M[s - o - e][k + 1], D[s - e][k + 1]);

            M[s][j] = max(I[s][j], D[s][j]);

            if let Some((i1, j1)) = get_j(s, x, k) {
                // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                M[s][j] = max(M[i1][j1] + 1, M[s][j]);
            }

            // if M[s][j] > 0 || (j > 0 && j < M[s].len() - 1) {
            explore_diagonal(s1, s2, &mut M[s][j], s, j);
            // }

            if j as isize - (w / 2) as isize == s1.len() as isize - s2.len() as isize {
                if M[s][j] >= s1.len() as isize {
                    // print_vec2(&M);
                    // print_vec2(&I);
                    // print_vec2(&D);
                    // println!("A {s}\t{j}\n");
                    return s;
                }
            }
            k += 1;
        }
    }
    unreachable!("Error! Shouldn't be here!");
}

// if a(
//     D[i][((w / 2) as isize + k_max + 1) as usize],
//     Mr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
//     s1.len() + 1,
// ) {
//     f = 10;
// }
// if a(
//     M[i][((w / 2) as isize + k_max + 1) as usize],
//     Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
//     s1.len() + 1,
// ) {
//     f = 11;
// }
// if a(
//     D[i][((w / 2) as isize + k_max + 1) as usize],
//     Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
//     s1.len() + 1,
// ) {
//     return 2 * i - 1 - o;
// }

fn explore_diagonal2(
    //Function for exploring (checking matches on the diagonal). Only for standard DTM and compatible algorithms!
    s1: &Sequence,
    s2: &Sequence,
    A: &mut isize, // A = A[i][j]
    i: usize,
    j: usize,
    d: isize,
) {
    if *A < 0 || (d == -1 && s1.len() < (*A) as usize + 1) {
        return;
    }
    let k0: isize = s1.len() as isize - s2.len() as isize;
    let w: isize = if d == 1 {
        1 + (i as isize) * 2
    } else {
        -(j as isize) + i as isize + k0
    };
    let mut x = if d == 1 {
        *A as usize
    } else {
        s1.len() - (*A) as usize - 1
    };
    let mut y = if d == 1 {
        x + w as usize / 2 - j
    } else {
        (x as isize - w) as usize
    };
    while (d == 1 && x < s1.len() && y < s2.len()) || (d == -1) {
        if s1[x] == s2[y] {
            *A += 1;
            if d == -1 && (x == 0 || y == 0) {
                break;
            }
            x = (x as isize + d) as usize;
            y = (y as isize + d) as usize;
        } else {
            break;
        }
    }
}

pub fn biwfa_affine2<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    //Bug at n == 4, k = 0.3
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let k0: isize = s1.len() as isize - s2.len() as isize;
    let len1 = s1.len();
    let len2 = s2.len();

    //Affine cost constants
    const x: usize = 1; // mismatch cost
    const o: usize = 1; // open gap cost
    const e: usize = 1; // extand gap cost

    //procedures

    let cmp = |a1: isize, b1: isize, c1: usize| -> bool {
        if a1 > 0 && b1 > 0 && a1 + b1 == c1 as isize {
            return true;
        }
        return false;
    };

    let is_inside = |num: isize, from: isize, to: isize| -> bool {
        return num >= from && num <= to;
    };

    let get_j = |s: usize, d: usize, k: isize| -> Option<(usize, usize)> {
        if s < d {
            return None;
        }
        let p = (s - d) as isize + k;
        if p < 0 || p as usize > (s - d) * 2 {
            return None;
        }
        Some((s - d, p as usize))
    };

    let check_point = |set: &mut (Vec<Vec<isize>>, Vec<Vec<isize>>, Vec<Vec<isize>>),
                       s: usize,
                       j: usize,
                       k: isize|
     -> isize {
        let mut t = NEG;
        let (M, I, D) = set;
        if s > 0 {
            t = max(I[s][j], D[s][j]);

            if let Some((i1, j1)) = get_j(s, x, k) {
                // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                t = max(M[i1][j1] + 1, M[s][j]);
            }
        } else {
            return 0;
        }
        t
    };

    let compare = |set1: &mut (Vec<Vec<isize>>, Vec<Vec<isize>>, Vec<Vec<isize>>),
                   s1: usize,
                   set2: &mut (
        //set2 is reverse!!! Do not swap set1 and set2! They have different allocations!
        Vec<Vec<isize>>,
        Vec<Vec<isize>>,
        Vec<Vec<isize>>,
    ),
                   s2: usize|
     -> (bool, usize) {
        //bool - do these layers cover each other?; usize - if they do, how much do we need to substruct? (0 or o (open cost))

        let k_min1 = -(s1 as isize);
        let k_max1 = (s1 as isize);
        let k_min2 = -(s2 as isize) + k0;
        let k_max2 = (s2 as isize) + k0;
        let k_min = max(k_min1, k_min2);
        let k_max = min(k_max1, k_max2);
        let mut j1 = (s1 as isize + k_min) as usize;
        let mut j2 = (s2 as isize - k_min + k0) as usize;
        let mut f = false;
        if k_min - k_max <= 1 {
            let (M, I, D) = set1;
            let (Mr, Ir, Dr) = set2;
            // print!("\n{s1}\t{s2}\t{k_max}\t{k_min}\t{k0}\n");
            if ((s1 as isize + k_max + 1) as usize) < D[s1].len() - 1
                && cmp(
                    D[s1][(s1 as isize + k_max + 1) as usize],
                    Dr[s2][(s2 as isize - k_max + k0) as usize],
                    len1 + 1,
                )
            {
                return (true, o);
            }
            if ((s1 as isize + k_max + 1) as usize) < D[s1].len() - 1
                && (cmp(
                    M[s1][(s1 as isize + k_max + 1) as usize],
                    Dr[s2][(s2 as isize - k_max + k0) as usize],
                    len1 + 1,
                ) || cmp(
                    D[s1][(s1 as isize + k_max + 1) as usize],
                    Mr[s2][(s2 as isize - k_max + k0) as usize],
                    len1 + 1,
                ))
            {
                f = true;
            }
        }
        for k in k_min..=k_max {
            let (M, I, D) = set1;
            let (Mr, Ir, Dr) = set2;
            //comparing...
            if j2 + 1 < 1 + s2 * 2 {
                if cmp(D[s1][j1], Dr[s2][j2 + 1], len1 + 1) {
                    return (true, o);
                }
                // if cmp(M[s1][j1], Dr[s2][j2 + 1], len1 + 1)
                //     || cmp(D[s1][j1], Mr[s2][j2 + 1], len1 + 1)
                // {
                //     f = true;
                // }
            }
            if k > k_min {
                if cmp(I[s1][j1 - 1], Ir[s2][j2], len1 + 1) {
                    return (true, o);
                }
                // if cmp(M[s1][j1 - 1], Ir[s2][j2], len1 + 1)
                //     || cmp(I[s1][j1 - 1], Mr[s2][j2], len1 + 1)
                // {
                //     f = true;
                // }
            }

            let t1 = check_point(set1, s1, j1, k);
            let t2 = check_point(set2, s2, j2, -(k - k0));
            let (M, I, D) = set1;
            let (Mr, Ir, Dr) = set2;

            // println!("CMP:\t{k}\t{s1}\t{s2}\t{}\t{}\n", M[s1][j1], Mr[s2][j2]);
            // println!("{t2}");
            if cmp(M[s1][j1], Mr[s2][j2], len1)
                || (M[s1][j1] > 0
                    && Mr[s2][j2] > 0
                    && ((is_inside((len1 + 1) as isize - Mr[s2][j2], t1, M[s1][j1])
                        || is_inside(
                            M[s1][j1],
                            (len1 + 1) as isize - Mr[s2][j2],
                            (len1 + 1) as isize - t2,
                        ))
                        || M[s1][j1] + M[s2][j2] == len1 as isize))
            {
                f = true;
            }

            j1 += 1;
            if j2 > 0 {
                j2 -= 1;
            }
        }
        return (f, 0);
    };

    let expand_layer =
        |set: &mut (Vec<Vec<isize>>, Vec<Vec<isize>>, Vec<Vec<isize>>), s: usize, d: isize| -> () {
            //d == 1 for forward fron; d == -1 for reverse
            let (M, I, D) = set;
            let w = 1 + s * 2;
            //memory allocations
            if s > 0 {
                M.push(vec![NEG; w]);
                I.push(vec![NEG; w]);
                D.push(vec![NEG; w]);
                M[s] = vec![NEG; w];
            } else {
                M[s] = vec![0isize; w];
            }
            I[s] = vec![NEG; w];
            D[s] = vec![NEG; w];
            let mut k = -d * (w as isize / 2);
            for j in 0..w {
                if let Some((i1, j1)) = get_j(s, o + e, (k - d) * d) {
                    I[s][j] = M[i1][j1] + 1;
                }
                if let Some((i1, j1)) = get_j(s, e, (k - d) * d) {
                    I[s][j] = max(I[i1][j1] + 1, I[s][j]);
                }

                if let Some((i1, j1)) = get_j(s, o + e, (k + d) * d) {
                    D[s][j] = M[i1][j1];
                }
                if let Some((i1, j1)) = get_j(s, e, (k + d) * d) {
                    D[s][j] = max(D[i1][j1], D[s][j]);
                }

                if s > 0 {
                    M[s][j] = max(I[s][j], D[s][j]);

                    if let Some((i1, j1)) = get_j(s, x, k * d) {
                        // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                        M[s][j] = max(M[i1][j1] + 1, M[s][j]);
                    }
                }

                explore_diagonal2(s1, s2, &mut M[s][j], s, j, d);
                k += d;
            }
        };

    let print_vector = |C: &Vec<Vec<isize>>| -> () {
        print!("\n");
        for i in 0..C.len() {
            for j in 0..C[i].len() {
                print!("{} ", C[i][j]);
            }
            print!("\n");
        }
        print!("\n");
    };

    //intialization
    const NEG: isize = isize::MIN;
    let mut M = vec![vec![]]; // Main layer
    let mut I = vec![vec![]]; // Insertion layer
    let mut D = vec![vec![]]; // Deletion layer
    let mut Mr = vec![vec![]]; // Reverse main layer
    let mut Ir = vec![vec![]]; // Reverse insertion layer
    let mut Dr = vec![vec![]]; // Reverse deletion layer
    let mut set1 = (M, I, D);
    let mut set2 = (Mr, Ir, Dr);
    let mut w = 1;

    //main loop
    let mut i: usize = 0;
    loop {
        //explore new wavefront for M,I,D
        expand_layer(&mut set1, i, 1);
        //explore new wavefront for Mr,Ir,Dr
        expand_layer(&mut set2, i, -1);

        // println!("\n{i}\n");
        // println!("M:\n");
        // print_vector(&set1.0);
        // print_vector(&set1.1);
        // print_vector(&set1.2);
        // println!("Mrs:\n");
        // print_vector(&set2.0);
        // print_vector(&set2.1);
        // print_vector(&set2.2);

        let mut f1 = false;
        let mut f2 = false;
        let mut d1 = INF;
        let mut d2 = INF;

        if i > 1 {
            (f1, d1) = compare(&mut set1, i, &mut set2, i - 2);
            // if found {
            //     return i * 2 - 2 - d;
            // }
        }

        //comparison two
        if i > 1 {
            (f2, d2) = compare(&mut set1, i - 2, &mut set2, i);
            // if found {
            //     return i * 2 - 2 - d;
            // }
        }

        if f1 && f2 {
            return i * 2 - 2 - max(d1, d2);
        }
        if f1 {
            return i * 2 - 2 - d1;
        }
        if f2 {
            return i * 2 - 2 - d2;
        }

        f1 = false;
        f2 = false;
        //comparison one
        if i > 0 {
            (f1, d1) = compare(&mut set1, i, &mut set2, i - 1);
            // if found {
            //     return i * 2 - 1 - d;
            // }
        }

        //comparison two
        if i > 0 {
            (f2, d2) = compare(&mut set1, i - 1, &mut set2, i);
            // if found {
            //     return i * 2 - 1 - d;
            // }
        }
        if f1 && f2 {
            return i * 2 - 1 - max(d1, d2);
        }
        if f1 {
            return i * 2 - 1 - d1;
        }
        if f2 {
            return i * 2 - 1 - d2;
        }

        //comparison three
        let (found, d) = compare(&mut set1, i, &mut set2, i);
        if found {
            return i * 2 - d;
        }

        // println!("\n{i}\n");
        // println!("M:\n");
        // print_vector(&set1.0);
        // print_vector(&set1.1);
        // print_vector(&set1.2);
        // println!("Mrs:\n");
        // print_vector(&set2.0);
        // print_vector(&set2.1);
        // print_vector(&set2.2);
        // let b1 = std::io::stdin().read_line(&mut String::from("")).unwrap();

        i += 1;
        w += 2;
    }
}
