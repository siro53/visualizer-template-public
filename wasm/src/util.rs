#![allow(non_snake_case, unused_macros)]

use noise::{NoiseFn, Perlin};
use proconio::{input, marker::Chars};
use rand::prelude::*;
use std::ops::RangeBounds;
use svg::node::element::{Rectangle, Style, Text};
use svg::Node;
use web_sys::console::log_1;

/// SetMinMax トレイト: 小さい/大きい値を簡単に更新するための補助
pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

/// マクロ: 二次元配列などを生成する際に便利
#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

/// 入力を保持する構造体.
/// - N: マップの縦横サイズ(問題文により常に20)
/// - M: 鉱石の種類数(問題Aでは1, 問題Bでは3, 問題Cでは1)
/// - cs: マップ上の文字情報('@', 'a'..'z', 'A'..'Z', '.')
#[derive(Clone, Debug)]
pub struct Input {
    pub N: usize,
    pub M: usize,
    pub cs: Vec<Vec<char>>,
}

/// Inputを表示するためのトレイト実装(デバッグ用や入力書き出し用)
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.N, self.M)?;
        for i in 0..self.N {
            writeln!(f, "{}", self.cs[i].iter().collect::<String>())?;
        }
        Ok(())
    }
}

/// 出力解析などで数値や文字を安全に読み込むための補助関数
pub fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if !range.contains(&v) {
                Err(format!("Out of range: {}", v))
            } else {
                Ok(v)
            }
        } else {
            Err(format!("Parse error: {}", v))
        }
    } else {
        Err("Unexpected EOF".to_owned())
    }
}

/// 入力をパースする関数(AtCoder上で与えられる入力を想定)
pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        N: usize, M: usize,
        cs: [Chars; N],
    }
    Input { N, M, cs }
}

/// 方向は {U, D, L, R} の4通りを想定
const DIJ: [(usize, usize); 4] = [
    (!0, 0), // U(i-1, j)
    (1, 0),  // D(i+1, j)
    (0, !0), // L(i, j-1)
    (0, 1),  // R(i, j+1)
];
const DIR: [char; 4] = ['U', 'D', 'L', 'R'];

/// 行動(Action)を表す列挙型
/// - Move(d): 上下左右へ自分だけ移動
/// - Carry(d): 現在位置の岩や鉱石を隣へ運ぶ(自分もその隣に移動)
/// - Roll(d): 現在位置の岩や鉱石を「転がす」
#[derive(Clone, Debug, Copy)]
pub enum Action {
    Move(usize),
    Carry(usize),
    Roll(usize),
}

/// 出力を保持する構造体
/// - out: 行動(Action)の列
pub struct Output {
    pub out: Vec<Action>,
}

/// 出力文字列をパースし、問題の制約に合わない場合はErrを返す関数
/// - "a d" という形で、(a: 行動種類1..3, d: U/D/L/R) を並べたものを想定
pub fn parse_output(input: &Input, f: &str) -> Result<Output, String> {
    let mut out = vec![];
    let mut ss = f.split_whitespace().peekable();
    while ss.peek().is_some() {
        let a = read(ss.next(), 1..=3)?;
        let dir = read(ss.next(), 'A'..='Z')?;
        let Some(d) = DIR.iter().position(|&x| x == dir) else {
            return Err(format!("Invalid direction: {}", dir));
        };
        out.push(match a {
            1 => Action::Move(d),
            2 => Action::Carry(d),
            3 => Action::Roll(d),
            _ => unreachable!(),
        });
    }
    if out.len() > 10000 {
        return Err("Too many actions".to_owned());
    }
    Ok(Output { out })
}

/// 問題A/B/Cごとに、指定された方法で入力を生成する関数
/// - `problem` は "A" or "B" or "C" のいずれかを想定
/// - シード値に基づいてランダムに生成
pub fn gen(seed: u64, problem: &str) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    match problem {
        "A" => {
            // A問題: M=1, 穴(A)1個, 鉱石(a) 2N個, 岩(@) 2N個をランダム配置
            let N = 20;
            let M = 1;
            let mut cs = mat!['.'; N; N];
            let mut ps = vec![];
            for i in 0..N {
                for j in 0..N {
                    ps.push((i, j));
                }
            }
            // シャッフルして先頭から穴/鉱石/岩を置いていく
            ps.shuffle(&mut rng);
            let (i, j) = ps.pop().unwrap();
            cs[i][j] = 'A'; // 穴1個
            for _ in 0..2 * N {
                let (i, j) = ps.pop().unwrap();
                cs[i][j] = 'a'; // 鉱石
            }
            for _ in 0..2 * N {
                let (i, j) = ps.pop().unwrap();
                cs[i][j] = '@'; // 岩
            }
            Input { N, M, cs }
        }
        "B" => {
            // B問題: M=3, 穴ABC 各1個, 鉱石abc 各N個, 岩なし
            let N = 20;
            let M = 3;
            loop {
                let mut cs = mat!['.'; N; N];
                let mut ps = vec![];
                for i in 0..N {
                    for j in 0..N {
                        ps.push((i, j));
                    }
                }
                ps.shuffle(&mut rng);

                // 穴の配置
                let mut ss = vec![];
                for k in 0..3 {
                    let (i, j) = ps.pop().unwrap();
                    cs[i][j] = (b'A' + k as u8) as char;
                    ss.push((i, j)); // 各穴の位置を記録
                }
                // 鉱石の配置
                for k in 0..3 {
                    for _ in 0..N {
                        let (i, j) = ps.pop().unwrap();
                        cs[i][j] = (b'a' + k as u8) as char;
                    }
                }

                // 穴から同種類の鉱石へ到達可能かチェックし、全てOKであれば採用
                let mut ok = true;
                for k in 0..3 {
                    let t = (b'a' + k as u8) as char; // a, b, c
                    let s = ss[k]; // A, B, C の位置
                    let mut visited = mat![false; N; N];
                    let mut stack = vec![s];
                    visited[s.0][s.1] = true;
                    while let Some((i, j)) = stack.pop() {
                        for d in 0..4 {
                            let (di, dj) = DIJ[d];
                            let i2 = i.wrapping_add(di);
                            let j2 = j.wrapping_add(dj);
                            if i2 < N && j2 < N && !visited[i2][j2] {
                                // 穴に対応する鉱石か '.' なら通れる
                                if cs[i2][j2] == '.' || cs[i2][j2] == t {
                                    visited[i2][j2] = true;
                                    stack.push((i2, j2));
                                }
                            }
                        }
                    }
                    // 未到達の同種類鉱石がある場合は生成失敗
                    for i2 in 0..N {
                        for j2 in 0..N {
                            if cs[i2][j2] == t && !visited[i2][j2] {
                                ok = false;
                            }
                        }
                    }
                }
                if ok {
                    return Input { N, M, cs };
                }
            }
        }
        "C" => {
            // C問題: M=1, 岩大量。Perlin noiseに基づいて上位N*N/2マスを岩に。
            // 残りから穴Aを1マス、鉱石aを2Nマス配置。
            let N = 20;
            let M = 1;
            let perlin = Perlin::new(rng.gen());
            let D = 10.0;
            let mut ps = vec![];
            for i in 0..N {
                for j in 0..N {
                    let x = i as f64 / D;
                    let y = j as f64 / D;
                    // Perlin値を取得し、(perlin値, i, j) としてpush
                    ps.push((perlin.get([x, y]), i, j));
                }
            }
            // Perlin値が大きい順に上半分を岩に
            ps.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let mut cs = mat!['.'; N; N];
            for _ in 0..(N * N / 2) {
                let (_, i, j) = ps.pop().unwrap();
                cs[i][j] = '@';
            }
            // 残り半分に対してシャッフルし、穴Aと鉱石aを2N個配置
            ps.shuffle(&mut rng);
            let (_, i, j) = ps.pop().unwrap();
            cs[i][j] = 'A';
            for _ in 0..2 * N {
                let (_, i, j) = ps.pop().unwrap();
                cs[i][j] = 'a';
            }
            Input { N, M, cs }
        }
        _ => {
            panic!("Unknown problem: {}", problem);
        }
    }
}

/// 得点計算(不正等ある場合はscore=0にする)
pub fn compute_score(input: &Input, out: &Output) -> (i64, String) {
    let (mut score, err, _) = compute_score_details(input, &out.out);
    if !err.is_empty() {
        score = 0;
    }
    (score, err)
}

/// スコア計算の内部処理(途中経過なども知りたい場合に利用)
/// - 戻り値は (score, err文字列, () ) という形にする
pub fn compute_score_details(input: &Input, out: &[Action]) -> (i64, String, ()) {
    // マップをコピーして改変しながらシミュレーション
    let mut cs = input.cs.clone();
    let mut pos = (0, 0); // 自分の初期位置(穴Aの場所と問題文で指定)
    let mut K = 0; // 全鉱石数
    let mut A = 0; // 正しく穴に落とせた鉱石数

    // 初期盤面走査: 穴'A'..'Z' のうち 'A'(=0番目) にいることを想定しつつ
    // 鉱石'a'..'z'の数を数える
    for i in 0..input.N {
        for j in 0..input.N {
            if cs[i][j] == 'A' {
                pos = (i, j);
            } else if cs[i][j].is_ascii_lowercase() {
                K += 1; // 鉱石をカウント
            }
        }
    }

    // 行動を順に適用
    for (t, &action) in out.iter().enumerate() {
        match action {
            Action::Move(d) => {
                let (di, dj) = DIJ[d];
                pos.0 = pos.0.wrapping_add(di);
                pos.1 = pos.1.wrapping_add(dj);
                if pos.0 >= input.N || pos.1 >= input.N {
                    return (0, format!("Out of the board (turn {t})"), ());
                }
            }
            Action::Carry(d) => {
                let (di, dj) = DIJ[d];
                // 現在位置に岩/鉱石がなければエラー
                if !(('a'..='z').contains(&cs[pos.0][pos.1]) || cs[pos.0][pos.1] == '@') {
                    return (0, format!("No item to carry (turn {t})"), ());
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                // 自分もその方向へ移動
                pos.0 = pos.0.wrapping_add(di);
                pos.1 = pos.1.wrapping_add(dj);
                if pos.0 >= input.N || pos.1 >= input.N {
                    return (0, format!("Out of the board (turn {t})"), ());
                }
                // 行き先に別の岩/鉱石があれば衝突エラー
                if matches!(cs[pos.0][pos.1], '@' | 'a'..='z') {
                    return (0, format!("Collision (turn {t})"), ());
                } else if cs[pos.0][pos.1].is_ascii_uppercase() {
                    // 穴に落とす
                    if cs[pos.0][pos.1].to_ascii_lowercase() == c {
                        A += 1; // 正しく対応する穴に落とせた
                    }
                } else {
                    // '.' ならそこに置く
                    debug_assert_eq!(cs[pos.0][pos.1], '.');
                    cs[pos.0][pos.1] = c;
                }
            }
            Action::Roll(d) => {
                let (di, dj) = DIJ[d];
                if !(('a'..='z').contains(&cs[pos.0][pos.1]) || cs[pos.0][pos.1] == '@') {
                    return (0, format!("No item to roll (turn {t})"), ());
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                let mut crt = pos;
                // 転がして岩や鉱石や壁(盤外)に当たるか穴に落ちるまで進む
                loop {
                    let next = (crt.0.wrapping_add(di), crt.1.wrapping_add(dj));
                    if next.0 >= input.N || next.1 >= input.N {
                        // 盤外に出るならその直前で止まる
                        cs[crt.0][crt.1] = c;
                        break;
                    }
                    // 次マスに岩or鉱石があるなら衝突
                    if matches!(cs[next.0][next.1], '@' | 'a'..='z') {
                        cs[crt.0][crt.1] = c;
                        break;
                    }
                    // 穴があるなら判定
                    if cs[next.0][next.1].is_ascii_uppercase() {
                        // 穴の種類が一致すれば得点
                        if cs[next.0][next.1].to_ascii_lowercase() == c {
                            A += 1;
                        }
                        break; // 落ちて盤から消える
                    }
                    // 何も無ければさらに進む
                    crt = next;
                }
            }
        }
    }

    // スコア計算
    // - A == K の場合: 1e6 * (1 + log2(10000 / T)) を丸めた値
    // - それ以外の場合: 1e6 * (A / K) を丸めた値
    let score = if A == K {
        // 行動が0の場合はlog2が発散しないよう一応ガードする(実際には問題上T=0はありえない想定)
        let t = out.len().max(1) as f64;
        (1_000_000.0 * (1.0 + (10000.0 / t).log2())).round() as i64
    } else {
        (1_000_000.0 * (A as f64 / K as f64)).round() as i64
    };

    (score, String::new(), ())
}

/// 可視化のための返却用データ(スコア, エラーメッセージ, SVG文字列)
pub struct VisRet {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

/// 可視化描画関数.
/// - turn 回分の行動を適用した後の盤面をSVGで出力し、score・errも返す
pub fn vis(input: &Input, output: &Output, turn: usize) -> VisRet {
    // マップを再現し、turn回ぶんの行動だけ適用
    let mut cs = input.cs.clone();
    let mut pos = (0, 0);
    for i in 0..input.N {
        for j in 0..input.N {
            if cs[i][j] == 'A' {
                pos = (i, j);
            }
        }
    }

    // 途中までの行動を適用して盤面更新
    let mut A_ok = 0;
    let mut K = 0;
    for i in 0..input.N {
        for j in 0..input.N {
            if cs[i][j].is_ascii_lowercase() {
                K += 1;
            }
        }
    }

    let mut err = String::new();
    let mut done = false;

    for (t, &action) in output.out.iter().enumerate() {
        if t >= turn {
            break;
        }
        match action {
            Action::Move(d) => {
                let (di, dj) = DIJ[d];
                pos.0 = pos.0.wrapping_add(di);
                pos.1 = pos.1.wrapping_add(dj);
                if pos.0 >= input.N || pos.1 >= input.N {
                    err = format!("Out of the board (turn {t})");
                    done = true;
                    break;
                }
            }
            Action::Carry(d) => {
                let (di, dj) = DIJ[d];
                if !(('a'..='z').contains(&cs[pos.0][pos.1]) || cs[pos.0][pos.1] == '@') {
                    err = format!("No item to carry (turn {t})");
                    done = true;
                    break;
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                pos.0 = pos.0.wrapping_add(di);
                pos.1 = pos.1.wrapping_add(dj);
                if pos.0 >= input.N || pos.1 >= input.N {
                    err = format!("Out of the board (turn {t})");
                    done = true;
                    break;
                }
                if matches!(cs[pos.0][pos.1], '@' | 'a'..='z') {
                    err = format!("Collision (turn {t})");
                    done = true;
                    break;
                } else if cs[pos.0][pos.1].is_ascii_uppercase() {
                    if cs[pos.0][pos.1].to_ascii_lowercase() == c {
                        A_ok += 1;
                    }
                } else {
                    cs[pos.0][pos.1] = c;
                }
            }
            Action::Roll(d) => {
                let (di, dj) = DIJ[d];
                if !(('a'..='z').contains(&cs[pos.0][pos.1]) || cs[pos.0][pos.1] == '@') {
                    err = format!("No item to roll (turn {t})");
                    done = true;
                    break;
                }
                let c = cs[pos.0][pos.1];
                cs[pos.0][pos.1] = '.';
                let mut crt = pos;
                loop {
                    let next = (crt.0.wrapping_add(di), crt.1.wrapping_add(dj));
                    if next.0 >= input.N || next.1 >= input.N {
                        cs[crt.0][crt.1] = c;
                        break;
                    }
                    if matches!(cs[next.0][next.1], '@' | 'a'..='z') {
                        cs[crt.0][crt.1] = c;
                        break;
                    }
                    if cs[next.0][next.1].is_ascii_uppercase() {
                        if cs[next.0][next.1].to_ascii_lowercase() == c {
                            A_ok += 1;
                        }
                        break;
                    }
                    crt = next;
                }
            }
        }
        if done {
            break;
        }
    }

    // スコア計算 (途中までの行動なのであくまで暫定値)
    let score = if err.is_empty() {
        if A_ok == K {
            let t = turn.max(1) as f64;
            (1_000_000.0 * (1.0 + (10000.0 / t).log2())).round() as i64
        } else {
            (1_000_000.0 * (A_ok as f64 / K as f64)).round() as i64
        }
    } else {
        0
    };

    // SVG構築
    // (マスを白かピンクで塗り、文字を黒で書く)
    let W = 20; // 1マスあたりの描画幅
    let H = 20; // 1マスあたりの描画高さ
    let total_w = (input.N * W) as i32;
    let total_h = (input.N * H) as i32;

    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("viewBox", (0, 0, total_w, total_h))
        .set("width", total_w)
        .set("height", total_h)
        .set("style", "background-color:white");

    doc = doc.add(Style::new(format!(
        "text {{text-anchor: middle;dominant-baseline: central; font-size: {}}}",
        10
    )));

    for i in 0..input.N {
        for j in 0..input.N {
            // プレイヤー位置ならピンク、それ以外は白
            let fill_color = if (i, j) == pos { "#FFC0CB" } else { "#FFFFFF" };
            let rect = Rectangle::new()
                .set("x", (j * W) as i32)
                .set("y", (i * H) as i32)
                .set("width", W)
                .set("height", H)
                .set("fill", fill_color)
                .set("opacity", 0.7)
                .set("stroke", "black")
                .set("stroke-width", 1);
            doc = doc.add(rect);

            // マス上にあるキャラクター(岩/鉱石/穴など)を文字表示
            let ch = cs[i][j];
            if ch != '.' {
                let tx = Text::new("")
                    .set("x", (j * W + W / 2) as i32)
                    .set("y", (i * H + H / 2) as i32)
                    .set("fill", "black")
                    .add(svg::node::Text::new(ch.to_string()));
                doc = doc.add(tx);
            }
            // プレイヤー位置にいた場合は上から文字を書く必要はない
            // (マス自体を塗りつぶすだけにしている)
        }
    }

    VisRet {
        score,
        err,
        svg: doc.to_string(),
    }
}

/// 与えられた出力(out)の最大ターン数を返す(例: outの長さ)
/// 今回は単に行動列の長さを最大ターン数とする
pub fn get_max_turn(_input: &Input, output: &Output) -> usize {
    output.out.len()
}
