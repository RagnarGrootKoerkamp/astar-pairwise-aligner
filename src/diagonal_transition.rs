use std::mem::swap;

use crate::prelude::*;

pub fn diagonal_transition_linear_fast<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut w = 1;

    let mut A = vec![0usize; w];
    let mut B = vec![0usize; w];
    let mut k = 0;
    while k < s2.len() && A[0] < s1.len() {
        if s1[A[0]] == s2[k] {
            A[0] += 1;
            k += 1;
        } else {
            break;
        }
    }
    if s2.len() == s1.len() && A[0] >= s1.len() {
        return 0;
    }
    for i in 1..s2.len() {
        w += 2;
        B.resize(w, 0);
        //A[i] = vec![0usize; w];
        for j in 0..w {
            B[j] = 0;

            if j > 1 && j - 2 < w - 2 {
                B[j] = max(B[j], A[j - 2] + 1);
            }
            if j > 0 && j - 1 < w - 2 {
                B[j] = max(B[j], A[j - 1] + 1);
            }
            if j < w - 2 {
                B[j] = max(B[j], A[j]);
            }

            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            k = (B[j] as isize - d) as usize;
            while k < s2.len() && B[j] < s1.len() {
                if s1[B[j]] == s2[k] {
                    B[j] += 1;
                    k += 1;
                } else {
                    break;
                }
            }
            if d == s1.len() as isize - s2.len() as isize {
                if B[j] >= s1.len() {
                    return i;
                }
            }
        }
        swap(&mut A, &mut B);
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn diagonal_transition_linear<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut w = 1;

    let mut A = vec![0usize; w];
    let mut k = 0;
    while k < s2.len() && A[0] < s1.len() {
        if s1[A[0]] == s2[k] {
            A[0] += 1;
            k += 1;
        } else {
            break;
        }
    }
    if s2.len() == s1.len() && A[0] >= s1.len() {
        return 0;
    }
    for i in 1..s2.len() {
        w += 2;
        let mut B = vec![0usize; w];
        //A[i] = vec![0usize; w];
        for j in 0..w {
            B[j] = 0;

            if j > 1 && j - 2 < w - 2 {
                B[j] = max(B[j], A[j - 2] + 1);
            }
            if j > 0 && j - 1 < w - 2 {
                B[j] = max(B[j], A[j - 1] + 1);
            }
            if j < w - 2 {
                B[j] = max(B[j], A[j]);
            }

            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            k = (B[j] as isize - d) as usize;
            while k < s2.len() && B[j] < s1.len() {
                if s1[B[j]] == s2[k] {
                    B[j] += 1;
                    k += 1;
                } else {
                    break;
                }
            }
            if d == s1.len() as isize - s2.len() as isize {
                if B[j] >= s1.len() {
                    return i;
                }
            }
        }
        A = B;
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn diagonal_transition<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

    let mut A = vec![vec![]];
    let mut w = 1;

    A[0] = vec![0usize; w];
    let mut k = 0;
    while k < s2.len() && A[0][0] < s1.len() {
        if s1[A[0][0]] == s2[k] {
            A[0][0] += 1;
            k += 1;
        } else {
            break;
        }
    }
    if s2.len() == s1.len() && A[0][0] >= s1.len() {
        return 0;
    }
    for i in 1..s2.len() {
        w += 2;
        A.push(vec![0usize; w]);
        //A[i] = vec![0usize; w];
        for j in 0..w {
            A[i][j] = 0;

            if j > 1 && j - 2 < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j - 2] + 1);
            }
            if j > 0 && j - 1 < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j - 1] + 1);
            }
            if j < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j]);
            }

            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            k = (A[i][j] as isize - d) as usize;
            while k < s2.len() && A[i][j] < s1.len() {
                if s1[A[i][j]] == s2[k] {
                    A[i][j] += 1;
                    k += 1;
                } else {
                    break;
                }
            }
            if d == s1.len() as isize - s2.len() as isize {
                if A[i][j] >= s1.len() {
                    return i;
                }
            }
        }
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn diagonal_transition_short<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

    let mut A: Vec<Vec<u32>> = vec![vec![]];
    let mut w = 1;

    A[0] = vec![0; w];
    let mut k = 0;
    while k < s2.len() && (A[0][0] as usize) < s1.len() {
        if s1[A[0][0] as usize] == s2[k] {
            A[0][0] += 1;
            k += 1;
        } else {
            break;
        }
    }
    if s2.len() == s1.len() && A[0][0] as usize >= s1.len() {
        return 0;
    }
    for i in 1..s2.len() {
        w += 2;
        A.push(vec![0; w]);
        //A[i] = vec![0usize; w];
        for j in 0..w {
            A[i][j] = 0;

            if j > 1 && j - 2 < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j - 2] + 1);
            }
            if j > 0 && j - 1 < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j - 1] + 1);
            }
            if j < w - 2 {
                A[i][j] = max(A[i][j], A[i - 1][j]);
            }

            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            k = (A[i][j] as isize - d) as usize;
            while k < s2.len() && (A[i][j] as usize) < s1.len() {
                if s1[A[i][j] as usize] == s2[k] {
                    A[i][j] += 1;
                    k += 1;
                } else {
                    break;
                }
            }
            if d == s1.len() as isize - s2.len() as isize {
                if A[i][j] as usize >= s1.len() {
                    return i;
                }
            }
        }
    }
    unreachable!("Error! Shouldn't be here!");
}

fn explore_diagonal(
    s1: &Sequence,
    s2: &Sequence,
    mut A: usize, // A = A[j][i]
    a: usize,
    b: usize,
) -> usize {
    let mut d: isize = 0;
    if a % 2 == 0 {
        d = (a / 2) as isize - b as isize;
    } else {
        d = ((a - 1) / 2) as isize - b as isize;
    }
    //println!("d = {d}\nA = {A}\n");
    let mut k: usize = (A as isize - d) as usize;
    while A < s1.len() {
        if s1[A] == s2[k] {
            A += 1;
            k += 1;
        } else {
            break;
        }
    }
    A
}

pub fn diagonal_transition_a<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut A = vec![vec![]; 0];
    let d0 = s2.len() - s1.len(); // +1??
                                  //A[0] = vec![0usize; d0];

    let mut i = 0;
    let mut len = d0;
    /*println!(
        "First line is {}\nSecond line is {}\n\n",
        s1.len(),
        s2.len()
    );*/
    loop {
        if i % 2 == 0 {
            len += 1;
        }

        A.push(vec![0usize; len - 1]);

        for j in 0..(A.last().expect("A shouldn't be empty... Error...").len()) {
            A[i][j] = 0;
            if j > 0 {
                A[i][j] = A[i][j - 1];
            }
            if i > 0 && j >= (if i % 2 == 1 { 0 } else { 1 }) {
                A[i][j] = max(A[i][j], A[i - 1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
            }
            if i > 1 {
                A[i][j] = max(A[i][j], A[i - 2][j] + 1);
            }

            /*A[i][j] = max(
                A[i][j - 1],
                max(
                    A[i - 1][j - (if i % 2 == 1 { 0 } else { 1 })],
                    A[i - 2][j - 1],
                ) + 1,
            );*/
            /*println!(
                "i = {i}\nj = {j}\nA len = {}\nA last len = {}\n\n",
                A.len(),
                A.last().unwrap().len()
            );*/
            A[i][j] = explore_diagonal(s1, s2, A[i][j], i, j);
            /*print!("\n");
            for k in 0..(A.len()) {
                for k2 in 0..(A[k].len()) {
                    print!("{} ", A[k][k2]);
                }
                print!("\n");
            }*/
        }
        //print!("\nSecond Loop\n");
        for j in 0..=i {
            if j != i && (i + j + d0) % 2 == 1 {
                continue;
            }
            let mut tmp = 0;
            if A[j].len() > 0 {
                tmp = A[j][A[j].len() - 1];
            }
            if j > 0 && A[j - 1].len() > A[j].len() - (if j % 2 == 1 { 0 } else { 1 }) {
                tmp = max(
                    tmp,
                    A[j - 1][A[j].len() - (if j % 2 == 1 { 0 } else { 1 })] + 1,
                );
            }
            if j > 1 && A[j - 2].len() > A[j].len() {
                tmp = max(tmp, A[j - 2][A[j].len()] + 1);
            }
            /*let tmp = max(
                A[j][A[j].len() - 1],
                max(A[j - 1][A[j].len() - 2], A[j - 2][A[j].len() - 1]) + 1,
            );*/
            A[j].push(tmp);
            let tmp = explore_diagonal(s1, s2, A[j][A[j].len() - 1], j, A[j].len() - 1);
            let tmp2 = A[j].len() - 1;
            A[j][tmp2] = tmp;
            /*for k in 0..(A.len()) {
                for k2 in 0..(A[k].len()) {
                    print!("{} ", A[k][k2]);
                }
                print!("\n");
            }*/
        }

        if *A[i].last().expect("A is empty") >= s1.len() {
            let t = A[i].len() - 1;
            if i % 2 == 0 {
                return i / 2 + t;
            } else {
                return (i + 2 * t + 1) / 2;
            }
        }

        i += 1;
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn diagonal_transition_a_oxy<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut A = vec![vec![]; 0];
    let d0 = s2.len() - s1.len();

    let mut i = 0;
    let mut len = d0;
    loop {
        if i % 2 == 0 {
            len += 1;
        }

        let sz = len + i / 2;
        A.push(vec![0usize; sz]);

        for j in 0..(len - 1) {
            A[i][j] = 0;
            if j > 0 {
                A[i][j] = A[i][j - 1];
            }
            if i > 0 && j >= (if i % 2 == 1 { 0 } else { 1 }) {
                A[i][j] = max(A[i][j], A[i - 1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
            }
            if i > 1 {
                A[i][j] = max(A[i][j], A[i - 2][j] + 1);
            }

            A[i][j] = explore_diagonal(s1, s2, A[i][j], i, j);
        }
        let mut j1 = 1;
        for j in 0..i {
            if (i + j + d0) % 2 == 1 {
                continue;
            }
            A[i][sz - j1] = 0;
            if j1 > 1 {
                A[i][sz - j1] = A[i][sz - j1 + 1] + 1;
            }
            if i > 0 && j1 > (i + 1) % 2 {
                A[i][sz - j1] = max(A[i][sz - j1], A[i - 1][sz - j1 - 3 * (i + 1) % 2] + 1);
                if i > 1 {
                    A[i][sz - j1] = max(A[i][sz - j1], A[i - 2][sz - j1 - 2]);
                }
            }
            A[i][sz - j1] = explore_diagonal(s1, s2, A[i][sz - j1], j, d0 + i / 2);
            j1 += 1;
        }
        let last_index = sz - j1;
        A[i][last_index] = 0;
        if i > 0 {
            A[i][last_index] = A[i - 1][len - 1 - (i + 1) % 2] + 1;
            if i > 1 {
                A[i][last_index] = max(A[i][last_index], A[i][last_index + 1] + 1);
            }
        }
        if len > 1 {
            A[i][last_index] = max(A[i][last_index], A[i][len - 2]);
        }
        A[i][last_index] = explore_diagonal(s1, s2, A[i][last_index], i, d0 + i / 2);
        if A[i][last_index] >= s1.len() {
            let t = len - 1;
            if i % 2 == 0 {
                return i / 2 + t;
            } else {
                return (i + 2 * t + 1) / 2;
            }
        }

        i += 1;
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn diagonal_transition_a_oxy_linear<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut A = vec![vec![]; 3];
    let d0 = s2.len() - s1.len();

    let mut i = 0;
    let mut len = d0;
    let mut num = 2;
    let mut num1 = 1;
    let mut num2 = 0;
    let mut tmp = 0;
    let mut tmp1 = 0;
    loop {
        if i % 2 == 0 {
            len += 1;
        }

        let sz = len + i / 2;
        //A.push(vec![0usize; sz]);
        A[num].resize(sz, 0);

        for j in 0..(len - 1) {
            A[num][j] = 0;
            if j > 0 {
                A[num][j] = A[num][j - 1];
            }
            if i > 0 && j >= (if i % 2 == 1 { 0 } else { 1 }) {
                A[num][j] = max(A[num][j], A[num1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
            }
            if i > 1 {
                A[num][j] = max(A[num][j], A[num2][j] + 1);
            }

            A[num][j] = explore_diagonal(s1, s2, A[num][j], i, j);
        }
        let mut j1 = 1;
        for j in 0..i {
            if (i + j + d0) % 2 == 1 {
                continue;
            }
            A[num][sz - j1] = 0;
            if j1 > 1 {
                A[num][sz - j1] = A[num][sz - j1 + 1] + 1;
            }
            if i > 0 && j1 > (i + 1) % 2 {
                A[num][sz - j1] = max(A[num][sz - j1], A[num1][sz - j1 - 3 * (i + 1) % 2] + 1);
                if i > 1 {
                    A[num][sz - j1] = max(A[num][sz - j1], A[num2][sz - j1 - 2]);
                }
            }
            A[num][sz - j1] = explore_diagonal(s1, s2, A[num][sz - j1], j, d0 + i / 2);
            j1 += 1;
        }
        let last_index = sz - j1;
        A[num][last_index] = 0;
        if i > 0 {
            A[num][last_index] = A[num1][len - 1 - (i + 1) % 2] + 1;
            if i > 1 {
                A[num][last_index] = max(A[num][last_index], A[num][last_index + 1] + 1);
            }
        }
        if len > 1 {
            A[num][last_index] = max(A[num][last_index], A[num][len - 2]);
        }
        A[num][last_index] = explore_diagonal(s1, s2, A[num][last_index], i, d0 + i / 2);
        if A[num][last_index] >= s1.len() {
            let t = len - 1;
            if i % 2 == 0 {
                return i / 2 + t;
            } else {
                return (i + 2 * t + 1) / 2;
            }
        }

        i += 1;
        tmp = num;
        num = num2;
        tmp1 = num1;
        num1 = tmp;
        num2 = tmp1;
    }
    unreachable!("Error! Shouldn't be here!");
}
