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
        t[i][0] = A + (i - 1) * B;
    }
    for i in 1..=s2.len() {
        u[0][i] = A + (i - 1) * B;
    }
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            t[i][j] = min(t[i - 1][j] + B, s[i - 1][j] + A);
            u[i][j] = min(u[i][j - 1] + B, s[i][j - 1] + A);
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
