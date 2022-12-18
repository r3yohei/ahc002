#![allow(non_snake_case, unused)]

use proconio::{*, marker::*};
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use std::cmp::*;
use std::collections::*;
use std::vec;

// 型の定義
type Action = usize;
type Actions = Vec<usize>;
type ScoreType = i32;
pub type Output = String;

// 定数
const INF: ScoreType = 1000000000;
const TILE_SIZE: usize = 50;
const DIJ: [(usize, usize); 4] = [(0, !0), (0, 1), (!0, 0), (1, 0)];
const DIR: [char; 4] = ['L', 'R', 'U', 'D'];
const SOLUTION_SIZE: usize = 3; // DFSの探索方向の順序をいくつかためすやつでは3がベスト
const T0: f64 = 2000.;
const T1: f64 = 600.;

// 好みで変更する
const TIME_LIMIT: f64 = 1.985;
// const TIME_LIMIT: f64 = 120.;
const SEED: u64 = 20210325;
const VIEW_POINTS: bool = false; // デバッグの時得点を表示するかどうか

/// 入力で与えられる情報をまとめた構造体
/// s: 開始位置  
/// tiles: タイルの位置  
/// ps: 座標ごとの得点 
#[derive(Clone)]
pub struct Input {
    pub s: (usize, usize),
    pub tiles: Vec<Vec<usize>>,
    pub ps: Vec<Vec<i32>>,
}

#[derive(Clone)]
/// 位置を表す構造体
struct Position {
    i_: usize,
    j_: usize,
}

#[derive(Clone)]
/// END_TURN_: 探索を終了するターン<br>
/// turn_: 現在のターン<br>
/// seen_: タイルを踏んだかどうか<br>
/// pos_: 現在位置<br>
/// output_: 経路の出力<br>
/// steps_: 移動経路の座標<br>
/// game_score_: 得点(実際の得点)<br>
/// evaluated_score_: 探索上で評価したスコア<br>
/// first_action_: 探索木のルートノードで最初に選択した行動<br>
struct TileState {
    END_TURN_: usize,
    turn_: usize,
    seen_: Vec<bool>,
    pos_: Position,
    pub output_: Output,
    pub steps_: Vec<(usize, usize)>,
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
    pub first_action_: Action,
}

impl TileState {
    pub fn new(input: &Input, end_turn: usize, pos: (usize, usize)) -> Self {
        let M_ = input
            .tiles
            .iter()
            .map(|t| t.iter().max().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut seen_ = vec![false; M_];
        let pos_ = Position {
            i_: pos.0,
            j_: pos.1,
        };
        seen_[input.tiles[pos_.i_][pos_.j_]] = true;
        let steps_ = vec![(pos_.i_, pos_.j_)];
        let game_score_ = input.ps[pos_.i_][pos_.j_];
        let evaluated_score_ = 0;

        Self {
            END_TURN_: end_turn,
            turn_: 0,
            seen_,
            pos_,
            steps_,
            output_: String::new(),
            game_score_,
            evaluated_score_,
            first_action_: !0,
        }
    }
    
    /// [どのゲームでも実装する]: 探索用の盤面評価をする
    /// 探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
    pub fn evaluateScore(&mut self) {
        // 得点が大きいほど+
        self.evaluated_score_ = self.game_score_ / 1000;
        // // turnが多いほど+
        // self.evaluated_score_ += self.turn_ as i32;
        // // 端に到達していると+
        // if self.seen_[self.input_.tiles[0][0]] {
        //     self.evaluated_score_ += 100;
        // } 
        // if self.seen_[self.input_.tiles[0][TILE_SIZE-1]] {
        //     self.evaluated_score_ += 100;
        // } 
        // if self.seen_[self.input_.tiles[TILE_SIZE-1][0]] {
        //     self.evaluated_score_ += 100;
        // } 
        // if self.seen_[self.input_.tiles[TILE_SIZE-1][TILE_SIZE-1]] {
        //     self.evaluated_score_ += 100;
        // }
        // 分岐をたくさん持つ方がいい (複数方向に進めるマスをたくさん持つ方がいい)
        // self.evaluated_score_ += self.branch_list_.len() as i32 * 10;
    }

    /// [どのゲームでも実装する]: ゲームの終了判定
    pub fn isDone(&self) -> bool {
        self.turn_ == self.END_TURN_
    }

    /// [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
    pub fn advance(&mut self, input: &Input, action: Action) {
        self.pos_.i_ = self.pos_.i_.wrapping_add(DIJ[action].0);
        self.pos_.j_ = self.pos_.j_.wrapping_add(DIJ[action].1);
        self.steps_.push((self.pos_.i_, self.pos_.j_));
        self.game_score_ += input.ps[self.pos_.i_][self.pos_.j_];
        self.seen_[input.tiles[self.pos_.i_][self.pos_.j_]] = true;
        self.turn_ += 1;
        self.output_.push(DIR[action]);
    }

    /// [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
    pub fn legalActions(&self, input: &Input) -> Actions {
        let mut actions: Actions = vec![];
        for action in 0..4 {
            let ni = self.pos_.i_.wrapping_add(DIJ[action].0);
            let nj = self.pos_.j_.wrapping_add(DIJ[action].1);
            if ni < TILE_SIZE && nj < TILE_SIZE && !self.seen_[input.tiles[ni][nj]] {
                actions.push(action);
            }
        }
        actions
    }

    /// [実装しなくてもよいが実装すると便利]: 現在のゲーム状況を標準エラー出力に出力する
    pub fn toString(&self, input: &Input) {
        let mut path = vec![vec!["  "; TILE_SIZE]; TILE_SIZE];
        let string: Vec<Vec<String>> = input
            .ps
            .iter()
            .map(|pvec| pvec.iter().map(|p| format!("{:02}", p)).collect())
            .collect();
        if VIEW_POINTS {
            for i in 0..TILE_SIZE {
                for j in 0..TILE_SIZE {
                    path[i][j] = string[i][j].as_str();
                }
            }
        }
        // 移動経路に罫線を引く
        let (i, j) = input.s;
        path[i][j] = "@@";
        for i in 1..self.turn_ {
            let (h, w) = self.steps_[i];
            let mut dir = String::new();
            dir.push(self.output_.chars().nth(i - 1).unwrap());
            dir.push(self.output_.chars().nth(i).unwrap());
            // 直前の移動方向 + 今回の移動方向によって引く罫線を決定
            path[h][w] = match dir.as_str() {
                "LL" => "━━",
                "LU" => "┗━",
                "LD" => "┏━",
                "RR" => "━━",
                "RU" => "┛ ",
                "RD" => "┓ ",
                "UL" => "┓ ",
                "UR" => "┏━",
                "UU" => "┃ ",
                "DL" => "┛ ",
                "DR" => "┗━",
                "DD" => "┃ ",
                _ => unreachable!(),
            }
        }
        // 出力パート
        let isConnectHorizontal =
            |h: usize, w: usize| w + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h][w + 1];
        let isConnectVertical =
            |h: usize, w: usize| h + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h + 1][w];
        for h in 0..TILE_SIZE {
            for w in 0..TILE_SIZE {
                if !isConnectVertical(h, w) {
                    // 下のタイルとつながっていなかったら下線を引く
                    eprint!("\x1b[4m");
                }
                if self.seen_[input.tiles[h][w]] {
                    // 踏んだタイルなら色を塗る
                    eprint!("\x1b[46m");
                }
                eprint!("{}", path[h][w]);
                if isConnectHorizontal(h, w) {
                    // 右のタイルと繋がっていたら文字修飾を引き継いで空白を出力
                    eprint!(" ")
                } else {
                    // 右のタイルと繋がっていなかったら修飾をリセットして|を出力
                    eprint!("\x1b[0m");
                    eprint!("|");
                }
            }
            eprintln!();
        }
        eprintln!("turn : {}", self.turn_);
        eprintln!("score: {}", self.game_score_);
    }
}

/// [どのゲームでも実装する] : 探索時のソート用に評価を比較する
impl Ord for TileState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score_.cmp(&other.evaluated_score_)
    }
}
impl PartialEq for TileState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score_ == other.evaluated_score_
    }
}
impl Eq for TileState {} // ここは空でOK
impl PartialOrd for TileState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluated_score_.partial_cmp(&other.evaluated_score_)
    }
}

type State = TileState;

#[allow(dead_code)]
/// ランダムに行動を決定する
fn randomAction(rng: &mut Pcg64Mcg, input: &Input, state: &State) -> Option<Action> {
    let legalActions = state.legalActions(input);
    if legalActions.is_empty() {
        return None;
    }
    return Some(legalActions[rng.gen_range(0, 100) as usize % (legalActions.len())]);
}

// 初期解構築のためのDFS
// 後で焼きやすいようにスカスカに作るオプションもつけられます
fn dfs_making_first_solution(d: &Vec<usize>, x: usize, y: usize, prev_action: usize, h: usize, w: usize, input: &Input, mut seen: &mut Vec<bool>, mut actions: &mut Vec<usize>, mut best_actions_bh: &mut BinaryHeap<(Reverse<i32>, i32, Vec<usize>)>, mut score: i32, mut best_score: i32, crt_time: f64, tl: f64) {
    // 再帰の開始からtl秒たったらやめることにする
    if get_time() - crt_time > tl {return;}

    for &i in d {
        // 範囲外参照を防ぐ
        let to_x = x.wrapping_add(DIJ[i].0);
        let to_y = y.wrapping_add(DIJ[i].1);
        if to_x < h && to_y < w {
            if seen[input.tiles[to_x][to_y]] {
                continue;
            }
            // スカスカに作るオプション
            // let left_x = to_x.wrapping_add(DIJ[0].0);
            // let left_y = to_y.wrapping_add(DIJ[0].1);
            // let right_x = to_x.wrapping_add(DIJ[1].0);
            // let right_y = to_y.wrapping_add(DIJ[1].1);
            // let up_x = to_x.wrapping_add(DIJ[2].0);
            // let up_y = to_y.wrapping_add(DIJ[2].1);
            // let down_x = to_x.wrapping_add(DIJ[3].0);
            // let down_y = to_y.wrapping_add(DIJ[3].1);
            // if i == 0 {
            //     // 左に来たとき，その左/上/下/左上/左下が空いていないなら進まない
            //     if (left_x < h && left_y < w && seen[input.tiles[left_x][left_y]]) || (up_x < h && up_y < w && seen[input.tiles[up_x][up_y]]) || (down_x < h && down_y < w && seen[input.tiles[down_x][down_y]]) {//|| (up_x < h && left_y < w && seen[input.tiles[up_x][left_y]]) || (down_x < h && left_y < w && seen[input.tiles[down_x][left_y]]) {
            //         continue;
            //     }
            // } else if i == 1 {
            //     // 右に来たとき
            //     if (right_x < h && right_y < w && seen[input.tiles[right_x][right_y]]) || (up_x < h && up_y < w && seen[input.tiles[up_x][up_y]]) || (down_x < h && down_y < w && seen[input.tiles[down_x][down_y]]) {//|| (up_x < h && right_y < w && seen[input.tiles[up_x][right_y]]) || (down_x < h && right_y < w && seen[input.tiles[down_x][right_y]]) {
            //         continue;
            //     }
            // } else if i == 2 {
            //     // 上に来たとき
            //     if (left_x < h && left_y < w && seen[input.tiles[left_x][left_y]]) || (right_x < h && right_y < w && seen[input.tiles[right_x][right_y]]) || (up_x < h && up_y < w && seen[input.tiles[up_x][up_y]]) {//|| (up_x < h && left_y < w && seen[input.tiles[up_x][left_y]]) || (up_x < h && right_y < w && seen[input.tiles[up_x][right_y]]) {
            //         continue;
            //     }
            // } else {
            //     // 下に来たとき
            //     if (left_x < h && left_y < w && seen[input.tiles[left_x][left_y]]) || (right_x < h && right_y < w && seen[input.tiles[right_x][right_y]]) || (down_x < h && down_y < w && seen[input.tiles[down_x][down_y]]) {//|| (down_x < h && left_y < w && seen[input.tiles[down_x][left_y]]) || (down_x < h && right_y < w && seen[input.tiles[down_x][right_y]]) {
            //         continue;
            //     }
            // }
            seen[input.tiles[to_x][to_y]] = true;
            actions.push(i);
            score += input.ps[to_x][to_y];
            // scoreバージョン
            if score >= best_score {
                // 注：heapにactionsを入れすぎるとすぐにMLEになる
                // そのため，一定サイズ以下に保つ機構を入れる
                // スコアの低いものをドロップしたいので，heapの先頭にはReverse(score)を入れて置き，サイズが大きくなったらpopする
                best_actions_bh.push((Reverse(score), score, actions.clone()));
                if best_actions_bh.len() >= 2 {
                    best_actions_bh.pop();
                }
                best_score = score;
            }
            // sizeバージョン
            dfs_making_first_solution(&d, to_x, to_y, i, h, w, &input, &mut seen, &mut actions, &mut best_actions_bh, score, best_score, crt_time, tl);
            seen[input.tiles[to_x][to_y]] = false;
            actions.pop();
            score -= input.ps[to_x][to_y];
        }
    }
}

// p1->p2へたどり着く経路を探すためのdfs
// p2へ着いた時点で終了する
// (スコア, action)をbinaryheapに格納する
// actionの単体での取りだしが容易にできないため, それを格納するbhへ書き込むことで取り出す
fn dfs_to_destination(x1: usize, y1: usize, x2: usize, y2: usize, h: usize, w: usize, input: &Input, mut seen: &mut Vec<bool>, mut actions: &mut Vec<usize>, mut action_bh: &mut BinaryHeap<(i32, Vec<usize>)>, mut score: i32, crt_time: f64, tl: f64) {
    // x1 == x2 && y1 == y2だけじゃネストされた他の再帰関数全てを終わらせられない
    // 再帰の開始からtl秒たったらやめることにする
    if get_time() - crt_time > tl {return;}

    if x1 == x2 && y1 == y2 {
        action_bh.push((score, actions.clone()));
    }

    let d = vec![0,2,1,3]; // [ToDO] これも方向の並び2通りくらい試したほうがいいか
    for &i in &d {
        // 範囲外参照を防ぐ
        let to_x = x1.wrapping_add(DIJ[i].0);
        let to_y = y1.wrapping_add(DIJ[i].1);
        if to_x < h && to_y < w {
            if seen[input.tiles[to_x][to_y]] {
                continue;
            }
            seen[input.tiles[to_x][to_y]] = true;
            actions.push(i);
            score += input.ps[to_x][to_y];
            dfs_to_destination(to_x, to_y, x2, y2, h, w, &input, &mut seen, &mut actions, &mut action_bh, score, crt_time, tl);
            seen[input.tiles[to_x][to_y]] = false;
            actions.pop();
            score -= input.ps[to_x][to_y];
        }
    }
}

pub fn get_time() -> f64 {
	static mut STIME: f64 = -1.0;
	let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
	let ms = t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9;
	unsafe {
		if STIME < 0.0 {
			STIME = ms;
		}
		// ローカル環境とジャッジ環境の実行速度差はget_timeで吸収しておくと便利
		#[cfg(feature="local")]
		{
			(ms - STIME) * 10.0
		}
		#[cfg(not(feature="local"))]
		{
			(ms - STIME)
		}
	}
}

fn main() {
    get_time();
    input! {
        s: (usize, usize),
        tiles: [[usize; TILE_SIZE]; TILE_SIZE],
        ps: [[i32; TILE_SIZE]; TILE_SIZE],
    }
    // [debug] 入力受け取りを待たないため
    // get_time();
    let input = Input { s, tiles, ps };
    let mut rng = rand_pcg::Pcg64Mcg::new(42);

    // [part1] sを始点とする初期解をいくつかDFSで構築する
    let M = input
        .tiles
        .iter()
        .map(|t| t.iter().max().unwrap())
        .max()
        .unwrap()
        + 1;
    // 初期解をSOLUTION_SIZE個格納するbinaryheap
    let mut fist_action_bh = BinaryHeap::new();
    // DFSでの初期解構築で，探索の順序を[左or右, 上or下, 左or右, 上or下], [上or下, 左or右, 上or下, 左or右]の計8通り試す
    let dir_lists = vec![vec![0, 2, 1, 3], vec![0, 3, 1, 2], vec![1, 2, 0, 3], vec![1, 3, 0, 2], vec![2, 0, 3, 1], vec![2, 1, 3, 0], vec![3, 0, 2, 1], vec![3, 1, 2, 0]];    
    for d in dir_lists {
        let mut seen = vec![false; M];
        seen[input.tiles[input.s.0][input.s.1]] = true;
        let mut actions = vec![];
        let mut best_actions_bh = BinaryHeap::new();
        let mut score = input.ps[input.s.0][input.s.1];
        let crt_time = get_time();
        dfs_making_first_solution(&d, s.0, s.1, !0, TILE_SIZE, TILE_SIZE, &input, &mut seen, &mut actions, &mut best_actions_bh, score, 0, crt_time, 0.005);
        let best_actions = best_actions_bh.pop().unwrap();
        let best_score = best_actions.1;
        let best_action = best_actions.2;
        eprintln!("first score {}", best_score);
        // 大きい方からSOLUTION_SIZE個残す
        fist_action_bh.push((Reverse(best_score), best_action.clone()));
        if fist_action_bh.len() > SOLUTION_SIZE {
            fist_action_bh.pop();
        }
    }
    
    // DFSで見つけたactionをstateに施し，初期解とする
    let mut state_bh = BinaryHeap::new();
    while let Some(score_action) = fist_action_bh.pop() {
        let mut crt_state = State::new(&input, !0, input.s);
        for &action in &score_action.1 {
            crt_state.advance(&input, action);
        }
        crt_state.evaluateScore();
        state_bh.push(crt_state);
    }
    
    // eprintln!("score {}", crt_state.game_score_);
    // crt_state.toString(&input);
    // input! {s:usize}

    // state_bh.push(state);
    // }
    // [debug] 初期解構築の確認
    // while let Some(state) = state_bh.pop() {
    //     eprintln!("score {}", state.game_score_);
    //     state.toString(&input);
    //     input! {s:usize}
    // }

    // [part2] S以外のすでに訪問した頂点から2点を選び，テキトーに繋ぎ変える
    // let mut crt_state = state_bh.pop().unwrap(); // 初期解の中で最高得点のものを取り出す
    // let mut best_output = crt_state.output_.clone();
    let mut best_output = String::new(); // 表示用
    // let mut best_output = state_bh.pop().unwrap().output_;
    // let mut best_score = crt_state.game_score_;
    let mut best_score = 0;
    // 全体の制限時間内
    let mut state_iter = 0;
    while get_time() < TIME_LIMIT && state_iter < SOLUTION_SIZE {
        // 各初期解それぞれに対して焼きなましで改善するかを調べる
        state_iter += 1;
        let mut crt_state = state_bh.pop().unwrap();
        // eprintln!("{}番目の初期解の改善を行う", state_iter);
        // 各初期会の持ち時間はTL/SOLUTION_SIZEずつ与えられる
        while get_time() < TIME_LIMIT * state_iter as f64 / SOLUTION_SIZE as f64 {
            let t = get_time() / (TIME_LIMIT * state_iter as f64 / SOLUTION_SIZE as f64);
            let T = T0.powf(1.0 - t) * T1.powf(t);
            // eprintln!("crt state score {}", crt_state.evaluated_score_);
            // crt_state.toString(&input);

            // p1, p2の候補を探す
            // [ToDo] p1->p2の再検索幅を模索する．結構大きくてもいいかも
            // なるべく空きます周辺のp1, p2を選びたい
            let step = crt_state.steps_.clone(); // 今までの軌跡から近い2点を選ぶ
            let p1_idx = rng.gen_range(1, step.len()-1);
            let p2_idx = rng.gen_range(p1_idx + 1, std::cmp::min(p1_idx + 50, step.len()));
            let p1 = step[p1_idx];
            let p2 = step[p2_idx];

            // もとのaction = [0...p1....p2......n]みたいな感じとして，
            // ここではp1...p2だけ求め，
            // 新しいstateをaction[0:p1]，newaction[p1:p2], action[p2:n]進めるみたいなことをすれば実現可能か
            // 速度が気になる

            // p1->p2への経路のseenをfalseにする
            // 注：p1上から始めるのでp1のseenはfalseにしない
            let mut seen = crt_state.seen_.clone();
            for p_idx in p1_idx+1..=p2_idx {
                seen[input.tiles[step[p_idx].0][step[p_idx].1]] = false;
            }
            // [debug] p1->p2へのseenを消したことの確認
            // let mut tmp_state = crt_state.clone();
            // tmp_state.seen_ = seen.clone();
            // eprintln!("tmp state");
            // tmp_state.toString(&input);

            // p1->p2への経路のひとつをDFSで探す
            let mut actions = vec![];
            let mut action_bh = BinaryHeap::new();
            let crt_time = get_time();
            dfs_to_destination(p1.0, p1.1, p2.0, p2.1, TILE_SIZE, TILE_SIZE, &input, &mut seen, &mut actions, &mut action_bh, 0, crt_time, 0.001);
            // eprintln!("p1, p2: {:?}, {:?}", p1, p2);
            // eprintln!("new action: {:?}", action_bh);
            // p1->p2への経路がなければやり直し，あればどれか選ぶ
            if let Some(actions_tuple) = action_bh.pop() {
                actions = actions_tuple.1;
            } else {
                continue;
            }
            // 古いactionを保持(LRUD)
            let old_actions = crt_state.output_.clone();
            // 新しいstateを作り，スコアが高いかどうか判定する
            let mut next_state = State::new(&input, !0, input.s);
            // eprintln!("action確認");
            // eprintln!("{:?}", old_actions);
            // eprintln!("{:?}", &old_actions[0..=p1_idx]);
            // eprintln!("{:?}", actions);
            // eprintln!("{:?}", &old_actions[p2_idx..]);

            // 最初からp1までは過去のactionで進める
            // 注：action[i]でi+1番目のマスに行くことに注意
            for c in old_actions[0..p1_idx].chars() {
                let action = match c {
                    // const DIR: [char; 4] = ['L', 'R', 'U', 'D'];
                    'L' => 0,
                    'R' => 1,
                    'U' => 2,
                    'D' => 3,
                    _ => unreachable!()
                };
                next_state.advance(&input, action);
            }
            // p1からp2まで新しいactionで進める
            for &action in &actions {
                next_state.advance(&input, action);
            }
            // p2の次から最後まで過去のactionで進める
            for c in old_actions[p2_idx..].chars() {
                let action = match c {
                    // const DIR: [char; 4] = ['L', 'R', 'U', 'D'];
                    'L' => 0,
                    'R' => 1,
                    'U' => 2,
                    'D' => 3,
                    _ => unreachable!()
                };
                next_state.advance(&input, action);
            }
            next_state.evaluateScore();

            // 実際のゲームの得点に対し，best_scoreを更新したら常にbest_outputを更新する
            let next_score = next_state.game_score_;
            if next_score >= best_score {
                best_score = next_score;
                best_output = next_state.output_.clone();
                // eprintln!("updated best score: {}", best_score);
                // input!{
                //     tmp: usize,
                // }
            }

            // 評価関数の評価値に対し，crt_evaluated_score <= next_evaluated_score か 焼きなましの許容範囲ならnext_stateをcrt_stateとする
            let crt_evaluated_score = crt_state.evaluated_score_;
            let next_evaluated_score = next_state.evaluated_score_;
            if crt_evaluated_score <= next_evaluated_score || rng.gen_bool(((next_evaluated_score - crt_evaluated_score) as f64 / T).exp()) {
                crt_state = next_state;
                // if crt_evaluated_score > next_evaluated_score {
                //     eprintln!("annealing");
                //     eprintln!("next state evaluated score {}", next_evaluated_score);
                //     crt_state.toString(&input);
                //     // input!{
                //     //     tmp: usize,
                //     // }
                // }
            }
        }
    }
    println!("{}", best_output);
    eprintln!("score: {}", best_score);
    eprintln!("time: {:.3}", get_time());
}

// [ToDo]
// submit前にtimelimitを確認
// 温度チェック
// TLチェック(全体/初期解構築/DFS探索とか)
// 初期解数チェック
// 長らく更新しなくなったら次の初期解にいく
// 評価関数の設計
// 空きマスの近くのp1,p2を見に行く
// 時間半分でやってみて，スコアあんま変わらないなら2nd bestの初期解でも探索してもいいかも
// 時間1/8でもあまり変わらないので多点スタートすべき