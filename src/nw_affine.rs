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

pub fn biwfa_affine<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
    explored: &mut Vec<Pos>,
) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

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

    let get_j2 = |s: usize, d: usize, k: isize| -> Option<(usize, usize)> {
        if s < d {
            return None;
        }
        let p = (s - d) as isize - k;
        if p < 0 || p as usize > (s - d) * 2 {
            return None;
        }
        Some((s - d, p as usize))
    };

    let mut check_point5 = |C: &mut Vec<Vec<usize>>, i: usize, j: usize| -> () {
        //unused
        C[i][j] = 0;
        if i > 0 {
            //println!("HALOOOOOOOOOOOOOOOOOO!!!!!{i} {j}");
            if j > 1 && j - 2 < C[i - 1].len() {
                C[i][j] = max(C[i][j], C[i - 1][j - 2] + 1);
            }
            if j > 0 && j - 1 < C[i - 1].len() {
                C[i][j] = max(C[i][j], C[i - 1][j - 1] + 1);
            }
            if j < C[i - 1].len() {
                C[i][j] = max(C[i][j], C[i - 1][j]);
            }
        }
    };

    let check_point2 = |M: &Vec<Vec<isize>>,
                        I: &Vec<Vec<isize>>,
                        D: &Vec<Vec<isize>>,
                        i: usize,
                        j: usize,
                        k: isize|
     -> usize {
        let mut t = 0;
        let s = i;
        if i > 0 {
            //println!("HALOOOOOOOOOOOOOOOOOO!!!!!{i} {j}");
            // if j > 1 && j - 2 < C[i - 1].len() {
            //     t = max(t, C[i - 1][j - 2] + 1);
            // }
            // if j > 0 && j - 1 < C[i - 1].len() {
            //     t = max(t, C[i - 1][j - 1] + 1);
            // }
            // if j < C[i - 1].len() {
            //     t = max(t, C[i - 1][j]);
            // }
            t = max(I[s][j], D[s][j]);

            if let Some((i1, j1)) = get_j(s, x, k) {
                // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                t = max(M[i1][j1] + 1, M[s][j]);
            }
        }
        t as usize
    };

    let check_point3 = |M: &Vec<Vec<isize>>,
                        I: &Vec<Vec<isize>>,
                        D: &Vec<Vec<isize>>,
                        i: usize,
                        j: usize,
                        k: isize|
     -> usize {
        let mut t = 0;
        let s = i;
        if i > 0 {
            t = max(I[s][j], D[s][j]);

            if let Some((i1, j1)) = get_j2(s, x, -(j as isize) + (s as isize - x as isize) / 2) {
                // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                t = max(M[i1][j1] + 1, M[s][j]);
            }
        }
        t as usize
    };

    let a = |a1: isize, b1: isize, c1: usize| -> bool {
        if a1 > 0 && b1 > 0 && a1 + b1 == c1 as isize {
            return true;
        }
        return false;
    };

    let k0: isize = s1.len() as isize - s2.len() as isize;
    const NEG: isize = isize::MIN;
    let mut M = vec![vec![]]; // Main layer
    let mut I = vec![vec![]]; // Insertion layer
    let mut D = vec![vec![]]; // Deletion layer
    let mut Mr = vec![vec![]]; // Reverse main layer
    let mut Ir = vec![vec![]]; // Reverse insertion layer
    let mut Dr = vec![vec![]]; // Reverse deletion layer
    const x: usize = 1; // mismatch cost
    const o: usize = 1; // open gap cost
    const e: usize = 1; // extand gap cost
    let mut w = 1;

    for i in 0..(3 * s2.len()) {
        println!("\ni = {i} first\n\n");
        if i > 0 {
            M.push(vec![NEG; w]);
            I.push(vec![NEG; w]);
            D.push(vec![NEG; w]);
            Mr.push(vec![NEG; w]);
            Ir.push(vec![NEG; w]);
            Dr.push(vec![NEG; w]);
            M[i] = vec![NEG; w];
            Mr[i] = vec![NEG; w];
        } else {
            M[i] = vec![0isize; w];
            Mr[i] = vec![0isize; w];
        }
        I[i] = vec![NEG; w];
        D[i] = vec![NEG; w];
        Ir[i] = vec![NEG; w];
        Dr[i] = vec![NEG; w];
        let s = i;
        let mut k: isize = -(w as isize / 2);
        for j in 0..w {
            if let Some((i1, j1)) = get_j(s, o + e, k - 1) {
                I[s][j] = M[i1][j1] + 1;
            }
            if let Some((i1, j1)) = get_j(s, e, k - 1) {
                I[s][j] = max(I[i1][j1] + 1, I[s][j]);
            }

            if let Some((i1, j1)) = get_j(s, o + e, k + 1) {
                D[s][j] = M[i1][j1];
            }
            if let Some((i1, j1)) = get_j(s, e, k + 1) {
                D[s][j] = max(D[i1][j1], D[s][j]);
            }

            if i > 0 {
                M[s][j] = max(I[s][j], D[s][j]);

                if let Some((i1, j1)) = get_j(s, x, k) {
                    // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                    M[s][j] = max(M[i1][j1] + 1, M[s][j]);
                }
            }

            explore_diagonal(s1, s2, &mut M[s][j], s, j);
            k += 1;
        }

        print_vector(&M);
        print_vector(&I);
        print_vector(&D);
        println!("Mrs:\n");
        print_vector(&Mr);
        print_vector(&Ir);
        print_vector(&Dr);

        let mut k_min = 0;
        if w > 1 {
            k_min = max(-((w / 2) as isize), -(((w - 2) / 2) as isize) + k0);
        }
        let mut k_max = 0;

        if i > 0 {
            k_max = (((w - 2) / 2) as isize) + k0;
        } else {
            k_min = 2;
        }

        // print_vector(&M);
        // print_vector(&I);
        // print_vector(&D);
        // println!("Mr:\n");
        // print_vector(&Mr);

        println!("k_min = {k_min}\nk_max = {k_max}\n");
        println!("Comparison\n");
        let mut f = 0usize;
        if i > 0 {
            Mr[i - 1].reverse();
            Ir[i - 1].reverse();
            Dr[i - 1].reverse();
        }
        //fix for gaps!!!
        if i > 0 && k_min - k_max == 1 {
            if a(
                D[i][((w / 2) as isize + k_min) as usize],
                Mr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                f = 1;
            }
            if a(
                M[i][((w / 2) as isize + k_min) as usize],
                Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                f = 2;
            }
            if a(
                D[i][((w / 2) as isize + k_min) as usize],
                Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                return 2 * i - 1 - o;
            }
        } else if (((w / 2) as isize + k_max + 1) as usize) < w {
            if a(
                D[i][((w / 2) as isize + k_max + 1) as usize],
                Mr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = 10;
            }
            if a(
                M[i][((w / 2) as isize + k_max + 1) as usize],
                Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = 11;
            }
            if a(
                D[i][((w / 2) as isize + k_max + 1) as usize],
                Dr[i - 1][(((w - 2) / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                return 2 * i - 1 - o;
            }
        }
        for k in k_min..=k_max {
            println!(
                "I&D SECTIONS:\t{i}\t{k}\t{}\t{}\n",
                ((w / 2) as isize + k),
                (((w - 2) / 2) as isize + k - k0 + 1)
            );

            println!("\nACCESSED\n");
            if ((w - 2) / 2) as isize + k - k0 - 1 > 0 {
                if (s1.len() != s2.len() || k > k_min) {
                    //D section
                    // if a(
                    //     D[i][((w / 2) as isize + k) as usize],
                    //     Mr[i - 1][(((w - 2) / 2) as isize + k - k0 - 1) as usize],
                    //     s1.len() + 1,
                    // ) {
                    //     f = 3;
                    // }
                    // if a(
                    //     M[i][((w / 2) as isize + k) as usize],
                    //     Dr[i - 1][(((w - 2) / 2) as isize + k - k0 - 1) as usize],
                    //     s1.len() + 1,
                    // ) {
                    //     f = 4;
                    // }
                    if a(
                        D[i][((w / 2) as isize + k) as usize],
                        Dr[i - 1][(((w - 2) / 2) as isize + k - k0 - 1) as usize],
                        s1.len() + 1,
                    ) {
                        println!("EXIT 1");
                        return 2 * i - 1 - o;
                    }
                }
            }
            if k < k_max {
                //I section
                // if a(
                //     // Can be optimized by adding !f && ...
                //     I[i][((w / 2) as isize + k) as usize],
                //     Mr[i - 1][(((w - 2) / 2) as isize + k - k0 + 1) as usize],
                //     s1.len(),
                // ) {
                //     f = 5;
                // }
                // if a(
                //     M[i][((w / 2) as isize + k) as usize],
                //     Ir[i - 1][(((w - 2) / 2) as isize + k - k0 + 1) as usize],
                //     s1.len(),
                // ) {
                //     f = 6;
                // }
                if a(
                    I[i][((w / 2) as isize + k) as usize],
                    Ir[i - 1][(((w - 2) / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    println!("EXIT 2");
                    return 2 * i - o - 1;
                }
            }

            if M[i][((w / 2) as isize + k) as usize] < 0
                || Mr[i - 1][(((w - 2) / 2) as isize + k - k0) as usize] < 0
            {
                continue;
            }

            if k == s1.len() as isize - s2.len() as isize {
                if M[i][((w / 2) as isize + k) as usize] >= s1.len() as isize {
                    println!("EXIT 3");
                    return i;
                }
            }

            // println!(
            //     "{} {}",
            //     A[i][((w / 2) as isize + k) as usize],
            //     B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
            // );
            let t1 = check_point2(&M, &I, &D, i, ((w / 2) as isize + k) as usize, k);
            let t2 = check_point3(
                //k+1 -> k-1; k-1 -> k+1
                &Mr,
                &Ir,
                &Dr,
                i - 1,
                Mr[i - 1].len() - 1 - (((w - 2) / 2) as isize + k - k0) as usize,
                k,
            );
            //println!("t1 == {} t2 == {}", t1, t2);
            if M[i][((w / 2) as isize + k) as usize] as usize
                + Mr[i - 1][(((w - 2) / 2) as isize + k - k0) as usize] as usize
                == s1.len()
            {
                f = 8
            }
            if (M[i][((w / 2) as isize + k) as usize] as usize
                + Mr[i - 1][(((w - 2) / 2) as isize + k - k0) as usize] as usize
                >= (s1.len() + 1)
                && M[i][((w / 2) as isize + k) as usize] as usize + t2 <= s1.len() + 1)
            {
                f = 9;
            }
            if ((s1.len() + 1)
                >= t1 + Mr[i - 1][(((w - 2) / 2) as isize + k - k0) as usize] as usize
                && (s1.len() + 1 - Mr[i - 1][(((w - 2) / 2) as isize + k - k0) as usize] as usize)
                    <= M[i][((w / 2) as isize + k) as usize] as usize)
            {
                f = 7;
            }
        }
        if i > 0 {
            Mr[i - 1].reverse();
            Ir[i - 1].reverse();
            Dr[i - 1].reverse();
        }
        if f > 0 {
            println!("F flag {f}\n");
            return 2 * i - 1;
        }
        println!("Comparison End\n");

        println!("\ni = {i}\n\n");
        let mut k: isize = (w as isize / 2);
        for j in 0..w {
            if let Some((i1, j1)) = get_j2(s, o + e, k + 1) {
                Ir[s][j] = Mr[i1][j1] + 1;
                println!("Ir\t{i}\t{j}\t{}\tMr\t{i1}\t{j1}", Ir[s][j]);
            }
            if let Some((i1, j1)) = get_j2(s, e, k + 1) {
                Ir[s][j] = max(Ir[i1][j1] + 1, Ir[s][j]);
                println!("Ir\t{i}\t{j}\t{}\tIr\t{i1}\t{j1}", Ir[s][j]);
            }

            if let Some((i1, j1)) = get_j2(s, o + e, k - 1) {
                Dr[s][j] = Mr[i1][j1];
            }
            if let Some((i1, j1)) = get_j2(s, e, k - 1) {
                Dr[s][j] = max(Dr[i1][j1], Dr[s][j]);
            }
            if i > 0 {
                Mr[s][j] = max(Ir[s][j], Dr[s][j]);

                if let Some((i1, j1)) = get_j2(s, x, k) {
                    // println!("S {i1}\t{j1}\t{s}\t{k}\n");
                    Mr[s][j] = max(Mr[i1][j1] + 1, Mr[s][j]);
                }
            }

            if Mr[i][j] >= 0 && s1.len() >= Mr[i][j] as usize {
                explored.push(Pos(
                    (s1.len() - Mr[i][j] as usize) as u32,
                    ((s1.len() - Mr[i][j] as usize) as isize
                        - (Mr[i].len() as isize / 2 - j as isize + k0)) as u32,
                ));
            }
            if Mr[i][j] >= 0 && s1.len() >= Mr[i][j] as usize + 1 {
                let d: isize = -(j as isize) + (w - 1) as isize - (w / 2) as isize + k0;
                //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
                // println!("B[i][j] == {}\ni == {i}\nj == {j}\nd == {d}\n\n", B[i][j]);
                let mut x2 = s1.len() - Mr[i][j] as usize - 1;
                let mut k = (x2 as isize - d) as usize;
                while k < s2.len() && x2 < s1.len() {
                    if s1[x2] == s2[k] {
                        Mr[i][j] += 1;
                        explored.push(Pos(
                            (s1.len() - Mr[i][j] as usize) as u32,
                            ((s1.len() - Mr[i][j] as usize) as isize
                                - (Mr[i].len() as isize / 2 - j as isize + k0))
                                as u32,
                        ));
                        if x2 == 0 || k == 0 {
                            break;
                        }
                        k -= 1;
                        x2 -= 1;
                    } else {
                        break;
                    }
                }
            }
            k -= 1;
        }

        let mut k_min = 0;
        if w > 1 {
            k_min = max(-(((w - 2) / 2) as isize), -((w / 2) as isize) + k0);
        }
        let mut k_max = 0;

        if i > 0 {
            k_max = min((((w - 2) / 2) as isize) + k0, (w as isize - 2) / 2);
        } else {
            k_min = 2;
        }

        println!("k_min = {k_min}\nk_max = {k_max}\n");
        println!("Comparison Middle\n");
        let mut f = false;
        // if i > 0 {
        //     Mr[i - 1].reverse();
        // }
        Mr[i].reverse();
        Ir[i].reverse();
        Dr[i].reverse();
        if i > 0 && k_min - k_max == 1 {
            if a(
                D[i - 1][(((w - 2) / 2) as isize + k_min) as usize],
                Mr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                f = true;
            }
            if a(
                M[i - 1][(((w - 2) / 2) as isize + k_min) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                f = true;
            }
            if a(
                D[i - 1][(((w - 2) / 2) as isize + k_min) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                return 2 * i - 1 - o;
            }
        } else if i > 0 && ((((w - 2) / 2) as isize + k_max + 1) as usize) < w {
            if a(
                D[i - 1][(((w - 2) / 2) as isize + k_max + 1) as usize],
                Mr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = true;
            }
            if a(
                M[i - 1][(((w - 2) / 2) as isize + k_max + 1) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = true;
            }
            if a(
                D[i - 1][(((w - 2) / 2) as isize + k_max + 1) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                return 2 * i - 1 - o;
            }
        }
        for k in k_min..=k_max {
            // println!(
            //     "I&D SECTIONS:\t{i}\t{k}\t{}\t{}\n",
            //     ((w / 2) as isize + k),
            //     (((w - 2) / 2) as isize + k - k0 + 1)
            // );

            // println!("\nACCESSED\n");
            if true {
                if (s1.len() != s2.len() || k > k_min) {
                    //D section
                    if a(
                        D[i - 1][(((w - 2) / 2) as isize + k) as usize],
                        Mr[i][((w / 2) as isize + k - k0 - 1) as usize],
                        s1.len() + 1,
                    ) {
                        f = true;
                    }
                    if a(
                        M[i - 1][(((w - 2) / 2) as isize + k) as usize],
                        Dr[i][((w / 2) as isize + k - k0 - 1) as usize],
                        s1.len() + 1,
                    ) {
                        f = true;
                    }
                    if a(
                        D[i - 1][(((w - 2) / 2) as isize + k) as usize],
                        Dr[i][((w / 2) as isize + k - k0 - 1) as usize],
                        s1.len() + 1,
                    ) {
                        return 2 * i - 1 - o;
                    }
                }
            }
            if k < k_max {
                //I section
                if a(
                    // Can be optimized by adding !f && ...
                    I[i - 1][(((w - 2) / 2) as isize + k) as usize],
                    Mr[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    f = true;
                }
                if a(
                    M[i - 1][(((w - 2) / 2) as isize + k) as usize],
                    Ir[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    f = true;
                }
                if a(
                    I[i - 1][(((w - 2) / 2) as isize + k) as usize],
                    Ir[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    return 2 * i - o - 1;
                }
            }

            if M[i - 1][(((w - 2) / 2) as isize + k) as usize] < 0
                || Mr[i][((w / 2) as isize + k - k0) as usize] < 0
            {
                continue;
            }

            if k == s1.len() as isize - s2.len() as isize {
                if M[i - 1][(((w - 2) / 2) as isize + k) as usize] >= s1.len() as isize {
                    return i;
                }
                if Mr[i][((w / 2) as isize + k - k0) as usize] >= s1.len() as isize {
                    return i - 1;
                }
            }

            // println!(
            //     "{} {}",
            //     A[i][((w / 2) as isize + k) as usize],
            //     B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
            // );
            let t1 = check_point2(&M, &I, &D, i - 1, (((w - 2) / 2) as isize + k) as usize, k);
            let t2 = check_point3(
                //k+1 -> k-1; k-1 -> k+1
                &Mr,
                &Ir,
                &Dr,
                i,
                Mr[i].len() - 1 - ((w / 2) as isize + k - k0) as usize,
                k,
            );
            //println!("t1 == {} t2 == {}", t1, t2);
            if M[i - 1][(((w - 2) / 2) as isize + k) as usize] as usize
                + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                == s1.len()
                || (M[i - 1][(((w - 2) / 2) as isize + k) as usize] as usize
                    + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                    >= (s1.len() + 1)
                    && M[i - 1][(((w - 2) / 2) as isize + k) as usize] as usize + t2
                        <= s1.len() + 1)
                || ((s1.len() + 1) >= t1 + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                    && (s1.len() + 1 - Mr[i][((w / 2) as isize + k - k0) as usize] as usize)
                        <= M[i - 1][(((w - 2) / 2) as isize + k) as usize] as usize)
            {
                f = true;
            }
        }
        // if i > 0 {
        //     Mr[i - 1].reverse();
        // }
        Mr[i].reverse();
        Ir[i].reverse();
        Dr[i].reverse();
        if f {
            return 2 * i - 1;
        }
        println!("Comparison Middle End\n");

        print_vector(&M);
        print_vector(&I);
        print_vector(&D);
        println!("Mrs:\n");
        print_vector(&Mr);
        print_vector(&Ir);
        print_vector(&Dr);

        let k_min = -((w / 2) as isize);
        let k_max = ((w / 2) as isize) + k0;
        println!("k_min = {k_min}\nk_max = {k_max}\n");
        println!("Comparison\n");
        let mut f = false;
        /*if i > 0 {
            B[i - 1].reverse();
        }*/
        Mr[i].reverse();
        Ir[i].reverse();
        Dr[i].reverse();
        if k_min - k_max == 1 {
            if a(D[i][0], Mr[i][w - 1], s1.len()) {
                f = true;
            }
            if a(
                M[i][((w / 2) as isize + k_min) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                f = true;
            }
            if a(
                D[i][((w / 2) as isize + k_min) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len(),
            ) {
                return 2 * i - o;
            }
        } else if (((w / 2) as isize + k_max + 1) as usize) < w {
            if a(
                D[i][((w / 2) as isize + k_max + 1) as usize],
                Mr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = true;
            }
            if a(
                M[i][((w / 2) as isize + k_max + 1) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                f = true;
            }
            if a(
                D[i][((w / 2) as isize + k_max + 1) as usize],
                Dr[i][((w / 2) as isize + k_max - k0) as usize],
                s1.len() + 1,
            ) {
                return 2 * i - o;
            }
        }
        for k in k_min..=k_max {
            if s1.len() != s2.len() || k > k_min {
                //D section
                println!(
                    "D-indexes:\t{i}\t{}\t{}",
                    ((w / 2) as isize + k) as usize,
                    ((w / 2) as isize + k - k0 - 1) as usize
                );
                println!(
                    "{}\t{}",
                    M[i][((w / 2) as isize + k) as usize],
                    Dr[i][((w / 2) as isize + k - k0 - 1) as usize]
                );
                if a(
                    D[i][((w / 2) as isize + k) as usize],
                    Mr[i][((w / 2) as isize + k - k0 - 1) as usize],
                    s1.len() + 1,
                ) {
                    f = true;
                }
                if a(
                    M[i][((w / 2) as isize + k) as usize],
                    Dr[i][((w / 2) as isize + k - k0 - 1) as usize],
                    s1.len() + 1,
                ) {
                    f = true;
                }
                if a(
                    D[i][((w / 2) as isize + k) as usize],
                    Dr[i][((w / 2) as isize + k - k0 - 1) as usize],
                    s1.len() + 1,
                ) {
                    return 2 * i - o;
                }
            }
            if k < k_max {
                //I section
                if a(
                    // Can be optimized by adding !f && ...
                    I[i][((w / 2) as isize + k) as usize],
                    Mr[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    f = true;
                }
                if a(
                    M[i][((w / 2) as isize + k) as usize],
                    Ir[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    f = true;
                }
                if a(
                    I[i][((w / 2) as isize + k) as usize],
                    Ir[i][((w / 2) as isize + k - k0 + 1) as usize],
                    s1.len(),
                ) {
                    return 2 * i - o;
                }
            }

            if M[i][((w / 2) as isize + k) as usize] < 0
                || Mr[i][((w / 2) as isize + k - k0) as usize] < 0
            {
                continue;
            }

            println!(
                "{} {}",
                M[i][((w / 2) as isize + k) as usize],
                Mr[i][((w / 2) as isize + k - k0) as usize]
            );
            let t1 = check_point2(&M, &I, &D, i, ((w / 2) as isize + k) as usize, k);
            let t2 = check_point3(
                //k+1 -> k-1; k-1 -> k+1
                &Mr,
                &Ir,
                &Dr,
                i,
                Mr[i].len() - 1 - ((w / 2) as isize + k - k0) as usize,
                k,
            );
            // println!("{k} {t1} {t2}");
            // println!("{} {}", ((w / 2) as isize + k), ((w / 2) as isize + k - k0));
            if M[i][((w / 2) as isize + k) as usize] as usize
                + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                == s1.len()
                || (M[i][((w / 2) as isize + k) as usize] as usize
                    + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                    >= (s1.len() + 1)
                    && M[i][((w / 2) as isize + k) as usize] as usize + t2 <= (s1.len() + 1))
                || ((s1.len() + 1) >= t1 + Mr[i][((w / 2) as isize + k - k0) as usize] as usize
                    && (s1.len() + 1 - Mr[i][((w / 2) as isize + k - k0) as usize] as usize)
                        <= M[i][((w / 2) as isize + k) as usize] as usize)
            {
                f = true;
            }
        }
        /*if i > 0 {
            B[i - 1].reverse();
        }*/
        Mr[i].reverse();
        Ir[i].reverse();
        Dr[i].reverse();
        println!("Comparison End\n");
        if f {
            return 2 * i;
        }

        w += 2;
    }
    unreachable!("Error! Shouldn't be here!");
}

fn explore_diagonal2(
    //Function for exploring (checking matches on the diagonal). Only for standard DTM and compatible algorithms!
    s1: &Sequence,
    s2: &Sequence,
    A: &mut isize, // A = A[i][j]
    i: usize,
    j: usize,
    d: isize,
) {
    if *A < 0 {
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

fn biwfa_affine2<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
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

    let check_point = |set: &mut (
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
    ),
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
        }
        t
    };

    let compare = |set1: &mut (
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
    ),
                   s1: usize,
                   set2: &mut (
        //set2 is reverse!!! Do not swap set1 and set2! They have different allocations!
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
    ),
                   s2: usize|
     -> (bool, usize) {
        //bool - do these layers cover each other?; usize - if they do, how much do we need to substruct? (0 or o (open cost))

        let (M, I, D) = set1;
        let (Mr, Ir, Dr) = set2;

        let k_min1 = -(s1 as isize);
        let k_max1 = (s1 as isize);
        let k_min2 = -(s2 as isize) + k0;
        let k_max2 = (s2 as isize) + k0;
        let k_min = max(k_min1, k_min2);
        let k_max = min(k_max1, k_max2);
        let mut j1 = (s1 as isize + k_min) as usize;
        let mut j2 = (2 * s2 as isize - k_min - k0) as usize;
        let mut f = false;
        if k_min - k_max <= 1 {
            if cmp(
                D[s1][(s1 as isize + k_max + 1) as usize],
                Dr[s2][(2 * s2 as isize - k_max) as usize],
                len1 + 1,
            ) {
                return (true, o);
            }
        }
        for k in k_min..=k_max {
            //comparing...
            if j2 + 1 < 1 + s2 * 2 && cmp(D[s1][j1], Dr[s2][j2 + 1], len1 + 1) {
                return (true, o);
            }
            if k > k_min && cmp(I[s1][j1 - 1], Ir[s2][j2], len1 + 1) {
                return (true, o);
            }

            let t1 = check_point(&mut set1, s1, j1, k);
            let t2 = check_point(&mut set2, s2, j2, -(k - k0));

            if cmp(M[s1][j1], Mr[s2][j2], len1)
                || (M[s1][j1] > 0
                    && Mr[s2][j2] > 0
                    && (is_inside((len1 + 1) as isize - Mr[s2][j2], t1, M[s1][j1])
                        || is_inside(
                            M[s1][j1],
                            (len1 + 1) as isize - Mr[s2][j2],
                            (len1 + 1) as isize - t2,
                        )))
            {
                f = true;
            }

            j1 += 1;
            j2 -= 1;
        }
        return (f, 0);
    };

    let expand_layer = |set: &mut (
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
        &mut Vec<Vec<isize>>,
    ),
                        s: usize,
                        d: isize|
     -> () {
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

    //intialization
    const NEG: isize = isize::MIN;
    let mut M = vec![vec![]]; // Main layer
    let mut I = vec![vec![]]; // Insertion layer
    let mut D = vec![vec![]]; // Deletion layer
    let mut Mr = vec![vec![]]; // Reverse main layer
    let mut Ir = vec![vec![]]; // Reverse insertion layer
    let mut Dr = vec![vec![]]; // Reverse deletion layer
    let mut set1 = (&mut M, &mut I, &mut D);
    let mut set2 = (&mut Mr, &mut Ir, &mut Dr);
    let mut w = 1;

    //main loop
    let mut i: usize = 0;
    loop {
        //explore new wavefront for M,I,D
        expand_layer(&mut set1, i, 1);

        //comparison one
        let (found, d) = compare(&mut set1, i, &mut set2, i - 1);
        if found {
            return i * 2 - 1 - d;
        }

        //explore new wavefront for Mr,Ir,Dr
        expand_layer(&mut set2, i, -1);

        //comparison two
        let (found, d) = compare(&mut set1, i - 1, &mut set2, i);
        if found {
            return i * 2 - 1 - d;
        }

        //comparison three
        let (found, d) = compare(&mut set1, i, &mut set2, i);
        if found {
            return i * 2 - d;
        }

        i += 1;
        w += 2;
    }
}
