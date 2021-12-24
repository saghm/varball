#[cfg(test)]
mod test;

use array_macro::array;
use num_format::{Buffer, Locale};
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};
use structopt::StructOpt;

const NUM_INNINGS_DEFAULT: u8 = 9;

const NUM_EXTRA_INNING_LENGTHS: usize = (u8::MAX - 10) as usize;
static INNING_COUNTS: [AtomicUsize; NUM_EXTRA_INNING_LENGTHS] =
    array![_ => AtomicUsize::new(0); NUM_EXTRA_INNING_LENGTHS];

#[derive(StructOpt)]
struct Options {
    /// How many games to simulate.
    #[structopt(short, long, default_value = "1000000")]
    num_games: usize,

    /// The geometric factor for a chance to score in a non-extra inning.
    #[structopt(short, long, default_value = "40")]
    regular_score_percent: u8,

    /// The geometric factor for a chance to score in an extra inning.
    #[structopt(short, long, default_value = "40")]
    extra_innings_score_percent: u8,

    /// Whether to skip the first nine innings and assume all games will go into extra innings.
    #[structopt(short, long)]
    skip_first_nine_innings: bool,

    /// Disable simulating games in parallel.
    #[structopt(short, long)]
    disable_parallel: bool,
}

type RunChecker = Box<dyn Fn(&GameState) -> u8>;

struct Game {
    state: GameState,
    run_checker: RunChecker,
}

#[derive(Debug, Default)]
struct GameState {
    home_team_runs: u8,
    away_team_runs: u8,
    inning: u8,
    half_inning: HalfInning,
}

#[derive(Debug)]
struct FinalScore {
    home_team: u8,
    away_team: u8,
    inning: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum HalfInning {
    Top,
    Bottom,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Team {
    Home,
    Away,
}

impl Default for HalfInning {
    fn default() -> Self {
        Self::Bottom
    }
}

impl HalfInning {
    fn flip(&mut self) {
        *self = match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        };
    }
}

impl Game {
    fn new(run_checker: RunChecker) -> Self {
        Self {
            state: Default::default(),
            run_checker,
        }
    }
}

impl GameState {
    fn is_over(&self) -> bool {
        (self.inning == NUM_INNINGS_DEFAULT &&
            self.half_inning == HalfInning::Top &&
            self.home_team_runs > self.away_team_runs) ||
            (self.inning >= NUM_INNINGS_DEFAULT &&
                self.half_inning == HalfInning::Bottom &&
                self.home_team_runs != self.away_team_runs)
    }

    fn step(&mut self, runs_scored: u8) {
        if self.is_over() {
            return;
        }

        self.half_inning.flip();

        if self.half_inning == HalfInning::Top {
            self.inning += 1;
        }

        if runs_scored > 0 {
            match self.half_inning {
                HalfInning::Top => self.away_team_runs += runs_scored,
                HalfInning::Bottom => self.home_team_runs += runs_scored,
            };
        }
    }
}

impl Game {
    fn complete(&mut self) -> FinalScore {
        while !self.state.is_over() {
            self.state.step(self.run_checker.as_ref()(&self.state));
        }

        FinalScore {
            home_team: self.state.home_team_runs,
            away_team: self.state.away_team_runs,
            inning: self.state.inning,
        }
    }
}

#[allow(dead_code)]
impl FinalScore {
    fn winner(&self) -> Team {
        match self.home_team.cmp(&self.away_team) {
            Ordering::Greater => Team::Home,
            Ordering::Less => Team::Away,
            Ordering::Equal => unreachable!(),
        }
    }
}

fn simulate_game(
    regular_score_percent: u8,
    extra_innings_score_percent: u8,
    skip_first_nine_innings: bool,
) -> FinalScore {
    let mut game = Game::new(Box::new(move |state| {
        std::iter::repeat(1)
            .take_while(|_| {
                if state.inning > NUM_INNINGS_DEFAULT {
                    thread_rng().gen_range(1..=100) < extra_innings_score_percent
                } else {
                    thread_rng().gen_range(1..=100) < regular_score_percent
                }
            })
            .sum()
    }));

    if skip_first_nine_innings {
        game.state.inning = NUM_INNINGS_DEFAULT;
    }

    game.complete()
}

fn update_inning_count(num_innings: u8) {
    let inning_index = num_innings as usize - 10;
    INNING_COUNTS[inning_index].fetch_add(1, SeqCst);
}

macro_rules! sim_games {
    ($iter:expr, $opts:expr) => {{
        let Options {
            regular_score_percent,
            extra_innings_score_percent,
            skip_first_nine_innings,
            ..
        } = *$opts;

        $iter
            .filter_map(move |_| {
                let inning = simulate_game(
                    regular_score_percent,
                    extra_innings_score_percent,
                    skip_first_nine_innings,
                )
                .inning;

                (inning > NUM_INNINGS_DEFAULT).then(|| inning)
            })
            .inspect(|num_innings| update_inning_count(*num_innings))
            .count()
    }};
}

fn simulate_inning_counts(opts: &Options) -> usize {
    let iter = 0..opts.num_games;

    if opts.disable_parallel {
        sim_games!(iter, opts)
    } else {
        sim_games!(iter.into_par_iter(), opts)
    }
}

fn formatted_usize(number: usize) -> Buffer {
    let mut buffer = Buffer::new();
    buffer.write_formatted(&number, &Locale::en);
    buffer
}

fn print_inning_counts(total_games: usize, num_extra_innings_games: usize) {
    let total_games_display = format!("{}", formatted_usize(total_games));

    println!("Total games played: {}", total_games_display);
    println!(
        "Number of extra inning games: {}",
        formatted_usize(num_extra_innings_games)
    );
    println!("\nNumber of games with <n> innings:");

    for (inning, count) in INNING_COUNTS
        .iter()
        .map(|count| count.load(SeqCst))
        .enumerate()
        .filter(|(_, count)| *count > 0)
    {
        print!("  {} innings: ", inning + 10);
        println!(
            "{count:>width$}",
            count = formatted_usize(count).as_str(),
            width = total_games_display.chars().count(),
        );
    }
}

fn main() -> Result<(), u8> {
    let args = Options::from_args();

    if args.regular_score_percent < 1 || args.regular_score_percent > 99 {
        eprintln!("--regular-score-percent must be between 1 and 99 (inclusive)");
        return Err(1);
    }

    if args.extra_innings_score_percent < 1 || args.regular_score_percent > 99 {
        eprintln!("--extra-innings-score-percent must be between 1 and 99 (inclusive)");
        return Err(2);
    }

    let inning_counts = simulate_inning_counts(&args);
    print_inning_counts(args.num_games, inning_counts);

    Ok(())
}
