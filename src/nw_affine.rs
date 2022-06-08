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
