use std::mem::swap;

use sdl2::{render::Canvas, video::Window};

use crate::{
    drawing::{display2, display3},
    prelude::*,
};

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

fn explore_diagonal1(
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

fn explore_diagonal(
    s1: &Sequence,
    s2: &Sequence,
    A: &mut usize, // A = A[j][i]
    a: usize,
    b: usize,
) {
    let mut d: isize = 0;
    if a % 2 == 0 {
        d = (a / 2) as isize - b as isize;
    } else {
        d = ((a - 1) / 2) as isize - b as isize;
    }
    //println!("d = {d}\nA = {A}\n");
    let mut k: usize = (*A as isize - d) as usize;
    while *A < s1.len() && k < s2.len() {
        if s1[*A] == s2[k] {
            *A += 1;
            k += 1;
        } else {
            break;
        }
    }
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
            explore_diagonal(s1, s2, &mut A[i][j], i, j);
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
            let tmp = A[j].len() - 1;
            explore_diagonal(s1, s2, &mut A[j][tmp], j, tmp);
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

            explore_diagonal(s1, s2, &mut A[i][j], i, j);
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
            explore_diagonal(s1, s2, &mut A[i][sz - j1], j, d0 + i / 2);
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
        explore_diagonal(s1, s2, &mut A[i][last_index], i, d0 + i / 2);
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

            explore_diagonal(s1, s2, &mut A[num][j], i, j);
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
            explore_diagonal(s1, s2, &mut A[num][sz - j1], j, d0 + i / 2);
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
        explore_diagonal(s1, s2, &mut A[num][last_index], i, d0 + i / 2);
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

pub fn diagonal_transition_a_oxy_linear_oxy<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }
    let mut A = vec![vec![]; 3];
    let d0 = s2.len() - s1.len();

    let mut i = 0;
    let mut len = d0 + 1;
    let mut num = 2;
    let mut num1 = 1;
    let mut num2 = 0;
    let mut tmp = 0;
    let mut tmp1 = 0;

    let sz = len + i / 2;
    //A.push(vec![0usize; sz]);
    A[num].resize(sz, 0);

    let mut j = 0;
    A[num][j] = 0;
    j = 1;
    if j < sz {
        A[num][j] = A[num][j - 1];
        explore_diagonal(s1, s2, &mut A[num][j], i, j);
    }

    for j in 2..(len - 1) {
        A[num][j] = A[num][j - 1];

        explore_diagonal(s1, s2, &mut A[num][j], i, j);
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
        explore_diagonal(s1, s2, &mut A[num][sz - j1], j, d0 + i / 2);
        j1 += 1;
    }
    let last_index = sz - j1;
    A[num][last_index] = 0;
    if len > 1 {
        A[num][last_index] = max(A[num][last_index], A[num][len - 2]);
    }
    explore_diagonal(s1, s2, &mut A[num][last_index], i, d0 + i / 2);
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

    A[num].resize(sz, 0);

    let mut j = 0;
    A[num][j] = 0;
    if j >= (if i % 2 == 1 { 0 } else { 1 }) {
        A[num][j] = max(A[num][j], A[num1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
    }
    j = 1;
    if j < sz {
        A[num][j] = A[num][j - 1];
        A[num][j] = max(A[num][j], A[num1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
        explore_diagonal(s1, s2, &mut A[num][j], i, j);
    }

    for j in 2..(len - 1) {
        A[num][j] = A[num][j - 1];
        A[num][j] = max(A[num][j], A[num1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);

        explore_diagonal(s1, s2, &mut A[num][j], i, j);
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
        if j1 > (i + 1) % 2 {
            A[num][sz - j1] = max(A[num][sz - j1], A[num1][sz - j1 - 3 * (i + 1) % 2] + 1);
        }
        explore_diagonal(s1, s2, &mut A[num][sz - j1], j, d0 + i / 2);
        j1 += 1;
    }
    let last_index = sz - j1;
    A[num][last_index] = 0;
    A[num][last_index] = A[num1][len - 1 - (i + 1) % 2] + 1;
    if len > 1 {
        A[num][last_index] = max(A[num][last_index], A[num][len - 2]);
    }
    explore_diagonal(s1, s2, &mut A[num][last_index], i, d0 + i / 2);
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

    loop {
        if i % 2 == 0 {
            len += 1;
        }

        let sz = len + i / 2;
        //A.push(vec![0usize; sz]);
        A[num].resize(sz, 0);

        let mut j = 0;
        A[num][j] = 0;
        if j >= (if i % 2 == 1 { 0 } else { 1 }) {
            A[num][j] = max(A[num][j], A[num1][j - (if i % 2 == 1 { 0 } else { 1 })] + 1);
        }
        A[num][j] = max(A[num][j], A[num2][j] + 1);
        j = 1;
        A[num][j] = max(
            A[num][j - 1],
            max(
                A[num1][j - (if i % 2 == 1 { 0 } else { 1 })],
                if A[num2].len() > j { A[num2][j] } else { 0 },
            ) + 1,
        );

        explore_diagonal(s1, s2, &mut A[num][j], i, j);

        for j in 2..(len - 1) {
            A[num][j] = max(
                A[num][j - 1],
                max(A[num1][j - (if i % 2 == 1 { 0 } else { 1 })], A[num2][j]) + 1,
            );

            explore_diagonal(s1, s2, &mut A[num][j], i, j);
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
            if j1 > (i + 1) % 2 {
                A[num][sz - j1] = max(A[num][sz - j1], A[num1][sz - j1 - 3 * (i + 1) % 2] + 1);
                A[num][sz - j1] = max(A[num][sz - j1], A[num2][sz - j1 - 2]);
            }
            explore_diagonal(s1, s2, &mut A[num][sz - j1], j, d0 + i / 2);
            j1 += 1;
        }
        let last_index = sz - j1;
        A[num][last_index] = max(A[num1][len - 1 - (i + 1) % 2], A[num][last_index + 1]) + 1;
        if len > 1 {
            A[num][last_index] = max(A[num][last_index], A[num][len - 2]);
        }
        explore_diagonal(s1, s2, &mut A[num][last_index], i, d0 + i / 2);
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

fn path_dtm<'a>(mut A: Vec<Vec<usize>>, s1: &'a Sequence, s2: &'a Sequence) -> Vec<Pos> {
    let mut k: i32 = s1.len() as i32 - s2.len() as i32;
    let mut i = A.len() - 1;
    let mut j = A[i].len() / 2 + k as usize;
    let mut x = s1.len();
    let mut y = s2.len();
    let mut path: Vec<Pos> = vec![];

    loop {
        x = A[i][j];
        k = j as i32 - (A[i].len() / 2) as i32;
        y = (x as i32 - k) as usize;
        path.push(Pos(x as u32, y as u32));
        if i == 0 {
            path.reverse();
            return path;
        }
        let mut tmp = 0;
        if j < A[i - 1].len() {
            tmp = A[i - 1][j];
        }
        if j < A[i - 1].len() + 2 {
            tmp = max(tmp, A[i - 1][j - 2] + 1);
        }
        if j < A[i - 1].len() + 1 {
            tmp = max(tmp, A[i - 1][j - 1] + 1);
        }

        if j < A[i - 1].len() && tmp == A[i - 1][j] {
            // we came from the upper cell
            i -= 1;
            x = A[i][j];
            k = j as i32 - (A[i].len() / 2) as i32;
            y = (x as i32 - k + 1) as usize;
            path.push(Pos(x as u32, y as u32));
        } else if j < A[i - 1].len() + 2 && tmp == A[i - 1][j - 2] + 1 {
            // we came from the left cell
            i -= 1;
            j -= 2;
            x = A[i][j];
            k = j as i32 - (A[i].len() / 2) as i32;
            y = (x as i32 - k) as usize;
            x += 1;
            path.push(Pos(x as u32, y as u32));
        } else if j < A[i - 1].len() + 1 && tmp == A[i - 1][j - 1] + 1 {
            // we came from the diagonal cell
            i -= 1;
            j -= 1;
            x = A[i][j];
            k = j as i32 - (A[i].len() / 2) as i32;
            y = (x as i32 - k) as usize;
            x += 1;
            y += 1;
            path.push(Pos(x as u32, y as u32));
        }
    }

    path.reverse();
    path
}

pub fn diagonal_transition2<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
    explored: &mut Vec<Pos>,
) -> (usize, Vec<Pos>) {
    //for making a picture
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

    let mut A = vec![vec![]];
    let mut w = 1;

    let mut save_pos = |i: usize, j: usize, r: &usize, w: &usize| -> () {
        let k = j as i32 - (*w / 2) as i32;
        explored.push(Pos((*r as i32 - k) as u32, *r as u32));
    };

    A[0] = vec![0usize; w];
    save_pos(0, 0, &A[0][0], &w);
    let mut k = 0;
    while k < s2.len() && A[0][0] < s1.len() {
        if s1[A[0][0]] == s2[k] {
            A[0][0] += 1;
            k += 1;
            save_pos(0, 0, &A[0][0], &w);
        } else {
            break;
        }
    }
    if s2.len() == s1.len() && A[0][0] >= s1.len() {
        return (0, vec![Pos(s1.len() as u32, s2.len() as u32)]);
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

            save_pos(i, j, &A[i][j], &w);
            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            k = (A[i][j] as isize - d) as usize;
            while k < s2.len() && A[i][j] < s1.len() {
                if s1[A[i][j]] == s2[k] {
                    A[i][j] += 1;
                    k += 1;
                    save_pos(i, j, &A[i][j], &w);
                } else {
                    break;
                }
            }
            if d == s1.len() as isize - s2.len() as isize {
                if A[i][j] >= s1.len() {
                    let path = path_dtm(A, s1, s2);
                    return (i, path);
                }
            }
        }
    }
    unreachable!("Error! Shouldn't be here!");
}

pub fn biwfa<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence, explored: &mut Vec<Pos>) -> usize {
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
    }

    let print_vector = |C: &Vec<Vec<usize>>| -> () {
        print!("\n");
        for i in 0..C.len() {
            for j in 0..C[i].len() {
                print!("{} ", C[i][j]);
            }
            print!("\n");
        }
        print!("\n");
    };

    let mut check_point = |C: &mut Vec<Vec<usize>>, i: usize, j: usize| -> () {
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

    let check_point2 = |C: &Vec<Vec<usize>>, i: usize, j: usize| -> usize {
        let mut t = 0;
        if i > 0 {
            //println!("HALOOOOOOOOOOOOOOOOOO!!!!!{i} {j}");
            if j > 1 && j - 2 < C[i - 1].len() {
                t = max(t, C[i - 1][j - 2] + 1);
            }
            if j > 0 && j - 1 < C[i - 1].len() {
                t = max(t, C[i - 1][j - 1] + 1);
            }
            if j < C[i - 1].len() {
                t = max(t, C[i - 1][j]);
            }
        }
        t
    };

    let k0: isize = s1.len() as isize - s2.len() as isize;
    let mut A = vec![vec![]];
    let mut B = vec![vec![]];
    let mut w = 1;

    for i in 0..s2.len() {
        println!("\ni = {i} first\n\n");
        if i > 0 {
            A.push(vec![0usize; w]);
            B.push(vec![0usize; w]);
        }
        A[i] = vec![0usize; w];
        B[i] = vec![0usize; w];
        for j in 0..w {
            check_point(&mut A, i, j);
            explored.push(Pos(A[i][j] as u32, (A[i][j] + A[i].len() / 2 - j) as u32));

            let d: isize = j as isize - (w / 2) as isize;
            //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
            let mut k = (A[i][j] as isize - d) as usize;
            while k < s2.len() && A[i][j] < s1.len() {
                if s1[A[i][j]] == s2[k] {
                    A[i][j] += 1;
                    explored.push(Pos(A[i][j] as u32, (A[i][j] + A[i].len() / 2 - j) as u32));
                    k += 1;
                } else {
                    break;
                }
            }
        }

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

        println!("k_min = {k_min}\nk_max = {k_max}\n");
        println!("Comparison\n");
        if i > 0 {
            B[i - 1].reverse();
        }
        for k in k_min..=k_max {
            // println!(
            //     "{} {}",
            //     A[i][((w / 2) as isize + k) as usize],
            //     B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
            // );
            let t1 = check_point2(&A, i, ((w / 2) as isize + k) as usize);
            let t2 = check_point2(
                &B,
                i - 1,
                B[i - 1].len() - 1 - (((w - 2) / 2) as isize + k - k0) as usize,
            );
            //println!("t1 == {} t2 == {}", t1, t2);
            if A[i][((w / 2) as isize + k) as usize]
                + B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
                == s1.len()
                || (A[i][((w / 2) as isize + k) as usize]
                    >= (s1.len() + 1 - B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize])
                    && A[i][((w / 2) as isize + k) as usize] <= s1.len() + 1 - t2)
                || ((s1.len() + 1 - B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]) >= t1
                    && (s1.len() + 1 - B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize])
                        <= A[i][((w / 2) as isize + k) as usize])
            {
                return 2 * i - 1;
            }
        }
        if i > 0 {
            B[i - 1].reverse();
        }
        println!("Comparison End\n");

        print_vector(&A);
        print_vector(&B);

        println!("\ni = {i}\n\n");
        for j in 0..w {
            check_point(&mut B, i, j);
            if s1.len() >= B[i][j] {
                explored.push(Pos(
                    (s1.len() - B[i][j]) as u32,
                    ((s1.len() - B[i][j]) as isize - (B[i].len() as isize / 2 - j as isize + k0))
                        as u32,
                ));
            }
            if s1.len() >= B[i][j] + 1 {
                let d: isize = -(j as isize) + (w - 1) as isize - (w / 2) as isize + k0;
                //print!("i = {i}\nj = {j}\nA[i][j] = {}\nd ={d}\nw = {w}", A[i][j]);
                // println!("B[i][j] == {}\ni == {i}\nj == {j}\nd == {d}\n\n", B[i][j]);
                let mut x = s1.len() - B[i][j] - 1;
                let mut k = (x as isize - d) as usize;
                while k < s2.len() && x < s1.len() {
                    if s1[x] == s2[k] {
                        B[i][j] += 1;
                        explored.push(Pos(
                            (s1.len() - B[i][j]) as u32,
                            ((s1.len() - B[i][j]) as isize
                                - (B[i].len() as isize / 2 - j as isize + k0))
                                as u32,
                        ));
                        if x == 0 || k == 0 {
                            break;
                        }
                        k -= 1;
                        x -= 1;
                    } else {
                        break;
                    }
                }
            }
        }

        print_vector(&A);
        print_vector(&B);

        let k_min = -((w / 2) as isize);
        let k_max = ((w / 2) as isize) + k0;
        println!("k_min = {k_min}\nk_max = {k_max}\n");
        println!("Comparison\n");
        /*if i > 0 {
            B[i - 1].reverse();
        }*/
        B[i].reverse();
        for k in k_min..=k_max {
            // println!(
            //     "{} {}",
            //     A[i][((w / 2) as isize + k) as usize],
            //     B[i][((w / 2) as isize + k - k0) as usize]
            // );
            let t1 = check_point2(&A, i, ((w / 2) as isize + k) as usize);
            let t2 = check_point2(&B, i, B[i].len() - 1 - ((w / 2) as isize + k - k0) as usize);
            println!("{k} {t1} {t2}");
            println!("{} {}", ((w / 2) as isize + k), ((w / 2) as isize + k - k0));
            if A[i][((w / 2) as isize + k) as usize] + B[i][((w / 2) as isize + k - k0) as usize]
                == s1.len()
                || (A[i][((w / 2) as isize + k) as usize]
                    >= (s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize])
                    && A[i][((w / 2) as isize + k) as usize] <= (s1.len() + 1 - t2))
                || ((s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize]) >= t1
                    && (s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize])
                        <= A[i][((w / 2) as isize + k) as usize])
            {
                return 2 * i;
            }
        }
        /*if i > 0 {
            B[i - 1].reverse();
        }*/
        B[i].reverse();
        println!("Comparison End\n");

        w += 2;
    }
    unreachable!("Error! Shouldn't be here!");
}

#[derive(Clone)]
pub struct Args {
    pub a1: usize,
    pub a2: usize,
    pub b1: usize,
    pub b2: usize,
    pub x_offset: u32,
    pub y_offset: u32,
}

pub fn biwfa5<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
    explored: &mut Vec<Pos>,
    mut x_offset: u32,
    mut y_offset: u32,
    _target: Pos,
    mut file_number: usize,
    canvas: &mut Canvas<Window>,
    queue: &mut Vec<Args>,
) -> (usize, usize) {
    let expl = explored.len();
    let file_number1 = (file_number).clone();
    let mut f: bool = true; // if we switch sequences, proably, we need to switch positions also, I am not sure. It's only an assumption.
    if s1.len() > s2.len() {
        (s1, s2) = (s2, s1);
        (x_offset, y_offset) = (y_offset, x_offset);
        f = false;
    }

    if s1.len() == 1 {
        return (0, file_number);
    }

    let mut convert = |a: Pos| -> Pos {
        if f {
            return a;
        } else {
            return Pos(a.1, a.0);
        }
    };

    // println!(
    //     "{} {}\n{}\n{}\n",
    //     s1.len(),
    //     s2.len(),
    //     to_string(s1),
    //     to_string(s2)
    // );

    let mut check_point = |C: &mut Vec<Vec<usize>>, i: usize, j: usize| -> () {
        C[i][j] = 0;
        if i > 0 {
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

    let check_point2 = |C: &Vec<Vec<usize>>, i: usize, j: usize| -> usize {
        let mut t = 0;
        if i > 0 {
            if j > 1 && j - 2 < C[i - 1].len() {
                t = max(t, C[i - 1][j - 2] + 1);
            }
            if j > 0 && j - 1 < C[i - 1].len() {
                t = max(t, C[i - 1][j - 1] + 1);
            }
            if j < C[i - 1].len() {
                t = max(t, C[i - 1][j]);
            }
        }
        t
    };

    let mut start_wfa = |index: usize,
                         k: isize,
                         explored1: &mut Vec<Pos>,
                         canvas2: &mut Canvas<Window>,
                         mut file_number3: usize|
     -> usize {
        if index > 0
            && (index < s1.len() || (index as isize - k) < s2.len() as isize)
            && (index as isize - k) > 0
            && index <= s1.len()
            && (index as isize - k) as usize <= s2.len()
        {
            if f {
                let args = Args {
                    a1: x_offset as usize,
                    a2: x_offset as usize + index,
                    b1: y_offset as usize,
                    b2: y_offset as usize + (index as isize - k) as usize,
                    x_offset: x_offset,
                    y_offset: y_offset,
                };
                queue.push(args);
                /*(d, file_number3) = biwfa4(
                    &s1[..index].to_vec(),
                    &s2[..(index as isize - k) as usize].to_vec(),
                    explored1,
                    x_offset,
                    y_offset,
                    _target,
                    file_number3,
                    canvas2,
                );*/
            } else {
                let args = Args {
                    a1: y_offset as usize,
                    a2: y_offset as usize + (index as isize - k) as usize,
                    b1: x_offset as usize,
                    b2: x_offset as usize + index,
                    x_offset: y_offset,
                    y_offset: x_offset,
                };
                queue.push(args);
                /*(d, file_number3) = biwfa4(
                    &s2[..(index as isize - k) as usize].to_vec(),
                    &s1[..index].to_vec(),
                    explored1,
                    y_offset,
                    x_offset,
                    _target,
                    file_number3,
                    canvas2,
                );*/
            }
        }
        if s2.len() > (index as isize - k + 1) as usize
            && s1.len() > index + 1
            && index + 1 < s1.len()
            && ((index as isize - k + 1) as usize) < s2.len()
        {
            if f {
                let args = Args {
                    a1: x_offset as usize + (index + 1),
                    a2: x_offset as usize + s1.len(),
                    b1: y_offset as usize + ((index as isize - k + 1) as usize),
                    b2: y_offset as usize + s2.len(),
                    x_offset: x_offset + index as u32 + 1,
                    y_offset: y_offset + (index as isize - k + 1) as u32,
                };
                queue.push(args);
                /*(d, file_number3) = biwfa4(
                    &s1[(index + 1)..].to_vec(),
                    &s2[((index as isize - k + 1) as usize)..].to_vec(),
                    explored1,
                    x_offset + index as u32 + 1,
                    y_offset + (index as isize - k + 1) as u32,
                    _target,
                    file_number3,
                    canvas2,
                );*/
            } else {
                let args = Args {
                    a1: y_offset as usize + ((index as isize - k + 1) as usize),
                    a2: y_offset as usize + s2.len(),
                    b1: x_offset as usize + (index + 1),
                    b2: x_offset as usize + s1.len(),
                    x_offset: y_offset + (index as isize - k + 1) as u32,
                    y_offset: x_offset + index as u32 + 1,
                };
                queue.push(args);
                /*(d, file_number3) = biwfa4(
                    &s2[((index as isize - k + 1) as usize)..].to_vec(),
                    &s1[(index + 1)..].to_vec(),
                    explored1,
                    y_offset + (index as isize - k + 1) as u32,
                    x_offset + index as u32 + 1,
                    _target,
                    file_number3,
                    canvas2,
                );*/
            }
        }
        file_number3
    };

    let mut draw = |explored2: &Vec<Pos>, canvas1: &mut Canvas<Window>| -> () {
        println!(
            "{file_number1}\t{}\t{}:\n{}\n{}\n",
            s1.len(),
            s2.len(),
            to_string(s1),
            to_string(s2)
        );
        display3(
            _target,
            explored2,
            file_number1,
            canvas1,
            "evals/astar-visualization/test2/",
        );
    };

    let k0: isize = s1.len() as isize - s2.len() as isize;
    let mut A = vec![vec![]];
    let mut B = vec![vec![]];
    let mut w = 1;

    for i in 0..s2.len() {
        if i > 0 {
            A.push(vec![0usize; w]);
            B.push(vec![0usize; w]);
        }
        A[i] = vec![0usize; w];
        B[i] = vec![0usize; w];
        for j in 0..w {
            check_point(&mut A, i, j);

            explored.push(convert(Pos(
                A[i][j] as u32 + x_offset,
                (A[i][j] + A[i].len() / 2 - j) as u32 + y_offset,
            )));

            let d: isize = j as isize - (w / 2) as isize;
            let mut k = (A[i][j] as isize - d) as usize;
            while k < s2.len() && A[i][j] < s1.len() {
                if s1[A[i][j]] == s2[k] {
                    A[i][j] += 1;
                    explored.push(convert(Pos(
                        A[i][j] as u32 + x_offset,
                        (A[i][j] + A[i].len() / 2 - j) as u32 + y_offset,
                    )));
                    k += 1;
                } else {
                    break;
                }
            }
        }

        let mut k_min = 0;
        if w > 1 {
            k_min = max(-((w / 2) as isize), -(((w - 2) / 2) as isize) + k0);
        }
        let mut k_max = 0;

        if i > 0 {
            k_max = (((w - 2) / 2) as isize) + k0;
        } else {
            if s1.len() == s2.len() && A[0][0] >= s1.len() {
                return (0, file_number);
            }
            k_min = 2;
        }

        if i > 0 {
            B[i - 1].reverse();
        }
        for k in k_min..=k_max {
            let t1 = check_point2(&A, i, ((w / 2) as isize + k) as usize);
            let t2 = check_point2(
                &B,
                i - 1,
                B[i - 1].len() - 1 - (((w - 2) / 2) as isize + k - k0) as usize,
            );
            if (A[i][((w / 2) as isize + k) as usize]
                + B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
                >= (s1.len() + 1)
                && A[i][((w / 2) as isize + k) as usize] + t2 <= s1.len() + 1)
            {
                draw(explored, canvas);
                if 2 * i - 1 > 1 {
                    file_number = start_wfa(
                        A[i][((w / 2) as isize + k) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i - 1, file_number);
            }
            if ((s1.len() + 1) >= t1 + B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
                && (s1.len() + 1 - B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize])
                    <= A[i][((w / 2) as isize + k) as usize])
            {
                draw(explored, canvas);
                if 2 * i - 1 > 1 {
                    file_number = start_wfa(
                        s1.len() + 1 - B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i - 1, file_number);
            }
            if B[i - 1][(((w - 2) / 2) as isize + k - k0) as usize]
                + A[i][((w / 2) as isize + k) as usize]
                == s1.len()
            {
                draw(explored, canvas);
                if 2 * i - 1 > 1 {
                    file_number = start_wfa(
                        A[i][((w / 2) as isize + k) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i - 1, file_number);
            }
        }
        if i > 0 {
            B[i - 1].reverse();
        }

        for j in 0..w {
            check_point(&mut B, i, j);
            if s1.len() >= B[i][j]
                && s1.len() as isize + j as isize + k0 >= B[i][j] as isize + B[i].len() as isize / 2
            {
                /*println!(
                    "{}\n",
                    (s1.len() - B[i][j]) as isize - (B[i].len() as isize / 2 - j as isize + k0)
                );*/
                explored.push(convert(Pos(
                    (s1.len() - B[i][j]) as u32 + x_offset,
                    ((s1.len() - B[i][j]) as isize - (B[i].len() as isize / 2 - j as isize + k0))
                        as u32
                        + y_offset,
                )));
            }

            if s1.len() >= B[i][j] + 1 {
                let d: isize = -(j as isize) + (w - 1) as isize - (w / 2) as isize + k0;
                let mut x = s1.len() - B[i][j] - 1;
                let mut k = (x as isize - d) as usize;
                while k < s2.len() && x < s1.len() {
                    if s1[x] == s2[k] {
                        B[i][j] += 1;
                        explored.push(convert(Pos(
                            (s1.len() - B[i][j]) as u32 + x_offset,
                            ((s1.len() - B[i][j]) as isize
                                - (B[i].len() as isize / 2 - j as isize + k0))
                                as u32
                                + y_offset,
                        )));
                        if x == 0 || k == 0 {
                            break;
                        }
                        k -= 1;
                        x -= 1;
                    } else {
                        break;
                    }
                }
            }
        }

        let k_min = -((w / 2) as isize);
        let k_max = ((w / 2) as isize) + k0;
        B[i].reverse();
        for k in k_min..=k_max {
            let t1 = check_point2(&A, i, ((w / 2) as isize + k) as usize);
            let t2 = check_point2(&B, i, B[i].len() - 1 - ((w / 2) as isize + k - k0) as usize);
            if (A[i][((w / 2) as isize + k) as usize] + B[i][((w / 2) as isize + k - k0) as usize]
                >= (s1.len() + 1)
                && A[i][((w / 2) as isize + k) as usize] + t2 <= (s1.len() + 1))
            {
                draw(explored, canvas);
                if i > 0 {
                    file_number = start_wfa(
                        A[i][((w / 2) as isize + k) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i, file_number);
            }
            if ((s1.len() + 1) >= t1 + B[i][((w / 2) as isize + k - k0) as usize]
                && (s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize])
                    <= A[i][((w / 2) as isize + k) as usize])
            {
                draw(explored, canvas);
                if i > 0 {
                    file_number = start_wfa(
                        s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i, file_number);
            }
            if B[i][((w / 2) as isize + k - k0) as usize] + A[i][((w / 2) as isize + k) as usize]
                == s1.len()
            {
                draw(explored, canvas);
                if i > 0 {
                    file_number = start_wfa(
                        s1.len() + 1 - B[i][((w / 2) as isize + k - k0) as usize],
                        k,
                        explored,
                        canvas,
                        file_number + 1,
                    );
                } else {
                    file_number += 1;
                }
                return (2 * i, file_number);
            }
        }
        B[i].reverse();

        w += 2;
    }
    //*file_number -= 1;
    // let expl2 = explored.len();
    // for i in expl..expl2 {
    //     explored.pop();
    // }
    // (0, file_number)
    println!("{}\n{}", to_string(s1), to_string(s2));
    unreachable!("Error! Shouldn't be here!");
}
