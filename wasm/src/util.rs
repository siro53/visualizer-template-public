#![allow(non_snake_case, unused_macros)]

// ====== ここから共通ツール類/マクロ定義 ======
use proconio::input;
use rand::prelude::*;
use std::io::prelude::*;
use std::ops::RangeBounds;

/// 比較で最小・最大を更新するためのトレイト
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

/// 多次元配列作成用のマクロ(使わない場合もあるがそのままにしておく)
#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

/// 最大ターン数
const MAX_T: usize = 5000;

// ====== ここから問題固有の定義 ======
// 今回の問題では、以下のような情報を入力として扱う:
//   - eps, delta: 風・計測誤差を制御するパラメータ
//   - s: 初期ドローン位置
//   - ps: 目的地の座標
//   - walls: 内部壁および外周(プログラム内部で扱うため生成・参照)
//   - alphas: 計測誤差の係数(ターンごと)
//   - fs: 風の影響による速度の変化量(ターンごと)
#[derive(Clone, Debug)]
pub struct Input {
    pub eps: f64,
    pub delta: f64,
    pub s: (i64, i64),
    pub ps: Vec<(i64, i64)>,
    pub walls: Vec<(i64, i64, i64, i64)>,
    pub fs: Vec<(i64, i64)>,
    pub alphas: Vec<f64>,
}

/// Input 構造体を表示できるようにする(Displayトレイト必須)
/// ここでは問題の入出力形式に従って出力している。
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // N=目的地数, M=内壁数, eps, delta の順に出力
        writeln!(
            f,
            "{} {} {:.2} {:.2}",
            self.ps.len(),
            self.walls.len(),
            self.eps,
            self.delta
        )?;
        // ドローン初期位置
        writeln!(f, "{} {}", self.s.0, self.s.1)?;
        // 目的地座標
        for i in 0..self.ps.len() {
            writeln!(f, "{} {}", self.ps[i].0, self.ps[i].1)?;
        }
        // 内壁座標
        for i in 0..self.walls.len() {
            writeln!(
                f,
                "{} {} {} {}",
                self.walls[i].0, self.walls[i].1, self.walls[i].2, self.walls[i].3
            )?;
        }
        // 計測誤差パラメータ(α)を各ターン分
        for i in 0..MAX_T {
            writeln!(f, "{}", self.alphas[i])?;
        }
        // 風による速度変化を各ターン分
        for i in 0..MAX_T {
            writeln!(f, "{} {}", self.fs[i].0, self.fs[i].1)?;
        }
        Ok(())
    }
}

/// 入力文字列から Input 構造体を生成(問題の入力フォーマットに従う)
/// lib.rs 側の vis 関数などが内部で使う想定
pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        N: usize,               // 目的地の数
        M: usize,               // 内壁の数
        eps: f64,               // 風の強さパラメータ
        delta: f64,             // 計測誤差パラメータ
        s: (i64, i64),          // ドローン初期位置
        ps: [(i64, i64); N],    // 目的地座標
        walls: [(i64, i64, i64, i64); M],   // 内部壁
        alphas: [f64; MAX_T],   // 各ターン計測誤差用 α
        fs: [(i64, i64); MAX_T],// 各ターンの風の影響 (f_x, f_y)
    }
    Input {
        eps,
        delta,
        s,
        ps,
        walls,
        fs,
        alphas,
    }
}

/// 出力に相当する情報を格納する構造体
/// 今回は "A ax ay" や "S bx by" といった行動指示(加速 or 計測)の列を保持している
pub struct Output {
    pub out: Vec<(char, i64, i64)>,
}

/// parse_output で使用する範囲付き読み取りヘルパ
fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
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

/// 出力文字列(ユーザ解答)をパースして Output 構造体を作る
/// 元コードと同様、関数シグネチャは変えられないため、エラーは panic で対処
pub fn parse_output(f: &str) -> Output {
    let mut out = vec![];
    for line in f.lines() {
        // '#' で始まる行はコメントとみなす
        if line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        // コマンド種類('A' or 'S')
        let a = match read(it.next(), 'A'..'Z') {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        // 座標x, y
        let x = match read(it.next(), -100000..=100000) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        let y = match read(it.next(), -100000..=100000) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        // コマンド検証(問題仕様に基づく)
        if a != 'A' && a != 'S' {
            panic!("Invalid action: {}", a);
        } else if a == 'A' && x * x + y * y > 500 * 500 {
            panic!("Out of range acceleration: ({}, {})", x, y);
        } else if a == 'S' && x * x + y * y > 10000000000 {
            panic!("Out of range sensing: ({}, {})", x, y);
        } else if a == 'S' && (x, y) == (0, 0) {
            panic!("Zero vector not allowed for sensing: ({}, {})", x, y);
        }

        out.push((a, x, y));
    }
    // ターン数制限
    if out.len() > MAX_T {
        panic!("Too many actions: {}", out.len());
    }
    Output { out }
}

// ====== ここから入力生成ロジック(問題文に基づく) ======
//   問題では A, B, C の3種類を想定しているが、lib.rs とインターフェースを
//   合わせるため、引数は seed のみとし、問題タイプは seed から決定している。
pub fn gen(seed: u64, problem_id: char) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);

    // 目的地数は問題通り N=10
    let N = 10;
    // 問題タイプ毎に壁数M, eps, deltaを決める
    let (M, eps, delta) = match problem_id {
        'A' => {
            // M=0, eps=rand(1..=100), delta=rand(1..=20)*0.01
            (
                0,
                rng.gen_range(1..=100) as f64,
                rng.gen_range(1..=20) as f64 * 0.01,
            )
        }
        'B' => {
            // M=10, eps=rand(0..=1), delta=0.01
            (10, rng.gen_range(0..=1) as f64, 0.01)
        }
        'C' => {
            // M=rand(1..=10), eps=rand(1..=100), delta=rand(1..=20)*0.01
            (
                rng.gen_range(1..=10) as usize,
                rng.gen_range(1..=100) as f64,
                rng.gen_range(1..=20) as f64 * 0.01,
            )
        }
        _ => panic!("Unknown problem type"),
    };

    // ドローン初期位置(s)を生成
    let s = (rng.gen_range(-99999..=99999), rng.gen_range(-99999..=99999));

    // 目的地の生成(距離5000以上で衝突しないように)
    let mut ps: Vec<(i64, i64)> = vec![];
    while ps.len() < N {
        let p = (
            rng.gen_range(-100000..=100000),
            rng.gen_range(-100000..=100000),
        );
        // 既に入れた座標 or s の近く(距離5000以下)はNG
        if ps
            .iter()
            .chain(&[s])
            .any(|&q| (p.0 - q.0).pow(2) + (p.1 - q.1).pow(2) < 5000 * 5000)
        {
            continue;
        }
        ps.push(p);
    }

    // 内壁を生成(M個)
    let mut walls: Vec<(i64, i64, i64, i64)> = vec![];
    while walls.len() < M {
        let x1 = rng.gen_range(-90000..=90000);
        let y1 = rng.gen_range(-90000..=90000);
        let x2_tmp = x1 + rng.gen_range(-100000..=100000);
        let y2_tmp = y1 + rng.gen_range(-100000..=100000);

        // (x2_tmp, y2_tmp)が完全範囲外か、同一点ならやり直し
        if (x2_tmp < -100000 || x2_tmp > 100000) && (y2_tmp < -100000 || y2_tmp > 100000) {
            continue;
        }
        if (x1, y1) == (x2_tmp, y2_tmp) {
            continue;
        }

        // クリップ(外周範囲に収める)
        let x2 = x2_tmp.min(100000).max(-100000);
        let y2 = y2_tmp.min(100000).max(-100000);

        // 既存の壁と交差しないか & 初期位置(s)上にないか
        if walls.iter().all(|&(ax1, ay1, ax2, ay2)| {
            !P::crs_ss(
                (P(x1 as f64, y1 as f64), P(x2 as f64, y2 as f64)),
                (P(ax1 as f64, ay1 as f64), P(ax2 as f64, ay2 as f64)),
            )
        }) {
            // 壁上に初期位置があればやり直し
            if !P::crs_sp(
                (P(x1 as f64, y1 as f64), P(x2 as f64, y2 as f64)),
                P(s.0 as f64, s.1 as f64),
            ) {
                walls.push((x1, y1, x2, y2));
            }
        }
    }

    // 計測誤差パラメータ alphas (正規乱数を使う)
    // 平均1, 標準偏差delta, ただし alpha <= 0 は再度サンプリング
    let alphas = (0..MAX_T)
        .map(|_| loop {
            let t = 1.0 + rng.sample::<f64, _>(rand_distr::StandardNormal) * delta;
            if t > 0.0 {
                break t;
            }
        })
        .collect::<Vec<_>>();

    // 風の影響 fs (平均0, 標準偏差eps) -> roundして i64
    let fs = (0..MAX_T)
        .map(|_| {
            (
                (rng.sample::<f64, _>(rand_distr::StandardNormal) * eps).round() as i64,
                (rng.sample::<f64, _>(rand_distr::StandardNormal) * eps).round() as i64,
            )
        })
        .collect::<Vec<_>>();

    Input {
        eps,
        delta,
        s,
        ps,
        walls,
        fs,
        alphas,
    }
}

// ====== ここからスコア計算・可視化用ロジック ======
//   今回は壁に衝突するとペナルティ、目的地到達でボーナスなど、
//   問題文に記載の通りのロジックを実装している。

/// compute_score は最終スコアを求める関数
/// (スコア値, エラー文字列) を返す
pub fn compute_score(input: &Input, out: &Output) -> (i64, String) {
    let (mut score, mut err, (_, _, visited)) = compute_score_details(input, &out.out);
    // もし全目的地を訪問していなければエラーとする(例)
    if visited.iter().any(|&b| !b) {
        err = "Not all destinations visited".to_owned();
    }
    if !err.is_empty() {
        score = 0;
    }
    (score, err)
}

/// シミュレータ本体。各ターンの操作を順番に適用し、最終的な(スコア,エラー,最終状態)を返す。
pub fn compute_score_details(
    input: &Input,
    out: &[(char, i64, i64)],
) -> (i64, String, (P, P, Vec<bool>)) {
    let mut sim = Sim::new(input);
    for &(mv, x, y) in out {
        sim.query(input, mv, x, y);
    }
    // エラー文字列は本サンプルでは空文字(代わりに parse 時などで panic させている)
    (sim.score, String::new(), (sim.p, sim.v, sim.visited))
}

// シミュレーション用の構造体。
//   - visited: 各目的地を訪れたかどうか
//   - score, crt_score: スコア管理
//   - p: 現在位置
//   - v: 現在速度
//   - t: 現在ターン
struct Sim {
    visited: Vec<bool>,
    score: i64,
    crt_score: i64,
    p: P,
    v: P,
    t: usize,
}

impl Sim {
    fn new(input: &Input) -> Self {
        Sim {
            visited: vec![false; input.ps.len()],
            score: 0,
            crt_score: 0,
            p: P(input.s.0 as f64, input.s.1 as f64),
            v: P(0.0, 0.0),
            t: 0,
        }
    }

    /// 1ターン分の処理:
    ///   - 加速 or 計測を行う
    ///   - 風の影響を反映
    ///   - 壁衝突の判定
    ///   - 目的地到達判定
    fn query(&mut self, input: &Input, mv: char, x: i64, y: i64) -> (i32, Vec<usize>, i64) {
        let mut ret = -1;
        // 加速(A)か計測(S)かで処理を分岐
        match mv {
            'A' => {
                // 速度に加速度を足す
                self.v = self.v + P(x as f64, y as f64);
            }
            'S' => {
                // 計測: 指定方向に最も近い壁までの距離を測定
                // 外周壁(-10^5, 10^5)も含めて最小距離を出す
                let mut d = 1e9; // 大きめに初期化
                let dir = P(x as f64, y as f64);

                // 内壁 + 外周壁 でループ
                for wall in input.walls.iter().chain(
                    [
                        // 外周4辺を(wallsと同じ形式で)見る
                        (-100000, -100000, -100000, 100000),
                        (-100000, 100000, 100000, 100000),
                        (100000, 100000, 100000, -100000),
                        (100000, -100000, -100000, -100000),
                    ]
                    .iter(),
                ) {
                    let w1 = P(wall.0 as f64, wall.1 as f64);
                    let w2 = P(wall.2 as f64, wall.3 as f64);
                    // 半直線 (p -> p+dir) と 壁(w1-w2) の交点を求める
                    if let Some(cross_p) = P::pi_ll((self.p, self.p + dir), (w1, w2)) {
                        // 壁内に含まれる/半直線の向き合う側にある かを判定
                        if sig(dir.det(w1 - self.p)) * sig(dir.det(w2 - self.p)) <= 0
                            && (cross_p - self.p).dot(dir) >= 0.0
                        {
                            d.setmin((cross_p - self.p).abs2().sqrt());
                        }
                    }
                }
                // 正規乱数からサンプルされた alphas[t] を掛け、四捨五入
                d *= input.alphas[self.t];
                ret = d.round() as i64;
            }
            _ => unreachable!(),
        }

        // 風の影響を速度に加える
        self.v = self.v + P(input.fs[self.t].0 as f64, input.fs[self.t].1 as f64);

        // 毎ターンペナルティ -2
        self.crt_score -= 2;

        // ターン経過
        self.t += 1;

        // 新しい位置qを試算
        let q = self.p + self.v;

        // 壁衝突判定:
        //   (p->q)の線分が枠外 or 壁に衝突していれば、衝突とみなし速度を0に戻す
        if q.0 < -100000.0
            || q.0 > 100000.0
            || q.1 < -100000.0
            || q.1 > 100000.0
            || input.walls.iter().any(|&(x1, y1, x2, y2)| {
                P::crs_ss(
                    (self.p, q),
                    (P(x1 as f64, y1 as f64), P(x2 as f64, y2 as f64)),
                )
            })
        {
            // 壁衝突ペナルティ -100
            self.crt_score -= 100;
            self.v = P(0.0, 0.0);
            // 衝突した場合、位置は更新しない(pのまま)
            return (1, vec![], ret);
        } else {
            // 衝突なし → 目的地到達判定
            let mut hit = vec![];
            for i in 0..input.ps.len() {
                // 未達の目的地について、移動軌跡(p->q)と目的地座標の距離<=1000なら到達
                if !self.visited[i]
                    && P::dist2_sp((self.p, q), P(input.ps[i].0 as f64, input.ps[i].1 as f64))
                        <= 1000.0_f64.powi(2)
                {
                    self.visited[i] = true;
                    // 到達ボーナス +1000
                    self.crt_score += 1000;
                    hit.push(i);
                }
            }
            // 位置を更新
            self.p = q;
            // スコアの最大値を更新
            self.score.setmax(self.crt_score);
            (0, hit, ret)
        }
    }
}

// ====== 幾何ライブラリ(元コードそのまま構成) ======
use std::cmp::Ordering;
use std::ops::*;
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct P(pub f64, pub f64);

impl Add for P {
    type Output = P;
    fn add(self, a: P) -> P {
        P(self.0 + a.0, self.1 + a.1)
    }
}
impl Sub for P {
    type Output = P;
    fn sub(self, a: P) -> P {
        P(self.0 - a.0, self.1 - a.1)
    }
}
impl Mul<f64> for P {
    type Output = P;
    fn mul(self, a: f64) -> P {
        P(self.0 * a, self.1 * a)
    }
}

impl P {
    pub fn dot(self, a: P) -> f64 {
        self.0 * a.0 + self.1 * a.1
    }
    pub fn det(self, a: P) -> f64 {
        self.0 * a.1 - self.1 * a.0
    }
    pub fn abs2(self) -> f64 {
        self.dot(self)
    }
}

/// 符号判定用
fn sig<T>(x: T) -> i32
where
    T: Default + PartialOrd,
{
    match x.partial_cmp(&T::default()) {
        Some(Ordering::Greater) => 1,
        Some(Ordering::Less) => -1,
        _ => 0,
    }
}

impl P {
    /// 線分(p1->p2)と点qの距離の2乗を返す(垂線 or 最近傍点)
    pub fn dist2_sp((p1, p2): (P, P), q: P) -> f64 {
        if (p2 - p1).dot(q - p1) <= 0.0 {
            (q - p1).abs2()
        } else if (p1 - p2).dot(q - p2) <= 0.0 {
            (q - p2).abs2()
        } else {
            P::dist2_lp((p1, p2), q)
        }
    }
    /// 直線(p1->p2)と点qの距離の2乗
    pub fn dist2_lp((p1, p2): (P, P), q: P) -> f64 {
        let det = (p2 - p1).det(q - p1);
        det * det / (p2 - p1).abs2()
    }
    /// 点qが線分上にあるか(座標が一直線上かつ、p1-p2間にあるか)を判定
    pub fn crs_sp((p1, p2): (P, P), q: P) -> bool {
        P::crs_lp((p1, p2), q) && (q - p1).dot(q - p2) <= 0.0
    }
    /// 点qが直線上にあるかどうか
    pub fn crs_lp((p1, p2): (P, P), q: P) -> bool {
        (p2 - p1).det(q - p1) == 0.0
    }
    /// 線分同士が交差しているかどうか
    pub fn crs_ss((p1, p2): (P, P), (q1, q2): (P, P)) -> bool {
        let sort = |a, b| if a < b { (a, b) } else { (b, a) };
        let (lp0, up0) = sort(p1.0, p2.0);
        let (lq0, uq0) = sort(q1.0, q2.0);
        let (lp1, up1) = sort(p1.1, p2.1);
        let (lq1, uq1) = sort(q1.1, q2.1);
        if up0 < lq0 || uq0 < lp0 || up1 < lq1 || uq1 < lp1 {
            return false;
        }
        sig((p2 - p1).det(q1 - p1)) * sig((p2 - p1).det(q2 - p1)) <= 0
            && sig((q2 - q1).det(p1 - q1)) * sig((q2 - q1).det(p2 - q1)) <= 0
    }
    /// 直線と直線の交点(存在すれば)を返す(パラメータ計算)
    pub fn pi_ll((p1, p2): (P, P), (q1, q2): (P, P)) -> Option<P> {
        let d = (q2 - q1).det(p2 - p1);
        if d == 0.0 {
            // 平行 or 一致
            return None;
        }
        let r = p1 * d + (p2 - p1) * (q2 - q1).det(q1 - p1);
        Some(P(r.0 / d, r.1 / d))
    }
}

use svg::node::element::{
    Circle, Definitions, Line, Marker, Path, Rectangle, Style, Text as SvgText,
};
use svg::node::{Node, Text as NodeText};
use svg::Document;

// vis関数: turnターン目までシミュレーションしてSVGを生成
// 戻り値: (現在までの最高スコア, エラーメッセージ, SVG文字列)
pub fn vis(input: &Input, output: &Output, turn: usize) -> (i64, String, String) {
    // 1) 部分シミュレーション
    let max_step = turn.min(output.out.len());
    let mut sim = Sim::new(input);
    let mut positions = vec![sim.p];
    for i in 0..max_step {
        let (mv, ax, ay) = output.out[i];
        sim.query(input, mv, ax, ay);
        positions.push(sim.p);
    }
    let partial_score = sim.score;
    let err = String::new();

    // 2) SVGドキュメント作成
    let W = 800;
    let H = 800;
    let mut doc = Document::new()
        .set("id", "vis")
        .set("viewBox", (0, 0, W, H))
        .set("width", W)
        .set("height", H)
        .set("style", "background-color:white");

    // テキスト等のスタイル
    doc = doc.add(Style::new(
        "text { font-size: 10px; fill: #000000; font-family: monospace; }",
    ));

    // 2-1) <defs> に矢印マーカーを追加
    let arrow_marker = create_arrow_marker(); // 下部で定義
    let mut defs = Definitions::new();
    defs = defs.add(arrow_marker);

    // doc に <defs> を追加
    doc = doc.add(defs);

    // 2-2) <script> を追加 (マウス座標を表示するJS)
    let script_node = create_script_node(); // 下部で定義
                                            // Document は Node を実装しているため .add() 可能
    doc = doc.add(script_node);

    // 2-3) 壁(内壁)の描画
    for &(x1, y1, x2, y2) in &input.walls {
        let (sx1, sy1) = to_screen(x1, y1);
        let (sx2, sy2) = to_screen(x2, y2);
        let wall_line = Line::new()
            .set("x1", sx1)
            .set("y1", sy1)
            .set("x2", sx2)
            .set("y2", sy2)
            .set("stroke", "brown")
            .set("stroke-width", 2);
        doc = doc.add(wall_line);
    }

    // 2-4) 目的地の描画
    for (i, &(px, py)) in input.ps.iter().enumerate() {
        let (sx, sy) = to_screen(px, py);
        let fill_color = if sim.visited[i] { "lime" } else { "red" };
        let circle = Circle::new()
            .set("cx", sx)
            .set("cy", sy)
            .set("r", 5)
            .set("fill", fill_color);
        doc = doc.add(circle);
    }

    // 2-5) ドローン軌跡を線で描画
    for w in positions.windows(2) {
        let p1 = w[0];
        let p2 = w[1];
        let (sx1, sy1) = to_screen(p1.0 as i64, p1.1 as i64);
        let (sx2, sy2) = to_screen(p2.0 as i64, p2.1 as i64);
        let path_line = Line::new()
            .set("x1", sx1)
            .set("y1", sy1)
            .set("x2", sx2)
            .set("y2", sy2)
            .set("stroke", "blue")
            .set("stroke-width", 1);
        doc = doc.add(path_line);
    }

    // 2-6) ドローンの現在位置(青丸)
    let (dx, dy) = to_screen(sim.p.0 as i64, sim.p.1 as i64);
    let drone_circle = Circle::new()
        .set("cx", dx)
        .set("cy", dy)
        .set("r", 6)
        .set("fill", "blue");
    doc = doc.add(drone_circle);

    // 2-7) ドローンの速度ベクトル(矢印)
    let velocity_scale = 5.0; // 適宜調整
    let v_end = sim.p + (sim.v * velocity_scale);
    let (vx1, vy1) = to_screen(sim.p.0 as i64, sim.p.1 as i64);
    let (vx2, vy2) = to_screen(v_end.0 as i64, v_end.1 as i64);
    let arrow_line = Line::new()
        .set("x1", vx1)
        .set("y1", vy1)
        .set("x2", vx2)
        .set("y2", vy2)
        .set("stroke", "blue")
        .set("stroke-width", 2)
        .set("marker-end", "url(#arrow)"); // 矢印マーカーを適用
    doc = doc.add(arrow_line);

    // 2-8) 全面透明レクトでマウスイベントを拾う
    let cover_rect = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "none")
        .set("pointer-events", "all")
        .set("onmousemove", "showCoords(evt)");
    doc = doc.add(cover_rect);

    // 2-9) マウス座標表示用 <text id="coordText">
    let coord_text = SvgText::new("")
        .set("id", "coordText")
        .set("x", 10)
        .set("y", 20)
        .set("font-size", 12)
        .set("fill", "black")
        .add(NodeText::new(""));
    doc = doc.add(coord_text);

    // 3) 結果を返す
    (partial_score, err, doc.to_string())
}

/// 矢印マーカー (arrow) を定義するヘルパー
fn create_arrow_marker() -> Marker {
    let arrow_path = Path::new().set("d", "M0,0 L0,6 L6,3 Z").set("fill", "blue");
    Marker::new()
        .set("id", "arrow")
        .set("markerWidth", 6)
        .set("markerHeight", 6)
        .set("refX", 2)
        .set("refY", 3)
        .set("orient", "auto")
        .add(arrow_path)
}

/// `<script>` ノードを作成し、マウス座標をSVG座標→問題座標へ逆変換するJSを埋め込む
fn create_script_node() -> svg::node::element::Script {
    use svg::node::element::Script;

    // ※ 注意: React + wasm 環境ではインライン<script>が動かない/ブロックされることもある
    let script_body = r#"
<![CDATA[
function showCoords(evt) {
    var svg = document.getElementById('vis');
    if(!svg) return;
    var pt = svg.createSVGPoint();
    pt.x = evt.clientX;
    pt.y = evt.clientY;

    // SVG座標(0..800)へ変換
    var ctm = svg.getScreenCTM();
    if(!ctm) return;
    pt = pt.matrixTransform(ctm.inverse());

    // scaleは (800 / 200000)
    var scale = 800 / 200000;
    // 逆変換
    var problemX = (pt.x / scale) - 100000;
    var problemY = 100000 - (pt.y / scale);

    var textElem = document.getElementById('coordText');
    if(textElem) {
        textElem.textContent =
            "mouse: (" + problemX.toFixed(1) + ", " + problemY.toFixed(1) + ")";
    }
}
]]>
"#;

    // Script要素を生成
    let mut script_node = Script::new("");
    script_node = script_node.set("type", "text/ecmascript");
    // script_body を Textノードとして追加
    script_node.append(NodeText::new(script_body));

    script_node
}

/// 画面座標への変換( -100000..100000 => 0..800 程度 )
/// 上が y=0、下が y=800 のシンプルな座標系とする
fn to_screen(x: i64, y: i64) -> (f64, f64) {
    let W = 800.0;
    let H = 800.0;
    let scale = W / 200000.0; // 200000 = (100000 - (-100000))
    let sx = (x as f64 + 100000.0) * scale;
    // yは下にいくほど値が大きくなるように変換したければ以下のようにする
    let sy = (100000.0 - y as f64) * scale;
    (sx, sy)
}
